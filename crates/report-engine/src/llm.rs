//! Optional opt-in LLM interpret path (Ollama / OpenAI-compatible).
//!
//! Default **OFF**. Never call from app startup — hosts must invoke only on
//! explicit user action (Dashboard wiring → P4-E3-T3). Consumes
//! [`crate::ReportDocument::llm_prompt`] only; does **not** compute Features.
//!
//! Optional Bearer token via [`LOCAL_LLM_API_KEY_ENV`] for remote OpenAI-compatible
//! providers (incl. Gemini OpenAI-compat gateways). Prefer localhost; remote URL
//! + key is the operator’s choice. Never log or surface the key in UI/status.

use std::env;
use std::fmt;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::builder::ReportDocument;
use crate::error::{ReportEngineError, ReportResult};

/// Env: set to `1` / `true` / `yes` / `on` to enable local LLM HTTP.
pub const LOCAL_LLM_ENV: &str = "BIOFOCUS_LOCAL_LLM";
/// Env: OpenAI-compatible API base (default Ollama localhost).
pub const LOCAL_LLM_BASE_URL_ENV: &str = "BIOFOCUS_LOCAL_LLM_BASE_URL";
/// Env: model id passed to `/chat/completions`.
pub const LOCAL_LLM_MODEL_ENV: &str = "BIOFOCUS_LOCAL_LLM_MODEL";
/// Env: HTTP timeout in seconds (default 30).
pub const LOCAL_LLM_TIMEOUT_SECS_ENV: &str = "BIOFOCUS_LOCAL_LLM_TIMEOUT_SECS";
/// Env: optional Bearer API key for OpenAI-compatible endpoints that require auth.
///
/// Empty / missing → no `Authorization` header (typical for local Ollama).
pub const LOCAL_LLM_API_KEY_ENV: &str = "BIOFOCUS_LOCAL_LLM_API_KEY";

const DEFAULT_BASE_URL: &str = "http://127.0.0.1:11434/v1";
const DEFAULT_MODEL: &str = "llama3.2";
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Configuration for the optional local LLM adapter.
#[derive(Clone, PartialEq, Eq)]
pub struct LocalLlmConfig {
    /// When false, [`interpret_llm_prompt`] returns [`ReportEngineError::LocalLlmDisabled`]
    /// without opening a network connection.
    pub enabled: bool,
    /// OpenAI-compatible base URL (e.g. `http://127.0.0.1:11434/v1`).
    pub base_url: String,
    /// Model name for chat completions.
    pub model: String,
    /// Per-request HTTP timeout.
    pub timeout: Duration,
    /// Optional Bearer token. Never include in UI/status DTOs or logs.
    pub api_key: Option<String>,
}

impl fmt::Debug for LocalLlmConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LocalLlmConfig")
            .field("enabled", &self.enabled)
            .field("base_url", &self.base_url)
            .field("model", &self.model)
            .field("timeout", &self.timeout)
            .field(
                "api_key",
                &self.api_key.as_ref().map(|_| "<redacted>"),
            )
            .finish()
    }
}

impl LocalLlmConfig {
    /// Disabled config — safe default (no network).
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            base_url: DEFAULT_BASE_URL.into(),
            model: DEFAULT_MODEL.into(),
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
            api_key: None,
        }
    }

    /// Load from process env. Missing / unrecognized enable flag → disabled.
    pub fn from_env() -> Self {
        Self::from_env_lookup(|key| env::var(key).ok())
    }

    fn from_env_lookup(lookup: impl Fn(&str) -> Option<String>) -> Self {
        let enabled = lookup(LOCAL_LLM_ENV)
            .as_deref()
            .map(env_flag_enabled)
            .unwrap_or(false);
        let base_url = lookup(LOCAL_LLM_BASE_URL_ENV)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| DEFAULT_BASE_URL.into());
        let model = lookup(LOCAL_LLM_MODEL_ENV)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| DEFAULT_MODEL.into());
        let timeout_secs = lookup(LOCAL_LLM_TIMEOUT_SECS_ENV)
            .and_then(|s| s.trim().parse::<u64>().ok())
            .filter(|&n| n > 0)
            .unwrap_or(DEFAULT_TIMEOUT_SECS);
        let api_key = lookup(LOCAL_LLM_API_KEY_ENV)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        Self {
            enabled,
            base_url,
            model,
            timeout: Duration::from_secs(timeout_secs),
            api_key,
        }
    }
}

fn env_flag_enabled(raw: &str) -> bool {
    let v = raw.trim().to_ascii_lowercase();
    matches!(v.as_str(), "1" | "true" | "yes" | "on")
}

/// Interprets [`ReportDocument::llm_prompt`] via the local LLM when enabled.
///
/// Explicit call only — do not invoke on host startup.
pub async fn interpret_report(
    document: &ReportDocument,
    config: &LocalLlmConfig,
) -> ReportResult<String> {
    interpret_llm_prompt(&document.llm_prompt, config).await
}

/// POSTs `llm_prompt` to an OpenAI-compatible `/chat/completions` endpoint.
///
/// No Feature math: the body is the prompt string only.
pub async fn interpret_llm_prompt(
    llm_prompt: &str,
    config: &LocalLlmConfig,
) -> ReportResult<String> {
    if !config.enabled {
        return Err(ReportEngineError::LocalLlmDisabled);
    }

    let url = chat_completions_url(&config.base_url);
    let client = reqwest::Client::builder()
        .timeout(config.timeout)
        .build()
        .map_err(|err| ReportEngineError::LocalLlmHttp {
            message: format!("failed to build HTTP client: {err}"),
        })?;

    let body = ChatCompletionRequest {
        model: config.model.clone(),
        messages: vec![ChatMessage {
            role: "user",
            content: llm_prompt,
        }],
        stream: false,
    };

    let mut request = client.post(&url).json(&body);
    if let Some(key) = config.api_key.as_deref() {
        request = request.bearer_auth(key);
    }

    let response = request.send().await.map_err(|err| {
        if err.is_timeout() {
            ReportEngineError::LocalLlmTimeout {
                timeout_ms: u64::try_from(config.timeout.as_millis()).unwrap_or(u64::MAX),
            }
        } else {
            ReportEngineError::LocalLlmHttp {
                message: format!("request to {url} failed: {err}"),
            }
        }
    })?;

    let status = response.status();
    if !status.is_success() {
        let detail = response.text().await.unwrap_or_else(|_| String::new());
        let truncated = truncate_for_error(&detail, 240);
        return Err(ReportEngineError::LocalLlmHttp {
            message: format!("HTTP {status} from {url}: {truncated}"),
        });
    }

    let parsed: ChatCompletionResponse =
        response
            .json()
            .await
            .map_err(|err| ReportEngineError::LocalLlmResponse {
                message: format!("invalid JSON from {url}: {err}"),
            })?;

    let content = parsed
        .choices
        .into_iter()
        .find_map(|c| c.message.content)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    match content {
        Some(text) => Ok(text),
        None => Err(ReportEngineError::LocalLlmResponse {
            message: "chat completion returned no message content".into(),
        }),
    }
}

fn chat_completions_url(base_url: &str) -> String {
    let trimmed = base_url.trim().trim_end_matches('/');
    format!("{trimmed}/chat/completions")
}

fn truncate_for_error(raw: &str, max: usize) -> String {
    let compact: String = raw.chars().take(max).collect();
    if raw.chars().count() > max {
        format!("{compact}…")
    } else {
        compact
    }
}

#[derive(Debug, Serialize)]
struct ChatCompletionRequest<'a> {
    model: String,
    messages: Vec<ChatMessage<'a>>,
    stream: bool,
}

#[derive(Debug, Serialize)]
struct ChatMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    #[serde(default)]
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    #[serde(default)]
    message: ChatMessageOwned,
}

#[derive(Debug, Default, Deserialize)]
struct ChatMessageOwned {
    #[serde(default)]
    content: Option<String>,
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    use axum::extract::State;
    use axum::http::StatusCode;
    use axum::routing::post;
    use axum::{Json, Router};
    use serde_json::{json, Value};
    use tokio::net::TcpListener;
    use tokio::sync::oneshot;

    use super::*;
    use crate::build_report;

    #[test]
    fn from_env_lookup_defaults_disabled() {
        let cfg = LocalLlmConfig::from_env_lookup(|_| None);
        assert!(!cfg.enabled);
        assert_eq!(cfg.base_url, DEFAULT_BASE_URL);
        assert_eq!(cfg.model, DEFAULT_MODEL);
        assert_eq!(cfg.timeout, Duration::from_secs(DEFAULT_TIMEOUT_SECS));
        assert!(cfg.api_key.is_none());
    }

    #[test]
    fn from_env_lookup_honors_opt_in_and_overrides() {
        let cfg = LocalLlmConfig::from_env_lookup(|key| match key {
            LOCAL_LLM_ENV => Some("1".into()),
            LOCAL_LLM_BASE_URL_ENV => Some("http://127.0.0.1:9999/v1".into()),
            LOCAL_LLM_MODEL_ENV => Some("mistral".into()),
            LOCAL_LLM_TIMEOUT_SECS_ENV => Some("12".into()),
            LOCAL_LLM_API_KEY_ENV => Some(" sk-test-secret ".into()),
            _ => None,
        });
        assert!(cfg.enabled);
        assert_eq!(cfg.base_url, "http://127.0.0.1:9999/v1");
        assert_eq!(cfg.model, "mistral");
        assert_eq!(cfg.timeout, Duration::from_secs(12));
        assert_eq!(cfg.api_key.as_deref(), Some("sk-test-secret"));
        let dbg = format!("{cfg:?}");
        assert!(dbg.contains("<redacted>"));
        assert!(!dbg.contains("sk-test-secret"));
    }

    #[test]
    fn from_env_lookup_ignores_blank_api_key() {
        let cfg = LocalLlmConfig::from_env_lookup(|key| match key {
            LOCAL_LLM_ENV => Some("1".into()),
            LOCAL_LLM_API_KEY_ENV => Some("   ".into()),
            _ => None,
        });
        assert!(cfg.api_key.is_none());
    }

    #[test]
    fn env_flag_parses_truthy() {
        assert!(env_flag_enabled("1"));
        assert!(env_flag_enabled("true"));
        assert!(env_flag_enabled("YES"));
        assert!(env_flag_enabled("on"));
        assert!(!env_flag_enabled("0"));
        assert!(!env_flag_enabled("false"));
        assert!(!env_flag_enabled(""));
    }

    #[test]
    fn chat_url_joins_base() {
        assert_eq!(
            chat_completions_url("http://127.0.0.1:11434/v1/"),
            "http://127.0.0.1:11434/v1/chat/completions"
        );
    }

    #[tokio::test]
    async fn disabled_returns_error_without_http() {
        let hits = Arc::new(AtomicUsize::new(0));
        let (_base, _shutdown) = spawn_mock(hits.clone(), MockMode::Ok).await;

        let doc = build_report(&[], &[]).expect("build");
        let err = interpret_report(&doc, &LocalLlmConfig::disabled())
            .await
            .expect_err("disabled");
        assert_eq!(err, ReportEngineError::LocalLlmDisabled);
        assert_eq!(hits.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn enabled_posts_llm_prompt_only() {
        let hits = Arc::new(AtomicUsize::new(0));
        let (base, _shutdown) = spawn_mock(hits.clone(), MockMode::Ok).await;

        let doc = build_report(&[], &[]).expect("build");
        let config = LocalLlmConfig {
            enabled: true,
            base_url: base,
            model: "test-model".into(),
            timeout: Duration::from_secs(5),
            api_key: None,
        };

        let text = interpret_report(&doc, &config).await.expect("interpret");
        assert_eq!(text, "Calm local summary.");
        assert_eq!(hits.load(Ordering::SeqCst), 1);
        assert!(doc.llm_prompt.contains("Не выдумывай метрики"));
    }

    #[tokio::test]
    async fn enabled_sends_bearer_when_api_key_set() {
        let hits = Arc::new(AtomicUsize::new(0));
        let (base, _shutdown) = spawn_mock(hits.clone(), MockMode::RequireBearer).await;

        let config = LocalLlmConfig {
            enabled: true,
            base_url: base,
            model: "test-model".into(),
            timeout: Duration::from_secs(5),
            api_key: Some("test-api-key".into()),
        };

        let text = interpret_llm_prompt("prompt only", &config)
            .await
            .expect("interpret with bearer");
        assert_eq!(text, "Calm local summary.");
        assert_eq!(hits.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn timeout_maps_to_typed_error() {
        let hits = Arc::new(AtomicUsize::new(0));
        let (base, _shutdown) = spawn_mock(hits.clone(), MockMode::Slow).await;

        let config = LocalLlmConfig {
            enabled: true,
            base_url: base,
            model: "test-model".into(),
            timeout: Duration::from_millis(80),
            api_key: None,
        };

        let err = interpret_llm_prompt("prompt only", &config)
            .await
            .expect_err("timeout");
        assert!(matches!(err, ReportEngineError::LocalLlmTimeout { .. }));
    }

    #[tokio::test]
    async fn empty_choices_are_response_error() {
        let hits = Arc::new(AtomicUsize::new(0));
        let (base, _shutdown) = spawn_mock(hits.clone(), MockMode::EmptyChoices).await;

        let config = LocalLlmConfig {
            enabled: true,
            base_url: base,
            model: "test-model".into(),
            timeout: Duration::from_secs(5),
            api_key: None,
        };

        let err = interpret_llm_prompt("x", &config)
            .await
            .expect_err("empty");
        assert!(matches!(err, ReportEngineError::LocalLlmResponse { .. }));
    }

    #[tokio::test]
    async fn http_error_status_is_typed() {
        let hits = Arc::new(AtomicUsize::new(0));
        let (base, _shutdown) = spawn_mock(hits.clone(), MockMode::Fail).await;

        let config = LocalLlmConfig {
            enabled: true,
            base_url: base,
            model: "test-model".into(),
            timeout: Duration::from_secs(5),
            api_key: None,
        };

        let err = interpret_llm_prompt("x", &config).await.expect_err("http");
        assert!(matches!(err, ReportEngineError::LocalLlmHttp { .. }));
    }

    enum MockMode {
        Ok,
        Slow,
        EmptyChoices,
        Fail,
        RequireBearer,
    }

    async fn spawn_mock(
        hits: Arc<AtomicUsize>,
        mode: MockMode,
    ) -> (String, oneshot::Sender<()>) {
        use axum::http::HeaderMap;

        #[derive(Clone)]
        struct AppState {
            hits: Arc<AtomicUsize>,
            mode: MockModeKind,
        }

        #[derive(Clone, Copy)]
        enum MockModeKind {
            Ok,
            Slow,
            EmptyChoices,
            Fail,
            RequireBearer,
        }

        let mode = match mode {
            MockMode::Ok => MockModeKind::Ok,
            MockMode::Slow => MockModeKind::Slow,
            MockMode::EmptyChoices => MockModeKind::EmptyChoices,
            MockMode::Fail => MockModeKind::Fail,
            MockMode::RequireBearer => MockModeKind::RequireBearer,
        };

        let state = AppState { hits, mode };
        let app = Router::new()
            .route(
                "/v1/chat/completions",
                post(
                    |State(state): State<AppState>,
                     headers: HeaderMap,
                     Json(body): Json<Value>| async move {
                        state.hits.fetch_add(1, Ordering::SeqCst);
                        // Prompt-only contract: user messages only; no Feature payload.
                        assert!(body.get("messages").is_some());
                        assert!(body.get("features").is_none());

                        if matches!(state.mode, MockModeKind::RequireBearer) {
                            let auth = headers
                                .get(axum::http::header::AUTHORIZATION)
                                .and_then(|v| v.to_str().ok());
                            if auth != Some("Bearer test-api-key") {
                                return (
                                    StatusCode::UNAUTHORIZED,
                                    Json(json!({ "error": "missing bearer" })),
                                );
                            }
                        }

                        match state.mode {
                            MockModeKind::Ok | MockModeKind::RequireBearer => (
                                StatusCode::OK,
                                Json(json!({
                                    "choices": [{
                                        "message": { "content": "Calm local summary." }
                                    }]
                                })),
                            ),
                            MockModeKind::Slow => {
                                tokio::time::sleep(Duration::from_secs(2)).await;
                                (
                                    StatusCode::OK,
                                    Json(json!({
                                        "choices": [{
                                            "message": { "content": "late" }
                                        }]
                                    })),
                                )
                            }
                            MockModeKind::EmptyChoices => {
                                (StatusCode::OK, Json(json!({ "choices": [] })))
                            }
                            MockModeKind::Fail => (
                                StatusCode::BAD_GATEWAY,
                                Json(json!({ "error": "upstream" })),
                            ),
                        }
                    },
                ),
            )
            .with_state(state);

        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind mock");
        let addr = listener.local_addr().expect("addr");
        let (tx, rx) = oneshot::channel::<()>();
        tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    let _ = rx.await;
                })
                .await
                .expect("serve");
        });

        (format!("http://{addr}/v1"), tx)
    }
}
