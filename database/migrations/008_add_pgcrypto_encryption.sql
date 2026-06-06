-- Add pgcrypto extension for encryption
CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- Create encryption key management table
CREATE TABLE IF NOT EXISTS encryption_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    key_name VARCHAR(100) UNIQUE NOT NULL,
    key_value BYTEA NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Generate and store a default encryption key
INSERT INTO encryption_keys (key_name, key_value)
VALUES ('default', gen_random_bytes(32))
ON CONFLICT (key_name) DO NOTHING;

-- Function to encrypt data
CREATE OR REPLACE FUNCTION encrypt_data(data TEXT, key_name VARCHAR DEFAULT 'default')
RETURNS BYTEA AS $$
DECLARE
    key BYTEA;
BEGIN
    SELECT key_value INTO key FROM encryption_keys WHERE key_name = encryption_keys.key_name LIMIT 1;
    RETURN pgp_sym_encrypt(data, convert_from(key, 'UTF8'));
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- Function to decrypt data
CREATE OR REPLACE FUNCTION decrypt_data(data BYTEA, key_name VARCHAR DEFAULT 'default')
RETURNS TEXT AS $$
DECLARE
    key BYTEA;
BEGIN
    SELECT key_value INTO key FROM encryption_keys WHERE key_name = encryption_keys.key_name LIMIT 1;
    RETURN pgp_sym_decrypt(data, convert_from(key, 'UTF8'));
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- Add encrypted columns for PII in hospital_patients
ALTER TABLE hospital_patients
ADD COLUMN emergency_contact_name_encrypted BYTEA,
ADD COLUMN emergency_contact_phone_encrypted BYTEA,
ADD COLUMN insurance_provider_encrypted BYTEA,
ADD COLUMN insurance_policy_number_encrypted BYTEA;

-- Migrate existing data to encrypted columns
UPDATE hospital_patients
SET emergency_contact_name_encrypted = encrypt_data(emergency_contact_name),
    emergency_contact_phone_encrypted = encrypt_data(emergency_contact_phone),
    insurance_provider_encrypted = encrypt_data(insurance_provider),
    insurance_policy_number_encrypted = encrypt_data(insurance_policy_number)
WHERE emergency_contact_name IS NOT NULL OR emergency_contact_phone IS NOT NULL
   OR insurance_provider IS NOT NULL OR insurance_policy_number IS NOT NULL;

-- Drop original plain text columns (after successful migration)
-- Uncomment after verifying migration success:
-- ALTER TABLE hospital_patients DROP COLUMN emergency_contact_name;
-- ALTER TABLE hospital_patients DROP COLUMN emergency_contact_phone;
-- ALTER TABLE hospital_patients DROP COLUMN insurance_provider;
-- ALTER TABLE hospital_patients DROP COLUMN insurance_policy_number;

-- Add encrypted columns for PII in facilities
ALTER TABLE facilities
ADD COLUMN phone_encrypted BYTEA,
ADD COLUMN email_encrypted BYTEA;

-- Migrate existing data
UPDATE facilities
SET phone_encrypted = encrypt_data(phone),
    email_encrypted = encrypt_data(email)
WHERE phone IS NOT NULL OR email IS NOT NULL;

-- Add encrypted columns for PII in insurance_providers
ALTER TABLE insurance_providers
ADD COLUMN contact_email_encrypted BYTEA,
ADD COLUMN contact_phone_encrypted BYTEA;

-- Migrate existing data
UPDATE insurance_providers
SET contact_email_encrypted = encrypt_data(contact_email),
    contact_phone_encrypted = encrypt_data(contact_phone)
WHERE contact_email IS NOT NULL OR contact_phone IS NOT NULL;

-- Create views for convenient access (decrypt on read)
CREATE OR REPLACE VIEW hospital_patients_secure AS
SELECT 
    id,
    profile_id,
    patient_id,
    blood_type,
    COALESCE(decrypt_data(emergency_contact_name_encrypted), emergency_contact_name) AS emergency_contact_name,
    COALESCE(decrypt_data(emergency_contact_phone_encrypted), emergency_contact_phone) AS emergency_contact_phone,
    COALESCE(decrypt_data(insurance_provider_encrypted), insurance_provider) AS insurance_provider,
    COALESCE(decrypt_data(insurance_policy_number_encrypted), insurance_policy_number) AS insurance_policy_number,
    admission_date,
    discharge_date,
    status,
    created_at,
    updated_at
FROM hospital_patients;

CREATE OR REPLACE VIEW facilities_secure AS
SELECT
    id,
    name,
    address,
    COALESCE(decrypt_data(phone_encrypted), phone) AS phone,
    COALESCE(decrypt_data(email_encrypted), email) AS email,
    timezone,
    is_active,
    created_at,
    updated_at
FROM facilities;

CREATE OR REPLACE VIEW insurance_providers_secure AS
SELECT
    id,
    name,
    COALESCE(decrypt_data(contact_email_encrypted), contact_email) AS contact_email,
    COALESCE(decrypt_data(contact_phone_encrypted), contact_phone) AS contact_phone,
    claims_address,
    is_active,
    created_at,
    updated_at
FROM insurance_providers;

-- Grant necessary permissions
GRANT EXECUTE ON FUNCTION encrypt_data TO PUBLIC;
GRANT EXECUTE ON FUNCTION decrypt_data TO PUBLIC;
GRANT SELECT ON encryption_keys TO PUBLIC;
