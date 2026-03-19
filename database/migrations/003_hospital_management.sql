-- Hospital Management System Database Schema

-- Hospital patients table (extends existing user_profiles)
CREATE TABLE hospital_patients (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    profile_id UUID REFERENCES user_profiles(id) ON DELETE CASCADE,
    patient_id VARCHAR(50) UNIQUE NOT NULL,
    blood_type VARCHAR(10),
    emergency_contact_name VARCHAR(255),
    emergency_contact_phone VARCHAR(20),
    insurance_provider VARCHAR(255),
    insurance_policy_number VARCHAR(100),
    admission_date TIMESTAMP WITH TIME ZONE,
    discharge_date TIMESTAMP WITH TIME ZONE,
    status VARCHAR(20) DEFAULT 'active' CHECK (status IN ('active', 'discharged', 'transferred', 'deceased')),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Medical encounters
CREATE TABLE medical_encounters (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    patient_id UUID REFERENCES hospital_patients(id) ON DELETE CASCADE,
    doctor_id UUID,
    encounter_type VARCHAR(50) NOT NULL CHECK (encounter_type IN ('admission', 'consultation', 'procedure', 'surgery', 'emergency', 'follow_up', 'discharge')),
    start_time TIMESTAMP WITH TIME ZONE NOT NULL,
    end_time TIMESTAMP WITH TIME ZONE,
    diagnosis TEXT,
    treatment TEXT,
    notes TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Patient vitals
CREATE TABLE patient_vitals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    patient_id UUID REFERENCES hospital_patients(id) ON DELETE CASCADE,
    blood_pressure_systolic INTEGER,
    blood_pressure_diastolic INTEGER,
    heart_rate INTEGER,
    temperature DECIMAL(4,1),
    weight DECIMAL(5,2),
    height DECIMAL(5,2),
    oxygen_saturation INTEGER,
    recorded_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    recorded_by UUID
);

-- Patient allergies
CREATE TABLE patient_allergies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    patient_id UUID REFERENCES hospital_patients(id) ON DELETE CASCADE,
    allergen VARCHAR(255) NOT NULL,
    severity VARCHAR(20) NOT NULL CHECK (severity IN ('mild', 'moderate', 'severe', 'life_threatening')),
    reaction TEXT,
    notes TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Patient medications
CREATE TABLE patient_medications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    patient_id UUID REFERENCES hospital_patients(id) ON DELETE CASCADE,
    medication_name VARCHAR(255) NOT NULL,
    dosage VARCHAR(100) NOT NULL,
    frequency VARCHAR(100) NOT NULL,
    route VARCHAR(50) NOT NULL,
    start_date DATE NOT NULL,
    end_date DATE,
    prescribed_by UUID NOT NULL,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Patient events for timeline
CREATE TABLE patient_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    patient_id UUID REFERENCES hospital_patients(id) ON DELETE CASCADE,
    event_type VARCHAR(100) NOT NULL,
    metadata JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Appointments
CREATE TABLE appointments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    patient_id UUID REFERENCES hospital_patients(id) ON DELETE CASCADE,
    doctor_id UUID NOT NULL,
    facility_id UUID NOT NULL,
    appointment_type VARCHAR(50) NOT NULL CHECK (appointment_type IN ('consultation', 'follow_up', 'procedure', 'surgery', 'emergency', 'check_up', 'vaccination', 'test')),
    start_time TIMESTAMP WITH TIME ZONE NOT NULL,
    end_time TIMESTAMP WITH TIME ZONE NOT NULL,
    status VARCHAR(20) DEFAULT 'scheduled' CHECK (status IN ('scheduled', 'confirmed', 'in_progress', 'completed', 'cancelled', 'no_show', 'rescheduled')),
    notes TEXT,
    reminder_sent BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Doctor availability
CREATE TABLE doctor_availability (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    doctor_id UUID NOT NULL,
    day_of_week INTEGER NOT NULL CHECK (day_of_week BETWEEN 0 AND 6),
    start_time TIME NOT NULL,
    end_time TIME NOT NULL,
    is_available BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(doctor_id, day_of_week)
);

-- Facilities
CREATE TABLE facilities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    address VARCHAR(500) NOT NULL,
    phone VARCHAR(20) NOT NULL,
    email VARCHAR(255) NOT NULL,
    timezone VARCHAR(50) NOT NULL,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Appointment reminders
CREATE TABLE appointment_reminders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    appointment_id UUID REFERENCES appointments(id) ON DELETE CASCADE,
    reminder_type VARCHAR(20) NOT NULL CHECK (reminder_type IN ('email', 'sms', 'push', 'in_app')),
    scheduled_for TIMESTAMP WITH TIME ZONE NOT NULL,
    sent_at TIMESTAMP WITH TIME ZONE,
    status VARCHAR(20) DEFAULT 'scheduled' CHECK (status IN ('scheduled', 'sent', 'failed', 'cancelled')),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Appointment events for audit trail
CREATE TABLE appointment_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    appointment_id UUID REFERENCES appointments(id) ON DELETE CASCADE,
    event_type VARCHAR(100) NOT NULL,
    metadata JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Invoices
CREATE TABLE invoices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    patient_id UUID REFERENCES hospital_patients(id) ON DELETE CASCADE,
    invoice_number VARCHAR(50) UNIQUE NOT NULL,
    issue_date DATE NOT NULL,
    due_date DATE NOT NULL,
    total_amount DECIMAL(10,2) NOT NULL,
    status VARCHAR(20) DEFAULT 'pending' CHECK (status IN ('pending', 'sent', 'paid', 'overdue', 'cancelled')),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Invoice items
CREATE TABLE invoice_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    invoice_id UUID REFERENCES invoices(id) ON DELETE CASCADE,
    description TEXT NOT NULL,
    quantity INTEGER NOT NULL,
    unit_price DECIMAL(10,2) NOT NULL,
    total_price DECIMAL(10,2) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Payments
CREATE TABLE payments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    invoice_id UUID REFERENCES invoices(id) ON DELETE CASCADE,
    amount DECIMAL(10,2) NOT NULL,
    payment_method VARCHAR(50) NOT NULL,
    payment_date TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    transaction_id VARCHAR(255),
    status VARCHAR(20) DEFAULT 'completed' CHECK (status IN ('pending', 'completed', 'failed', 'refunded')),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Service charges
CREATE TABLE service_charges (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    category VARCHAR(100) NOT NULL,
    base_price DECIMAL(10,2) NOT NULL,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Insurance providers
CREATE TABLE insurance_providers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    contact_email VARCHAR(255),
    contact_phone VARCHAR(20),
    claims_address TEXT,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create indexes for performance
CREATE INDEX idx_hospital_patients_profile_id ON hospital_patients(profile_id);
CREATE INDEX idx_hospital_patients_patient_id ON hospital_patients(patient_id);
CREATE INDEX idx_hospital_patients_status ON hospital_patients(status);
CREATE INDEX idx_medical_encounters_patient_id ON medical_encounters(patient_id);
CREATE INDEX idx_medical_encounters_doctor_id ON medical_encounters(doctor_id);
CREATE INDEX idx_medical_encounters_start_time ON medical_encounters(start_time);
CREATE INDEX idx_patient_vitals_patient_id ON patient_vitals(patient_id);
CREATE INDEX idx_patient_vitals_recorded_at ON patient_vitals(recorded_at);
CREATE INDEX idx_patient_allergies_patient_id ON patient_allergies(patient_id);
CREATE INDEX idx_patient_medications_patient_id ON patient_medications(patient_id);
CREATE INDEX idx_patient_events_patient_id ON patient_events(patient_id);
CREATE INDEX idx_appointments_patient_id ON appointments(patient_id);
CREATE INDEX idx_appointments_doctor_id ON appointments(doctor_id);
CREATE INDEX idx_appointments_facility_id ON appointments(facility_id);
CREATE INDEX idx_appointments_start_time ON appointments(start_time);
CREATE INDEX idx_appointments_status ON appointments(status);
CREATE INDEX idx_doctor_availability_doctor_id ON doctor_availability(doctor_id);
CREATE INDEX idx_appointment_reminders_appointment_id ON appointment_reminders(appointment_id);
CREATE INDEX idx_appointment_reminders_scheduled_for ON appointment_reminders(scheduled_for);
CREATE INDEX idx_appointment_events_appointment_id ON appointment_events(appointment_id);
CREATE INDEX idx_invoices_patient_id ON invoices(patient_id);
CREATE INDEX idx_invoices_status ON invoices(status);
CREATE INDEX idx_invoice_items_invoice_id ON invoice_items(invoice_id);
CREATE INDEX idx_payments_invoice_id ON payments(invoice_id);

-- Create updated_at trigger function (if not exists)
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create triggers for updated_at
CREATE TRIGGER update_hospital_patients_updated_at BEFORE UPDATE ON hospital_patients
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_appointments_updated_at BEFORE UPDATE ON appointments
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_facilities_updated_at BEFORE UPDATE ON facilities
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_invoices_updated_at BEFORE UPDATE ON invoices
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_service_charges_updated_at BEFORE UPDATE ON service_charges
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_insurance_providers_updated_at BEFORE UPDATE ON insurance_providers
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
