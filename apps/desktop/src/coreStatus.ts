import { invoke } from "@tauri-apps/api/core";

/** Neutral Core states for Phase 1 Menubar (non-judgmental copy). */
export type CoreStatusKind = "idle" | "ready" | "error";

/** Menubar traffic-light from Core (`get_status.alertLevel`). UX colors → P3-E3-T3. */
export type AlertLevel = "green" | "yellow" | "red";

export type StatusSource = "get_status" | "core_ping" | "mock";

export type CoreStatusView = {
  kind: CoreStatusKind;
  /** Short label shown in UI / tray: Idle | Ready | Error */
  label: string;
  /** One calm supporting line — no charts, no evaluation of the user. */
  detail: string;
  source: StatusSource;
  /** Optional secondary meta (crate names / version) — never DB paths. */
  meta?: string;
  /** From IPC only; independent of Idle/Ready/Error. */
  alertLevel?: AlertLevel;
};

const COPY: Record<
  CoreStatusKind,
  { label: string; detail: string }
> = {
  idle: {
    label: "Idle",
    detail: "Waiting for Core.",
  },
  ready: {
    label: "Ready",
    detail: "Core is available.",
  },
  error: {
    label: "Error",
    detail: "Could not reach Core.",
  },
};

export function statusView(
  kind: CoreStatusKind,
  source: StatusSource,
  meta?: string,
): CoreStatusView {
  const copy = COPY[kind];
  return {
    kind,
    label: copy.label,
    detail: copy.detail,
    source,
    meta,
  };
}

export function trayTooltipFor(view: CoreStatusView): string {
  const alert = view.alertLevel
    ? ` · ${alertCopy(view.alertLevel).label}`
    : "";
  return `BioFocus — ${view.label}${alert}`;
}

/** Calm, non-evaluative labels for Menubar alert level (P3-E3-T3). */
export function alertCopy(level: AlertLevel): { label: string; detail: string } {
  switch (level) {
    case "green":
      return { label: "Steady", detail: "Load looks steady." };
    case "yellow":
      return { label: "Elevated", detail: "Load is elevated." };
    case "red":
      return { label: "High", detail: "Load is high right now." };
  }
}

/** QA: `?mockAlert=green|yellow|red` forces alert without Core Features. */
export function mockAlertFromLocation(
  search: string = typeof window !== "undefined" ? window.location.search : "",
): AlertLevel | null {
  const raw = new URLSearchParams(search).get("mockAlert");
  if (raw === "green" || raw === "yellow" || raw === "red") {
    return raw;
  }
  return null;
}

/** QA / T4: `?mockStatus=idle|ready|error` forces a state without DB access. */
export function mockStatusFromLocation(
  search: string = typeof window !== "undefined" ? window.location.search : "",
): CoreStatusKind | null {
  const raw = new URLSearchParams(search).get("mockStatus");
  if (raw === "idle" || raw === "ready" || raw === "error") {
    return raw;
  }
  return null;
}

/** Mirrors desktop `get_status` IPC (`docs/09-api.md` § Desktop Tauri IPC). */
type GetStatusPayload = {
  version?: string;
  dbStatus?: string;
  db_status?: string;
  dbError?: string;
  db_error?: string;
  alertLevel?: string;
  alert_level?: string;
};

function parseAlertLevel(raw: string | undefined): AlertLevel | undefined {
  const v = raw?.toLowerCase();
  if (v === "green" || v === "yellow" || v === "red") {
    return v;
  }
  return undefined;
}

type CorePingPayload = {
  status: string;
  runtime: string;
  storage: string;
  schemaVersion: number;
};

function isCommandMissing(err: unknown): boolean {
  const text = err instanceof Error ? err.message : String(err);
  return /not found|unknown command|command.*get_status/i.test(text);
}

async function fromGetStatus(): Promise<CoreStatusView | null> {
  try {
    const payload = await invoke<GetStatusPayload>("get_status");
    const db = payload.dbStatus ?? payload.db_status ?? "";
    const kind: CoreStatusKind =
      db.toLowerCase() === "ok" ? "ready" : "error";
    const versionMeta = payload.version ? `v${payload.version}` : undefined;
    const dbError = payload.dbError ?? payload.db_error;
    // Keep calm primary copy; surface short reason only as meta on error.
    const meta =
      kind === "error" && dbError
        ? [versionMeta, dbError].filter(Boolean).join(" · ")
        : versionMeta;
    const view = statusView(kind, "get_status", meta);
    view.alertLevel = parseAlertLevel(
      payload.alertLevel ?? payload.alert_level,
    );
    return view;
  } catch (err) {
    if (isCommandMissing(err)) {
      return null;
    }
    return statusView("error", "get_status");
  }
}

async function fromCorePing(): Promise<CoreStatusView> {
  try {
    const ping = await invoke<CorePingPayload>("core_ping");
    if (ping.status === "ok") {
      return statusView(
        "ready",
        "core_ping",
        `${ping.runtime} · ${ping.storage} · schema ${ping.schemaVersion}`,
      );
    }
    return statusView("error", "core_ping");
  } catch {
    return statusView("error", "core_ping");
  }
}

/**
 * Loads Core status via IPC only.
 * Prefers `get_status`; falls back to `core_ping` if the command is missing.
 */
export async function fetchCoreStatus(): Promise<CoreStatusView> {
  const mocked = mockStatusFromLocation();
  if (mocked) {
    const view = statusView(mocked, "mock");
    const mockAlert = mockAlertFromLocation();
    if (mockAlert) {
      view.alertLevel = mockAlert;
    }
    return view;
  }

  const fromStatus = await fromGetStatus();
  if (fromStatus) {
    const mockAlert = mockAlertFromLocation();
    if (mockAlert) {
      fromStatus.alertLevel = mockAlert;
    }
    return fromStatus;
  }

  return fromCorePing();
}
