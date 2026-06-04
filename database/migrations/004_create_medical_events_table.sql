-- Create medical events table (event store)
CREATE TABLE IF NOT EXISTS medical_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    patient_id UUID NOT NULL REFERENCES patient_profiles(id) ON DELETE CASCADE,
    event_type VARCHAR(50) NOT NULL,
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
    payload JSONB NOT NULL,
    
    -- Metadata
    source VARCHAR(100) NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create indexes for efficient querying
CREATE INDEX IF NOT EXISTS idx_medical_events_patient_id ON medical_events(patient_id);
CREATE INDEX IF NOT EXISTS idx_medical_events_timestamp ON medical_events(timestamp);
CREATE INDEX IF NOT EXISTS idx_medical_events_event_type ON medical_events(event_type);
CREATE INDEX IF NOT EXISTS idx_medical_events_patient_timestamp ON medical_events(patient_id, timestamp);
CREATE INDEX IF NOT EXISTS idx_medical_events_payload ON medical_events USING GIN(payload);

-- Add trigger for updated_at
CREATE TRIGGER update_medical_events_updated_at 
    BEFORE UPDATE ON medical_events 
    FOR EACH ROW 
    EXECUTE FUNCTION update_updated_at_column();

-- Add constraint for valid event types
ALTER TABLE medical_events 
ADD CONSTRAINT check_event_type 
CHECK (event_type IN (
    'SymptomCreated',
    'MedicationStarted', 
    'MedicationStopped',
    'LabResultReceived',
    'DoctorVisit',
    'Diagnosis',
    'DocumentUploaded',
    'ReminderTriggered'
));
