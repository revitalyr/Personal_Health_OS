-- Create patient profiles table
CREATE TABLE IF NOT EXISTS patient_profiles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    relationship VARCHAR(50) NOT NULL, -- self, child, parent, spouse, etc.
    date_of_birth DATE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create indexes
CREATE INDEX IF NOT EXISTS idx_patient_profiles_user_id ON patient_profiles(user_id);
CREATE INDEX IF NOT EXISTS idx_patient_profiles_relationship ON patient_profiles(relationship);

-- Add trigger for updated_at
CREATE TRIGGER update_patient_profiles_updated_at 
    BEFORE UPDATE ON patient_profiles 
    FOR EACH ROW 
    EXECUTE FUNCTION update_updated_at_column();
