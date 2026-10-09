//! Companion client: post `Observation` batches to desktop ingest.
//!
//! HealthKit-shaped samples for Desktop ingest dogfood (HR + ADR-018 wearable
//! types). No Feature math, no cloud. Pairing UX is Desktop IPC.

#![forbid(unsafe_code)]

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use bio_spec::{
    DATA_TYPE_ACTIVE_ENERGY, DATA_TYPE_OXYGEN_SATURATION, DATA_TYPE_SLEEP_INTERVAL,
    DATA_TYPE_STEP_COUNT, Observation, UnixTimestamp,
};
use ingest::{DEFAULT_INGEST_PORT, IngestResponse, StatusResponse};
use serde_json::json;
use thiserror::Error;
use uuid::Uuid;

/// Matches `docs/07-contracts.md` Apple Health provider.
pub const APPLE_HEALTH_PROVIDER_ID: &str = "com.biofocus.applehealth";

/// Canonical `data_type` for heart-rate Observations.
pub const HEART_RATE_DATA_TYPE: &str = "heart_rate";

/// Default loopback ingest base URL (Desktop host).
pub const DEFAULT_INGEST_BASE_URL: &str = "http://127.0.0.1:8787";

/// Default HTTP timeout for status preflight (`GET /v1/status`).
pub const STATUS_REQUEST_TIMEOUT: Duration = Duration::from_secs(5);

/// Default HTTP timeout for ingest POST.
pub const INGEST_REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

/// Crate identity.
pub const CRATE_NAME: &str = "companion";

/// Classifies transport failures for calm companion UX (P28-E1-T1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkErrorKind {
    Timeout,
    Unreachable,
    Other,
}

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

    /// Plain HTTP is only allowed to loopback. LAN must be https.
    #[error("plain HTTP is only allowed for 127.0.0.1")]
    PlainHttpOffLoopback,

    /// https ingest needs the certificate fingerprint from the pairing QR.
    #[error("https ingest requires a certificate fingerprint")]
    PinRequired,

    /// Fingerprint was not 64 hex characters, or rustls rejected the pin config.
    #[error("could not pin the ingest certificate")]
    PinRejected,
}

/// Result alias for companion operations.
pub type CompanionResult<T> = Result<T, CompanionError>;

/// Maps `reqwest` transport errors into [`NetworkErrorKind`].
#[must_use]
pub fn classify_network_error(err: &reqwest::Error) -> NetworkErrorKind {
    if err.is_timeout() {
        NetworkErrorKind::Timeout
    } else if err.is_connect() || err.is_request() {
        NetworkErrorKind::Unreachable
    } else {
        NetworkErrorKind::Other
    }
}

fn now_ts() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Builds a privacy-safe sample `heart_rate` Observation (bpm only + optional source).
pub fn sample_heart_rate_observation(
    bpm: f64,
    source: Option<&str>,
) -> CompanionResult<Observation> {
    let mut payload = json!({ "bpm": bpm });
    if let Some(src) = source {
        payload["source"] = json!(src);
    }

    Observation::try_new(
        Uuid::now_v7(),
        UnixTimestamp(now_ts()),
        APPLE_HEALTH_PROVIDER_ID,
        HEART_RATE_DATA_TYPE,
        payload,
        0.98,
    )
    .map_err(Into::into)
}

/// Builds a scripted `step_count` Observation (ADR-018).
pub fn sample_step_count_observation(
    count: u64,
    window_secs: Option<u64>,
) -> CompanionResult<Observation> {
    let mut payload = json!({ "count": count });
    if let Some(w) = window_secs {
        payload["window_secs"] = json!(w);
    }
    Observation::try_new(
        Uuid::now_v7(),
        UnixTimestamp(now_ts()),
        APPLE_HEALTH_PROVIDER_ID,
        DATA_TYPE_STEP_COUNT,
        payload,
        0.9,
    )
    .map_err(Into::into)
}

/// Builds a scripted `active_energy` Observation (ADR-018).
pub fn sample_active_energy_observation(kcal: f64) -> CompanionResult<Observation> {
    Observation::try_new(
        Uuid::now_v7(),
        UnixTimestamp(now_ts()),
        APPLE_HEALTH_PROVIDER_ID,
        DATA_TYPE_ACTIVE_ENERGY,
        json!({ "kcal": kcal }),
        0.9,
    )
    .map_err(Into::into)
}

/// Builds a scripted `sleep_interval` Observation (ADR-018).
pub fn sample_sleep_interval_observation(
    start: i64,
    end: i64,
    stage: Option<&str>,
) -> CompanionResult<Observation> {
    let mut payload = json!({ "start": start, "end": end });
    if let Some(s) = stage {
        payload["stage"] = json!(s);
    }
    Observation::try_new(
        Uuid::now_v7(),
        UnixTimestamp(now_ts()),
        APPLE_HEALTH_PROVIDER_ID,
        DATA_TYPE_SLEEP_INTERVAL,
        payload,
        0.85,
    )
    .map_err(Into::into)
}

/// Builds a scripted soft-optional `oxygen_saturation` Observation (ADR-018).
pub fn sample_oxygen_saturation_observation(spo2_percent: f64) -> CompanionResult<Observation> {
    Observation::try_new(
        Uuid::now_v7(),
        UnixTimestamp(now_ts()),
        APPLE_HEALTH_PROVIDER_ID,
        DATA_TYPE_OXYGEN_SATURATION,
        json!({ "spo2_percent": spo2_percent }),
        0.85,
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
        Self::with_timeout(base_url, token, INGEST_REQUEST_TIMEOUT, None)
    }

    /// Client tuned for status preflight (shorter timeout).
    pub fn for_status(
        base_url: impl Into<String>,
        token: impl Into<String>,
    ) -> CompanionResult<Self> {
        Self::with_timeout(base_url, token, STATUS_REQUEST_TIMEOUT, None)
    }

    /// Like [`Self::new`], plus the SHA-256 certificate fingerprint for https.
    pub fn with_pin(
        base_url: impl Into<String>,
        token: impl Into<String>,
        pin: Option<&str>,
    ) -> CompanionResult<Self> {
        Self::with_timeout(base_url, token, INGEST_REQUEST_TIMEOUT, pin)
    }

    fn with_timeout(
        base_url: impl Into<String>,
        token: impl Into<String>,
        timeout: Duration,
        pin: Option<&str>,
    ) -> CompanionResult<Self> {
        let base_url = base_url.into().trim_end_matches('/').to_string();
        enforce_ingest_transport(&base_url, pin)?;
        let mut builder = reqwest::Client::builder().timeout(timeout);
        if base_url.starts_with("https://") {
            let pin = pin
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .ok_or(CompanionError::PinRequired)?;
            let config =
                ingest::pinned_client_config(pin).map_err(|_| CompanionError::PinRejected)?;
            builder = builder.use_preconfigured_tls(config);
        }
        let http = builder.build()?;
        Ok(Self {
            base_url,
            token: token.into(),
            http,
        })
    }

    /// Default loopback URL + given pairing token.
    pub fn loopback(token: impl Into<String>) -> CompanionResult<Self> {
        Self::new(DEFAULT_INGEST_BASE_URL, token)
    }

    /// `GET /v1/status` — unauthenticated reachability probe (P28-E1-T1).
    pub async fn get_status(&self) -> CompanionResult<StatusResponse> {
        let url = format!("{}/v1/status", self.base_url);
        let response = self.http.get(&url).send().await?;

        let status = response.status();
        let body = response.text().await.unwrap_or_default();

        if !status.is_success() {
            return Err(CompanionError::Http {
                status: status.as_u16(),
                body,
            });
        }

        serde_json::from_str::<StatusResponse>(&body)
            .map_err(|err| CompanionError::BadResponse(format!("{err}; body={body}")))
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

/// Loopback may use plain HTTP. Any other host must be https with a fingerprint.
pub fn enforce_ingest_transport(base_url: &str, pin: Option<&str>) -> CompanionResult<()> {
    if let Some(rest) = base_url.strip_prefix("http://") {
        let host = rest.split([':', '/']).next().unwrap_or("");
        if host == "127.0.0.1" || host == "localhost" {
            return Ok(());
        }
        return Err(CompanionError::PlainHttpOffLoopback);
    }
    if base_url.starts_with("https://") {
        let present = pin.map(str::trim).is_some_and(|value| !value.is_empty());
        if present {
            return Ok(());
        }
        return Err(CompanionError::PinRequired);
    }
    Err(CompanionError::BadResponse(
        "base URL must start with http:// or https://".into(),
    ))
}

/// Default ingest port constant (re-export for CLI/docs).
pub fn default_ingest_port() -> u16 {
    DEFAULT_INGEST_PORT
}

#[cfg(test)]
mod transport_tests {
    use super::*;

    #[test]
    fn plain_http_off_loopback_is_rejected_and_https_needs_a_pin() {
        assert!(matches!(
            enforce_ingest_transport("http://192.168.1.10:8787", None),
            Err(CompanionError::PlainHttpOffLoopback)
        ));
        assert!(enforce_ingest_transport("http://127.0.0.1:8787", None).is_ok());
        assert!(matches!(
            enforce_ingest_transport("https://10.0.0.8:8787", None),
            Err(CompanionError::PinRequired)
        ));
        assert!(enforce_ingest_transport("https://10.0.0.8:8787", Some("ab")).is_ok());
    }
}
