import { invoke } from "@tauri-apps/api/core";

/** v1 Life Event kinds (ADR-006 / `docs/07-contracts.md`). */
export const LIFE_EVENT_KINDS = ["coffee", "walk", "lunch", "workout"] as const;

export type LifeEventKind = (typeof LIFE_EVENT_KINDS)[number];

/**
 * Mirrors desktop `log_life_event` / `list_recent_life_events` IPC.
 * `timestamp` = when it happened (what the data uses); `loggedAt` = when tapped.
 */
export type LifeEventInfo = {
  id: string;
  kind: string;
  timestamp: number;
  loggedAt: number;
  edited: boolean;
  providerId: string;
};

type LifeEventPayload = {
  id?: string;
  kind?: string;
  timestamp?: number;
  loggedAt?: number;
  edited?: boolean;
  providerId?: string;
  provider_id?: string;
};

/** "When did it happen?" presets (minutes before now). */
export const BACKDATE_OPTIONS = [
  { minutes: 0, label: "Сейчас" },
  { minutes: 15, label: "15 мин" },
  { minutes: 30, label: "30 мин" },
  { minutes: 60, label: "1 ч" },
] as const;

/** Re-time presets for an existing row (minutes before now). */
export const RETIME_OPTIONS = [
  { minutes: 15, label: "15 мин назад" },
  { minutes: 30, label: "30 мин назад" },
  { minutes: 60, label: "1 ч назад" },
  { minutes: 120, label: "2 ч назад" },
] as const;

export type ActionResult<T> = { ok: true; value: T } | { ok: false; detail: string };

export type LifeEventsListView =
  | { kind: "idle" }
  | { kind: "loading" }
  | { kind: "ready"; events: LifeEventInfo[] }
  | { kind: "error"; detail: string };

export type LogLifeEventView =
  | { kind: "idle" }
  | { kind: "logging"; eventKind: LifeEventKind }
  | { kind: "ok"; event: LifeEventInfo; message: string }
  | { kind: "error"; detail: string };

const LABELS: Record<LifeEventKind, string> = {
  coffee: "Кофе",
  walk: "Прогулка",
  lunch: "Обед",
  workout: "Тренировка",
};

const ICONS: Record<LifeEventKind, string> = {
  coffee: "☕",
  walk: "🚶",
  lunch: "🍽️",
  workout: "🏋️",
};

/** Simple emoji icon for a kind (decorative; label is always shown too). */
export function lifeEventIcon(kind: string): string {
  if ((LIFE_EVENT_KINDS as readonly string[]).includes(kind)) {
    return ICONS[kind as LifeEventKind];
  }
  return "•";
}

/** How many recent events the panel shows. */
export const RECENT_LIMIT = 5;

/** "just now", "5 min ago", "2 h ago", "yesterday", "3 days ago". */
export function formatRelativeTime(timestamp: number, nowSecs = Date.now() / 1000): string {
  const diff = Math.max(0, Math.round(nowSecs - timestamp));
  if (diff < 45) return "только что";
  const min = Math.round(diff / 60);
  if (min < 60) return `${min} мин назад`;
  const h = Math.round(min / 60);
  if (h < 24) return `${h} ч назад`;
  const d = Math.round(h / 24);
  if (d === 1) return "вчера";
  return `${d} дн. назад`;
}

/** Absolute local time; adds the date when not today. */
export function formatAbsoluteTime(timestamp: number, now = new Date()): string {
  try {
    const d = new Date(timestamp * 1000);
    const sameDay = d.toDateString() === now.toDateString();
    const time = d.toLocaleTimeString("ru-RU", { hour: "2-digit", minute: "2-digit" });
    if (sameDay) return `Сегодня, ${time}`;
    const date = d.toLocaleDateString("ru-RU", { weekday: "short", day: "numeric", month: "short" });
    return `${date}, ${time}`;
  } catch {
    return "";
  }
}

/** Calm, non-evaluative label for a v1 kind. */
export function lifeEventLabel(kind: string): string {
  if ((LIFE_EVENT_KINDS as readonly string[]).includes(kind)) {
    return LABELS[kind as LifeEventKind];
  }
  return kind;
}

/** Short confirmation after a successful log. */
export function loggedMessage(kind: string): string {
  return `Записано: ${lifeEventLabel(kind).toLowerCase()}.`;
}

/** Local clock time only (e.g. "14:05"). */
export function formatClock(timestamp: number): string {
  try {
    return new Date(timestamp * 1000).toLocaleTimeString("ru-RU", {
      hour: "2-digit",
      minute: "2-digit",
    });
  } catch {
    return "";
  }
}

/** True when the event was back-dated / re-timed by at least a minute. */
export function isBackdated(event: LifeEventInfo): boolean {
  return Math.abs(event.loggedAt - event.timestamp) >= 60;
}

/** Secondary row text: "Today, 14:05" or "Today, 14:05 · logged 14:20". */
export function lifeEventWhen(event: LifeEventInfo, now = new Date()): string {
  const abs = formatAbsoluteTime(event.timestamp, now);
  return isBackdated(event) ? `${abs} · записано ${formatClock(event.loggedAt)}` : abs;
}

function normalize(payload: LifeEventPayload): LifeEventInfo | null {
  const id = payload.id?.trim();
  const kind = payload.kind?.trim();
  const providerId = (
    payload.providerId ??
    payload.provider_id ??
    ""
  ).trim();
  const timestamp = payload.timestamp;
  if (!id || !kind || typeof timestamp !== "number" || !Number.isFinite(timestamp)) {
    return null;
  }
  const loggedAt =
    typeof payload.loggedAt === "number" && Number.isFinite(payload.loggedAt)
      ? payload.loggedAt
      : timestamp;
  return {
    id,
    kind,
    timestamp,
    loggedAt,
    edited: payload.edited === true,
    providerId: providerId || "com.biofocus.desktop",
  };
}

let mockStore: LifeEventInfo[] | null = null;
/** Mock: removed rows (id → row) so "Undo" can restore them. */
const mockRemoved = new Map<string, LifeEventInfo>();
let mockSeq = 0;

function nowSecs(): number {
  return Math.floor(Date.now() / 1000);
}

function sortNewestFirst(rows: LifeEventInfo[]): void {
  rows.sort((a, b) => b.timestamp - a.timestamp || b.id.localeCompare(a.id));
}

function mockFromQuery(): LifeEventInfo[] | null {
  if (typeof window === "undefined") {
    return null;
  }
  const raw = new URLSearchParams(window.location.search).get("mockLifeEvents");
  if (!raw || raw === "error") {
    return null;
  }
  if (mockStore) {
    return mockStore;
  }
  if (raw === "empty") {
    mockStore = [];
    return mockStore;
  }
  if (raw === "ready") {
    const now = Math.floor(Date.now() / 1000);
    // [kind, happened ago, logged ago]
    const rows: [LifeEventKind, number, number][] = [
      ["coffee", 5 * 60, 5 * 60],
      ["walk", 52 * 60, 22 * 60],
      ["lunch", 3 * 3600, 3 * 3600],
      ["coffee", 5 * 3600, 5 * 3600],
      ["workout", 26 * 3600, 26 * 3600],
      ["walk", 2 * 86400, 2 * 86400],
    ];
    mockStore = rows.map(([kind, ago, loggedAgo], i) => ({
      id: `mock-${i}`,
      kind,
      timestamp: now - ago,
      loggedAt: now - loggedAgo,
      edited: false,
      providerId: "com.biofocus.desktop",
    }));
    return mockStore;
  }
  return null;
}

function mockErrorFromQuery(): boolean {
  if (typeof window === "undefined") {
    return false;
  }
  return new URLSearchParams(window.location.search).get("mockLifeEvents") === "error";
}

/** Load recent Life Events via IPC (or QA mock). No busy-loop. */
export async function fetchRecentLifeEvents(
  limit = RECENT_LIMIT,
): Promise<LifeEventsListView> {
  if (mockErrorFromQuery()) {
    return { kind: "error", detail: "Не удалось загрузить недавние события." };
  }
  const mocked = mockFromQuery();
  if (mocked) {
    return { kind: "ready", events: mocked.slice(0, limit) };
  }

  try {
    const payload = await invoke<LifeEventPayload[]>("list_recent_life_events", {
      limit,
    });
    if (!Array.isArray(payload)) {
      return { kind: "error", detail: "Не удалось загрузить недавние события." };
    }
    const events = payload
      .map(normalize)
      .filter((row): row is LifeEventInfo => row !== null);
    return { kind: "ready", events };
  } catch (err) {
    const detail =
      typeof err === "string" && err.trim()
        ? err.trim()
        : "Не удалось загрузить недавние события.";
    return { kind: "error", detail };
  }
}

/**
 * Log one v1 Life Event via IPC. `minutesAgo` back-dates "happened at"
 * (Core keeps `loggedAt` = now). Mock mode returns a synthetic row.
 */
export async function logLifeEvent(
  kind: LifeEventKind,
  minutesAgo = 0,
): Promise<LogLifeEventView> {
  if (mockErrorFromQuery()) {
    return { kind: "error", detail: "Не удалось записать событие." };
  }
  const now = nowSecs();
  const happenedAt = minutesAgo > 0 ? now - minutesAgo * 60 : undefined;
  const mocked = mockFromQuery();
  if (mocked) {
    mockSeq += 1;
    const event: LifeEventInfo = {
      id: `mock-${kind}-${now}-${mockSeq}`,
      kind,
      timestamp: happenedAt ?? now,
      loggedAt: now,
      edited: false,
      providerId: "com.biofocus.desktop",
    };
    mocked.unshift(event);
    sortNewestFirst(mocked);
    return { kind: "ok", event, message: loggedMessage(kind) };
  }

  try {
    const payload = await invoke<LifeEventPayload>(
      "log_life_event",
      happenedAt === undefined ? { kind } : { kind, happenedAt },
    );
    const event = normalize(payload);
    if (!event) {
      return { kind: "error", detail: "Не удалось записать событие." };
    }
    return { kind: "ok", event, message: loggedMessage(event.kind) };
  } catch (err) {
    const detail =
      typeof err === "string" && err.trim()
        ? err.trim()
        : "Не удалось записать событие.";
    return { kind: "error", detail };
  }
}

function errorDetail(err: unknown, fallback: string): string {
  return typeof err === "string" && err.trim() ? err.trim() : fallback;
}

/** Remove (retract) a Life Event. Core appends a marker; nothing is deleted. */
export async function retractLifeEvent(id: string): Promise<ActionResult<string>> {
  const mocked = mockFromQuery();
  if (mocked) {
    const idx = mocked.findIndex((e) => e.id === id);
    if (idx < 0) return { ok: false, detail: "That life event wasn’t found." };
    const [row] = mocked.splice(idx, 1);
    mockRemoved.set(id, row);
    return { ok: true, value: id };
  }
  try {
    await invoke("retract_life_event", { id });
    return { ok: true, value: id };
  } catch (err) {
    return { ok: false, detail: errorDetail(err, "Не удалось убрать событие.") };
  }
}

/** Undo a removal (Core appends a copy with the same times). */
export async function restoreLifeEvent(id: string): Promise<ActionResult<LifeEventInfo>> {
  const mocked = mockFromQuery();
  if (mocked) {
    const row = mockRemoved.get(id);
    if (!row) return { ok: false, detail: "That life event is still logged." };
    mockRemoved.delete(id);
    mockSeq += 1;
    const copy = { ...row, id: `${row.id}-r${mockSeq}`, edited: true };
    mocked.push(copy);
    sortNewestFirst(mocked);
    return { ok: true, value: copy };
  }
  try {
    const payload = await invoke<LifeEventPayload>("restore_life_event", { id });
    const event = normalize(payload);
    return event ? { ok: true, value: event } : { ok: false, detail: "Не удалось вернуть событие." };
  } catch (err) {
    return { ok: false, detail: errorDetail(err, "Не удалось вернуть событие.") };
  }
}

/** Change when an event happened (minutes before now). Keeps `loggedAt`. */
export async function retimeLifeEvent(
  id: string,
  minutesAgo: number,
): Promise<ActionResult<LifeEventInfo>> {
  const happenedAt = nowSecs() - Math.max(0, minutesAgo) * 60;
  const mocked = mockFromQuery();
  if (mocked) {
    const idx = mocked.findIndex((e) => e.id === id);
    if (idx < 0) return { ok: false, detail: "That life event wasn’t found." };
    mockSeq += 1;
    const moved = { ...mocked[idx], id: `${id}-t${mockSeq}`, timestamp: happenedAt, edited: true };
    mocked.splice(idx, 1, moved);
    sortNewestFirst(mocked);
    return { ok: true, value: moved };
  }
  try {
    const payload = await invoke<LifeEventPayload>("retime_life_event", { id, happenedAt });
    const event = normalize(payload);
    return event ? { ok: true, value: event } : { ok: false, detail: "Не удалось изменить время." };
  } catch (err) {
    return { ok: false, detail: errorDetail(err, "Не удалось изменить время.") };
  }
}

/**
 * Life Events with happened-at in `[start, end]` for chart markers.
 * Soft-fails to `[]`. QA: with `?mockLifeEvents=ready` returns a few
 * synthetic events spread across the requested window.
 */
export async function fetchLifeEventsBetween(start: number, end: number): Promise<LifeEventInfo[]> {
  if (!(end > start)) return [];
  if (typeof window !== "undefined") {
    const raw = new URLSearchParams(window.location.search).get("mockLifeEvents");
    if (raw === "ready") {
      const span = end - start;
      const at: [LifeEventKind, number][] = [
        ["coffee", 0.22],
        ["walk", 0.5],
        ["lunch", 0.64],
        ["coffee", 0.83],
      ];
      return at.map(([kind, f], i) => {
        const t = Math.round(start + span * f);
        return { id: `mock-range-${i}`, kind, timestamp: t, loggedAt: t, edited: false, providerId: "com.biofocus.desktop" };
      });
    }
    if (raw) return [];
  }
  try {
    const payload = await invoke<LifeEventPayload[]>("list_life_events_between", {
      start: Math.floor(start),
      end: Math.ceil(end),
    });
    return Array.isArray(payload)
      ? payload.map(normalize).filter((row): row is LifeEventInfo => row !== null)
      : [];
  } catch {
    return [];
  }
}

/**
 * QA only: `?mockLifeEventsUi=actions` pre-opens the Undo toast, a removed row
 * and the re-time picker so screenshots can show them without clicks.
 */
export function mockLifeEventsUiFromLocation(): "actions" | null {
  if (typeof window === "undefined") return null;
  return new URLSearchParams(window.location.search).get("mockLifeEventsUi") === "actions"
    ? "actions"
    : null;
}

/** Local time for a Unix-seconds timestamp (Menubar meta). */
export function formatLifeEventTime(timestamp: number): string {
  try {
    return new Date(timestamp * 1000).toLocaleTimeString("ru-RU", {
      hour: "2-digit",
      minute: "2-digit",
    });
  } catch {
    return "";
  }
}
