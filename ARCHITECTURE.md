# Health OS Architecture

## System Overview

Health OS is designed as a high-performance, event-driven system for managing personal medical records. The architecture prioritizes:

- **Data integrity** through event sourcing
- **Scalability** via microservices and message queues
- **Security** with comprehensive access controls
- **Performance** with optimized algorithms and caching

## Core Architectural Patterns

### 1. Event Sourcing

All medical data is stored as immutable events in the `medical_events` table. This provides:

- **Complete audit trail**: Every change is recorded
- **Temporal queries**: Reconstruct state at any point in time
- **Data recovery**: Replay events to rebuild state
- **Analytics**: Rich event-based analysis capabilities

```sql
CREATE TABLE medical_events (
    id UUID PRIMARY KEY,
    patient_id UUID NOT NULL,
    event_type VARCHAR(50) NOT NULL,
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
    payload JSONB NOT NULL,
    source VARCHAR(100) NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
```

### 2. CQRS (Command Query Responsibility Segregation)

- **Commands**: Write operations that modify state (create events)
- **Queries**: Read operations that project state (build timelines)

This separation allows for independent scaling of read and write operations.

### 3. Microservices Architecture

```
┌─────────────────┐    ┌─────────────────┐
│   Mobile App    │    │   Web Client    │
└─────────┬───────┘    └─────────┬───────┘
          │                      │
          └──────────┬───────────┘
                     │
          ┌─────────────────┐
          │  API Gateway    │
          │  (Port 8080)    │
          └─────────┬───────┘
                    │
          ┌─────────────────┐
          │  Event Bus      │
          │  (NATS)         │
          └─────────┬───────┘
     ┌──────────┼──────────┬──────────┐
     ▼          ▼          ▼          ▼
┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐
│Timeline │ │Document │ │ AI      │ │Doctor   │
│Service  │ │Processor│ │Service  │ │Access   │
│(8081)   │ │(8082)   │ │(8083)   │ │(8084)   │
└─────────┘ └─────────┘ └─────────┘ └─────────┘
```

## Service Details

### API Gateway (Port 8080)

**Responsibilities:**
- HTTP request routing and load balancing
- Authentication and authorization
- Request validation and rate limiting
- CORS handling
- API versioning

**Key Features:**
- JWT token generation and validation
- Request/response transformation
- Circuit breaker for downstream services
- Comprehensive logging and tracing

**Technology Stack:**
- Rust + Axum web framework
- JWT authentication
- OpenTelemetry tracing
- Tower middleware

### Timeline Service (Port 8081)

**Responsibilities:**
- Event aggregation and timeline building
- Timeline filtering and pagination
- Anomaly detection
- Performance optimization

**Core Algorithm:**
```rust
pub fn build_timeline(events: Vec<MedicalEvent>) -> Timeline {
    let mut sorted_events = events;
    sorted_events.sort_by_key(|e| e.timestamp); // O(n log n)
    Timeline::new(sorted_events)
}
```

**Performance Characteristics:**
- 1M events sorted in <30ms
- Memory-efficient streaming for large datasets
- Parallel processing for multiple patients

**Technology Stack:**
- Rust + Tokio async runtime
- PostgreSQL with optimized queries
- Redis for caching frequent timelines

### Document Processor (Port 8082)

**Responsibilities:**
- File upload and storage
- OCR processing with Tesseract
- Medical entity extraction
- Document classification

**Processing Pipeline:**
```
Upload → Validation → Storage → OCR → Extraction → Indexing
```

**Supported Formats:**
- PDF documents
- Medical images (X-ray, MRI, DICOM)
- Prescriptions and lab reports
- Handwritten notes

**Technology Stack:**
- Tesseract OCR engine
- AWS S3/Cloudflare R2 storage
- Parallel processing workers
- Medical NLP models

### AI Report Service (Port 8083)

**Responsibilities:**
- LLM integration for report generation
- Context building from patient data
- Report template management
- Quality assurance and validation

**AI Pipeline:**
```
Patient Data → Context Building → LLM Prompt → Report Generation → Validation
```

**Supported Models:**
- OpenAI GPT-4/GPT-3.5
- Local models via Ollama (Llama, Mistral)
- Custom fine-tuned medical models

**RAG Implementation:**
- Vector embeddings for medical documents
- Semantic search for relevant context
- Retrieval-augmented generation

### Doctor Access Service (Port 8084)

**Responsibilities:**
- Temporary access token generation
- QR code creation
- Access logging and audit
- Token revocation

**Security Features:**
- 15-minute token expiration
- Single-use or limited-use tokens
- IP-based restrictions
- Comprehensive audit trail

## Data Flow

### 1. Event Creation Flow

```
Client → API Gateway → Event Bus → Timeline Service → Database
```

1. Client submits medical event via API
2. API Gateway validates authentication
3. Event is published to NATS
4. Timeline Service processes and stores event
5. Database stores immutable event record

### 2. Timeline Query Flow

```
Client → API Gateway → Timeline Service → Cache/Database → Response
```

1. Client requests patient timeline
2. Timeline Service checks cache first
3. If cache miss, queries database
4. Events are sorted and filtered
5. Timeline is cached and returned

### 3. Document Processing Flow

```
Client → API Gateway → Document Processor → Storage → OCR → Extraction → Database
```

1. Client uploads document
2. Document is stored in object storage
3. OCR processing extracts text
4. Medical entities are identified
5. Structured data is stored

### 4. AI Report Generation Flow

```
Client → API Gateway → AI Service → Context Builder → LLM → Report → Database
```

1. Client requests AI report
2. Patient data is aggregated
3. Context is built for LLM
4. Report is generated
5. Report is stored and returned

## Security Architecture

### Authentication & Authorization

- **JWT Tokens**: 24-hour expiration with refresh tokens
- **Role-Based Access**: Patient, doctor, admin roles
- **Scope-Based Access**: Limited data access per role
- **Token Revocation**: Immediate token invalidation

### Data Protection

- **Encryption at Rest**: PostgreSQL TDE, encrypted storage
- **Encryption in Transit**: TLS 1.3 for all communications
- **Data Masking**: Sensitive data redaction in logs
- **Access Logging**: Complete audit trail

### Compliance

- **HIPAA**: Healthcare data protection standards
- **GDPR**: EU data protection regulations
- **Data Retention**: Configurable retention policies
- **Right to Deletion**: Complete data removal

## Performance Architecture

### Caching Strategy

- **Redis**: Session data, frequent timelines
- **Application Cache**: In-memory for hot data
- **CDN**: Static assets and document previews
- **Database Query Cache**: Frequently accessed queries

### Database Optimization

- **Indexing Strategy**: Optimized for timeline queries
- **Partitioning**: By patient_id for large datasets
- **Connection Pooling**: Efficient resource utilization
- **Read Replicas**: Query load distribution

### Scalability Patterns

- **Horizontal Scaling**: Stateless services
- **Event-Driven Decoupling**: NATS message bus
- **Circuit Breakers**: Fault tolerance
- **Rate Limiting**: Protection against abuse

## Monitoring & Observability

### Tracing

- **OpenTelemetry**: Distributed tracing
- **Jaeger**: Trace visualization
- **Correlation IDs**: Request tracking across services
- **Span Context**: Operation timing analysis

### Metrics

- **Prometheus**: Metrics collection
- **Grafana**: Visualization dashboards
- **Custom Metrics**: Business and performance KPIs
- **Alerting**: Proactive issue detection

### Logging

- **Structured Logging**: JSON format with correlation
- **Log Levels**: Debug, Info, Warn, Error
- **Log Aggregation**: Centralized log management
- **Security Events**: Separate security audit log

## Deployment Architecture

### Container Strategy

- **Multi-stage Builds**: Optimized Docker images
- **Resource Limits**: CPU and memory constraints
- **Health Checks**: Container readiness and liveness
- **Graceful Shutdown**: Zero-downtime deployments

### Orchestration

- **Kubernetes**: Container orchestration
- **Helm Charts**: Deployment templates
- **ConfigMaps**: Configuration management
- **Secrets Management**: Secure credential storage

### Infrastructure

- **Cloud Provider**: AWS/GCP/Azure
- **Database**: Managed PostgreSQL with HA
- **Storage**: Object storage with lifecycle policies
- **Networking**: VPC with security groups

## Development Architecture

### Code Organization

```
services/          # Microservices
├── api-gateway/
├── timeline-service/
├── document-processor/
├── ai-report-service/
└── doctor-access-service/

crates/            # Shared libraries
├── event-model/
├── timeline-engine/
├── auth/
├── storage/
└── telemetry/
```

### Development Workflow

- **Local Development**: Docker Compose with all services
- **Feature Branches**: Git flow with pull requests
- **CI/CD Pipeline**: Automated testing and deployment
- **Code Quality**: Automated linting and security scanning

### Testing Strategy

- **Unit Tests**: Individual component testing
- **Integration Tests**: Service interaction testing
- **E2E Tests**: Complete workflow testing
- **Performance Tests**: Load and stress testing

## Future Architecture Considerations

### Scalability Enhancements

- **Event Sourcing Evolution**: Kafka for high-throughput events
- **CQRS Evolution**: Separate read/write databases
- **Microservice Evolution**: Further service decomposition
- **Data Evolution**: Time-series databases for analytics

### Technology Evolution

- **AI Enhancement**: Custom medical language models
- **Real-time Features**: WebSocket for live updates
- **Mobile Enhancement**: Offline-first mobile clients
- **Integration Enhancement**: EHR system connections

### Security Evolution

- **Zero Trust**: Enhanced security model
- **Blockchain**: Immutable audit trails
- **Homomorphic Encryption**: Privacy-preserving computation
- **Multi-Party Computation**: Secure data sharing
