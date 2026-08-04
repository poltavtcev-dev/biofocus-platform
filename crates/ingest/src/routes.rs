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
    /// Bounded Observation ingress (persist → P2-E1-T3).
    pub tx: ObservationSender,
}

/// Success body for `POST /v1/ingest` (`docs/09-api.md`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IngestResponse {
    /// Always `"queued"` on 202 for this skeleton.
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

/// Builds the ingest router (`POST /v1/ingest` only in T1).
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

    let count = observations.len();
    for observation in observations {
        if let Err(err) = state.tx.try_send(observation) {
            match err {
                tokio::sync::mpsc::error::TrySendError::Full(_) => {
                    warn!("ingest channel full; rejecting batch");
                    return (
                        StatusCode::SERVICE_UNAVAILABLE,
                        Json(error_body("queue_full")),
                    )
                        .into_response();
                }
                tokio::sync::mpsc::error::TrySendError::Closed(_) => {
                    warn!("ingest channel closed");
                    return (
                        StatusCode::SERVICE_UNAVAILABLE,
                        Json(error_body("queue_closed")),
                    )
                        .into_response();
                }
            }
        }
    }

    (
        StatusCode::ACCEPTED,
        Json(IngestResponse::queued(count)),
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
