//! Axum routes for local ingest.

use std::sync::Arc;

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::post;
use axum::{Json, Router};
use bio_spec::Observation;
use runtime::ObservationSender;
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::auth::{bearer_token, tokens_equal};

/// Shared state for ingest handlers.
#[derive(Clone)]
pub struct IngestState {
    /// Expected Bearer token.
    pub token: Arc<str>,
    /// Bounded Observation ingress (drained by persist worker).
    pub tx: ObservationSender,
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

/// Builds the ingest router (`POST /v1/ingest`).
#[must_use]
pub fn ingest_router(state: IngestState) -> Router {
    Router::new()
        .route("/v1/ingest", post(post_ingest))
        .with_state(state)
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
