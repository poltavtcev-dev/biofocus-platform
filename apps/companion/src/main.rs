//! CLI: post one sample `heart_rate` Observation to local BioFocus ingest.
//!
//! ```text
//! BIOFOCUS_INGEST_TOKEN=… cargo run -p companion --bin biofocus-companion-sample -- 74
//! ```

use std::env;
use std::process::ExitCode;

use companion::{CompanionClient, CompanionError, DEFAULT_INGEST_BASE_URL};
use ingest::resolve_ingest_token;

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(code) => code,
    }
}

async fn run() -> Result<(), ExitCode> {
    let bpm = env::args()
        .nth(1)
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(74.0);

    let base_url =
        env::var("BIOFOCUS_INGEST_URL").unwrap_or_else(|_| DEFAULT_INGEST_BASE_URL.to_string());

    let token = resolve_ingest_token().map_err(|err| {
        eprintln!("pairing token required ({err}); set BIOFOCUS_INGEST_TOKEN or create ~/.biofocus/pairing_token");
        ExitCode::from(3)
    })?;

    let pin = env::var("BIOFOCUS_INGEST_PIN").ok();
    let pin = pin.as_deref().filter(|value| !value.trim().is_empty());
    let client = CompanionClient::with_pin(&base_url, &token, pin).map_err(|err| {
        eprintln!("client init failed: {err}");
        ExitCode::FAILURE
    })?;

    match client
        .post_sample_heart_rate(bpm, Some("companion-sample-cli"))
        .await
    {
        Ok(resp) => {
            println!("queued count={}", resp.count);
            Ok(())
        }
        Err(CompanionError::Unauthorized) => {
            eprintln!("401 unauthorized — set BIOFOCUS_INGEST_TOKEN or ~/.biofocus/pairing_token");
            Err(ExitCode::from(3))
        }
        Err(CompanionError::Network(err)) => {
            eprintln!("network error (is Desktop ingest running?): {err}");
            Err(ExitCode::from(2))
        }
        Err(err) => {
            eprintln!("ingest failed: {err}");
            Err(ExitCode::FAILURE)
        }
    }
}
