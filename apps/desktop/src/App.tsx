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
  baseUrlPlaceholder,
  companionStatusCopy,
  copyText,
  fetchPairingToken,
  isPrimaryUrlCopyable,
  rotatePairingToken,
  maskToken,
  networkModeDetail,
  networkModeLabel,
  primaryBaseUrl,
  type PairingView,
} from "./pairing";
import {
  fetchIngestLanPreference,
  setIngestLanPreference,
  type IngestLanPreference,
} from "./ingestLan";
import { LifeEventsBlock } from "./LifeEventsBlock";
import {
  fetchGitWatchedRoots,
  saveGitWatchedRoots,
  shortenRootPath,
  type GitWatchedRootsSaveView,
  type GitWatchedRootsView,
} from "./gitWatchedRoots";
import "./App.css";
import "./theme.css";

function GitRootPath({ path }: { path: string }) {
  const shown = shortenRootPath(path);
  const bits = shown.split("/");
  return (
    <span className="git-roots-path" title={path}>
      {bits.map((bit, i) => (
        <span key={`${i}-${bit}`}>
          {bit}
          {i < bits.length - 1 ? (
            <>
              /<wbr />
            </>
          ) : null}
        </span>
      ))}
    </span>
  );
}

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
  const [lanPref, setLanPref] = useState<IngestLanPreference | null>(null);
  const [lanNote, setLanNote] = useState<string | null>(null);
  const [lanBusy, setLanBusy] = useState(false);
  const [dashNote, setDashNote] = useState<string | null>(null);
  const [gitRoots, setGitRoots] = useState<GitWatchedRootsView>({ kind: "idle" });
  const [gitSave, setGitSave] = useState<GitWatchedRootsSaveView>({
    kind: "idle",
  });
  const [gitDraft, setGitDraft] = useState("");
  const [gitLocalRoots, setGitLocalRoots] = useState<string[]>([]);

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
    void fetchIngestLanPreference().then((next) => {
      if (!cancelled) {
        setLanPref(next);
      }
    });
    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    let cancelled = false;
    setGitRoots({ kind: "loading" });
    void fetchGitWatchedRoots().then((next) => {
      if (cancelled) {
        return;
      }
      setGitRoots(next);
      if (next.kind === "ready") {
        setGitLocalRoots(next.info.roots);
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

  const onRotateToken = async () => {
    if (pairing.kind !== "ready" || pairing.info.fromEnv) {
      return;
    }
    const ok = window.confirm(
      "Rotate the pairing token? The phone will need the new QR before it can sync.",
    );
    if (!ok) {
      return;
    }
    const next = await rotatePairingToken();
    setPairing(next);
    setCopyNote(next.kind === "ready" ? "Token rotated. Scan the QR again." : null);
  };

  const onCopyToken = async () => {
    if (pairing.kind !== "ready") {
      return;
    }
    const ok = await copyText(pairing.info.token);
    setCopyNote(ok ? "Token copied." : "Could not copy.");
  };

  const onCopyBaseUrl = async () => {
    if (pairing.kind !== "ready" || !isPrimaryUrlCopyable(pairing.info)) {
      setCopyNote("Not ready for a phone yet — follow the message above.");
      return;
    }
    const ok = await copyText(primaryBaseUrl(pairing.info));
    setCopyNote(ok ? "Base URL copied." : "Could not copy.");
  };

  const onToggleLanPref = async (enabled: boolean) => {
    if (lanBusy || lanPref?.fromEnv) {
      return;
    }
    setLanBusy(true);
    setLanNote(null);
    const result = await setIngestLanPreference(enabled);
    setLanBusy(false);
    if (!result.ok) {
      setLanNote(result.detail);
      return;
    }
    setLanPref(result.pref);
    // Refresh the status card so "Restart needed" shows immediately.
    void fetchPairingToken().then(setPairing);
    setLanNote(
      result.pref.needsRestart
        ? "Saved. Quit and reopen BioFocus to apply, then press Reload."
        : enabled
          ? "LAN preference saved."
          : "LAN turned off. Quit and reopen BioFocus to stop listening on the network.",
    );
  };

  const onOpenDashboard = () => {
    setDashNote(null);
    void openDashboardWindow().then((ok) => {
      if (!ok) {
        setDashNote("Could not open Dashboard.");
      }
    });
  };

  const onReloadGitRoots = () => {
    setGitRoots({ kind: "loading" });
    setGitSave({ kind: "idle" });
    void fetchGitWatchedRoots().then((next) => {
      setGitRoots(next);
      if (next.kind === "ready") {
        setGitLocalRoots(next.info.roots);
      }
    });
  };

  const onAddGitRoot = () => {
    const next = gitDraft.trim();
    if (!next) {
      return;
    }
    if (gitLocalRoots.includes(next)) {
      setGitDraft("");
      return;
    }
    setGitLocalRoots([...gitLocalRoots, next]);
    setGitDraft("");
    setGitSave({ kind: "idle" });
  };

  const onRemoveGitRoot = (path: string) => {
    setGitLocalRoots(gitLocalRoots.filter((r) => r !== path));
    setGitSave({ kind: "idle" });
  };

  const onSaveGitRoots = () => {
    if (gitSave.kind === "saving") {
      return;
    }
    setGitSave({ kind: "saving" });
    void saveGitWatchedRoots(gitLocalRoots).then((result) => {
      setGitSave(result);
      if (result.kind === "ok") {
        setGitLocalRoots(result.info.roots);
        setGitRoots({ kind: "ready", info: result.info });
      }
    });
  };

  const alertLevel = view.alertLevel ?? "green";
  const gitSaving = gitSave.kind === "saving";

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

      <LifeEventsBlock />

      <section className="git-roots-block" aria-label="Git watched folders">
        <h2 className="pairing-title">Git folders</h2>
        <p className="pairing-detail">
          Folders you choose for your own Git activity. BioFocus keeps only
          coarse event kinds — not paths, remotes, or commit messages in
          Observations.
        </p>
        <p className="status-meta">
          Also set <code>BIOFOCUS_GIT_ACTIVITY=1</code> so the collector can run.
          Empty list → idle (no whole-disk scan).
        </p>

        {gitRoots.kind === "loading" && (
          <p className="status-meta">Loading watched folders…</p>
        )}
        {gitRoots.kind === "error" && (
          <>
            <p className="status-meta">{gitRoots.detail}</p>
            <button type="button" className="retry" onClick={onReloadGitRoots}>
              Try again
            </button>
          </>
        )}

        {(gitRoots.kind === "ready" ||
          gitRoots.kind === "idle" ||
          gitLocalRoots.length > 0) && (
          <>
            <div className="git-roots-add">
              <input
                className="git-roots-input"
                type="text"
                value={gitDraft}
                placeholder="/Users/you/Developer/…"
                aria-label="Folder path to watch"
                onChange={(e) => setGitDraft(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") {
                    e.preventDefault();
                    onAddGitRoot();
                  }
                }}
              />
              <button
                type="button"
                className="retry"
                onClick={onAddGitRoot}
                disabled={!gitDraft.trim() || gitSaving}
              >
                Add
              </button>
            </div>

            {gitLocalRoots.length === 0 ? (
              <p className="status-meta">No folders yet — Git activity stays idle.</p>
            ) : (
              <ul className="git-roots-rows">
                {gitLocalRoots.map((path) => (
                  <li key={path} className="git-roots-row">
                    <GitRootPath path={path} />
                    <button
                      type="button"
                      className="retry"
                      onClick={() => onRemoveGitRoot(path)}
                      disabled={gitSaving}
                    >
                      Remove
                    </button>
                  </li>
                ))}
              </ul>
            )}

            <div className="git-roots-actions">
              <button
                type="button"
                className="retry"
                onClick={onSaveGitRoots}
                disabled={gitSaving}
              >
                Save folders
              </button>
              <button
                type="button"
                className="retry"
                onClick={onReloadGitRoots}
                disabled={gitSaving || gitRoots.kind === "loading"}
              >
                Reload
              </button>
            </div>
            {gitSave.kind === "ok" && (
              <p className="status-meta" aria-live="polite">
                {gitSave.message}
              </p>
            )}
            {gitSave.kind === "error" && (
              <p className="status-meta" aria-live="polite">
                {gitSave.detail}
              </p>
            )}
            {gitSave.kind === "saving" && (
              <p className="status-meta" aria-live="polite">
                Saving…
              </p>
            )}
          </>
        )}
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
            {(() => {
              const st = companionStatusCopy(pairing.info);
              const cls = `companion-card companion-card--${st.tone}`;
              return (
                <div className={cls} role={st.tone === "error" ? "alert" : "status"}>
                  <strong>{st.title}</strong>
                  <p>{st.detail}</p>
                </div>
              );
            })()}

            <div className="pairing-lan-toggle">
              <label className="pairing-lan-label">
                <input
                  type="checkbox"
                  checked={Boolean(lanPref?.persisted)}
                  disabled={lanBusy || Boolean(lanPref?.fromEnv)}
                  onChange={(e) => void onToggleLanPref(e.target.checked)}
                />
                Enable LAN ingest for physical iPhone (opt-in)
              </label>
              {lanPref?.fromEnv && (
                <p className="status-meta">
                  LAN bind is controlled by environment variables for this launch.
                </p>
              )}
              {lanNote && <p className="status-meta">{lanNote}</p>}
            </div>

            <div className="pairing-url-block">
              <p className="pairing-subtitle">Base URL</p>
              <p className="pairing-url" aria-live="polite">
                {baseUrlPlaceholder(pairing.info)}
              </p>
              <div className="pairing-actions">
                <button
                  type="button"
                  className="retry"
                  disabled={!isPrimaryUrlCopyable(pairing.info)}
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
                <button
                  type="button"
                  className="retry"
                  disabled={pairing.info.fromEnv}
                  onClick={() => void onRotateToken()}
                >
                  Rotate token
                </button>
              </div>
              {pairing.info.certFingerprint && (
                <>
                  <p className="pairing-subtitle">Certificate fingerprint</p>
                  <p className="pairing-token" aria-live="polite">
                    {pairing.info.certFingerprint}
                  </p>
                  <p className="pairing-detail">
                    The QR includes this fingerprint. The phone rejects a different certificate.
                  </p>
                </>
              )}
              {qrVisible && (
                <div
                  className="pairing-qr"
                  role="img"
                  aria-label="QR code for pairing"
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
