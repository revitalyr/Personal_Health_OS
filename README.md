# Health OS

Microservice-based personal health record and hospital management platform. Backend in Rust (Axum), database PostgreSQL, message bus NATS, cache Redis.

## Repository Structure

```
crates/              Shared libraries
├── auth/            JWT verification + DB-backed auth (email, Google, Apple, OTP)
├── event-model/     MedicalEvent, EventType, typed payloads
├── storage/         EventStore trait + PostgresEventStore
├── timeline-engine/ Timeline build/filter/summary/anomaly detection/export
├── ocr-processor/   OCR text extraction (stub)
├── dicom-processor/ DICOM metadata parsing (stub)
├── nlp-processor/   Medical entity extraction (keyword/regex)
├── health-os-types/ Domain newtypes with validation
└── telemetry/       OpenTelemetry + Jaeger tracing

services/            Microservices
├── api-gateway/           Port 8080 — single entry point, auth, routing
├── timeline-service/      Port 8081 — event store, timeline computation
├── document-processor/    Port 8082 — document upload, OCR, DICOM, NLP, manual entry
├── ai-report-service/     Port 8083 — AI report generation (stub)
├── doctor-access-service/ Port 8084 — QR-based temporary doctor access
├── patient-management/    Port 8085 — hospital patient CRUD, encounters, vitals, allergies, medications
├── appointment-scheduling/ Appointment lifecycle, availability, reminders
├── billing-invoicing/     Invoice generation, payments, insurance claims
└── licence-management/    RSA-signed licence keys, hardware fingerprinting, usage tracking
```

## Dependencies

- Rust 1.75+
- PostgreSQL 15+
- NATS
- Redis
- Docker (for infrastructure)

## Quick Start

```bash
# Start infrastructure
docker-compose up postgres redis nats -d

# Run database migrations
cargo install sqlx-cli --no-default-features --features postgres
sqlx migrate run --database-url "postgres://postgres:postgres@localhost:5432/health_os"

# Start services
cargo run --package api-gateway &
cargo run --package timeline-service &
cargo run --package document-processor &
cargo run --package doctor-access-service &
cargo run --package patient-management &
```

## Architecture

Event-sourced medical records with append-only `medical_events` table (JSONB payload). Timeline reconstruction via `timeline-engine` crate. Services communicate through NATS for async event processing.

9 shared crates provide reusable domain logic. 9 microservices expose HTTP APIs via Axum. All services use `telemetry` crate for OpenTelemetry tracing to Jaeger.

## Documentation

- `docs/index.html` — interactive architecture diagrams (Mermaid)
- `ARCHITECTURE.md` — system architecture
- `HMS_ARCHITECTURE.md` — hospital management system architecture
