import type { FeatureDto } from "./featureSnapshot";

/** Catalog Feature ids drawn in v1 charts (order = legend). */
export const CHART_FEATURE_IDS = [
  "FocusScore",
  "StressIndex",
  "FatigueIndex",
  "ContextSwitchRate",
  "ActivityBalance",
  "EnergyScore",
  "SleepDebt",
  "RecoveryScore",
  "CognitiveLoad",
] as const;

export type ChartFeatureId = (typeof CHART_FEATURE_IDS)[number];

/** Primary Y-axis series (0–100 catalog scores). */
export const SCORE_SERIES_IDS: ChartFeatureId[] = [
  "FocusScore",
  "StressIndex",
  "FatigueIndex",
  "ActivityBalance",
  "EnergyScore",
  "SleepDebt",
  "RecoveryScore",
  "CognitiveLoad",
];

/** Calm UI labels — Feature names only, no evaluative / medical claims. */
export const CHART_SERIES_META: Record<
  ChartFeatureId,
  { label: string; unit: string; color: string }
> = {
  FocusScore: {
    label: "Focus",
    unit: "0–100",
    color: "#5b6b8c",
  },
  StressIndex: {
    label: "Stress index",
    unit: "0–100",
    color: "#8a7a5c",
  },
  FatigueIndex: {
    label: "Fatigue index",
    unit: "0–100",
    color: "#6e7a86",
  },
  ContextSwitchRate: {
    label: "Context switches",
    unit: "per window min",
    color: "#7a8a7a",
  },
  ActivityBalance: {
    label: "Activity",
    unit: "0–100",
    color: "#6a8f7a",
  },
  EnergyScore: {
    label: "Energy",
    unit: "0–100",
    color: "#8c7a5b",
  },
  SleepDebt: {
    label: "Sleep shortfall",
    unit: "0–100",
    color: "#6b7088",
  },
  RecoveryScore: {
    label: "Recovery",
    unit: "0–100",
    color: "#7a6b8c",
  },
  CognitiveLoad: {
    label: "Combined demand",
    unit: "0–100",
    color: "#6b7a8a",
  },
};

export type ChartPoint = {
  /** Window end (unix seconds) — x-axis. */
  t: number;
  FocusScore?: number;
  StressIndex?: number;
  FatigueIndex?: number;
  ContextSwitchRate?: number;
  ActivityBalance?: number;
  EnergyScore?: number;
  SleepDebt?: number;
  RecoveryScore?: number;
  CognitiveLoad?: number;
};

function scalarValue(value: FeatureDto["value"]): number | null {
  return typeof value === "number" && Number.isFinite(value) ? value : null;
}

function isChartFeatureId(id: string): id is ChartFeatureId {
  return (CHART_FEATURE_IDS as readonly string[]).includes(id);
}

/**
 * Build Recharts rows from Feature series: one point per distinct
 * `timeWindow.end`, merging Features that share that end.
 */
export function buildChartPoints(features: FeatureDto[]): ChartPoint[] {
  const byEnd = new Map<number, ChartPoint>();

  for (const feature of features) {
    if (!isChartFeatureId(feature.featureId)) {
      continue;
    }
    const scalar = scalarValue(feature.value);
    if (scalar === null) {
      continue;
    }
    const end = feature.timeWindow.end;
    const row = byEnd.get(end) ?? { t: end };
    row[feature.featureId] = scalar;
    byEnd.set(end, row);
  }

  return Array.from(byEnd.values()).sort((a, b) => a.t - b.t);
}

/** Which catalog series appear in the built points. */
export function presentSeriesIds(points: ChartPoint[]): ChartFeatureId[] {
  return CHART_FEATURE_IDS.filter((id) =>
    points.some((p) => typeof p[id] === "number"),
  );
}

/** Short time label for axis ticks (local clock). */
export function formatChartTime(unixSecs: number): string {
  const d = new Date(unixSecs * 1000);
  if (Number.isNaN(d.getTime())) {
    return "—";
  }
  return d.toLocaleTimeString(undefined, {
    hour: "2-digit",
    minute: "2-digit",
  });
}
