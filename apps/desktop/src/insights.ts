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

/** Short evidence label for list rows (Feature / Signal / Insight id). */
export function formatEvidenceRef(ref: EvidenceRefDto): string {
  if (ref.kind === "feature") {
    return `Feature ${ref.id}`;
  }
  if (ref.kind === "signal") {
    return `Signal ${ref.id}`;
  }
  if (ref.kind === "insight") {
    return `Insight ${ref.id}`;
  }
  if (ref.kind === "observation") {
    return `Event ${ref.id.slice(0, 8)}`;
  }
  return `${ref.kind} ${ref.id}`;
}

/**
 * Calm category label for list rows (Core `category` as returned).
 * No clinical framing — personal pattern / focus / stress only.
 */
export function formatInsightCategory(category: string): string {
  const key = category.trim().toLowerCase();
  if (key === "pattern") {
    return "Pattern";
  }
  if (key === "focus") {
    return "Focus";
  }
  if (key === "stress") {
    return "Stress";
  }
  if (!key) {
    return "";
  }
  return category.trim();
}

/** Sample baseline / pattern Insight (mirrors `focus_vs_recent_baseline_v1` copy). */
const MOCK_PATTERN_INSIGHT: InsightDto = {
  id: "01900000-0000-7000-8000-000000000003",
  title: "Focus relative to your recent average",
  description: "Focus looks higher than your recent afternoon average.",
  category: "pattern",
  evidenceList: [{ kind: "feature", id: "FocusScore" }],
  actionRecommendation:
    "Noticing a stronger focus stretch than recent afternoons — keep the setup that is working if it still feels right.",
};

/** QA: `?mockInsights=empty|ready|pattern|error` forces Insights state without Core. */
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
  if (raw === "pattern") {
    return insightsView("ready", "mock", [MOCK_PATTERN_INSIGHT]);
  }
  if (raw === "ready") {
    return insightsView("ready", "mock", [
      MOCK_PATTERN_INSIGHT,
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
