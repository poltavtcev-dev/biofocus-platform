import { useEffect, useState } from "react";
import { TrayIcon } from "@tauri-apps/api/tray";
import {
  alertCopy,
  fetchCoreStatus,
  statusView,
  trayTooltipFor,
  type AlertLevel,
  type CoreStatusView,
} from "./coreStatus";
import { Dashboard } from "./Dashboard";
import { isDashboardSurface, openDashboardWindow } from "./dashboardWindow";
import {
  copyText,
  fetchPairingToken,
  maskToken,
  networkModeDetail,
  networkModeLabel,
  primaryBaseUrl,
  type PairingView,
} from "./pairing";
import {
  fetchRecentLifeEvents,
  formatLifeEventTime,
  LIFE_EVENT_KINDS,
  lifeEventLabel,
  logLifeEvent,
  type LifeEventKind,
  type LifeEventsListView,
  type LogLifeEventView,
} from "./lifeEvents";
import "./App.css";

const TRAY_ID = "main";
/** Soft refresh so Menubar alert tracks Core without busy-loop. */
const STATUS_POLL_MS = 5_000;

async function syncTrayTooltip(view: CoreStatusView): Promise<void> {
  try {
    const tray = await TrayIcon.getById(TRAY_ID);
    if (tray) {
      await tray.setTooltip(trayTooltipFor(view));
    }
  } catch {
    // Browser/Vite preview has no tray — ignore.
  }
}

function AlertIndicator({ level }: { level: AlertLevel }) {
  const copy = alertCopy(level);
  return (
    <section className="alert-block" aria-live="polite" data-alert={level}>
      <div className="status-row">
        <span className={`alert-dot alert-dot--${level}`} aria-hidden />
        <p className="status-label">{copy.label}</p>
      </div>
      <p className="status-detail">{copy.detail}</p>
    </section>
  );
}

function MenubarShell() {
  const [view, setView] = useState<CoreStatusView>(() =>
    statusView("idle", "core_ping"),
  );
  const [busy, setBusy] = useState(true);
  const [pairing, setPairing] = useState<PairingView>({ kind: "idle" });
  const [tokenVisible, setTokenVisible] = useState(false);
  const [qrVisible, setQrVisible] = useState(false);
  const [copyNote, setCopyNote] = useState<string | null>(null);
  const [dashNote, setDashNote] = useState<string | null>(null);
  const [lifeEvents, setLifeEvents] = useState<LifeEventsListView>({
    kind: "idle",
  });
  const [logView, setLogView] = useState<LogLifeEventView>({ kind: "idle" });

  useEffect(() => {
    let cancelled = false;

    const load = async (isFirst: boolean) => {
      if (isFirst) {
        setBusy(true);
      }
      const next = await fetchCoreStatus();
      if (cancelled) {
        return;
      }
      setView(next);
      if (isFirst) {
        setBusy(false);
      }
      void syncTrayTooltip(next);
    };

    void load(true);
    const timer = window.setInterval(() => {
      void load(false);
    }, STATUS_POLL_MS);

    return () => {
      cancelled = true;
      window.clearInterval(timer);
    };
  }, []);

  useEffect(() => {
    let cancelled = false;
    setPairing({ kind: "loading" });
    void fetchPairingToken().then((next) => {
      if (!cancelled) {
        setPairing(next);
      }
    });
    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    let cancelled = false;
    setLifeEvents({ kind: "loading" });
    void fetchRecentLifeEvents().then((next) => {
      if (!cancelled) {
        setLifeEvents(next);
      }
    });
    return () => {
      cancelled = true;
    };
  }, []);

  const onRetry = () => {
    setBusy(true);
    setView(statusView("idle", view.source));
    void fetchCoreStatus().then((next) => {
      setView(next);
      setBusy(false);
      void syncTrayTooltip(next);
    });
  };

  const onReloadPairing = () => {
    setPairing({ kind: "loading" });
    setCopyNote(null);
    void fetchPairingToken().then(setPairing);
  };

  const onCopyToken = async () => {
    if (pairing.kind !== "ready") {
      return;
    }
    const ok = await copyText(pairing.info.token);
    setCopyNote(ok ? "Token copied." : "Could not copy.");
  };

  const onCopyBaseUrl = async () => {
    if (pairing.kind !== "ready") {
      return;
    }
    const ok = await copyText(primaryBaseUrl(pairing.info));
    setCopyNote(ok ? "Base URL copied." : "Could not copy.");
  };

  const onOpenDashboard = () => {
    setDashNote(null);
    void openDashboardWindow().then((ok) => {
      if (!ok) {
        setDashNote("Could not open Dashboard.");
      }
    });
  };

  const onLogLifeEvent = (kind: LifeEventKind) => {
    if (logView.kind === "logging") {
      return;
    }
    setLogView({ kind: "logging", eventKind: kind });
    void logLifeEvent(kind).then((result) => {
      setLogView(result);
      if (result.kind === "ok") {
        void fetchRecentLifeEvents().then(setLifeEvents);
      }
    });
  };

  const onReloadLifeEvents = () => {
    setLifeEvents({ kind: "loading" });
    void fetchRecentLifeEvents().then(setLifeEvents);
  };

  const alertLevel = view.alertLevel ?? "green";
  const logging = logView.kind === "logging";

  return (
    <main
      className="shell"
      data-status={view.kind}
      data-alert={alertLevel}
    >
      <header className="brand">
        <h1>BioFocus</h1>
      </header>

      <section className="status-block" aria-live="polite">
        <div className="status-row">
          <span className={`status-dot status-dot--${view.kind}`} aria-hidden />
          <p className="status-label">
            {busy && view.kind === "idle" ? "Idle" : view.label}
          </p>
        </div>
        <p className="status-detail">{view.detail}</p>
        {view.meta && <p className="status-meta">{view.meta}</p>}
      </section>

      {view.kind !== "error" && <AlertIndicator level={alertLevel} />}

      {view.kind === "error" && (
        <button type="button" className="retry" onClick={onRetry} disabled={busy}>
          Try again
        </button>
      )}

      <section className="dashboard-entry" aria-label="Dashboard">
        <button type="button" className="retry" onClick={onOpenDashboard}>
          Open Dashboard
        </button>
        {dashNote && <p className="status-meta">{dashNote}</p>}
      </section>

      <section className="life-events-block" aria-label="Life events">
        <h2 className="pairing-title">Life events</h2>
        <p className="pairing-detail">
          Note what happened — coffee, a walk, lunch, or a workout. No scores.
        </p>
        <div className="life-event-actions">
          {LIFE_EVENT_KINDS.map((kind) => (
            <button
              key={kind}
              type="button"
              className="retry"
              disabled={logging}
              onClick={() => onLogLifeEvent(kind)}
            >
              {lifeEventLabel(kind)}
            </button>
          ))}
        </div>
        {logView.kind === "ok" && (
          <p className="status-meta" aria-live="polite">
            {logView.message}
          </p>
        )}
        {logView.kind === "error" && (
          <p className="status-meta" aria-live="polite">
            {logView.detail}
          </p>
        )}
        {logView.kind === "logging" && (
          <p className="status-meta" aria-live="polite">
            Logging {lifeEventLabel(logView.eventKind).toLowerCase()}…
          </p>
        )}

        <div className="life-event-recent">
          <div className="life-event-recent-head">
            <p className="pairing-subtitle">Recent</p>
            <button
              type="button"
              className="retry"
              onClick={onReloadLifeEvents}
              disabled={lifeEvents.kind === "loading"}
            >
              Refresh
            </button>
          </div>
          {lifeEvents.kind === "loading" && (
            <p className="status-meta">Loading recent…</p>
          )}
          {lifeEvents.kind === "error" && (
            <p className="status-meta">{lifeEvents.detail}</p>
          )}
          {lifeEvents.kind === "ready" && lifeEvents.events.length === 0 && (
            <p className="status-meta">Nothing logged yet.</p>
          )}
          {lifeEvents.kind === "ready" && lifeEvents.events.length > 0 && (
            <ul className="life-event-rows">
              {lifeEvents.events.map((event) => (
                <li key={event.id} className="life-event-row">
                  <span className="life-event-kind">
                    {lifeEventLabel(event.kind)}
                  </span>
                  <span className="life-event-time">
                    {formatLifeEventTime(event.timestamp)}
                  </span>
                </li>
              ))}
            </ul>
          )}
        </div>
      </section>

      <section className="pairing-block" aria-label="Companion pairing">
        <h2 className="pairing-title">Companion</h2>
        <p className="pairing-detail">
          Local pairing only — no cloud account. Share this Mac’s base URL and
          token with a phone on the same network when LAN is enabled.
        </p>

        {pairing.kind === "loading" && (
          <p className="status-meta">Loading pairing…</p>
        )}

        {pairing.kind === "error" && (
          <>
            <p className="status-meta">{pairing.detail}</p>
            <button type="button" className="retry" onClick={onReloadPairing}>
              Try again
            </button>
          </>
        )}

        {pairing.kind === "ready" && (
          <>
            <div className="pairing-url-block">
              <p className="pairing-subtitle">Base URL</p>
              <p className="pairing-url" aria-live="polite">
                {primaryBaseUrl(pairing.info)}
              </p>
              <div className="pairing-actions">
                <button
                  type="button"
                  className="retry"
                  onClick={() => void onCopyBaseUrl()}
                >
                  Copy URL
                </button>
                <button
                  type="button"
                  className="retry"
                  onClick={onReloadPairing}
                >
                  Reload
                </button>
              </div>
              <p className="status-meta">
                {networkModeLabel(pairing.info)}
                {pairing.info.fromEnv ? " · token env override" : ""}
              </p>
              <p className="pairing-detail">{networkModeDetail(pairing.info)}</p>
            </div>

            <div className="pairing-token-block">
              <p className="pairing-subtitle">Token</p>
              <p className="pairing-token" aria-live="polite">
                {tokenVisible
                  ? pairing.info.token
                  : maskToken(pairing.info.token)}
              </p>
              <div className="pairing-actions">
                <button
                  type="button"
                  className="retry"
                  onClick={() => setTokenVisible((v) => !v)}
                >
                  {tokenVisible ? "Hide" : "Show"}
                </button>
                <button
                  type="button"
                  className="retry"
                  onClick={() => void onCopyToken()}
                >
                  Copy
                </button>
                <button
                  type="button"
                  className="retry"
                  onClick={() => setQrVisible((v) => !v)}
                >
                  {qrVisible ? "Hide QR" : "Show QR"}
                </button>
              </div>
              {qrVisible && (
                <div
                  className="pairing-qr"
                  role="img"
                  aria-label="QR code for pairing token"
                  dangerouslySetInnerHTML={{ __html: pairing.info.qrSvg }}
                />
              )}
            </div>

            {copyNote && <p className="status-meta">{copyNote}</p>}
          </>
        )}
      </section>
    </main>
  );
}

function App() {
  if (isDashboardSurface()) {
    return <Dashboard />;
  }
  return <MenubarShell />;
}

export default App;
