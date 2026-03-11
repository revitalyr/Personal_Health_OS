-- Create documents table
CREATE TABLE IF NOT EXISTS documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    patient_id UUID NOT NULL REFERENCES patient_profiles(id) ON DELETE CASCADE,
    filename VARCHAR(255) NOT NULL,
    file_type VARCHAR(50) NOT NULL,
    file_size BIGINT NOT NULL,
    storage_path VARCHAR(500) NOT NULL,
    document_type VARCHAR(50), -- prescription, lab_report, radiology, etc.
    facility VARCHAR(255),
    document_date DATE,
    processed BOOLEAN DEFAULT FALSE,
    ocr_extracted_text TEXT,
    extracted_data JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create indexes
CREATE INDEX IF NOT EXISTS idx_documents_patient_id ON documents(patient_id);
CREATE INDEX IF NOT EXISTS idx_documents_document_type ON documents(document_type);
CREATE INDEX IF NOT EXISTS idx_documents_document_date ON documents(document_date);
CREATE INDEX IF NOT EXISTS idx_documents_processed ON documents(processed);
CREATE INDEX IF NOT EXISTS idx_documents_extracted_data ON documents USING GIN(extracted_data);

-- Add trigger for updated_at
CREATE TRIGGER update_documents_updated_at 
    BEFORE UPDATE ON documents 
    FOR EACH ROW 
    EXECUTE FUNCTION update_updated_at_column();

-- Add constraint for valid document types
ALTER TABLE documents 
ADD CONSTRAINT check_document_type 
CHECK (document_type IN (
    'prescription',
    'lab_report', 
    'radiology',
    'discharge_summary',
    'referral',
    'insurance',
    'other'
) OR document_type IS NULL);
