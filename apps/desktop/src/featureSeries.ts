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
