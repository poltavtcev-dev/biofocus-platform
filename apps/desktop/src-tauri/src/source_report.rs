//! Which device streams reached the Mac in the last 24 hours.
//!
//! Reads Observation counts only (no payloads). The phone is the courier for
//! watch samples; it does not emit a separate phone sensor stream.

use std::time::{SystemTime, UNIX_EPOCH};

use bio_spec::UnixTimestamp;
use report_engine::format_unix_local;
use storage::{Database, ObservationRepository, ObservationTypeSummary};

const LOOKBACK_SECS: i64 = 24 * 60 * 60;

struct Channel {
    label: &'static str,
    data_type: &'static str,
    /// When set, a missing stream means the collector is off, not "no samples".
    enable_env: Option<&'static str>,
}

const MAC: &[Channel] = &[
    Channel {
        label: "Окно приложения",
        data_type: "context_window",
        enable_env: None,
    },
    Channel {
        label: "Темп набора",
        data_type: "keystrokes",
        enable_env: Some("BIOFOCUS_INPUT_AGGREGATES"),
    },
    Channel {
        label: "Календарь",
        data_type: "calendar_event",
        enable_env: Some("BIOFOCUS_CALENDAR"),
    },
    Channel {
        label: "Категории браузера",
        data_type: "browser_category",
        enable_env: Some("BIOFOCUS_BROWSER_CATEGORIES"),
    },
    Channel {
        label: "Музыка",
        data_type: "now_playing",
        enable_env: Some("BIOFOCUS_NOW_PLAYING"),
    },
    Channel {
        label: "Git-активность",
        data_type: "git_activity",
        enable_env: Some("BIOFOCUS_GIT_ACTIVITY"),
    },
    Channel {
        label: "Свет",
        data_type: "ambient_light",
        enable_env: Some("BIOFOCUS_AMBIENT_LIGHT"),
    },
    Channel {
        label: "Уведомления",
        data_type: "notification_event",
        enable_env: Some("BIOFOCUS_NOTIFICATION_EVENTS"),
    },
    Channel {
        label: "События дня (кофе, прогулка)",
        data_type: "life_event",
        enable_env: None,
    },
];

const WATCH: &[Channel] = &[
    Channel {
        label: "Пульс",
        data_type: "heart_rate",
        enable_env: None,
    },
    Channel {
        label: "Вариабельность пульса",
        data_type: "hrv",
        enable_env: None,
    },
    Channel {
        label: "Шаги",
        data_type: "step_count",
        enable_env: None,
    },
    Channel {
        label: "Активная энергия",
        data_type: "active_energy",
        enable_env: None,
    },
    Channel {
        label: "Сон",
        data_type: "sleep_interval",
        enable_env: None,
    },
    Channel {
        label: "Кислород в крови",
        data_type: "oxygen_saturation",
        enable_env: None,
    },
];

/// Markdown section appended to the report. Never includes payloads.
pub fn render_source_section() -> String {
    let offset = local_offset_secs();
    let mut out = String::new();
    out.push_str("## Откуда пришли данные\n\n");
    out.push_str(
        "_Последние 24 часа на этом компьютере. Телефон сам ничего не измеряет: \
         он только приносит данные часов, когда Mac в той же сети и BioFocus слушает LAN._\n\n",
    );

    let summaries = match load_summaries() {
        Ok(rows) => rows,
        Err(message) => {
            out.push_str(&format!(
                "Сводку источников прочитать не удалось: {message}\n"
            ));
            return out;
        }
    };

    let watch_hits = WATCH
        .iter()
        .filter(|ch| find_summary(&summaries, ch.data_type).is_some())
        .count();
    if watch_hits == 0 {
        out.push_str(
            "**Телефон → Mac:** за сутки данные часов не доехали. \
             Очередь может всё ещё лежать на iPhone (на экране Companion будет «Desktop not reachable»).\n\n",
        );
    } else {
        out.push_str(&format!(
            "**Телефон → Mac:** доехали {watch_hits} из {} потоков часов.\n\n",
            WATCH.len()
        ));
    }

    out.push_str("### Компьютер\n\n");
    for channel in MAC {
        out.push_str(&format_line(channel, &summaries, offset));
    }
    out.push('\n');
    out.push_str("### Часы (через телефон)\n\n");
    for channel in WATCH {
        out.push_str(&format_line(channel, &summaries, offset));
    }
    out.push('\n');
    out
}

fn load_summaries() -> Result<Vec<ObservationTypeSummary>, String> {
    let path = storage::default_db_path().map_err(|err| err.public_message())?;
    let db = Database::open(&path).map_err(|err| err.public_message())?;
    let repo = ObservationRepository::new(&db);
    let since = UnixTimestamp::from_secs(unix_now_secs().saturating_sub(LOOKBACK_SECS));
    repo.summarize_types_since(since)
        .map_err(|err| err.public_message())
}

fn format_line(
    channel: &Channel,
    summaries: &[ObservationTypeSummary],
    offset: i32,
) -> String {
    if let Some(row) = find_summary(summaries, channel.data_type) {
        let when = format_unix_local(row.latest_timestamp, offset);
        return format!(
            "- **{}** — пришло {}, последнее {}.\n",
            channel.label, row.count, when
        );
    }
    if let Some(env) = channel.enable_env {
        if !env_on(env) {
            return format!(
                "- **{}** — сбор выключен (`{env}`).\n",
                channel.label
            );
        }
    }
    format!(
        "- **{}** — за сутки на компьютере нет.\n",
        channel.label
    )
}

fn find_summary<'a>(
    summaries: &'a [ObservationTypeSummary],
    data_type: &str,
) -> Option<&'a ObservationTypeSummary> {
    summaries.iter().find(|row| row.data_type == data_type && row.count > 0)
}

fn env_on(name: &str) -> bool {
    let Ok(raw) = std::env::var(name) else {
        return false;
    };
    matches!(
        raw.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

fn unix_now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
        .unwrap_or(0)
}

fn local_offset_secs() -> i32 {
    chrono::Local::now().offset().local_minus_utc()
}
