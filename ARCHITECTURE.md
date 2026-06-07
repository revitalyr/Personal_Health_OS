# Architecture

## System Overview

Event-driven microservice architecture. Single API Gateway (port 8080) routes to 8 downstream services. PostgreSQL stores all data. NATS handles async event passing. Redis caches frequent queries. Jaeger collects distributed traces.

## Core Patterns

### Event Sourcing

All medical data stored as immutable events in `medical_events` table:

```sql
CREATE TABLE medical_events (
    id UUID PRIMARY KEY,
    patient_id UUID NOT NULL,
    event_type VARCHAR(50) NOT NULL CHECK (event_type IN ('Symptom','Medication','LabResult','DoctorVisit','Diagnosis','Document','Vitals','Encounter')),
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
    payload JSONB NOT NULL,
    source VARCHAR(100) NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_medical_events_patient_timestamp ON medical_events (patient_id, timestamp);
CREATE INDEX idx_medical_events_payload ON medical_events USING GIN (payload);
```

### Trait-Based Abstraction

- `EventStore` trait (in `crates/storage`) enables mockable event persistence. `PostgresEventStore` is the concrete implementation.
- Processor traits (`OcrProcessing`, `DicomProcessing`, `NlpProcessing`) in `document-processor` enable dependency injection via `Arc<dyn Trait>`.

### Dual Auth Service

- `AuthService` — lightweight JWT verification only (used by middleware in downstream services).
- `FullAuthService` — DB-backed auth: register, login (email, Google OAuth, Apple stub, phone OTP), profile management.

## Services

| Service | Port | Purpose |
|---------|------|---------|
| api-gateway | 8080 | Auth, routing, CORS, proxying to downstream services |
| timeline-service | 8081 | Event CRUD, timeline build/filter/summary, anomaly detection |
| document-processor | 8082 | Document upload, OCR pipeline, DICOM, NLP, manual entry |
| ai-report-service | 8083 | AI report generation (stub) |
| doctor-access-service | 8084 | 15-minute JWT tokens for temporary doctor access |
| patient-management | 8085 | Hospital patient CRUD, encounters, vitals, allergies, medications |
| appointment-scheduling | — | Appointment lifecycle, availability, reminders |
| billing-invoicing | — | Invoices, payments, insurance claims |
| licence-management | — | RSA-signed licence keys, hardware fingerprinting |

## Data Flows

### Document OCR Pipeline

```
Client -> API Gateway -> Document Processor
  1. Upload: INSERT document record, save file to storage
  2. Async pipeline (tokio::spawn):
     a. Read file bytes
     b. OcrProcessor::perform_ocr() -> text
     c. MedicalEntityExtractor::extract_entities() -> entities
     d. UPDATE document with OCR text + extracted data
     e. Publish ocr.completed + entities.extracted to NATS
```

### Timeline Query

```
Client -> API Gateway -> Timeline Service -> PostgresEventStore -> TimelineEngine
  1. get_events_by_patient() from DB
  2. build_timeline(events) -> sort by timestamp
  3. generate_summary(timeline) -> counts, date range, event type histogram, recent events
  4. detect_anomalies(timeline) -> duplicates, future events, high-frequency days
```

### Doctor Access

```
Patient: POST /doctor-access/:id -> generate_doctor_access_token (15min JWT) -> INSERT token hash
Doctor:  GET /doctor-view/:token   -> validate JWT -> query patient events -> return structured report
```

## Database

8 migrations covering:

1. `users` table (email + password_hash)
2. Multi-auth (`accounts`, `user_profiles`, `otp_sessions`, `medication_reminders`)
3. `medical_events` with JSONB payload
4. Hospital management (13 tables: patients, encounters, vitals, allergies, medications, appointments, facilities, invoices, etc.)
5. `documents` table with GIN index on extracted_data
6. `doctor_access_tokens` with access count limits
7. pgcrypto encryption for PII columns + secure views

## Infrastructure

| Component | Port |
|-----------|------|
| PostgreSQL | 5432 |
| NATS | 4222 |
| Redis | 6379 |
| Jaeger (UDP) | 6831 |

## Known Issues

- `licence-management`, `appointment-scheduling`, `billing-invoicing`, `patient-management`, `ai-report-service`, `document-processor` have pre-existing compilation errors (missing deps, broken imports, API mismatches).
- OCR, DICOM, and AI report processors are stubs (return mock data).
- No Kubernetes deployment configuration.
- No CI/CD pipeline configured.
- Mobile apps (SwiftUI, Kotlin) and web frontend (Next.js) are placeholders.
