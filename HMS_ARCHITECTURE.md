# Hospital Management System Architecture

## Overview

Extension of Personal Health OS with hospital operations: patient management, appointment scheduling, billing, and licence control.

## Services

### Patient Management (port 8085)

- CRUD for hospital patients (extends `user_profiles`)
- Medical encounters (admission, consultation, procedure, surgery, discharge, follow-up, emergency)
- Vitals recording (blood pressure, heart rate, temperature, weight, height, SpO2)
- Allergy management with severity levels (mild, moderate, severe, life-threatening)
- Medication prescriptions with dosage, frequency, route, active/inactive status
- Patient timeline merging encounters + vitals + medications, sorted by timestamp
- Audit trail via `patient_events` table

### Appointment Scheduling

- Doctor availability per day-of-week (start/end time, available flag)
- 30-minute slot generation excluding existing appointments
- Appointment CRUD with conflict detection (overlapping time ranges)
- Status: scheduled, confirmed, in_progress, completed, cancelled, no_show, rescheduled
- Automated reminder scheduling (24h + 2h before appointment)
- 8 appointment types: checkup, consultation, follow-up, emergency, surgery, procedure, vaccination, telemedicine

### Billing & Invoicing

- Invoice with line items, subtotal, tax, discount, total
- Payment tracking (cash, card, insurance, bank_transfer, online, cheque)
- Service charge catalog with categories and tax rates
- Insurance claim processing with provider management
- Aging report (AR buckets: current, 1-30, 31-60, 61-90, 90+ days)
- Invoice status: draft, pending, sent, paid, overdue, cancelled, refunded

### Licence Management

- RSA-signed licence key generation (SHA256 + PKCS1v15Sign)
- Licence validation: active flag, expiry date, hardware fingerprint match
- Usage tracking per action (check, login, feature_access, export, report)
- 4 tiers: Basic (3 features), Professional (5), Enterprise (9), Custom (all)
- CRUD operations: create, suspend, reactivate, reassign

## Database

Tables: `hospital_patients`, `medical_encounters`, `patient_vitals`, `patient_allergies`, `patient_medications`, `patient_events`, `appointments`, `doctor_availability`, `facilities`, `appointment_reminders`, `appointment_events`, `invoices`, `invoice_items`, `payments`, `service_charges`, `insurance_providers`, `insurance_claims`, `licences`, `licence_usage`

pgcrypto PGP encryption applied to PII columns in `hospital_patients`, `facilities`, `insurance_providers`. Secure views provide transparent decryption on read.

## Known Issues

- Licence management, appointment scheduling, billing invoicing have pre-existing compilation errors.
- Notification service and reporting service are not implemented.
