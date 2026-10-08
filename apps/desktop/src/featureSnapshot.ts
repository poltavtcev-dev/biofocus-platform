import { invoke } from "@tauri-apps/api/core";

/** Wire Feature from `get_feature_snapshot` (`docs/09-api.md`). */
export type FeatureDto = {
  featureId: string;
  timeWindow: { start: number; end: number };
  value: number | Record<string, unknown>;
  provenance: string[];
  /** Data-quality confidence 0–1 (not clinical). */
  confidence?: number;
  /** "Why this value" factors; shares sum to 1. */
  factors?: { id: string; label: string; share: number }[];
};

/** Wire Signal from `get_feature_snapshot`. */
export type SignalDto = {
  id: string;
  type: string;
  timestampStart: number;
  timestampEnd: number;
  severity: string;
};

export type FeatureSnapshotDto = {
  features: FeatureDto[];
  signals: SignalDto[];
};

export type SnapshotSource = "get_feature_snapshot" | "mock";

/** Calm Dashboard shell states (non-evaluative). */
export type SnapshotViewKind = "loading" | "empty" | "ready" | "error";

export type SnapshotView = {
  kind: SnapshotViewKind;
  label: string;
  detail: string;
  source: SnapshotSource;
  /** Present when kind is ready (and optionally empty for layout). */
  snapshot?: FeatureSnapshotDto;
};

const COPY: Record<
  Exclude<SnapshotViewKind, "loading">,
  { label: string; detail: string }
> = {
  empty: {
    label: "No metrics yet",
    detail: "No data yet. Keep BioFocus running and use your Mac normally — metrics appear after a few minutes.",
  },
  ready: {
    label: "Metrics available",
    detail: "Latest values from the last 15 minutes.",
  },
  error: {
    label: "Could not load",
    detail: "Could not load metrics from the local engine. Try again; if it persists, restart BioFocus.",
  },
};

export function snapshotView(
  kind: Exclude<SnapshotViewKind, "loading">,
  source: SnapshotSource,
  snapshot?: FeatureSnapshotDto,
): SnapshotView {
  const copy = COPY[kind];
  return {
    kind,
    label: copy.label,
    detail: copy.detail,
    source,
    snapshot,
  };
}

export function loadingView(): SnapshotView {
  return {
    kind: "loading",
    label: "Loading",
    detail: "Fetching Feature snapshot…",
    source: "get_feature_snapshot",
  };
}

/** QA: `?mockSnapshot=empty|ready|error` forces a Dashboard state without Core. */
export function mockSnapshotFromLocation(
  search: string = typeof window !== "undefined" ? window.location.search : "",
): SnapshotView | null {
  const raw = new URLSearchParams(search).get("mockSnapshot");
  if (raw === "empty") {
    return snapshotView("empty", "mock", { features: [], signals: [] });
  }
  if (raw === "error") {
    return snapshotView("error", "mock");
  }
  if (raw === "ready") {
    // Multi-window series for Recharts smoke (P4-E1-T3). Values are synthetic.
    const base = 1_700_000_000;
    const windows = [0, 60, 120, 180, 240].map((offset) => ({
      start: base + offset - 900,
      end: base + offset,
    }));
    const focus = [68, 71, 74, 72.5, 76];
    const stress = [38, 41, 44, 42, 40];
    const fatigue = [28, 30, 33, 35, 36];
    const csr = [0.8, 1.1, 0.9, 1.2, 1.0];
    const cognitive = [42, 45, 48, 46, 44];
    const deepWork = [70, 72, 75, 73, 74];
    const attentionStability = [88, 90, 86, 92, 91];
    const deskAway = [0, 55, 72, 40, 65];
    const circadian = [82, 80, 78, 85, 84];
    const sustainedLoad = [48, 51, 54, 52, 55];
    const features: FeatureDto[] = [];
    for (let i = 0; i < windows.length; i += 1) {
      const tw = windows[i];
      features.push(
        {
          featureId: "FocusScore",
          timeWindow: tw,
          value: focus[i],
          provenance: ["00000000-0000-0000-0000-000000000001"],
        },
        {
          featureId: "StressIndex",
          timeWindow: tw,
          value: stress[i],
          // QA: one thin metric so the "rough" marking is visible in mocks.
          confidence: 0.35,
          provenance: ["00000000-0000-0000-0000-000000000002"],
        },
        {
          featureId: "FatigueIndex",
          timeWindow: tw,
          value: fatigue[i],
          provenance: ["00000000-0000-0000-0000-000000000003"],
        },
        {
          featureId: "ContextSwitchRate",
          timeWindow: tw,
          value: csr[i],
          provenance: ["00000000-0000-0000-0000-000000000004"],
        },
        {
          featureId: "CognitiveLoad",
          timeWindow: tw,
          value: cognitive[i],
          provenance: ["00000000-0000-0000-0000-00000000000a"],
        },
        {
          featureId: "DeepWorkScore",
          timeWindow: tw,
          value: deepWork[i],
          provenance: ["00000000-0000-0000-0000-00000000000b"],
        },
        {
          featureId: "AttentionStability",
          timeWindow: tw,
          value: attentionStability[i],
          provenance: ["00000000-0000-0000-0000-00000000000c"],
        },
        {
          featureId: "DeskAwayPresence",
          timeWindow: tw,
          value: deskAway[i],
          provenance: ["00000000-0000-0000-0000-00000000000d"],
        },
        {
          featureId: "CircadianOffset",
          timeWindow: tw,
          value: circadian[i],
          provenance: ["00000000-0000-0000-0000-00000000000e"],
        },
        {
          featureId: "SustainedLoadIndicator",
          timeWindow: tw,
          value: sustainedLoad[i],
          provenance: ["00000000-0000-0000-0000-00000000000f"],
        },
      );
    }
    return snapshotView("ready", "mock", {
      features,
      signals: [
        {
          id: "00000000-0000-0000-0000-000000000009",
          type: "High_Stress",
          timestampStart: base + 180,
          timestampEnd: base + 540,
          severity: "high",
        },
      ],
    });
  }
  return null;
}

type WireSnapshot = {
  features?: unknown;
  signals?: unknown;
};

function asFeature(raw: unknown): FeatureDto | null {
  if (!raw || typeof raw !== "object") {
    return null;
  }
  const o = raw as Record<string, unknown>;
  const featureId = o.featureId ?? o.feature_id;
  const tw = o.timeWindow ?? o.time_window;
  if (typeof featureId !== "string" || !tw || typeof tw !== "object") {
    return null;
  }
  const window = tw as Record<string, unknown>;
  const start = window.start;
  const end = window.end;
  if (typeof start !== "number" || typeof end !== "number") {
    return null;
  }
  const provenance = Array.isArray(o.provenance)
    ? o.provenance.filter((id): id is string => typeof id === "string")
    : [];
  const value = o.value;
  if (typeof value !== "number" && (typeof value !== "object" || value === null)) {
    return null;
  }
  return {
    featureId,
    timeWindow: { start, end },
    value: value as number | Record<string, unknown>,
    provenance,
  };
}

function asSignal(raw: unknown): SignalDto | null {
  if (!raw || typeof raw !== "object") {
    return null;
  }
  const o = raw as Record<string, unknown>;
  const id = o.id;
  const type = o.type;
  const timestampStart = o.timestampStart ?? o.timestamp_start;
  const timestampEnd = o.timestampEnd ?? o.timestamp_end;
  const severity = o.severity;
  if (
    typeof id !== "string" ||
    typeof type !== "string" ||
    typeof timestampStart !== "number" ||
    typeof timestampEnd !== "number" ||
    typeof severity !== "string"
  ) {
    return null;
  }
  return { id, type, timestampStart, timestampEnd, severity };
}

function parseSnapshot(payload: WireSnapshot): FeatureSnapshotDto {
  const features = Array.isArray(payload.features)
    ? payload.features.map(asFeature).filter((f): f is FeatureDto => f !== null)
    : [];
  const signals = Array.isArray(payload.signals)
    ? payload.signals.map(asSignal).filter((s): s is SignalDto => s !== null)
    : [];
  return { features, signals };
}

/**
 * Loads Feature snapshot via IPC only (`get_feature_snapshot`).
 * Empty cache → empty state; invoke failure → error. No busy-loop.
 */
export async function fetchFeatureSnapshot(): Promise<SnapshotView> {
  const mocked = mockSnapshotFromLocation();
  if (mocked) {
    return mocked;
  }

  try {
    const payload = await invoke<WireSnapshot>("get_feature_snapshot");
    const snapshot = parseSnapshot(payload ?? {});
    if (snapshot.features.length === 0 && snapshot.signals.length === 0) {
      return snapshotView("empty", "get_feature_snapshot", snapshot);
    }
    return snapshotView("ready", "get_feature_snapshot", snapshot);
  } catch {
    return snapshotView("error", "get_feature_snapshot");
  }
}

/** Format a Feature value for shell list (no charts). */
export function formatFeatureValue(value: number | Record<string, unknown>): string {
  if (typeof value === "number") {
    return Number.isInteger(value) ? String(value) : value.toFixed(1);
  }
  return formatVitalObject(value);
}

function finite(value: unknown): number | null {
  return typeof value === "number" && Number.isFinite(value) ? value : null;
}

function signed(n: number): string {
  const body = Number.isInteger(n) ? String(n) : n.toFixed(1);
  return n > 0 ? `+${body}` : body;
}

function formatHours(mins: number): string {
  const h = Math.floor(mins / 60);
  const m = Math.round(mins % 60);
  if (h <= 0) return `${m}m`;
  return `${h}h ${m}m`;
}

function formatVitalObject(value: Record<string, unknown>): string {
  const bpm = finite(value.bpm);
  if (bpm != null && value.method == null && value.mean_percent == null) {
    const delta = finite(value.delta_bpm);
    const text = `${Math.round(bpm)} bpm`;
    return delta == null ? text : `${text} (${signed(delta)} vs usual)`;
  }
  const ms = finite(value.ms);
  if ((value.method === "sdnn" || value.method === "rmssd") && ms != null) {
    const label = value.method === "sdnn" ? "SDNN" : "RMSSD";
    const delta = finite(value.delta_ms);
    const text = `${label} ${Math.round(ms)} ms`;
    return delta == null ? text : `${text} (${signed(delta)} vs usual)`;
  }
  const total = finite(value.total_min);
  if (value.stages === false && total != null) {
    return `${formatHours(total)} · total sleep only`;
  }
  if (value.stages === true && total != null) {
    const parts = [formatHours(total)];
    const deep = finite(value.deep_share);
    const rem = finite(value.rem_share);
    const core = finite(value.core_share);
    if (deep != null) parts.push(`deep ${Math.round(deep * 100)}%`);
    if (rem != null) parts.push(`REM ${Math.round(rem * 100)}%`);
    if (core != null) parts.push(`core ${Math.round(core * 100)}%`);
    return parts.join(" · ");
  }
  const meanPct = finite(value.mean_percent);
  const minPct = finite(value.min_percent);
  if (meanPct != null && minPct != null) {
    return `mean ${Math.round(meanPct)}% · min ${Math.round(minPct)}%`;
  }
  const meanBreath = finite(value.mean_per_min);
  const minBreath = finite(value.min_per_min);
  if (meanBreath != null && minBreath != null) {
    return `mean ${meanBreath.toFixed(1)} · min ${minBreath.toFixed(1)}`;
  }
  const celsius = finite(value.celsius);
  if (celsius != null) {
    const delta = finite(value.delta_celsius);
    const text = `${signed(celsius)} °C`;
    return delta == null ? text : `${text} (${signed(delta)} vs usual)`;
  }
  try {
    return JSON.stringify(value);
  } catch {
    return "—";
  }
}
