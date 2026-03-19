-- Create accounts table for multi-provider authentication
CREATE TABLE accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE,
    phone VARCHAR(20) UNIQUE,
    password_hash VARCHAR(255),
    google_id VARCHAR(255) UNIQUE,
    apple_id VARCHAR(255) UNIQUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create user profiles table for multi-profile management
CREATE TABLE user_profiles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    relationship VARCHAR(50) NOT NULL CHECK (relationship IN ('self', 'child', 'parent', 'spouse', 'sibling', 'other')),
    date_of_birth TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create OTP sessions table for phone authentication
CREATE TABLE otp_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    phone_number VARCHAR(20) NOT NULL,
    code VARCHAR(10) NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    attempts INTEGER DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create indexes for performance
CREATE INDEX idx_accounts_email ON accounts(email);
CREATE INDEX idx_accounts_phone ON accounts(phone);
CREATE INDEX idx_accounts_google_id ON accounts(google_id);
CREATE INDEX idx_accounts_apple_id ON accounts(apple_id);
CREATE INDEX idx_user_profiles_account_id ON user_profiles(account_id);
CREATE INDEX idx_otp_sessions_phone ON otp_sessions(phone_number);
CREATE INDEX idx_otp_sessions_expires_at ON otp_sessions(expires_at);

-- Create updated_at trigger function
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create triggers for updated_at
CREATE TRIGGER update_accounts_updated_at BEFORE UPDATE ON accounts
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_user_profiles_updated_at BEFORE UPDATE ON user_profiles
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Create medication reminders table
CREATE TABLE medication_reminders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    profile_id UUID NOT NULL REFERENCES user_profiles(id) ON DELETE CASCADE,
    medication_name VARCHAR(255) NOT NULL,
    dosage VARCHAR(100) NOT NULL,
    frequency VARCHAR(100) NOT NULL,
    reminder_times TIME[] NOT NULL,
    start_date DATE NOT NULL,
    end_date DATE,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create trigger for medication reminders updated_at
CREATE TRIGGER update_medication_reminders_updated_at BEFORE UPDATE ON medication_reminders
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Create index for medication reminders
CREATE INDEX idx_medication_reminders_profile_id ON medication_reminders(profile_id);
CREATE INDEX idx_medication_reminders_active ON medication_reminders(is_active);
