import { invoke } from "@tauri-apps/api/core";

/** v1 Life Event kinds (ADR-006 / `docs/07-contracts.md`). */
export const LIFE_EVENT_KINDS = ["coffee", "walk", "lunch", "workout"] as const;

export type LifeEventKind = (typeof LIFE_EVENT_KINDS)[number];

/** Mirrors desktop `log_life_event` / `list_recent_life_events` IPC. */
export type LifeEventInfo = {
  id: string;
  kind: string;
  timestamp: number;
  providerId: string;
};

type LifeEventPayload = {
  id?: string;
  kind?: string;
  timestamp?: number;
  providerId?: string;
  provider_id?: string;
};

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
  coffee: "Coffee",
  walk: "Walk",
  lunch: "Lunch",
  workout: "Workout",
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
  if (diff < 45) return "just now";
  const min = Math.round(diff / 60);
  if (min < 60) return `${min} min ago`;
  const h = Math.round(min / 60);
  if (h < 24) return `${h} h ago`;
  const d = Math.round(h / 24);
  if (d === 1) return "yesterday";
  return `${d} days ago`;
}

/** Absolute local time; adds the date when not today. */
export function formatAbsoluteTime(timestamp: number, now = new Date()): string {
  try {
    const d = new Date(timestamp * 1000);
    const sameDay = d.toDateString() === now.toDateString();
    const time = d.toLocaleTimeString(undefined, { hour: "2-digit", minute: "2-digit" });
    if (sameDay) return `Today, ${time}`;
    const date = d.toLocaleDateString(undefined, { weekday: "short", day: "numeric", month: "short" });
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
  return `Logged ${lifeEventLabel(kind).toLowerCase()}.`;
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
  return {
    id,
    kind,
    timestamp,
    providerId: providerId || "com.biofocus.desktop",
  };
}

let mockStore: LifeEventInfo[] | null = null;

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
    const rows: [LifeEventKind, number][] = [
      ["coffee", 5 * 60],
      ["walk", 52 * 60],
      ["lunch", 3 * 3600],
      ["coffee", 5 * 3600],
      ["workout", 26 * 3600],
      ["walk", 2 * 86400],
    ];
    mockStore = rows.map(([kind, ago], i) => ({
      id: `mock-${i}`,
      kind,
      timestamp: now - ago,
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
    return { kind: "error", detail: "Could not load recent life events." };
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
      return { kind: "error", detail: "Could not load recent life events." };
    }
    const events = payload
      .map(normalize)
      .filter((row): row is LifeEventInfo => row !== null);
    return { kind: "ready", events };
  } catch (err) {
    const detail =
      typeof err === "string" && err.trim()
        ? err.trim()
        : "Could not load recent life events.";
    return { kind: "error", detail };
  }
}

/** Log one v1 Life Event via IPC. Mock mode returns a synthetic row. */
export async function logLifeEvent(
  kind: LifeEventKind,
): Promise<LogLifeEventView> {
  if (mockErrorFromQuery()) {
    return { kind: "error", detail: "Could not log that event." };
  }
  const mocked = mockFromQuery();
  if (mocked) {
    const event: LifeEventInfo = {
      id: `mock-${kind}-${Date.now()}`,
      kind,
      timestamp: Math.floor(Date.now() / 1000),
      providerId: "com.biofocus.desktop",
    };
    mocked.unshift(event);
    return { kind: "ok", event, message: loggedMessage(kind) };
  }

  try {
    const payload = await invoke<LifeEventPayload>("log_life_event", { kind });
    const event = normalize(payload);
    if (!event) {
      return { kind: "error", detail: "Could not log that event." };
    }
    return { kind: "ok", event, message: loggedMessage(event.kind) };
  } catch (err) {
    const detail =
      typeof err === "string" && err.trim()
        ? err.trim()
        : "Could not log that event.";
    return { kind: "error", detail };
  }
}

/** Local time for a Unix-seconds timestamp (Menubar meta). */
export function formatLifeEventTime(timestamp: number): string {
  try {
    return new Date(timestamp * 1000).toLocaleTimeString(undefined, {
      hour: "2-digit",
      minute: "2-digit",
    });
  } catch {
    return "";
  }
}
