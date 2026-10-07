//! Axum routes for local ingest.

use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::sync::Arc;

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use bio_spec::{validate_observation_payload, Observation};
use runtime::ObservationSender;
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::advertise::AdvertiseInfo;
use crate::auth::{bearer_token, tokens_equal};
use crate::config::{DEFAULT_INGEST_PORT, INGEST_BIND_HOST};
use crate::status::{probe_db_at, StatusResponse};

/// Soft-fail DB probe used by `GET /v1/status`.
pub type DbProbe = Arc<dyn Fn() -> Result<(), String> + Send + Sync>;

/// Shared state for ingest handlers.
#[derive(Clone)]
pub struct IngestState {
    /// Expected Bearer token.
    pub token: Arc<str>,
    /// Bounded Observation ingress (drained by persist worker).
    pub tx: ObservationSender,
    /// Version string returned by `GET /v1/status`.
    pub version: Arc<str>,
    /// Soft-fail DB probe for status (no Observation payload).
    pub db_probe: DbProbe,
    /// Bind mode + base URL hints for `GET /v1/status` (P5-E1-T2).
    pub advertise: AdvertiseInfo,
    /// Actual bind (host, port) when known — hints are re-derived per request so
    /// a Wi-Fi change after startup is picked up without restarting.
    pub bind: Option<(Ipv4Addr, u16)>,
}

impl IngestState {
    /// Builds state with crate version, loopback advertise, and an always-ok DB probe.
    #[must_use]
    pub fn new(token: impl Into<String>, tx: ObservationSender) -> Self {
        Self {
            token: Arc::from(token.into()),
            tx,
            version: Arc::from(env!("CARGO_PKG_VERSION")),
            db_probe: Arc::new(|| Ok(())),
            advertise: AdvertiseInfo::for_bind(INGEST_BIND_HOST, DEFAULT_INGEST_PORT),
            bind: None,
        }
    }

    /// Overrides the version string exposed by `GET /v1/status`.
    #[must_use]
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Arc::from(version.into());
        self
    }

    /// Overrides advertise hints (bind mode + base URLs).
    #[must_use]
    pub fn with_advertise(mut self, advertise: AdvertiseInfo) -> Self {
        self.advertise = advertise;
        self
    }

    /// Sets advertise from bind host + port (derives hints on call; no spin).
    #[must_use]
    pub fn with_bind(mut self, bind_host: Ipv4Addr, port: u16) -> Self {
        self.bind = Some((bind_host, port));
        self.with_advertise(AdvertiseInfo::for_bind(bind_host, port))
    }

    /// Overrides the soft-fail DB probe.
    #[must_use]
    pub fn with_db_probe(
        mut self,
        probe: impl Fn() -> Result<(), String> + Send + Sync + 'static,
    ) -> Self {
        self.db_probe = Arc::new(probe);
        self
    }

    /// Probes by opening `path` (WAL + migrate-on-open). Soft-fail via `Err(String)`.
    #[must_use]
    pub fn with_db_path(self, path: PathBuf) -> Self {
        self.with_db_probe(move || probe_db_at(&path))
    }
}

/// Success body for `POST /v1/ingest` (`docs/09-api.md`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IngestResponse {
    /// Always `"queued"` on 202.
    pub status: String,
    /// Number of Observations accepted into the channel.
    pub count: usize,
}

impl IngestResponse {
    #[must_use]
    pub fn queued(count: usize) -> Self {
        Self {
            status: "queued".to_owned(),
            count,
        }
    }
}

/// Backpressure body when the bounded channel cannot accept the full batch.
///
/// **Mid-batch contract (option C):** Observations already `try_send`'d stay in
/// the channel; further items in this request are not enqueued. HTTP status is
/// always `503`. See `docs/09-api.md`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QueuePressureBody {
    /// `"queue_full"` or `"queue_closed"`.
    pub error: String,
    /// Items from this request that were enqueued before pressure stopped the loop.
    pub accepted: usize,
    /// Remaining items in this request that were not enqueued.
    pub rejected: usize,
}

impl QueuePressureBody {
    #[must_use]
    pub fn queue_full(accepted: usize, rejected: usize) -> Self {
        Self {
            error: "queue_full".to_owned(),
            accepted,
            rejected,
        }
    }

    #[must_use]
    pub fn queue_closed(accepted: usize, rejected: usize) -> Self {
        Self {
            error: "queue_closed".to_owned(),
            accepted,
            rejected,
        }
    }
}

/// Builds the ingest router (`POST /v1/ingest`, `GET /v1/status`).
#[must_use]
pub fn ingest_router(state: IngestState) -> Router {
    Router::new()
        .route("/v1/ingest", post(post_ingest))
        .route("/v1/status", get(get_status))
        .with_state(state)
}

async fn get_status(State(state): State<IngestState>) -> impl IntoResponse {
    let probe = (state.db_probe)();
    let advertise = match state.bind {
        // LAN wildcard bind: re-detect the address (cheap, no spin) on each status call.
        Some((host, port)) if host.is_unspecified() => AdvertiseInfo::for_bind(host, port),
        _ => state.advertise.clone(),
    };
    let body = StatusResponse::from_probe(state.version.as_ref(), probe, &advertise);
    (StatusCode::OK, Json(body))
}

async fn post_ingest(
    State(state): State<IngestState>,
    headers: HeaderMap,
    body: Result<Json<Vec<Observation>>, axum::extract::rejection::JsonRejection>,
) -> impl IntoResponse {
    let Some(provided) = bearer_token(&headers) else {
        return (StatusCode::UNAUTHORIZED, Json(error_body("unauthorized"))).into_response();
    };
    if !tokens_equal(state.token.as_ref(), provided) {
        return (StatusCode::UNAUTHORIZED, Json(error_body("unauthorized"))).into_response();
    }

    let observations = match body {
        Ok(Json(items)) => items,
        Err(rejection) => {
            warn!(error = %rejection, "ingest rejected invalid JSON body");
            return (
                StatusCode::BAD_REQUEST,
                Json(error_body("invalid_json")),
            )
                .into_response();
        }
    };

    for observation in &observations {
        if let Err(err) = validate_observation_payload(observation) {
            warn!(error = %err, data_type = %observation.data_type, "ingest rejected Observation payload");
            let code = match &err {
                bio_spec::SpecError::InvalidCalendarEventPayload { .. } => {
                    "invalid_calendar_event"
                }
                bio_spec::SpecError::InvalidBrowserCategoryPayload { .. } => {
                    "invalid_browser_category"
                }
                bio_spec::SpecError::InvalidNowPlayingPayload { .. } => "invalid_now_playing",
                bio_spec::SpecError::InvalidGitActivityPayload { .. } => "invalid_git_activity",
                bio_spec::SpecError::InvalidAmbientLightPayload { .. } => "invalid_ambient_light",
                bio_spec::SpecError::InvalidNotificationEventPayload { .. } => {
                    "invalid_notification_event"
                }
                bio_spec::SpecError::InvalidStepCountPayload { .. } => "invalid_step_count",
                bio_spec::SpecError::InvalidActiveEnergyPayload { .. } => "invalid_active_energy",
                bio_spec::SpecError::InvalidSleepIntervalPayload { .. } => "invalid_sleep_interval",
                bio_spec::SpecError::InvalidOxygenSaturationPayload { .. } => {
                    "invalid_oxygen_saturation"
                }
                _ => "invalid_life_event",
            };
            return (StatusCode::BAD_REQUEST, Json(error_body(code))).into_response();
        }
    }

    let total = observations.len();
    let mut accepted = 0usize;

    for observation in observations {
        if let Err(err) = state.tx.try_send(observation) {
            let rejected = total.saturating_sub(accepted);
            match err {
                tokio::sync::mpsc::error::TrySendError::Full(_) => {
                    warn!(
                        accepted,
                        rejected, "ingest channel full; stopping further enqueue (mid-batch contract C)"
                    );
                    return (
                        StatusCode::SERVICE_UNAVAILABLE,
                        Json(QueuePressureBody::queue_full(accepted, rejected)),
                    )
                        .into_response();
                }
                tokio::sync::mpsc::error::TrySendError::Closed(_) => {
                    warn!(
                        accepted,
                        rejected, "ingest channel closed; stopping further enqueue"
                    );
                    return (
                        StatusCode::SERVICE_UNAVAILABLE,
                        Json(QueuePressureBody::queue_closed(accepted, rejected)),
                    )
                        .into_response();
                }
            }
        }
        accepted += 1;
    }

    (
        StatusCode::ACCEPTED,
        Json(IngestResponse::queued(accepted)),
    )
        .into_response()
}

#[derive(Serialize)]
struct ErrorBody {
    error: &'static str,
}

const fn error_body(error: &'static str) -> ErrorBody {
    ErrorBody { error }
}
