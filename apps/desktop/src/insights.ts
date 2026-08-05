import { invoke } from "@tauri-apps/api/core";

/** Evidence ref from `get_insights` (`docs/09-api.md`). */
export type EvidenceRefDto = {
  kind: "feature" | "signal" | string;
  id: string;
};

/** Wire Insight from `get_insights`. */
export type InsightDto = {
  id: string;
  title: string;
  description: string;
  category: string;
  evidenceList: EvidenceRefDto[];
  actionRecommendation?: string;
};

export type InsightsDto = {
  insights: InsightDto[];
};

export type InsightsSource = "get_insights" | "mock";

/** Calm Insights panel states (non-evaluative, non-clinical). */
export type InsightsViewKind = "loading" | "empty" | "ready" | "error";

export type InsightsView = {
  kind: InsightsViewKind;
  label: string;
  detail: string;
  source: InsightsSource;
  insights: InsightDto[];
};

const COPY: Record<
  Exclude<InsightsViewKind, "loading">,
  { label: string; detail: string }
> = {
  empty: {
    label: "No Insights yet",
    detail: "Patterns will show here when Features or Signals support them.",
  },
  ready: {
    label: "Insights",
    detail: "From the latest Feature snapshot.",
  },
  error: {
    label: "Could not load Insights",
    detail: "Could not reach Insights from Core.",
  },
};

export function insightsView(
  kind: Exclude<InsightsViewKind, "loading">,
  source: InsightsSource,
  insights: InsightDto[] = [],
): InsightsView {
  const copy = COPY[kind];
  return {
    kind,
    label: copy.label,
    detail: copy.detail,
    source,
    insights,
  };
}

export function loadingInsightsView(): InsightsView {
  return {
    kind: "loading",
    label: "Loading",
    detail: "Fetching Insights…",
    source: "get_insights",
    insights: [],
  };
}

/** Short evidence label for list rows (Feature / Signal id only). */
export function formatEvidenceRef(ref: EvidenceRefDto): string {
  if (ref.kind === "feature") {
    return `Feature ${ref.id}`;
  }
  if (ref.kind === "signal") {
    return `Signal ${ref.id}`;
  }
  return `${ref.kind} ${ref.id}`;
}

/** QA: `?mockInsights=empty|ready|error` forces Insights state without Core. */
export function mockInsightsFromLocation(
  search: string = typeof window !== "undefined" ? window.location.search : "",
): InsightsView | null {
  const raw = new URLSearchParams(search).get("mockInsights");
  if (raw === "empty") {
    return insightsView("empty", "mock", []);
  }
  if (raw === "error") {
    return insightsView("error", "mock");
  }
  if (raw === "ready") {
    return insightsView("ready", "mock", [
      {
        id: "01900000-0000-7000-8000-000000000001",
        title: "Sustained stress pattern",
        description:
          "Stress stayed elevated long enough in this period to raise a High_Stress signal.",
        category: "stress",
        evidenceList: [
          { kind: "signal", id: "01900000-0000-7000-8000-000000000009" },
          { kind: "feature", id: "StressIndex" },
        ],
        actionRecommendation:
          "A brief pause or slower pace may help when it fits your schedule.",
      },
      {
        id: "01900000-0000-7000-8000-000000000002",
        title: "Frequent context changes",
        description:
          "ContextSwitchRate was elevated in the latest window, a pattern that often aligns with a shallower FocusScore.",
        category: "focus",
        evidenceList: [{ kind: "feature", id: "ContextSwitchRate" }],
        actionRecommendation:
          "Grouping similar tasks for a stretch can reduce switching when useful.",
      },
    ]);
  }
  return null;
}

/**
 * Loads Insights via IPC only (`get_insights`).
 * Evaluated on the host from the last Feature snapshot cache — UI never opens SQLite.
 */
export async function fetchInsights(): Promise<InsightsView> {
  const mocked = mockInsightsFromLocation();
  if (mocked) {
    return mocked;
  }

  try {
    const payload = await invoke<InsightsDto>("get_insights");
    const list = Array.isArray(payload?.insights) ? payload.insights : [];
    if (list.length === 0) {
      return insightsView("empty", "get_insights", []);
    }
    return insightsView("ready", "get_insights", list);
  } catch {
    return insightsView("error", "get_insights");
  }
}
