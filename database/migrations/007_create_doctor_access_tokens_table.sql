-- Create doctor access tokens table
CREATE TABLE IF NOT EXISTS doctor_access_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    patient_id UUID NOT NULL REFERENCES patient_profiles(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL UNIQUE,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    accessed_at TIMESTAMP WITH TIME ZONE,
    access_count INTEGER DEFAULT 0,
    max_accesses INTEGER DEFAULT 1,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    created_by VARCHAR(100) NOT NULL -- user_id or system
);

-- Create indexes
CREATE INDEX IF NOT EXISTS idx_doctor_access_tokens_patient_id ON doctor_access_tokens(patient_id);
CREATE INDEX IF NOT EXISTS idx_doctor_access_tokens_token_hash ON doctor_access_tokens(token_hash);
CREATE INDEX IF NOT EXISTS idx_doctor_access_tokens_expires_at ON doctor_access_tokens(expires_at);

-- Add constraint for expiration in the future
ALTER TABLE doctor_access_tokens 
ADD CONSTRAINT check_expires_at_future 
CHECK (expires_at > created_at);

-- Add constraint for reasonable access limits
ALTER TABLE doctor_access_tokens 
ADD CONSTRAINT check_max_accesses 
CHECK (max_accesses > 0 AND max_accesses <= 10);
