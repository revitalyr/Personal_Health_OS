#!/bin/bash

# Health OS API Demo Script
# This script demonstrates all the new data input capabilities

BASE_URL="http://localhost:8080"
PATIENT_ID="550e8400-e29b-41d4-a716-446655440000"

echo "🏥 Health OS API Demo"
echo "===================="
echo ""

# Health check
echo "📊 Health Check"
curl -s "$BASE_URL/health" | jq '.'
echo ""

# Create a patient timeline first (this will create empty timeline)
echo "📋 Creating Patient Timeline"
TIMELINE_RESPONSE=$(curl -s -X GET "$BASE_URL/patients/$PATIENT_ID/timeline")
echo "$TIMELINE_RESPONSE" | jq '.'
echo ""

# Upload a document with OCR processing
echo "📄 Uploading Document with OCR"
DOCUMENT_RESPONSE=$(curl -s -X POST "$BASE_URL/documents/upload" \
  -H "Content-Type: application/json" \
  -d '{
    "patient_id": "'$PATIENT_ID'",
    "filename": "blood_test.pdf",
    "document_type": "lab_report"
  }')
echo "$DOCUMENT_RESPONSE" | jq '.'
DOCUMENT_ID=$(echo "$DOCUMENT_RESPONSE" | jq -r '.document_id')
echo ""

# Process OCR for the uploaded document
echo "🔍 Processing OCR"
OCR_RESPONSE=$(curl -s -X POST "$BASE_URL/ocr/process/$DOCUMENT_ID" \
  -H "Content-Type: application/json" \
  -d '{"languages": ["rus", "eng"], "extract_entities": true}')
echo "$OCR_RESPONSE" | jq '.'
echo ""

# Add a symptom manually
echo "🤒 Adding Symptom"
SYMPTOM_RESPONSE=$(curl -s -X POST "$BASE_URL/input/symptom" \
  -H "Content-Type: application/json" \
  -d '{
    "patient_id": "'$PATIENT_ID'",
    "name": "Головная боль",
    "severity": 6,
    "description": "Пульсирующая боль в лобной части"
  }')
echo "$SYMPTOM_RESPONSE" | jq '.'
echo ""

# Add a medication manually
echo "💊 Adding Medication"
MEDICATION_RESPONSE=$(curl -s -X POST "$BASE_URL/input/medication" \
  -H "Content-Type: application/json" \
  -d '{
    "patient_id": "'$PATIENT_ID'",
    "name": "Ибупрофен",
    "dosage": "400мг",
    "frequency": "3 раза в день"
  }')
echo "$MEDICATION_RESPONSE" | jq '.'
echo ""

# Upload a DICOM file
echo "🏥 Uploading DICOM File"
DICOM_RESPONSE=$(curl -s -X POST "$BASE_URL/dicom/upload" \
  -H "Content-Type: application/json" \
  -d '{
    "patient_id": "'$PATIENT_ID'",
    "modality": "CT",
    "patient_name": "Иванов Иван",
    "study_date": "20240115"
  }')
echo "$DICOM_RESPONSE" | jq '.'
echo ""

# List all documents
echo "📋 Listing Documents"
DOCUMENTS_RESPONSE=$(curl -s -X GET "$BASE_URL/documents?patient_id=$PATIENT_ID")
echo "$DOCUMENTS_RESPONSE" | jq '.'
echo ""

# Get the updated timeline
echo "📈 Getting Updated Timeline"
TIMELINE_UPDATED=$(curl -s -X GET "$BASE_URL/patients/$PATIENT_ID/timeline")
echo "$TIMELINE_UPDATED" | jq '.'
echo ""

echo "✅ Demo completed!"
echo "📊 Summary:"
echo "   - Document uploaded and processed with OCR"
echo "   - Symptom added manually"
echo "   - Medication added manually"
echo "   - DICOM file uploaded"
echo "   - Timeline shows all events chronologically"
echo ""
echo "🌐 Server is running at: $BASE_URL"
echo "📖 Check the logs to see all API calls"
