use criterion::{black_box, criterion_group, criterion_main, Criterion};
use event_model::{MedicalEvent, EventType, SymptomPayload};
use timeline_engine::{TimelineEngine, TimelineFilter};
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;

fn generate_test_events(count: usize) -> Vec<MedicalEvent> {
    let patient_id = Uuid::new_v4();
    let mut events = Vec::with_capacity(count);
    
    let base_time = Utc::now();
    
    for i in 0..count {
        let timestamp = base_time - Duration::days((i % 365) as i64) - Duration::hours((i % 24) as i64);
        
        let event_type = match i % 8 {
            0 => EventType::SymptomCreated,
            1 => EventType::MedicationStarted,
            2 => EventType::MedicationStopped,
            3 => EventType::LabResultReceived,
            4 => EventType::DoctorVisit,
            5 => EventType::Diagnosis,
            6 => EventType::DocumentUploaded,
            _ => EventType::ReminderTriggered,
        };
        
        let payload = match event_type {
            EventType::SymptomCreated => serde_json::to_value(SymptomPayload {
                name: format!("Symptom {}", i),
                severity: (i % 10 + 1) as u8,
                description: Some(format!("Description for symptom {}", i)),
                duration: None,
            }).unwrap(),
            _ => serde_json::json!({
                "index": i,
                "data": format!("Sample data for event {}", i)
            }),
        };
        
        let event = MedicalEvent::with_timestamp(
            patient_id,
            event_type,
            payload,
            "benchmark".to_string(),
            timestamp,
        );
        
        events.push(event);
    }
    
    events
}

fn bench_timeline_build_small(c: &mut Criterion) {
    let events = generate_test_events(100);
    
    c.bench_function("timeline_build_100", |b| {
        b.iter(|| TimelineEngine::build_timeline(black_box(events.clone())))
    });
}

fn bench_timeline_build_medium(c: &mut Criterion) {
    let events = generate_test_events(1000);
    
    c.bench_function("timeline_build_1000", |b| {
        b.iter(|| TimelineEngine::build_timeline(black_box(events.clone())))
    });
}

fn bench_timeline_build_large(c: &mut Criterion) {
    let events = generate_test_events(10000);
    
    c.bench_function("timeline_build_10000", |b| {
        b.iter(|| TimelineEngine::build_timeline(black_box(events.clone())))
    });
}

fn bench_timeline_build_xlarge(c: &mut Criterion) {
    let events = generate_test_events(100000);
    
    c.bench_function("timeline_build_100000", |b| {
        b.iter(|| TimelineEngine::build_timeline(black_box(events.clone())))
    });
}

fn bench_timeline_filtering(c: &mut Criterion) {
    let events = generate_test_events(10000);
    let filter = TimelineFilter {
        start_date: Some(Utc::now() - Duration::days(30)),
        end_date: Some(Utc::now()),
        event_types: Some(vec![EventType::SymptomCreated, EventType::MedicationStarted]),
        limit: Some(100),
    };
    
    c.bench_function("timeline_filtering", |b| {
        b.iter(|| TimelineEngine::build_filtered_timeline(black_box(events.clone()), black_box(filter.clone())))
    });
}

fn bench_summary_generation(c: &mut Criterion) {
    let events = generate_test_events(10000);
    let timeline = TimelineEngine::build_timeline(events);
    
    c.bench_function("summary_generation", |b| {
        b.iter(|| TimelineEngine::generate_summary(black_box(&timeline)))
    });
}

fn bench_anomaly_detection(c: &mut Criterion) {
    let events = generate_test_events(10000);
    let timeline = TimelineEngine::build_timeline(events);
    
    c.bench_function("anomaly_detection", |b| {
        b.iter(|| TimelineEngine::detect_anomalies(black_box(&timeline)))
    });
}

fn bench_related_events(c: &mut Criterion) {
    let events = generate_test_events(10000);
    let timeline = TimelineEngine::build_timeline(events);
    let event_id = timeline.events[5000].id;
    
    c.bench_function("find_related_events", |b| {
        b.iter(|| TimelineEngine::find_related_events(black_box(&timeline), black_box(event_id)))
    });
}

fn bench_export_json(c: &mut Criterion) {
    let events = generate_test_events(1000);
    let timeline = TimelineEngine::build_timeline(events);
    
    c.bench_function("export_json", |b| {
        b.iter(|| TimelineEngine::export_timeline(black_box(&timeline), &timeline_engine::ExportFormat::Json))
    });
}

fn bench_export_text(c: &mut Criterion) {
    let events = generate_test_events(1000);
    let timeline = TimelineEngine::build_timeline(events);
    
    c.bench_function("export_text", |b| {
        b.iter(|| TimelineEngine::export_timeline(black_box(&timeline), &timeline_engine::ExportFormat::Text))
    });
}

fn bench_export_csv(c: &mut Criterion) {
    let events = generate_test_events(1000);
    let timeline = TimelineEngine::build_timeline(events);
    
    c.bench_function("export_csv", |b| {
        b.iter(|| TimelineEngine::export_timeline(black_box(&timeline), &timeline_engine::ExportFormat::Csv))
    });
}

criterion_group!(
    benches,
    bench_timeline_build_small,
    bench_timeline_build_medium,
    bench_timeline_build_large,
    bench_timeline_build_xlarge,
    bench_timeline_filtering,
    bench_summary_generation,
    bench_anomaly_detection,
    bench_related_events,
    bench_export_json,
    bench_export_text,
    bench_export_csv
);

criterion_main!(benches);
