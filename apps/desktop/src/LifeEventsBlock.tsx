import { useEffect, useRef, useState } from "react";
import {
  fetchRecentLifeEvents,
  formatAbsoluteTime,
  formatRelativeTime,
  LIFE_EVENT_KINDS,
  lifeEventIcon,
  lifeEventLabel,
  logLifeEvent,
  type LifeEventKind,
  type LifeEventsListView,
} from "./lifeEvents";

/** How long a button shows its "Logged" state. */
const LOGGED_FLASH_MS = 1600;
/** Re-render relative times ("5 min ago") — cheap, no IPC. */
const RELATIVE_TICK_MS = 30_000;

type BtnState = { kind: LifeEventKind; phase: "logging" | "logged" | "error" } | null;

/**
 * Menubar "Life events": one-tap logging with instant feedback and a short
 * recent list. Life events are append-only in Core (no delete/undo IPC).
 */
export function LifeEventsBlock() {
  const [list, setList] = useState<LifeEventsListView>({ kind: "loading" });
  const [btn, setBtn] = useState<BtnState>(null);
  const [error, setError] = useState<string | null>(null);
  const [, setTick] = useState(0);
  const flashTimer = useRef<number | undefined>(undefined);

  const reload = () => {
    setList((prev) => (prev.kind === "ready" ? prev : { kind: "loading" }));
    void fetchRecentLifeEvents().then(setList);
  };

  useEffect(() => {
    void fetchRecentLifeEvents().then(setList);
    const tick = window.setInterval(() => setTick((t) => t + 1), RELATIVE_TICK_MS);
    return () => {
      window.clearInterval(tick);
      window.clearTimeout(flashTimer.current);
    };
  }, []);

  const onLog = (kind: LifeEventKind) => {
    if (btn?.phase === "logging") {
      return;
    }
    setError(null);
    setBtn({ kind, phase: "logging" });
    void logLifeEvent(kind).then((result) => {
      window.clearTimeout(flashTimer.current);
      if (result.kind === "ok") {
        setBtn({ kind, phase: "logged" });
        // Optimistic prepend, then confirm with a fresh list.
        setList((prev) =>
          prev.kind === "ready"
            ? { kind: "ready", events: [result.event, ...prev.events.filter((e) => e.id !== result.event.id)] }
            : { kind: "ready", events: [result.event] },
        );
        void fetchRecentLifeEvents().then(setList);
      } else if (result.kind === "error") {
        setBtn({ kind, phase: "error" });
        setError(result.detail);
      }
      flashTimer.current = window.setTimeout(() => setBtn(null), LOGGED_FLASH_MS);
    });
  };

  const events = list.kind === "ready" ? list.events.slice(0, 5) : [];

  return (
    <section className="life-events-block le" aria-label="Life events">
      <div className="le-head">
        <h2 className="pairing-title">Life events</h2>
        <button
          type="button"
          className="icon-btn"
          onClick={reload}
          disabled={list.kind === "loading"}
          aria-label="Refresh recent life events"
          title="Refresh"
        >
          <span aria-hidden>↻</span>
        </button>
      </div>
      <p className="le-help">One tap to note a moment. It shows up in your timeline — never scored.</p>

      <div className="le-grid" role="group" aria-label="Log a life event">
        {LIFE_EVENT_KINDS.map((kind) => {
          const state = btn?.kind === kind ? btn.phase : null;
          return (
            <button
              key={kind}
              type="button"
              className={`le-btn${state ? ` le-btn--${state}` : ""}`}
              disabled={btn?.phase === "logging"}
              onClick={() => onLog(kind)}
              aria-live="polite"
            >
              <span className="le-btn-icon" aria-hidden>
                {state === "logged" ? "✓" : lifeEventIcon(kind)}
              </span>
              <span className="le-btn-label">
                {state === "logging"
                  ? "Saving…"
                  : state === "logged"
                    ? "Logged"
                    : lifeEventLabel(kind)}
              </span>
            </button>
          );
        })}
      </div>
      {error && (
        <p className="le-error" role="alert">
          {error}
        </p>
      )}

      <p className="pairing-subtitle le-recent-title">Recent</p>
      {list.kind === "loading" && <p className="status-meta">Loading…</p>}
      {list.kind === "error" && (
        <p className="status-meta">
          {list.detail}{" "}
          <button type="button" className="link-btn" onClick={reload}>
            Try again
          </button>
        </p>
      )}
      {list.kind === "ready" && events.length === 0 && (
        <div className="le-empty">
          <span aria-hidden>🌱</span>
          <p>Nothing logged yet. Tap a button above after your next coffee or walk.</p>
        </div>
      )}
      {events.length > 0 && (
        <ul className="le-list">
          {events.map((event, i) => (
            <li
              key={event.id}
              className={`le-row${i === 0 && btn?.phase === "logged" ? " le-row--new" : ""}`}
              title={formatAbsoluteTime(event.timestamp)}
            >
              <span className="le-row-icon" aria-hidden>
                {lifeEventIcon(event.kind)}
              </span>
              <span className="le-row-text">
                <span className="le-row-kind">{lifeEventLabel(event.kind)}</span>
                <span className="le-row-abs">{formatAbsoluteTime(event.timestamp)}</span>
              </span>
              <span className="le-row-rel">{formatRelativeTime(event.timestamp)}</span>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
