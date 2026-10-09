//! «Источники данных» — Tauri read of wearable rows plus priority order.
//!
//! UI never opens SQLite. Counts come from [`pipeline::summarize_sources`].

use std::time::{SystemTime, UNIX_EPOCH};

use bio_spec::{UnixTimestamp, WEARABLE_SUMMARY_TYPES};
use pipeline::{SourcePriority, SourceSample, SourcesReport, summarize_sources};
use serde::Serialize;
use storage::{Database, ObservationCreated, ObservationRepository};

/// One type's rolling counts (camelCase).
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TypeCountDto {
    pub data_type: String,
    pub last_24h: u32,
    pub last_7d: u32,
    pub last_30d: u32,
}

/// One source card.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SourceCardDto {
    pub kind: String,
    pub app: String,
    pub device_model: String,
    pub last_sample_unix: i64,
    pub last_received_unix: i64,
    pub counts: Vec<TypeCountDto>,
    pub coverage: f64,
    pub priority_rank: usize,
    pub active: bool,
}

/// Companion progress posted to `POST /v1/companion/status`. No health values.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CompanionSyncDto {
    pub phase: String,
    pub types_ok: u32,
    pub types_empty: u32,
    pub types_total: u32,
    pub pending: u32,
    pub received_at: i64,
}

/// Payload for `get_data_sources`.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DataSourcesDto {
    pub sources: Vec<SourceCardDto>,
    pub priority: Vec<String>,
    pub hints: Vec<String>,
    pub companion: Option<CompanionSyncDto>,
    /// Display zone for sample and receipt times.
    pub timezone: String,
}

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs() as i64)
        .unwrap_or(0)
}

fn sample_from(row: &ObservationCreated) -> Option<SourceSample> {
    let src = row.observation.payload.get("src")?;
    let kind = src.get("kind")?.as_str()?.to_owned();
    if !bio_spec::is_src_kind(&kind) {
        return None;
    }
    let app = src
        .get("app_name")
        .and_then(|value| value.as_str())
        .or_else(|| src.get("bundle_id").and_then(|value| value.as_str()))
        .unwrap_or("unknown")
        .to_owned();
    let device_model = src
        .get("device_model")
        .and_then(|value| value.as_str())
        .unwrap_or("")
        .to_owned();
    let sleep_stage = row
        .observation
        .payload
        .get("stage")
        .and_then(|value| value.as_str())
        .map(str::to_owned);
    Some(SourceSample {
        data_type: row.observation.data_type.clone(),
        timestamp: row.observation.timestamp.as_secs(),
        received_at: row.created_at,
        kind,
        app,
        device_model,
        sleep_stage,
    })
}

fn report_to_dto(report: SourcesReport, companion: Option<CompanionSyncDto>) -> DataSourcesDto {
    DataSourcesDto {
        sources: report
            .sources
            .into_iter()
            .map(|source| SourceCardDto {
                kind: source.kind,
                app: source.app,
                device_model: source.device_model,
                last_sample_unix: source.last_sample_unix,
                last_received_unix: source.last_received_unix,
                counts: source
                    .counts
                    .into_iter()
                    .map(|count| TypeCountDto {
                        data_type: count.data_type,
                        last_24h: count.last_24h,
                        last_7d: count.last_7d,
                        last_30d: count.last_30d,
                    })
                    .collect(),
                coverage: source.coverage,
                priority_rank: source.priority_rank,
                active: source.active,
            })
            .collect(),
        priority: report.priority,
        hints: report.hints,
        companion,
        timezone: "Europe/Belgrade".to_owned(),
    }
}

fn companion_dto() -> Option<CompanionSyncDto> {
    let status = ingest::published_companion_status()?;
    Some(CompanionSyncDto {
        phase: status.phase,
        types_ok: status.types_ok,
        types_empty: status.types_empty,
        types_total: status.types_total,
        pending: status.pending,
        received_at: status.received_at,
    })
}

fn load_report(now: i64) -> SourcesReport {
    let priority = SourcePriority::load_installed();
    let Ok(path) = storage::default_db_path() else {
        return summarize_sources(&[], now, &priority);
    };
    let Ok(db) = Database::open(&path) else {
        return summarize_sources(&[], now, &priority);
    };
    let repo = ObservationRepository::new(&db);
    let start = now.saturating_sub(30 * 86_400);
    let Ok(rows) = repo.list_with_created_in_range(
        WEARABLE_SUMMARY_TYPES,
        UnixTimestamp::from_secs(start),
        UnixTimestamp::from_secs(now),
    ) else {
        return summarize_sources(&[], now, &priority);
    };
    let samples: Vec<SourceSample> = rows.iter().filter_map(sample_from).collect();
    summarize_sources(&samples, now, &priority)
}

/// Latest 30 days of wearable sources. Soft-fails to an empty list.
pub fn get_data_sources() -> DataSourcesDto {
    report_to_dto(load_report(now_secs()), companion_dto())
}

/// Replaces `source-priority.toml`. Unknown names are dropped; omitted defaults stay.
pub fn set_source_priority(order: Vec<String>) -> Result<DataSourcesDto, String> {
    let priority = SourcePriority::from_order(order);
    let path = SourcePriority::installed_path()
        .ok_or_else(|| "Could not save source order.".to_owned())?;
    priority
        .write_to(&path)
        .map_err(|_| "Could not save source order.".to_owned())?;
    Ok(get_data_sources())
}
