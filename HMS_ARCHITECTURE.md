# Hospital Management System Architecture

## System Overview

The Hospital Management System (HMS) extends the existing Personal Health OS with enterprise-grade hospital operations, patient management, and license control.

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                    Hospital Management System                    │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Desktop App   │    │   Web Portal    │    │  Mobile App     │
│  (Tauri/Electron)│   │  (Next.js)     │    │ (React Native)  │
└─────────┬───────┘    └─────────┬───────┘    └─────────┬───────┘
          │                      │                      │
          └──────────────────────┼──────────────────────┘
                                 │
          ┌─────────────────────────────────────────────┐
          │            API Gateway (Port 8080)         │
          │  + Licence Verification Middleware          │
          └─────────────────────┬───────────────────────┘
                                │
          ┌─────────────────────────────────────────────┐
          │            Event Bus (NATS)                │
          └─────────────────────┬───────────────────────┘
                                │
    ┌───────────┬───────────┬───────────┬───────────┬───────────┐
    ▼           ▼           ▼           ▼           ▼           ▼
┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐
│Patient  │ │Appointment│ │Billing  │ │Licence  │ │Notification│ │Reporting│
│Service  │ │Service   │ │Service  │ │Service  │ │Service   │ │Service  │
│(8085)   │ │(8086)    │ │(8087)    │ │(8088)    │ │(8089)     │ │(8090)    │
└─────────┘ └─────────┘ └─────────┘ └─────────┘ └─────────┘ └─────────┘
    │           │           │           │           │           │
    └───────────┼───────────┼───────────┼───────────┼───────────┘
                ▼           ▼           ▼           ▼           ▼
        ┌─────────────────────────────────────────────────────────┐
        │              Database Layer                            │
        │  PostgreSQL + Redis + Object Storage                 │
        └─────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                    Licence/CRM System                         │
├─────────────────────────────────────────────────────────────────┤
│  Browser Dashboard (Next.js)                                  │
│  - Create/Suspend/Reassign licences                            │
│  - Usage analytics & reporting                                │
│  - Customer management                                        │
└─────────────────────────────────────────────────────────────────┘
```

## Core Services

### 1. Patient Management Service (Port 8085)
**Responsibilities:**
- Electronic Medical Records (EMR)
- Patient admission/discharge workflows
- Complete patient history timeline
- Integration with existing Health OS patient data

**Key Features:**
- Patient registration and demographics
- Medical history and allergies
- Admission/discharge tracking
- Integration with personal health profiles
- Secure data storage with HIPAA compliance

### 2. Appointment Scheduling Service (Port 8086)
**Responsibilities:**
- Shared availability calendar for doctors
- Patient appointment booking
- Automated reminder system (email/SMS)
- Resource management (rooms, equipment)

**Key Features:**
- Real-time calendar synchronization
- Multi-doctor scheduling
- Automated reminders
- Waitlist management
- Recurring appointments

### 3. Billing & Invoicing Service (Port 8087)
**Responsibilities:**
- Charge capture and billing
- Multi-payer billing (insurance, self-pay)
- Invoice generation and printing
- Basic financial reporting

**Key Features:**
- Service charge management
- Insurance claim processing
- Payment tracking
- Invoice templates
- Revenue reporting

### 4. Licence Management Service (Port 8088)
**Responsibilities:**
- Licence key generation and validation
- Subscription management
- Usage tracking and analytics
- Integration with desktop clients

**Key Features:**
- Licence key generation
- Real-time validation
- Usage analytics
- Subscription tiers
- Automated expiry handling

### 5. Notification Service (Port 8089)
**Responsibilities:**
- Email notifications
- SMS reminders
- Push notifications
- In-app notifications

**Key Features:**
- Multi-channel notifications
- Template management
- Delivery tracking
- Scheduling system

### 6. Reporting Service (Port 8090)
**Responsibilities:**
- Patient analytics
- Financial reporting
- Operational metrics
- Compliance reporting

**Key Features:**
- Real-time dashboards
- Custom report generation
- Data export
- Scheduled reports

## Database Schema

### Patient Management Tables
```sql
-- Hospital patients (extends existing user_profiles)
CREATE TABLE hospital_patients (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    profile_id UUID REFERENCES user_profiles(id),
    patient_id VARCHAR(50) UNIQUE NOT NULL,
    blood_type VARCHAR(10),
    emergency_contact_name VARCHAR(255),
    emergency_contact_phone VARCHAR(20),
    insurance_provider VARCHAR(255),
    insurance_policy_number VARCHAR(100),
    admission_date TIMESTAMP WITH TIME ZONE,
    discharge_date TIMESTAMP WITH TIME ZONE,
    status VARCHAR(20) DEFAULT 'active',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Medical encounters
CREATE TABLE medical_encounters (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    patient_id UUID REFERENCES hospital_patients(id),
    doctor_id UUID,
    encounter_type VARCHAR(50),
    start_time TIMESTAMP WITH TIME ZONE,
    end_time TIMESTAMP WITH TIME ZONE,
    diagnosis TEXT,
    treatment TEXT,
    notes TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
```

### Appointment Management Tables
```sql
-- Appointments
CREATE TABLE appointments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    patient_id UUID REFERENCES hospital_patients(id),
    doctor_id UUID,
    facility_id UUID,
    appointment_type VARCHAR(50),
    start_time TIMESTAMP WITH TIME ZONE NOT NULL,
    end_time TIMESTAMP WITH TIME ZONE NOT NULL,
    status VARCHAR(20) DEFAULT 'scheduled',
    notes TEXT,
    reminder_sent BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Doctor availability
CREATE TABLE doctor_availability (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    doctor_id UUID,
    day_of_week INTEGER,
    start_time TIME,
    end_time TIME,
    is_available BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
```

### Billing & Invoicing Tables
```sql
-- Invoices
CREATE TABLE invoices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    patient_id UUID REFERENCES hospital_patients(id),
    invoice_number VARCHAR(50) UNIQUE NOT NULL,
    issue_date DATE NOT NULL,
    due_date DATE NOT NULL,
    total_amount DECIMAL(10,2) NOT NULL,
    status VARCHAR(20) DEFAULT 'pending',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Invoice items
CREATE TABLE invoice_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    invoice_id UUID REFERENCES invoices(id),
    description TEXT NOT NULL,
    quantity INTEGER NOT NULL,
    unit_price DECIMAL(10,2) NOT NULL,
    total_price DECIMAL(10,2) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
```

### Licence Management Tables
```sql
-- Licences
CREATE TABLE licences (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    licence_key VARCHAR(255) UNIQUE NOT NULL,
    customer_name VARCHAR(255),
    customer_email VARCHAR(255),
    product_name VARCHAR(100),
    tier VARCHAR(50),
    max_users INTEGER,
    expiry_date TIMESTAMP WITH TIME ZONE,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Licence usage tracking
CREATE TABLE licence_usage (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    licence_id UUID REFERENCES licences(id),
    client_id VARCHAR(255),
    action VARCHAR(100),
    timestamp TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    metadata JSONB
);
```

## Desktop Client Architecture

### Technology Stack
- **Framework**: Tauri (Rust + Web frontend)
- **Frontend**: React + TypeScript
- **Database**: SQLite (local) + PostgreSQL (cloud sync)
- **Licence Verification**: Rust-based validation

### Licence Verification Flow
```
Desktop App Start
        ↓
Check Local Licence Cache
        ↓
If Valid → Start Application
        ↓
If Invalid/Expired → Prompt for Licence Key
        ↓
Validate with Licence Service (Port 8088)
        ↓
If Valid → Cache & Start
        ↓
If Invalid → Show Error & Exit
```

## Web Dashboard Architecture

### Technology Stack
- **Framework**: Next.js 13+ with TypeScript
- **UI**: Tailwind CSS + Headless UI
- **Charts**: Recharts
- **State Management**: React Query
- **Authentication**: NextAuth.js

### Dashboard Features
- Licence creation and management
- Customer relationship management
- Usage analytics and reporting
- Real-time monitoring
- Subscription management

## Security & Compliance

### Data Protection
- **HIPAA Compliance**: All patient data encrypted
- **GDPR Compliance**: Data handling and privacy
- **Access Control**: Role-based permissions
- **Audit Logging**: Complete activity tracking

### Licence Security
- **RSA Encryption**: Secure licence key generation
- **Digital Signatures**: Prevent tampering
- **Online Validation**: Real-time verification
- **Hardware Fingerprinting**: Prevent sharing

## Integration Points

### Health OS Integration
- **Patient Profiles**: Extend existing personal health profiles
- **Timeline Service**: Reuse medical timeline functionality
- **Document Storage**: Integrate with existing document processor
- **AI Reports**: Leverage existing AI report generation

### External Integrations
- **Email Services**: SendGrid/AWS SES
- **SMS Services**: Twilio
- **Payment Gateways**: Stripe/PayPal
- **Insurance APIs**: Standard medical billing APIs

## Deployment Architecture

### Microservices Deployment
- **Kubernetes**: Container orchestration
- **Service Mesh**: Istio for service communication
- **Load Balancing**: NGINX/HAProxy
- **Monitoring**: Prometheus + Grafana
- **Logging**: ELK Stack

### Database Deployment
- **PostgreSQL**: Primary database with replication
- **Redis**: Caching and session storage
- **Object Storage**: Medical documents and backups
- **Backup Strategy**: Automated daily backups

## Performance Considerations

### Scalability
- **Horizontal Scaling**: Stateless services
- **Database Sharding**: Patient data partitioning
- **Caching Strategy**: Redis for frequently accessed data
- **CDN**: Static asset delivery

### Performance Metrics
- **Response Time**: <200ms for API calls
- **Page Load**: <2 seconds for web dashboard
- **Desktop Startup**: <3 seconds for desktop app
- **Database Queries**: Optimized for <100ms response
