import { useEffect, useState } from "react";
import {
  fetchFeatureSnapshot,
  formatFeatureValue,
  loadingView,
  type SnapshotView,
} from "./featureSnapshot";

/** Soft refresh — idle-safe; no busy-loop. */
const SNAPSHOT_POLL_MS = 30_000;

function ChartSlot({ empty }: { empty: boolean }) {
  return (
    <section className="chart-slot" aria-label="Feature charts">
      <p className="chart-slot-title">Features</p>
      <div className="chart-slot-body">
        <p className="chart-slot-placeholder">
          {empty
            ? "Chart area — series arrive in a later update."
            : "Chart area reserved for Feature series."}
        </p>
      </div>
    </section>
  );
}

function InsightsSlot() {
  return (
    <section className="insights-slot" aria-label="Insights">
      <p className="chart-slot-title">Insights</p>
      <p className="status-meta">Insights will appear here later.</p>
    </section>
  );
}

export function Dashboard() {
  const [view, setView] = useState<SnapshotView>(() => loadingView());
  const [busy, setBusy] = useState(true);

  useEffect(() => {
    let cancelled = false;

    const load = async (isFirst: boolean) => {
      if (isFirst) {
        setBusy(true);
        setView(loadingView());
      }
      const next = await fetchFeatureSnapshot();
      if (cancelled) {
        return;
      }
      setView(next);
      if (isFirst) {
        setBusy(false);
      }
    };

    void load(true);
    const timer = window.setInterval(() => {
      void load(false);
    }, SNAPSHOT_POLL_MS);

    return () => {
      cancelled = true;
      window.clearInterval(timer);
    };
  }, []);

  const onRetry = () => {
    setBusy(true);
    setView(loadingView());
    void fetchFeatureSnapshot().then((next) => {
      setView(next);
      setBusy(false);
    });
  };

  const features = view.snapshot?.features ?? [];
  const signals = view.snapshot?.signals ?? [];
  const isEmpty = view.kind === "empty";
  const isReady = view.kind === "ready";

  return (
    <main className="shell shell--dashboard" data-snapshot={view.kind}>
      <header className="brand brand--dashboard">
        <h1>BioFocus</h1>
        <p className="brand-sub">Dashboard</p>
      </header>

      <section className="status-block" aria-live="polite">
        <div className="status-row">
          <span
            className={`status-dot status-dot--${
              view.kind === "error"
                ? "error"
                : view.kind === "loading"
                  ? "idle"
                  : "ready"
            }`}
            aria-hidden
          />
          <p className="status-label">
            {busy && view.kind === "loading" ? "Loading" : view.label}
          </p>
        </div>
        <p className="status-detail">{view.detail}</p>
      </section>

      {view.kind === "error" && (
        <button
          type="button"
          className="retry"
          onClick={onRetry}
          disabled={busy}
        >
          Try again
        </button>
      )}

      <ChartSlot empty={isEmpty || view.kind === "loading" || view.kind === "error"} />

      {isReady && features.length > 0 && (
        <section className="feature-list" aria-label="Feature snapshot">
          <p className="chart-slot-title">Snapshot</p>
          <ul className="feature-rows">
            {features.map((f) => (
              <li key={`${f.featureId}-${f.timeWindow.end}`} className="feature-row">
                <span className="feature-id">{f.featureId}</span>
                <span className="feature-value">{formatFeatureValue(f.value)}</span>
              </li>
            ))}
          </ul>
          {signals.length > 0 && (
            <p className="status-meta">
              Signals: {signals.map((s) => s.type).join(", ")}
            </p>
          )}
        </section>
      )}

      <InsightsSlot />
    </main>
  );
}
