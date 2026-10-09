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
  "DeepWorkScore",
  "AttentionStability",
  "DeskAwayPresence",
  "CircadianOffset",
  "SustainedLoadIndicator",
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
  "DeepWorkScore",
  "AttentionStability",
  "DeskAwayPresence",
  "CircadianOffset",
  "SustainedLoadIndicator",
];

/** Calm UI labels — Feature names only, no evaluative / medical claims. */
export const CHART_SERIES_META: Record<
  ChartFeatureId,
  { label: string; unit: string; color: string }
> = {
  FocusScore: {
    label: "Фокус",
    unit: "0–100",
    color: "#5b6b8c",
  },
  StressIndex: {
    label: "Индекс напряжения",
    unit: "0–100",
    color: "#8a7a5c",
  },
  FatigueIndex: {
    label: "Усталость",
    unit: "0–100",
    color: "#6e7a86",
  },
  ContextSwitchRate: {
    label: "Переключения",
    unit: "в минуту",
    color: "#7a8a7a",
  },
  ActivityBalance: {
    label: "Движение",
    unit: "0–100",
    color: "#6a8f7a",
  },
  EnergyScore: {
    label: "Энергия",
    unit: "0–100",
    color: "#8c7a5b",
  },
  SleepDebt: {
    label: "Недосып",
    unit: "0–100",
    color: "#6b7088",
  },
  RecoveryScore: {
    label: "Восстановление",
    unit: "0–100",
    color: "#7a6b8c",
  },
  CognitiveLoad: {
    label: "Общая нагрузка",
    unit: "0–100",
    color: "#6b7a8a",
  },
  DeepWorkScore: {
    label: "Глубокая работа",
    unit: "0–100",
    color: "#5c7a6b",
  },
  AttentionStability: {
    label: "Устойчивость внимания",
    unit: "0–100",
    color: "#5b7a8c",
  },
  DeskAwayPresence: {
    label: "Вдали от стола",
    unit: "0–100",
    color: "#7a6b5c",
  },
  CircadianOffset: {
    label: "Совпадение с ритмом",
    unit: "0–100",
    color: "#5c6b7a",
  },
  SustainedLoadIndicator: {
    label: "Длительная нагрузка",
    unit: "0–100",
    color: "#8a6b5c",
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
  DeepWorkScore?: number;
  AttentionStability?: number;
  DeskAwayPresence?: number;
  CircadianOffset?: number;
  SustainedLoadIndicator?: number;
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

/** Distinct, calm series colours for dark + light surfaces (design system v2). */
export const SERIES_COLORS: Partial<Record<ChartFeatureId, string>> = {
  FocusScore: "#8f9bff",
  CognitiveLoad: "#d9ad62",
  StressIndex: "#e08c8c",
  ContextSwitchRate: "#6fc3c9",
  DeepWorkScore: "#6fbf98",
  RecoveryScore: "#b493e6",
  FatigueIndex: "#c9a27a",
  AttentionStability: "#7fb1e8",
  EnergyScore: "#e6c25c",
  ActivityBalance: "#8cc76f",
  SleepDebt: "#9aa0c8",
  DeskAwayPresence: "#a3a8b3",
  CircadianOffset: "#d39bc4",
  SustainedLoadIndicator: "#d08a6a",
};

export function seriesColor(id: ChartFeatureId): string {
  return SERIES_COLORS[id] ?? CHART_SERIES_META[id].color;
}

/** Shown by default (first up to 3 present, in this order). */
export const DEFAULT_SERIES_ORDER: ChartFeatureId[] = [
  "FocusScore",
  "CognitiveLoad",
  "StressIndex",
  "DeepWorkScore",
  "RecoveryScore",
];

export const MAX_DEFAULT_SERIES = 3;

export function defaultSelectedSeries(present: ChartFeatureId[]): ChartFeatureId[] {
  const picks = DEFAULT_SERIES_ORDER.filter((id) => present.includes(id)).slice(
    0,
    MAX_DEFAULT_SERIES,
  );
  return picks.length > 0 ? picks : present.slice(0, MAX_DEFAULT_SERIES);
}
