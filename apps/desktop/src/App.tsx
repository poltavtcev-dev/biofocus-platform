import { useEffect, useState } from "react";
import { TrayIcon } from "@tauri-apps/api/tray";
import {
  fetchCoreStatus,
  statusView,
  trayTooltipFor,
  type CoreStatusView,
} from "./coreStatus";
import {
  copyText,
  fetchPairingToken,
  maskToken,
  type PairingView,
} from "./pairing";
import "./App.css";

const TRAY_ID = "main";

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

function App() {
  const [view, setView] = useState<CoreStatusView>(() =>
    statusView("idle", "core_ping"),
  );
  const [busy, setBusy] = useState(true);
  const [pairing, setPairing] = useState<PairingView>({ kind: "idle" });
  const [tokenVisible, setTokenVisible] = useState(false);
  const [qrVisible, setQrVisible] = useState(false);
  const [copyNote, setCopyNote] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;

    const load = async () => {
      setBusy(true);
      const next = await fetchCoreStatus();
      if (cancelled) {
        return;
      }
      setView(next);
      setBusy(false);
      void syncTrayTooltip(next);
    };

    void load();

    return () => {
      cancelled = true;
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
    setCopyNote(ok ? "Copied." : "Could not copy.");
  };

  return (
    <main className="shell" data-status={view.kind}>
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

      {view.kind === "error" && (
        <button type="button" className="retry" onClick={onRetry} disabled={busy}>
          Try again
        </button>
      )}

      <section className="pairing-block" aria-label="Companion pairing">
        <h2 className="pairing-title">Companion</h2>
        <p className="pairing-detail">
          Share this local token with your phone. No cloud account.
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
              <button type="button" className="retry" onClick={() => void onCopyToken()}>
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
            {copyNote && <p className="status-meta">{copyNote}</p>}
            {qrVisible && (
              <div
                className="pairing-qr"
                role="img"
                aria-label="QR code for pairing token"
                dangerouslySetInnerHTML={{ __html: pairing.info.qrSvg }}
              />
            )}
            <p className="status-meta">
              {pairing.info.ingestBaseUrl}
              {pairing.info.fromEnv ? " · env override" : ""}
            </p>
          </>
        )}
      </section>
    </main>
  );
}

export default App;
