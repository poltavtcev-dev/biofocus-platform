import { invoke } from "@tauri-apps/api/core";
import type { FeatureDto } from "./featureSnapshot";

/** Closed-set chart ranges (ADR-018). */
export const CHART_RANGES = ["1h", "8h", "12h", "1d", "1w"] as const;

export type ChartRange = (typeof CHART_RANGES)[number];

export type FeatureSeriesDto = {
  range: string;
  stepSecs: number;
  window: { start: number; end: number };
  features: FeatureDto[];
};

export type SeriesView =
  | { kind: "loading"; label: string; detail: string; series: null }
  | { kind: "empty"; label: string; detail: string; series: FeatureSeriesDto }
  | { kind: "ready"; label: string; detail: string; series: FeatureSeriesDto }
  | { kind: "error"; label: string; detail: string; series: null };

export function loadingSeriesView(): SeriesView {
  return {
    kind: "loading",
    label: "Loading",
    detail: "Loading Feature series…",
    series: null,
  };
}

export function isChartRange(value: string): value is ChartRange {
  return (CHART_RANGES as readonly string[]).includes(value);
}

export function rangeLabel(range: ChartRange): string {
  switch (range) {
    case "1h":
      return "1h";
    case "8h":
      return "8h";
    case "12h":
      return "12h";
    case "1d":
      return "1d";
    case "1w":
      return "1w";
  }
}

/**
 * Fetch recompute-on-read Feature series for a chart range.
 * Soft UI states only — never invents points.
 */
export async function fetchFeatureSeries(range: ChartRange): Promise<SeriesView> {
  const mocked = mockSeriesFromLocation(range);
  if (mocked) {
    return mocked;
  }
  try {
    const series = await invoke<FeatureSeriesDto>("get_feature_series", {
      range,
    });
    if (!series || !Array.isArray(series.features)) {
      return {
        kind: "empty",
        label: "No series",
        detail: "No Feature series for this range yet.",
        series: {
          range,
          stepSecs: 60,
          window: { start: 0, end: 0 },
          features: [],
        },
      };
    }
    if (series.features.length === 0) {
      return {
        kind: "empty",
        label: "No series",
        detail: "No Feature series for this range yet.",
        series,
      };
    }
    return {
      kind: "ready",
      label: "Series",
      detail: `Range ${series.range}`,
      series,
    };
  } catch (err) {
    const message =
      err instanceof Error
        ? err.message
        : typeof err === "string"
          ? err
          : "Could not load Feature series.";
    return {
      kind: "error",
      label: "Unavailable",
      detail: message,
      series: null,
    };
  }
}

/** QA: `?mockSeries=ready` draws synthetic smooth series without Core (layout / screenshots). */
export function mockSeriesFromLocation(
  range: ChartRange,
  search: string = typeof window !== "undefined" ? window.location.search : "",
): SeriesView | null {
  if (new URLSearchParams(search).get("mockSeries") !== "ready") {
    return null;
  }
  const end = 1_700_000_000;
  const n = 48;
  const step = 300;
  const ids: Record<string, (i: number) => number> = {
    FocusScore: (i) => 62 + 14 * Math.sin(i / 6),
    CognitiveLoad: (i) => 45 + 18 * Math.sin(i / 7 + 1),
    StressIndex: (i) => 38 + 8 * Math.cos(i / 9),
    FatigueIndex: (i) => 20 + i * 0.6,
    DeepWorkScore: (i) => 58 + 15 * Math.sin(i / 6 + 0.4),
    AttentionStability: (i) => 80 + 8 * Math.cos(i / 5),
    RecoveryScore: (i) => 66 + 5 * Math.sin(i / 11),
    ContextSwitchRate: (i) => 0.9 + 0.5 * Math.sin(i / 4),
  };
  const features: FeatureDto[] = [];
  for (let i = 0; i < n; i += 1) {
    const t = end - (n - 1 - i) * step;
    for (const [featureId, f] of Object.entries(ids)) {
      features.push({
        featureId,
        timeWindow: { start: t - 900, end: t },
        value: Math.round(f(i) * 10) / 10,
        provenance: [],
        confidence: 0.8,
      });
    }
  }
  return {
    kind: "ready",
    label: "Series",
    detail: `Range ${range}`,
    series: { range, stepSecs: step, window: { start: end - n * step, end }, features },
  };
}
