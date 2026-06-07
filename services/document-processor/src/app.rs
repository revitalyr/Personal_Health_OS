use storage::{PostgresEventStore, DatabaseConfig};
use std::sync::Arc;
use crate::{services::DocumentService, nats::NatsClient, storage::DocumentStorage};
use auth::FullAuthService;
use sqlx::PgPool;

#[derive(Clone)]
pub struct App {
    #[allow(dead_code)]
    pub event_store: Arc<dyn storage::EventStore>,
    pub document_service: Arc<DocumentService>,
    #[allow(dead_code)]
    pub nats_client: Arc<NatsClient>,
    pub document_storage: Arc<DocumentStorage>,
    pub auth_service: Arc<FullAuthService>,
}

impl App {
    pub async fn build(config: &crate::config::Config) -> anyhow::Result<Self> {
        let db_config = DatabaseConfig {
            host: "localhost".to_string(),
            port: 5432,
            database: "health_os".to_string(),
            username: "postgres".to_string(),
            password: "postgres".to_string(),
            max_connections: 10,
        };

        let pg_store = PostgresEventStore::new(db_config).await?;
        let pool = pg_store.pool().clone();
        let event_store: Arc<dyn storage::EventStore> = Arc::new(pg_store);

        let nats_client = Arc::new(NatsClient::new(&config.nats_url).await?);

        let document_storage = Arc::new(DocumentStorage::new(config.clone()).await.map_err(|e| anyhow::anyhow!("{:?}", e))?);

        let ocr_processor: Arc<dyn crate::services::OcrProcessing> = Arc::new(ocr_processor::OcrProcessor);
        let dicom_processor: Arc<dyn crate::services::DicomProcessing> = Arc::new(dicom_processor::DicomProcessor);
        let nlp_extractor: Arc<dyn crate::services::NlpProcessing> = Arc::new(nlp_processor::MedicalEntityExtractor);

        let document_service = Arc::new(DocumentService::new(
            event_store.clone(),
            pool,
            nats_client.clone(),
            document_storage.clone(),
            config.clone(),
            ocr_processor,
            dicom_processor,
            nlp_extractor,
        ));

        let db = PgPool::connect(&config.database_url).await?;
        let auth_service = Arc::new(FullAuthService::new(
            db,
            config.jwt_secret.clone(),
            "health_os".to_string(),
            "health_os_api".to_string(),
        )?);

        tracing::info!("Document Processor application initialized");

        Ok(Self {
            event_store,
            document_service,
            nats_client,
            document_storage,
            auth_service,
        })
    }
}
