# Health OS: Руководство по вводу данных

## 📋 Обзор

Health OS предоставляет удобные и разнообразные способы ввода медицинской информации:

- 📄 **Сканирование документов** с автоматическим OCR
- 🏥 **Импорт DICOM** файлов из медицинских учреждений
- ⌨️ **Ручной ввод** через удобные формы
- 📱 **Быстрый ввод** для частых операций
- 📦 **Пакетная обработка** множества документов

---

## 📄 Сканирование и OCR документов

### Поддерживаемые форматы
- PDF документы
- Изображения (JPG, PNG, TIFF)
- Сканы рецептов и анализов
- Выписки из больниц

### API Пример: Загрузка документа

```bash
curl -X POST http://localhost:8080/documents/upload \
  -H "Authorization: Bearer {token}" \
  -F "file=@/path/to/document.pdf" \
  -F "patient_id=550e8400-e29b-41d4-a716-446655440000"
```

**Ответ:**
```json
{
  "uploaded_files": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440001",
      "filename": "document.pdf",
      "document_type": "lab_report",
      "status": "uploaded",
      "ocr_processing": true
    }
  ],
  "total_files": 1,
  "message": "Documents uploaded successfully"
}
```

### OCR обработка

```bash
# Запустить OCR для конкретного документа
curl -X POST http://localhost:8080/ocr/process/550e8400-e29b-41d4-a716-446655440001 \
  -H "Authorization: Bearer {token}" \
  -H "Content-Type: application/json" \
  -d '{
    "languages": ["rus", "eng"],
    "extract_entities": true,
    "confidence_threshold": 0.7
  }'
```

**Ответ:**
```json
{
  "job_id": "550e8400-e29b-41d4-a716-446655440002",
  "document_id": "550e8400-e29b-41d4-a716-446655440001",
  "status": "processing_started"
}
```

### Проверка статуса OCR

```bash
curl -X GET http://localhost:8080/ocr/status/550e8400-e29b-41d4-a716-446655440002 \
  -H "Authorization: Bearer {token}"
```

### Получение результатов OCR

```bash
curl -X GET http://localhost:8080/ocr/result/550e8400-e29b-41d4-a716-446655440002 \
  -H "Authorization: Bearer {token}"
```

**Ответ с извлеченными данными:**
```json
{
  "job_id": "550e8400-e29b-41d4-a716-446655440002",
  "extracted_text": "Анализ крови\nГемоглобин: 145 г/л\nЭритроциты: 4.5 млн/мкл\n...",
  "confidence_score": 0.92,
  "language_detected": "rus",
  "extracted_entities": [
    {
      "text": "Гемоглобин: 145 г/л",
      "entity_type": "lab_value",
      "confidence": 0.95,
      "normalized_value": "гемоглобин 145 г/л"
    }
  ],
  "pages": [...]
}
```

---

## 🏥 Импорт DICOM файлов

### Поддерживаемые DICOM форматы
- Рентгеновские снимки (X-Ray)
- Компьютерная томография (CT)
- Магнитно-резонансная томография (MRI)
- Ультразвуковые исследования (Ultrasound)

### API Пример: Загрузка DICOM

```bash
curl -X POST http://localhost:8080/dicom/upload \
  -H "Authorization: Bearer {token}" \
  -F "file=@/path/to/scan.dcm" \
  -F "patient_id=550e8400-e29b-41d4-a716-446655440000"
```

**Ответ:**
```json
{
  "uploaded_files": [
    {
      "document_id": "550e8400-e29b-41d4-a716-446655440003",
      "filename": "scan.dcm",
      "status": "uploaded",
      "metadata": {
        "patient_id": "12345",
        "patient_name": "Иванов Иван",
        "study_date": "20240115",
        "modality": "CT",
        "study_description": "Грудная клетка"
      },
      "thumbnail_url": "/preview/550e8400-e29b-41d4-a716-446655440003"
    }
  ]
}
```

### Получение DICOM метаданных

```bash
curl -X GET http://localhost:8080/dicom/550e8400-e29b-41d4-a716-446655440003/metadata \
  -H "Authorization: Bearer {token}"
```

### Просмотр DICOM изображения

```bash
curl -X GET http://localhost:8080/dicom/550e8400-e29b-41d4-a716-446655440003/image \
  -H "Authorization: Bearer {token}"
```

---

## ⌨️ Ручной ввод данных

### Ввод симптомов

**Полная форма:**
```bash
curl -X POST http://localhost:8080/input/symptom \
  -H "Authorization: Bearer {token}" \
  -H "Content-Type: application/json" \
  -d '{
    "patient_id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "Головная боль",
    "severity": 6,
    "description": "Пульсирующая боль в лобной части",
    "duration": "2 часа",
    "location": "голова",
    "started_at": "2024-01-15T10:30:00Z"
  }'
```

**Быстрый ввод:**
```bash
curl -X POST http://localhost:8080/input/quick-symptom \
  -H "Authorization: Bearer {token}" \
  -H "Content-Type: application/json" \
  -d '{
    "patient_id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "Температура",
    "severity": 4
  }'
```

### Ввод лекарств

```bash
curl -X POST http://localhost:8080/input/medication \
  -H "Authorization: Bearer {token}" \
  -H "Content-Type: application/json" \
  -d '{
    "patient_id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "Ибупрофен",
    "dosage": "400мг",
    "frequency": "3 раза в день",
    "start_date": "2024-01-15T08:00:00Z",
    "end_date": "2024-01-17T20:00:00Z",
    "prescribed_by": "Доктор Smith",
    "reason": "Головная боль"
  }'
```

### Ввод результатов анализов

```bash
curl -X POST http://localhost:8080/input/lab-result \
  -H "Authorization: Bearer {token}" \
  -H "Content-Type: application/json" \
  -d '{
    "patient_id": "550e8400-e29b-41d4-a716-446655440000",
    "test_name": "Общий анализ крови",
    "value": "Гемоглобин 145 г/л",
    "unit": "г/л",
    "reference_range": "120-160",
    "status": "normal",
    "facility": "Лаборатория HealthTest",
    "test_date": "2024-01-15",
    "doctor": "Доктор Smith"
  }'
```

### Ввод визита к врачу

```bash
curl -X POST http://localhost:8080/input/doctor-visit \
  -H "Authorization: Bearer {token}" \
  -H "Content-Type: application/json" \
  -d '{
    "patient_id": "550e8400-e29b-41d4-a716-446655440000",
    "doctor_name": "Доктор Smith",
    "specialty": "Терапевт",
    "facility": "Поликлиника №1",
    "visit_date": "2024-01-15T14:00:00Z",
    "reason": "Профилактический осмотр",
    "recommendations": ["Продолжить прием витаминов", "Сдать анализ крови через месяц"],
    "follow_up_date": "2024-02-15T10:00:00Z"
  }'
```

### Ввод диагноза

```bash
curl -X POST http://localhost:8080/input/diagnosis \
  -H "Authorization: Bearer {token}" \
  -H "Content-Type: application/json" \
  -d '{
    "patient_id": "550e8400-e29b-41d4-a716-446655440000",
    "condition": "Грипп",
    "icd10_code": "J11.1",
    "diagnosed_by": "Доктор Smith",
    "diagnosis_date": "2024-01-15",
    "acute": true,
    "treatment_plan": ["Постельный режим", "Обильное питье", "Парацетамол при температуре"]
  }'
```

---

## 📦 Пакетная обработка

### Загрузка множества документов

```bash
curl -X POST http://localhost:8080/documents/upload/batch \
  -H "Authorization: Bearer {token}" \
  -F "files=@document1.pdf" \
  -F "files=@document2.jpg" \
  -F "files=@document3.dcm"
```

### Пакетная OCR обработка

```bash
curl -X POST http://localhost:8080/ocr/process/batch \
  -H "Authorization: Bearer {token}" \
  -H "Content-Type: application/json" \
  -d '{
    "document_ids": [
      "550e8400-e29b-41d4-a716-446655440001",
      "550e8400-e29b-41d4-a716-446655440002",
      "550e8400-e29b-41d4-a716-446655440003"
    ],
    "languages": ["rus", "eng"],
    "extract_entities": true,
    "priority": "normal"
  }'
```

---

## 🔍 Поиск и управление документами

### Поиск документов

```bash
curl -X GET "http://localhost:8080/documents/search?q=анализ крови&patient_id=550e8400-e29b-41d4-a716-446655440000" \
  -H "Authorization: Bearer {token}"
```

### Получение списка документов

```bash
curl -X GET "http://localhost:8080/documents?patient_id=550e8400-e29b-41d4-a716-446655440000&document_type=lab_report&limit=20" \
  -H "Authorization: Bearer {token}"
```

### Получение документа

```bash
curl -X GET http://localhost:8080/documents/550e8400-e29b-41d4-a716-446655440001 \
  -H "Authorization: Bearer {token}"
```

### Получение предпросмотра

```bash
curl -X GET http://localhost:8080/documents/550e8400-e29b-41d4-a716-446655440001/preview \
  -H "Authorization: Bearer {token}"
```

---

## 🎯 Лучшие практики

### OCR обработка
- Используйте высококачественные сканы (300 DPI минимум)
- Убедитесь что текст четкий и не размытый
- Для русских документов используйте язык "rus"
- Устанавливайте порог уверенности 0.7+ для лучших результатов

### DICOM файлы
- Проверьте совместимость формата перед загрузкой
- Убедитесь что метаданные пациента корректны
- Используйте сжатие для больших файлов

### Ручной ввод
- Используйте быстрый ввод для частых операций
- Заполняйте максимально полную информацию
- Используйте стандартизированные названия лекарств и диагнозов

### Пакетная обработка
- Ограничивайте пакеты до 50 файлов для стабильности
- Используйте приоритет "high" для срочных документов
- Мониторьте статус обработки через API

---

## 📱 Мобильное приложение

Мобильное приложение предоставляет удобные интерфейсы для всех этих операций:

- 📸 **Сканирование документов** через камеру
- ⚡ **Быстрый ввод** симптомов и лекарств
- 📋 **Шаблоны** для частых операций
- 🔔 **Уведомления** о завершении OCR
- 📊 **Предпросмотр** документов на лету

---

## 🛠️ Технические детали

### Поддерживаемые форматы файлов
- **Документы**: PDF, DOC, DOCX, TXT
- **Изображения**: JPG, JPEG, PNG, TIFF, BMP
- **Медицинские**: DICOM, DCM
- **Размер**: до 50MB на файл

### OCR языки
- Русский (rus)
- Английский (eng)
- Автоматическое определение языка

### Извлекаемые медицинские сущности
- Лекарства и дозировки
- Симптомы и состояния
- Результаты анализов
- Даты и временные метки
- Диагнозы и процедуры

### Время обработки
- **OCR**: 5-30 секунд на страницу
- **DICOM**: 10-60 секунд на исследование
- **Ручной ввод**: мгновенно
- **Пакетная обработка**: фоновая

---

## 🔗 Связанные разделы

- [API Documentation](./API_SPEC.md)
- [Mobile App Guide](./MOBILE_APP.md)
- [Security Guide](./SECURITY.md)
- [Troubleshooting](./TROUBLESHOOTING.md)

---

## 💡 Советы

1. **Начните с малого**: Попробуйте загрузить один документ перед пакетной обработкой
2. **Проверьте качество**: Убедитесь что сканы четкие для лучших OCR результатов
3. **Используйте шаблоны**: Создайте шаблоны для частых типов документов
4. **Мониторьте статус**: Следите за процессом обработки через API
5. **Регулярно обновляйте**: Загружайте новые документы регулярно для актуальной истории
