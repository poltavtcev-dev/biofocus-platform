import { useEffect, useRef, useState } from "react";
import "./lifeEventsActions.css";
import {
  BACKDATE_OPTIONS,
  fetchRecentLifeEvents,
  formatClock,
  formatRelativeTime,
  isBackdated,
  LIFE_EVENT_KINDS,
  lifeEventIcon,
  lifeEventLabel,
  lifeEventWhen,
  logLifeEvent,
  mockLifeEventsUiFromLocation,
  RECENT_LIMIT,
  restoreLifeEvent,
  retimeLifeEvent,
  retractLifeEvent,
  RETIME_OPTIONS,
  type LifeEventInfo,
  type LifeEventKind,
  type LifeEventsListView,
} from "./lifeEvents";

/** How long a button shows its "Logged" state. */
const LOGGED_FLASH_MS = 1600;
/** How long the "Logged … Undo" toast stays. */
const UNDO_TOAST_MS = 6000;
/** How long a removed row offers "Undo" in place. */
const REMOVED_ROW_MS = 10_000;
/** Re-render relative times ("5 min ago") — cheap, no IPC. */
const RELATIVE_TICK_MS = 30_000;

type BtnState = { kind: LifeEventKind; phase: "logging" | "logged" | "error" } | null;
type Toast = { event: LifeEventInfo; phase: "shown" | "busy" | "removed" } | null;

/**
 * Menubar "Life events": one-tap logging (optionally back-dated), an Undo toast
 * for the latest tap, and a recent list with remove / undo / change-time.
 * Core is append-only: remove = retraction marker, nothing is deleted.
 */
export function LifeEventsBlock() {
  const [list, setList] = useState<LifeEventsListView>({ kind: "loading" });
  const [btn, setBtn] = useState<BtnState>(null);
  const [error, setError] = useState<string | null>(null);
  const [minutesAgo, setMinutesAgo] = useState(0);
  const [toast, setToast] = useState<Toast>(null);
  const [removed, setRemoved] = useState<LifeEventInfo[]>([]);
  const [editing, setEditing] = useState<string | null>(null);
  const [busyId, setBusyId] = useState<string | null>(null);
  const [, setTick] = useState(0);
  const flashTimer = useRef<number | undefined>(undefined);
  const toastTimer = useRef<number | undefined>(undefined);
  const qaApplied = useRef(false);

  const refresh = () => fetchRecentLifeEvents().then(setList);

  const reload = () => {
    setList((prev) => (prev.kind === "ready" ? prev : { kind: "loading" }));
    void refresh();
  };

  useEffect(() => {
    void refresh();
    const tick = window.setInterval(() => setTick((t) => t + 1), RELATIVE_TICK_MS);
    return () => {
      window.clearInterval(tick);
      window.clearTimeout(flashTimer.current);
      window.clearTimeout(toastTimer.current);
    };
  }, []);

  // QA screenshot state (?mockLifeEventsUi=actions): toast + removed row + picker.
  useEffect(() => {
    if (qaApplied.current || list.kind !== "ready" || list.events.length < 3) return;
    if (mockLifeEventsUiFromLocation() !== "actions") return;
    qaApplied.current = true;
    const [first, second, third] = list.events;
    setToast({ event: first, phase: "shown" });
    setEditing(second.id);
    void retractLifeEvent(third.id).then((r) => {
      if (r.ok) {
        setRemoved([third]);
        void refresh();
      }
    });
  }, [list]);

  const showToast = (next: Toast, ms = UNDO_TOAST_MS) => {
    window.clearTimeout(toastTimer.current);
    setToast(next);
    if (next) toastTimer.current = window.setTimeout(() => setToast(null), ms);
  };

  const onLog = (kind: LifeEventKind) => {
    if (btn?.phase === "logging") {
      return;
    }
    setError(null);
    setBtn({ kind, phase: "logging" });
    void logLifeEvent(kind, minutesAgo).then((result) => {
      window.clearTimeout(flashTimer.current);
      if (result.kind === "ok") {
        setBtn({ kind, phase: "logged" });
        setMinutesAgo(0);
        showToast({ event: result.event, phase: "shown" });
        // Optimistic insert, then confirm with a fresh list.
        setList((prev) =>
          prev.kind === "ready"
            ? {
                kind: "ready",
                events: [result.event, ...prev.events.filter((e) => e.id !== result.event.id)].sort(
                  (a, b) => b.timestamp - a.timestamp,
                ),
              }
            : { kind: "ready", events: [result.event] },
        );
        void refresh();
      } else if (result.kind === "error") {
        setBtn({ kind, phase: "error" });
        setError(result.detail);
      }
      flashTimer.current = window.setTimeout(() => setBtn(null), LOGGED_FLASH_MS);
    });
  };

  const onToastUndo = () => {
    if (!toast || toast.phase !== "shown") return;
    const { event } = toast;
    setToast({ event, phase: "busy" });
    void retractLifeEvent(event.id).then((r) => {
      if (r.ok) {
        showToast({ event, phase: "removed" }, 2500);
        void refresh();
      } else {
        setError(r.detail);
        setToast(null);
      }
    });
  };

  const onRemove = (event: LifeEventInfo) => {
    setBusyId(event.id);
    setError(null);
    setEditing(null);
    void retractLifeEvent(event.id).then((r) => {
      setBusyId(null);
      if (!r.ok) {
        setError(r.detail);
        return;
      }
      if (toast?.event.id === event.id) setToast(null);
      setRemoved((prev) => [event, ...prev.filter((e) => e.id !== event.id)]);
      window.setTimeout(
        () => setRemoved((prev) => prev.filter((e) => e.id !== event.id)),
        REMOVED_ROW_MS,
      );
      void refresh();
    });
  };

  const onRestore = (event: LifeEventInfo) => {
    setBusyId(event.id);
    setError(null);
    void restoreLifeEvent(event.id).then((r) => {
      setBusyId(null);
      if (!r.ok) {
        setError(r.detail);
        return;
      }
      setRemoved((prev) => prev.filter((e) => e.id !== event.id));
      void refresh();
    });
  };

  const onRetime = (event: LifeEventInfo, minutes: number) => {
    setBusyId(event.id);
    setError(null);
    void retimeLifeEvent(event.id, minutes).then((r) => {
      setBusyId(null);
      if (!r.ok) {
        setError(r.detail);
        return;
      }
      setEditing(null);
      void refresh();
    });
  };

  const events = list.kind === "ready" ? list.events.slice(0, RECENT_LIMIT) : [];
  type Row = { event: LifeEventInfo; removed: boolean };
  const rows: Row[] = [
    ...events.map((event) => ({ event, removed: false })),
    ...removed.map((event) => ({ event, removed: true })),
  ].sort((a, b) => b.event.timestamp - a.event.timestamp);
  const newestId = events[0]?.id;

  return (
    <section className="life-events-block le" aria-label="События дня">
      <div className="le-head">
        <h2 className="pairing-title">События дня</h2>
        <button
          type="button"
          className="icon-btn"
          onClick={reload}
          disabled={list.kind === "loading"}
          aria-label="Обновить недавние события"
          title="Обновить"
        >
          <span aria-hidden>↻</span>
        </button>
      </div>
      <p className="le-help">
        Одно нажатие, чтобы отметить момент. Он появится на графике и в отчёте — без оценки.
      </p>

      <div className="le-when" role="radiogroup" aria-label="Когда это было?">
        <span className="le-when-label">Когда</span>
        {BACKDATE_OPTIONS.map((opt) => (
          <button
            key={opt.minutes}
            type="button"
            role="radio"
            aria-checked={minutesAgo === opt.minutes}
            className={`le-chip${minutesAgo === opt.minutes ? " le-chip--on" : ""}`}
            onClick={() => setMinutesAgo(opt.minutes)}
            title={opt.minutes === 0 ? "Только что" : opt.label}
          >
            {opt.minutes === 0 ? opt.label : `−${opt.label}`}
          </button>
        ))}
      </div>

      <div className="le-grid" role="group" aria-label="Записать событие дня">
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
                  ? "Запись…"
                  : state === "logged"
                    ? "Записано"
                    : lifeEventLabel(kind)}
              </span>
            </button>
          );
        })}
      </div>

      {toast && (
        <div className={`le-toast le-toast--${toast.phase}`} role="status" aria-live="polite">
          <span className="le-toast-text">
            {toast.phase === "removed"
              ? `Убрано: ${lifeEventLabel(toast.event.kind).toLowerCase()}.`
              : `Записано: ${lifeEventLabel(toast.event.kind).toLowerCase()} · ${formatClock(toast.event.timestamp)}`}
          </span>
          {toast.phase !== "removed" && (
            <button
              type="button"
              className="le-toast-undo"
              onClick={onToastUndo}
              disabled={toast.phase === "busy"}
            >
              {toast.phase === "busy" ? "Возвращаем…" : "Отменить"}
            </button>
          )}
        </div>
      )}

      {error && (
        <p className="le-error" role="alert">
          {error}
        </p>
      )}

      <p className="pairing-subtitle le-recent-title">Недавние</p>
      {list.kind === "loading" && <p className="status-meta">Загрузка…</p>}
      {list.kind === "error" && (
        <p className="status-meta">
          {list.detail}{" "}
          <button type="button" className="link-btn" onClick={reload}>
            Ещё раз
          </button>
        </p>
      )}
      {list.kind === "ready" && rows.length === 0 && (
        <div className="le-empty">
          <span aria-hidden>🌱</span>
          <p>Пока ничего не записано. Нажмите кнопку выше после кофе или прогулки.</p>
        </div>
      )}
      {rows.length > 0 && (
        <ul className="le-list">
          {rows.map(({ event, removed: isRemoved }) =>
            isRemoved ? (
              <li key={`rm-${event.id}`} className="le-row le-row--removed">
                <span className="le-row-icon" aria-hidden>
                  {lifeEventIcon(event.kind)}
                </span>
                <span className="le-row-text">
                  <span className="le-row-kind">{lifeEventLabel(event.kind)} убрано</span>
                  <span className="le-row-abs">Скрыто с графика, из наблюдений и отчётов</span>
                </span>
                <button
                  type="button"
                  className="le-row-undo"
                  onClick={() => onRestore(event)}
                  disabled={busyId === event.id}
                >
                  Отменить
                </button>
              </li>
            ) : (
              <li
                key={event.id}
                className={`le-row-wrap${event.id === newestId && btn?.phase === "logged" ? " le-row--new" : ""}`}
              >
                <div className="le-row" title={lifeEventWhen(event)}>
                  <span className="le-row-icon" aria-hidden>
                    {lifeEventIcon(event.kind)}
                  </span>
                  <span className="le-row-text">
                    <span className="le-row-kind">{lifeEventLabel(event.kind)}</span>
                    <span className="le-row-abs">
                      <span className="le-row-rel">{formatRelativeTime(event.timestamp)}</span>
                      {isBackdated(event) ? ` · записано ${formatClock(event.loggedAt)}` : ""}
                      {event.edited ? " · изменено" : ""}
                    </span>
                  </span>
                  <span className="le-row-actions">
                    <button
                      type="button"
                      className={`icon-btn le-act${editing === event.id ? " le-act--on" : ""}`}
                      onClick={() => setEditing(editing === event.id ? null : event.id)}
                      disabled={busyId === event.id}
                      aria-label={`Изменить время: ${lifeEventLabel(event.kind)}`}
                      aria-expanded={editing === event.id}
                      title="Изменить время"
                    >
                      <span aria-hidden>🕑</span>
                    </button>
                    <button
                      type="button"
                      className="icon-btn le-act le-act--remove"
                      onClick={() => onRemove(event)}
                      disabled={busyId === event.id}
                      aria-label={`Убрать: ${lifeEventLabel(event.kind)}`}
                      title="Убрать"
                    >
                      <span aria-hidden>✕</span>
                    </button>
                  </span>
                </div>
                {editing === event.id && (
                  <div className="le-retime" role="group" aria-label="Когда это было?">
                    <span className="le-when-label">Было</span>
                    {RETIME_OPTIONS.map((opt) => (
                      <button
                        key={opt.minutes}
                        type="button"
                        className="le-chip"
                        onClick={() => onRetime(event, opt.minutes)}
                        disabled={busyId === event.id}
                      >
                        {opt.label}
                      </button>
                    ))}
                    <button type="button" className="link-btn le-retime-cancel" onClick={() => setEditing(null)}>
                      Отмена
                    </button>
                  </div>
                )}
              </li>
            ),
          )}
        </ul>
      )}
    </section>
  );
}
