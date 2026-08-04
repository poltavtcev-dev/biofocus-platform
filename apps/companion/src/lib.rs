//! Companion client: post `Observation` batches to desktop ingest.
//!
//! Minimal Phase 2 path for HealthKit-shaped `heart_rate` samples. No Feature
//! math, no cloud. Pairing UX (QR/copy) is **P2-E3-T2**.

#![forbid(unsafe_code)]

use std::time::{SystemTime, UNIX_EPOCH};

use bio_spec::{Observation, UnixTimestamp};
use ingest::{IngestResponse, DEFAULT_INGEST_PORT};
use serde_json::json;
use thiserror::Error;
use uuid::Uuid;

/// Matches `docs/07-contracts.md` Apple Health provider.
pub const APPLE_HEALTH_PROVIDER_ID: &str = "com.biofocus.applehealth";

/// Canonical `data_type` for heart-rate Observations.
pub const HEART_RATE_DATA_TYPE: &str = "heart_rate";

/// Default loopback ingest base URL (Desktop host).
pub const DEFAULT_INGEST_BASE_URL: &str = "http://127.0.0.1:8787";

/// Crate identity.
pub const CRATE_NAME: &str = "companion";

/// Failures while building samples or talking to ingest.
#[derive(Debug, Error)]
pub enum CompanionError {
    /// Observation construction failed (invalid confidence, etc.).
    #[error("invalid Observation: {0}")]
    Spec(#[from] bio_spec::SpecError),

    /// HTTP client / transport failure (DNS, connection refused, timeout).
    #[error("network error talking to ingest: {0}")]
    Network(#[from] reqwest::Error),

    /// Ingest rejected the Bearer token (`401`).
    #[error("ingest unauthorized (check pairing token)")]
    Unauthorized,

    /// Non-success HTTP status other than 401.
    #[error("ingest HTTP {status}: {body}")]
    Http { status: u16, body: String },

    /// Response body was not the expected queued JSON.
    #[error("unexpected ingest response body: {0}")]
    BadResponse(String),
}

/// Result alias for companion operations.
pub type CompanionResult<T> = Result<T, CompanionError>;

/// Builds a privacy-safe sample `heart_rate` Observation (bpm only + optional source).
pub fn sample_heart_rate_observation(
    bpm: f64,
    source: Option<&str>,
) -> CompanionResult<Observation> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let mut payload = json!({ "bpm": bpm });
    if let Some(src) = source {
        payload["source"] = json!(src);
    }

    Observation::try_new(
        Uuid::now_v7(),
        UnixTimestamp(timestamp),
        APPLE_HEALTH_PROVIDER_ID,
        HEART_RATE_DATA_TYPE,
        payload,
        0.98,
    )
    .map_err(Into::into)
}

/// HTTP client for `POST /v1/ingest` (and optional status probe).
#[derive(Debug, Clone)]
pub struct CompanionClient {
    base_url: String,
    token: String,
    http: reqwest::Client,
}

impl CompanionClient {
    /// Creates a client. `base_url` should be like `http://127.0.0.1:8787` (no trailing slash).
    pub fn new(base_url: impl Into<String>, token: impl Into<String>) -> CompanionResult<Self> {
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()?;
        Ok(Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            token: token.into(),
            http,
        })
    }

    /// Default loopback URL + given pairing token.
    pub fn loopback(token: impl Into<String>) -> CompanionResult<Self> {
        Self::new(DEFAULT_INGEST_BASE_URL, token)
    }

    /// Posts a JSON array of Observations. Maps `401` → [`CompanionError::Unauthorized`].
    pub async fn post_observations(
        &self,
        observations: &[Observation],
    ) -> CompanionResult<IngestResponse> {
        let url = format!("{}/v1/ingest", self.base_url);
        let response = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("content-type", "application/json")
            .json(observations)
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await.unwrap_or_default();

        if status.as_u16() == 401 {
            return Err(CompanionError::Unauthorized);
        }
        if !status.is_success() {
            return Err(CompanionError::Http {
                status: status.as_u16(),
                body,
            });
        }

        serde_json::from_str::<IngestResponse>(&body)
            .map_err(|err| CompanionError::BadResponse(format!("{err}; body={body}")))
    }

    /// Convenience: one sample `heart_rate` Observation → ingest.
    pub async fn post_sample_heart_rate(
        &self,
        bpm: f64,
        source: Option<&str>,
    ) -> CompanionResult<IngestResponse> {
        let obs = sample_heart_rate_observation(bpm, source)?;
        self.post_observations(&[obs]).await
    }
}

/// Default ingest port constant (re-export for CLI/docs).
pub fn default_ingest_port() -> u16 {
    DEFAULT_INGEST_PORT
}
