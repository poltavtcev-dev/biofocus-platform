//! Integration: mock collectors → bounded channel → persist worker → SQLite.
//!
//! Covers emit Observation for `context_window`, opt-in `keystrokes`,
//! synthetic Calendar → `calendar_event`, opt-in Browser → `browser_category`,
//! opt-in Now Playing → `now_playing`, and opt-in Git activity → `git_activity`,
//! plus stop/idle: after `stop_stream` probe work must not keep ticking.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use bio_spec::Observation;
use ingest::spawn_persist_worker;
use macos_collector::{
    spawn_calendar_loop, ActiveWindowPlugin, BrowserCategoryPlugin, BrowserCategoryProbe,
    BrowserCategorySample, CalendarEvent, CalendarPlugin, CalendarProbe, FrontmostApp,
    FrontmostProbe, GitActivityPlugin, GitActivityProbe, GitActivitySample, IcsFileCalendarProbe,
    InputCountProbe, KeystrokeAggregatePlugin, NowPlayingPlugin, NowPlayingProbe,
    NowPlayingSample, ScriptedBrowserProbe, ScriptedCalendarProbe, ScriptedGitActivityProbe,
    ScriptedInputProbe, ScriptedNowPlayingProbe, BROWSER_CATEGORY_DATA_TYPE,
    CALENDAR_EVENT_DATA_TYPE, CONTEXT_WINDOW_DATA_TYPE, GIT_ACTIVITY_DATA_TYPE,
    KEYSTROKES_DATA_TYPE, NOW_PLAYING_DATA_TYPE,
};
use plugin_sdk::BioFocusPlugin;
use runtime::observation_channel;
use storage::{Database, ObservationRepository};

#[derive(Default)]
struct ScriptedFrontmost {
    apps: Mutex<Vec<Option<FrontmostApp>>>,
    calls: AtomicUsize,
}

impl FrontmostProbe for ScriptedFrontmost {
    fn frontmost(&self) -> macos_collector::CollectorResult<Option<FrontmostApp>> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let mut guard = self.apps.lock().expect("probe lock");
        if guard.is_empty() {
            return Ok(None);
        }
        Ok(guard.remove(0))
    }
}

/// Wraps [`ScriptedInputProbe`] and counts `take_count` / trust checks for idle asserts.
struct CountingInputProbe {
    inner: ScriptedInputProbe,
    drains: AtomicUsize,
}

impl CountingInputProbe {
    fn new() -> Self {
        Self {
            inner: ScriptedInputProbe::new(),
            drains: AtomicUsize::new(0),
        }
    }
}

impl InputCountProbe for CountingInputProbe {
    fn take_count(&self) -> macos_collector::CollectorResult<u64> {
        self.drains.fetch_add(1, Ordering::SeqCst);
        self.inner.take_count()
    }

    fn accessibility_trusted(&self) -> bool {
        self.inner.accessibility_trusted()
    }
}

struct CountingCalendarProbe {
    inner: ScriptedCalendarProbe,
    polls: AtomicUsize,
}

impl CountingCalendarProbe {
    fn new() -> Self {
        Self {
            inner: ScriptedCalendarProbe::new(),
            polls: AtomicUsize::new(0),
        }
    }
}

impl CalendarProbe for CountingCalendarProbe {
    fn events_in_range(
        &self,
        horizon_start: i64,
        horizon_end: i64,
    ) -> macos_collector::CollectorResult<Vec<CalendarEvent>> {
        self.polls.fetch_add(1, Ordering::SeqCst);
        self.inner.events_in_range(horizon_start, horizon_end)
    }
}

struct CountingBrowserProbe {
    sample: BrowserCategorySample,
    polls: AtomicUsize,
}

impl CountingBrowserProbe {
    fn new(sample: BrowserCategorySample) -> Self {
        Self {
            sample,
            polls: AtomicUsize::new(0),
        }
    }
}

impl BrowserCategoryProbe for CountingBrowserProbe {
    fn current(&self) -> macos_collector::CollectorResult<Option<BrowserCategorySample>> {
        self.polls.fetch_add(1, Ordering::SeqCst);
        Ok(Some(self.sample.clone()))
    }
}

struct CountingNowPlayingProbe {
    sample: NowPlayingSample,
    polls: AtomicUsize,
}

impl CountingNowPlayingProbe {
    fn new(sample: NowPlayingSample) -> Self {
        Self {
            sample,
            polls: AtomicUsize::new(0),
        }
    }
}

impl NowPlayingProbe for CountingNowPlayingProbe {
    fn current(&self) -> macos_collector::CollectorResult<Option<NowPlayingSample>> {
        self.polls.fetch_add(1, Ordering::SeqCst);
        Ok(Some(self.sample.clone()))
    }
}

struct CountingGitActivityProbe {
    sample: GitActivitySample,
    polls: AtomicUsize,
}

impl CountingGitActivityProbe {
    fn new(sample: GitActivitySample) -> Self {
        Self {
            sample,
            polls: AtomicUsize::new(0),
        }
    }
}

impl GitActivityProbe for CountingGitActivityProbe {
    fn current(&self) -> macos_collector::CollectorResult<Option<GitActivitySample>> {
        self.polls.fetch_add(1, Ordering::SeqCst);
        Ok(Some(self.sample.clone()))
    }
}

async fn wait_for_data_type(
    db_path: &std::path::Path,
    data_type: &str,
    min_count: usize,
    timeout: Duration,
) -> Vec<Observation> {
    let deadline = tokio::time::Instant::now() + timeout;
    loop {
        {
            let db = Database::open(db_path).expect("re-open db");
            let repo = ObservationRepository::new(&db);
            let listed = repo.list_by_data_type(data_type).expect("list");
            if listed.len() >= min_count {
                return listed;
            }
        }
        if tokio::time::Instant::now() >= deadline {
            panic!("timed out waiting for {min_count} Observation(s) of type {data_type}");
        }
        tokio::time::sleep(Duration::from_millis(15)).await;
    }
}

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[tokio::test]
async fn active_window_emits_observation_into_channel_and_storage() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("biofocus_main.db");
    let db = Database::open(&db_path).expect("open db");

    let (tx, rx) = observation_channel(8).expect("channel");
    let worker = spawn_persist_worker(rx, db);

    let probe = Arc::new(ScriptedFrontmost {
        apps: Mutex::new(vec![Some(FrontmostApp {
            bundle_id: "com.biofocus.integration".into(),
            app_name: "Integration".into(),
        })]),
        calls: AtomicUsize::new(0),
    });

    let plugin = ActiveWindowPlugin::with_probe(
        Arc::clone(&probe) as Arc<dyn FrontmostProbe>,
        Duration::from_millis(40),
    );
    plugin.start_stream(tx).await.expect("start");

    let listed = wait_for_data_type(&db_path, CONTEXT_WINDOW_DATA_TYPE, 1, Duration::from_secs(3)).await;
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].payload["bundle_id"], "com.biofocus.integration");
    assert_eq!(listed[0].payload["app_name"], "Integration");
    assert!(listed[0].payload.get("window_title").is_none());

    plugin.stop_stream().await.expect("stop");
    worker.join().expect("persist worker");
}

#[tokio::test]
async fn keystroke_aggregates_emit_observation_into_channel_and_storage() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("biofocus_main.db");
    let db = Database::open(&db_path).expect("open db");

    let (tx, rx) = observation_channel(8).expect("channel");
    let worker = spawn_persist_worker(rx, db);

    let probe = Arc::new(CountingInputProbe::new());
    probe.inner.push(15);

    let plugin = KeystrokeAggregatePlugin::with_probe(
        Arc::clone(&probe) as Arc<dyn InputCountProbe>,
        Duration::from_millis(40),
    );
    plugin.start_stream(tx).await.expect("start");

    let listed = wait_for_data_type(&db_path, KEYSTROKES_DATA_TYPE, 1, Duration::from_secs(3)).await;
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].payload["count"], 15);
    assert!(listed[0].payload.get("text").is_none());
    assert!(listed[0].payload.get("char").is_none());

    plugin.stop_stream().await.expect("stop");
    worker.join().expect("persist worker");
}

#[tokio::test]
async fn calendar_synthetic_emits_observation_into_channel_and_storage() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("biofocus_main.db");
    let db = Database::open(&db_path).expect("open db");

    let (tx, rx) = observation_channel(8).expect("channel");
    let worker = spawn_persist_worker(rx, db);

    let now = now_unix();
    let probe = Arc::new(ScriptedCalendarProbe::new());
    probe.push(CalendarEvent {
        uid: "synth-meet-1".into(),
        start: now + 60,
        end: now + 3_660,
        all_day: false,
        busy: true,
    });

    let plugin = CalendarPlugin::with_probe(
        Arc::clone(&probe) as Arc<dyn CalendarProbe>,
        Duration::from_millis(40),
    )
    .with_horizon(Duration::from_secs(3_600), Duration::from_secs(7_200));
    plugin.start_stream(tx).await.expect("start");

    let listed =
        wait_for_data_type(&db_path, CALENDAR_EVENT_DATA_TYPE, 1, Duration::from_secs(3)).await;
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].payload["uid"], "synth-meet-1");
    assert_eq!(listed[0].payload["busy"], true);
    assert!(listed[0].payload.get("title").is_none());
    assert!(listed[0].payload.get("summary").is_none());
    assert!(listed[0].payload.get("description").is_none());

    plugin.stop_stream().await.expect("stop");
    worker.join().expect("persist worker");
}

#[tokio::test]
async fn calendar_ics_fixture_round_trip_to_storage() {
    let dir = tempfile::tempdir().expect("tempdir");
    let ics_path = dir.path().join("dogfood.ics");
    let now = now_unix();
    let start = now + 120;
    let end = start + 1_800;
    let start_s = format_ical_utc(start);
    let end_s = format_ical_utc(end);
    std::fs::write(
        &ics_path,
        format!(
            "BEGIN:VCALENDAR\nVERSION:2.0\nBEGIN:VEVENT\nUID:ics-dogfood@biofocus\nDTSTART:{start_s}\nDTEND:{end_s}\nSUMMARY:Must Not Leak\nDESCRIPTION:secret body\nEND:VEVENT\nEND:VCALENDAR\n"
        ),
    )
    .expect("write ics");

    let db_path = dir.path().join("biofocus_main.db");
    let db = Database::open(&db_path).expect("open db");
    let (tx, rx) = observation_channel(8).expect("channel");
    let worker = spawn_persist_worker(rx, db);

    let plugin = CalendarPlugin::with_probe(
        Arc::new(IcsFileCalendarProbe::new(ics_path)),
        Duration::from_millis(40),
    )
    .with_horizon(Duration::from_secs(3_600), Duration::from_secs(7_200));
    plugin.start_stream(tx).await.expect("start");

    let listed =
        wait_for_data_type(&db_path, CALENDAR_EVENT_DATA_TYPE, 1, Duration::from_secs(3)).await;
    assert_eq!(listed[0].payload["uid"], "ics-dogfood@biofocus");
    assert!(listed[0].payload.get("SUMMARY").is_none());
    assert!(listed[0].payload.get("title").is_none());

    plugin.stop_stream().await.expect("stop");
    worker.join().expect("persist worker");
}

#[tokio::test]
async fn browser_category_emits_observation_into_channel_and_storage() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("biofocus_main.db");
    let db = Database::open(&db_path).expect("open db");

    let (tx, rx) = observation_channel(8).expect("channel");
    let worker = spawn_persist_worker(rx, db);

    let probe = Arc::new(ScriptedBrowserProbe::new());
    probe.push(Some(BrowserCategorySample {
        category: "entertainment".into(),
        browser_bundle_id: Some("com.apple.Safari".into()),
    }));

    let plugin = BrowserCategoryPlugin::with_probe(
        Arc::clone(&probe) as Arc<dyn BrowserCategoryProbe>,
        Duration::from_millis(40),
    );
    plugin.start_stream(tx).await.expect("start");

    let listed =
        wait_for_data_type(&db_path, BROWSER_CATEGORY_DATA_TYPE, 1, Duration::from_secs(3)).await;
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].payload["category"], "entertainment");
    assert_eq!(listed[0].payload["browser_bundle_id"], "com.apple.Safari");
    assert!(listed[0].payload.get("url").is_none());
    assert!(listed[0].payload.get("title").is_none());
    assert!(listed[0].payload.get("href").is_none());

    plugin.stop_stream().await.expect("stop");
    worker.join().expect("persist worker");
}

fn format_ical_utc(unix: i64) -> String {
    let days = unix.div_euclid(86_400);
    let tod = unix.rem_euclid(86_400) as u32;
    let (y, m, d) = civil_from_days(days);
    let hh = tod / 3_600;
    let mm = (tod % 3_600) / 60;
    let ss = tod % 60;
    format!("{y:04}{m:02}{d:02}T{hh:02}{mm:02}{ss:02}Z")
}

fn civil_from_days(days: i64) -> (i32, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = mp + if mp < 10 { 3 } else { -9 };
    let y = y + if m <= 2 { 1 } else { 0 };
    (y as i32, m as u32, d as u32)
}

#[tokio::test]
async fn active_window_stop_halts_periodic_probe_work() {
    let probe = Arc::new(ScriptedFrontmost::default());
    let (tx, _rx) = observation_channel(4).expect("channel");
    let plugin = ActiveWindowPlugin::with_probe(
        Arc::clone(&probe) as Arc<dyn FrontmostProbe>,
        Duration::from_millis(50),
    );

    plugin.start_stream(tx).await.expect("start");
    tokio::time::sleep(Duration::from_millis(120)).await;
    assert!(probe.calls.load(Ordering::SeqCst) >= 1);

    plugin.stop_stream().await.expect("stop");
    let after_stop = probe.calls.load(Ordering::SeqCst);

    tokio::time::sleep(Duration::from_millis(250)).await;
    assert_eq!(
        probe.calls.load(Ordering::SeqCst),
        after_stop,
        "frontmost probe must not keep polling after stop_stream (no busy-loop)"
    );
}

#[tokio::test]
async fn keystroke_stop_halts_periodic_probe_work() {
    let probe = Arc::new(CountingInputProbe::new());
    let (tx, _rx) = observation_channel(4).expect("channel");
    let plugin = KeystrokeAggregatePlugin::with_probe(
        Arc::clone(&probe) as Arc<dyn InputCountProbe>,
        Duration::from_millis(50),
    );

    plugin.start_stream(tx).await.expect("start");
    tokio::time::sleep(Duration::from_millis(180)).await;
    assert!(probe.drains.load(Ordering::SeqCst) >= 1);

    plugin.stop_stream().await.expect("stop");
    let after_stop = probe.drains.load(Ordering::SeqCst);

    tokio::time::sleep(Duration::from_millis(250)).await;
    assert_eq!(
        probe.drains.load(Ordering::SeqCst),
        after_stop,
        "input probe must not keep draining after stop_stream (no busy-loop)"
    );
}

#[tokio::test]
async fn calendar_stop_halts_periodic_probe_work() {
    let probe = Arc::new(CountingCalendarProbe::new());
    let (tx, _rx) = observation_channel(4).expect("channel");
    let counter = Arc::new(AtomicUsize::new(0));
    let handle = spawn_calendar_loop(
        tx,
        Arc::clone(&probe) as Arc<dyn CalendarProbe>,
        Duration::from_millis(50),
        Duration::from_secs(3_600),
        Duration::from_secs(3_600),
        Some(Arc::clone(&counter)),
    );

    tokio::time::sleep(Duration::from_millis(180)).await;
    assert!(probe.polls.load(Ordering::SeqCst) >= 1);
    assert!(counter.load(Ordering::SeqCst) >= 1);

    handle.stop().await;
    let after_stop = probe.polls.load(Ordering::SeqCst);

    tokio::time::sleep(Duration::from_millis(250)).await;
    assert_eq!(
        probe.polls.load(Ordering::SeqCst),
        after_stop,
        "calendar probe must not keep polling after stop (no busy-loop)"
    );
}

#[tokio::test]
async fn browser_category_stop_halts_periodic_probe_work() {
    let probe = Arc::new(CountingBrowserProbe::new(BrowserCategorySample {
        category: "work".into(),
        browser_bundle_id: Some("com.google.Chrome".into()),
    }));

    let (tx, _rx) = observation_channel(4).expect("channel");
    let plugin = BrowserCategoryPlugin::with_probe(
        Arc::clone(&probe) as Arc<dyn BrowserCategoryProbe>,
        Duration::from_millis(50),
    );

    plugin.start_stream(tx).await.expect("start");
    tokio::time::sleep(Duration::from_millis(180)).await;
    assert!(probe.polls.load(Ordering::SeqCst) >= 1);

    plugin.stop_stream().await.expect("stop");
    let after_stop = probe.polls.load(Ordering::SeqCst);

    tokio::time::sleep(Duration::from_millis(250)).await;
    assert_eq!(
        probe.polls.load(Ordering::SeqCst),
        after_stop,
        "browser category probe must not keep polling after stop_stream (no busy-loop)"
    );
}

#[tokio::test]
async fn now_playing_emits_observation_into_channel_and_storage() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("biofocus_main.db");
    let db = Database::open(&db_path).expect("open db");

    let (tx, rx) = observation_channel(8).expect("channel");
    let worker = spawn_persist_worker(rx, db);

    let probe = Arc::new(ScriptedNowPlayingProbe::new());
    probe.push(Some(NowPlayingSample {
        media_kind: "music".into(),
        is_playing: true,
    }));

    let plugin = NowPlayingPlugin::with_probe(
        Arc::clone(&probe) as Arc<dyn NowPlayingProbe>,
        Duration::from_millis(40),
    );
    plugin.start_stream(tx).await.expect("start");

    let listed =
        wait_for_data_type(&db_path, NOW_PLAYING_DATA_TYPE, 1, Duration::from_secs(3)).await;
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].payload["media_kind"], "music");
    assert_eq!(listed[0].payload["is_playing"], true);
    assert!(listed[0].payload.get("title").is_none());
    assert!(listed[0].payload.get("artist").is_none());
    assert!(listed[0].payload.get("album").is_none());
    assert!(listed[0].payload.get("lyrics").is_none());
    assert!(listed[0].payload.get("playlist_id").is_none());

    plugin.stop_stream().await.expect("stop");
    worker.join().expect("persist worker");
}

#[tokio::test]
async fn now_playing_stop_halts_periodic_probe_work() {
    let probe = Arc::new(CountingNowPlayingProbe::new(NowPlayingSample {
        media_kind: "podcast".into(),
        is_playing: true,
    }));

    let (tx, _rx) = observation_channel(4).expect("channel");
    let plugin = NowPlayingPlugin::with_probe(
        Arc::clone(&probe) as Arc<dyn NowPlayingProbe>,
        Duration::from_millis(50),
    );

    plugin.start_stream(tx).await.expect("start");
    tokio::time::sleep(Duration::from_millis(180)).await;
    assert!(probe.polls.load(Ordering::SeqCst) >= 1);

    plugin.stop_stream().await.expect("stop");
    let after_stop = probe.polls.load(Ordering::SeqCst);

    tokio::time::sleep(Duration::from_millis(250)).await;
    assert_eq!(
        probe.polls.load(Ordering::SeqCst),
        after_stop,
        "now_playing probe must not keep polling after stop_stream (no busy-loop)"
    );
}

#[tokio::test]
async fn git_activity_emits_observation_into_channel_and_storage() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("biofocus_main.db");
    let db = Database::open(&db_path).expect("open db");

    let (tx, rx) = observation_channel(8).expect("channel");
    let worker = spawn_persist_worker(rx, db);

    let probe = Arc::new(ScriptedGitActivityProbe::new());
    probe.push(Some(GitActivitySample {
        activity_kind: "commit".into(),
        event_count: Some(2),
    }));

    let plugin = GitActivityPlugin::with_probe(
        Arc::clone(&probe) as Arc<dyn GitActivityProbe>,
        Duration::from_millis(40),
    );
    plugin.start_stream(tx).await.expect("start");

    let listed =
        wait_for_data_type(&db_path, GIT_ACTIVITY_DATA_TYPE, 1, Duration::from_secs(3)).await;
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].payload["activity_kind"], "commit");
    assert_eq!(listed[0].payload["event_count"], 2);
    assert!(listed[0].payload.get("repo_path").is_none());
    assert!(listed[0].payload.get("remote").is_none());
    assert!(listed[0].payload.get("branch").is_none());
    assert!(listed[0].payload.get("sha").is_none());
    assert!(listed[0].payload.get("message").is_none());
    assert!(listed[0].payload.get("diff").is_none());
    assert!(listed[0].payload.get("author").is_none());

    plugin.stop_stream().await.expect("stop");
    worker.join().expect("persist worker");
}

#[tokio::test]
async fn git_activity_stop_halts_periodic_probe_work() {
    let probe = Arc::new(CountingGitActivityProbe::new(GitActivitySample {
        activity_kind: "sync".into(),
        event_count: Some(1),
    }));

    let (tx, _rx) = observation_channel(4).expect("channel");
    let plugin = GitActivityPlugin::with_probe(
        Arc::clone(&probe) as Arc<dyn GitActivityProbe>,
        Duration::from_millis(50),
    );

    plugin.start_stream(tx).await.expect("start");
    tokio::time::sleep(Duration::from_millis(180)).await;
    assert!(probe.polls.load(Ordering::SeqCst) >= 1);

    plugin.stop_stream().await.expect("stop");
    let after_stop = probe.polls.load(Ordering::SeqCst);

    tokio::time::sleep(Duration::from_millis(250)).await;
    assert_eq!(
        probe.polls.load(Ordering::SeqCst),
        after_stop,
        "git_activity probe must not keep polling after stop_stream (no busy-loop)"
    );
}
