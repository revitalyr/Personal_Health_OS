use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::Value;
use timeline_engine::{TimelineFilter, ExportFormat};
use uuid::Uuid;

use crate::{app::App, trace_request, trace_response};

#[derive(Debug, Deserialize)]
pub struct TimelineQuery {
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub event_types: Option<String>,
    pub limit: Option<usize>,
    pub page: Option<usize>,
}

pub async fn get_timeline(
    State(app): State<App>,
    Path(patient_id): Path<Uuid>,
    Query(params): Query<TimelineQuery>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("GET", format!("/patients/{}/timeline", patient_id));
    
    // Parse event types filter
    let event_types = params.event_types
        .map(|types| types.split(',').map(|s| s.trim().to_string()).collect::<Vec<_>>());

    // Build filter
    let filter = TimelineFilter {
        start_date: params.start_date,
        end_date: params.end_date,
        event_types: None, // TODO: Convert string to EventType enum
        limit: params.limit,
    };

    // Get events from storage
    let events = app.event_store
        .get_events_by_patient_with_filter(
            patient_id,
            filter.start_date,
            filter.end_date,
            filter.limit.map(|l| l as i64),
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to get events: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Build timeline
    let timeline = timeline_engine::TimelineEngine::build_filtered_timeline(events, filter)
        .map_err(|e| {
            tracing::error!("Failed to build timeline: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Generate summary
    let summary = timeline_engine::TimelineEngine::generate_summary(&timeline);

    let response = serde_json::json!({
        "patient_id": patient_id,
        "timeline": {
            "events": timeline.events,
            "total_events": timeline.events.len(),
            "generated_at": timeline.generated_at,
        },
        "summary": {
            "total_events": summary.total_events,
            "date_range": {
                "start": summary.date_range.0,
                "end": summary.date_range.1,
            },
            "event_types": summary.event_types,
            "recent_events_count": summary.recent_events.len(),
        },
        "filter": params,
    });

    trace_response!(StatusCode::OK, std::time::Duration::from_millis(25));
    Ok(Json(response))
}

pub async fn get_summary(
    State(app): State<App>,
    Path(patient_id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("GET", format!("/patients/{}/timeline/summary", patient_id));
    
    // Get all events for the patient
    let events = app.event_store.get_events_by_patient(patient_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get events: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Build timeline
    let timeline = timeline_engine::TimelineEngine::build_timeline(events);

    // Generate detailed summary
    let summary = timeline_engine::TimelineEngine::generate_summary(&timeline);
    let anomalies = timeline_engine::TimelineEngine::detect_anomalies(&timeline);

    // Get patient stats
    let stats = app.event_store.get_patient_stats(patient_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get patient stats: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let response = serde_json::json!({
        "patient_id": patient_id,
        "summary": {
            "total_events": summary.total_events,
            "date_range": {
                "start": summary.date_range.0,
                "end": summary.date_range.1,
            },
            "event_types": summary.event_types,
            "recent_events": summary.recent_events,
        },
        "statistics": {
            "total_events": stats.total_events,
            "unique_event_types": stats.unique_event_types,
            "first_event": stats.first_event,
            "last_event": stats.last_event,
        },
        "anomalies": {
            "count": anomalies.len(),
            "items": anomalies,
        },
        "generated_at": chrono::Utc::now(),
    });

    trace_response!(StatusCode::OK, std::time::Duration::from_millis(20));
    Ok(Json(response))
}

pub async fn export_timeline(
    State(app): State<App>,
    Path(patient_id): Path<Uuid>,
    Query(params): Query<TimelineQuery>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("GET", format!("/patients/{}/timeline/export", patient_id));
    
    // Parse export format
    let format_str = params.event_types.as_ref().and_then(|types| types.split(',').next());
    let export_format = match format_str {
        Some("json") => ExportFormat::Json,
        Some("text") => ExportFormat::Text,
        Some("csv") => ExportFormat::Csv,
        _ => ExportFormat::Json,
    };

    // Get events from storage
    let events = app.event_store.get_events_by_patient(patient_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get events: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Build timeline
    let timeline = timeline_engine::TimelineEngine::build_timeline(events);

    // Export timeline
    let exported_data = timeline_engine::TimelineEngine::export_timeline(&timeline, export_format)
        .map_err(|e| {
            tracing::error!("Failed to export timeline: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let response = serde_json::json!({
        "patient_id": patient_id,
        "format": format!("{:?}", export_format),
        "data": exported_data,
        "exported_at": chrono::Utc::now(),
        "events_count": timeline.events.len(),
    });

    trace_response!(StatusCode::OK, std::time::Duration::from_millis(50));
    Ok(Json(response))
}

pub async fn get_anomalies(
    State(app): State<App>,
    Path(patient_id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("GET", format!("/patients/{}/anomalies", patient_id));
    
    // Get all events for the patient
    let events = app.event_store.get_events_by_patient(patient_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get events: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Build timeline
    let timeline = timeline_engine::TimelineEngine::build_timeline(events);

    // Detect anomalies
    let anomalies = timeline_engine::TimelineEngine::detect_anomalies(&timeline);

    let response = serde_json::json!({
        "patient_id": patient_id,
        "anomalies": anomalies,
        "total_anomalies": anomalies.len(),
        "analyzed_at": chrono::Utc::now(),
        "events_analyzed": timeline.events.len(),
    });

    trace_response!(StatusCode::OK, std::time::Duration::from_millis(30));
    Ok(Json(response))
}
