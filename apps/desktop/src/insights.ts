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
    label: "Наблюдений пока нет",
    detail: "Закономерности появятся здесь, когда для них хватит метрик или сигналов.",
  },
  ready: {
    label: "Наблюдения",
    detail: "По последнему снимку метрик.",
  },
  error: {
    label: "Не удалось загрузить наблюдения",
    detail: "Не удалось получить наблюдения из ядра.",
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
    label: "Загрузка",
    detail: "Загружаем наблюдения…",
    source: "get_insights",
    insights: [],
  };
}

/** Short evidence label for list rows (Feature / Signal / Insight id). */
export function formatEvidenceRef(ref: EvidenceRefDto): string {
  if (ref.kind === "feature") {
    return `Метрика ${ref.id}`;
  }
  if (ref.kind === "signal") {
    return `Сигнал ${ref.id}`;
  }
  if (ref.kind === "insight") {
    return `Наблюдение ${ref.id}`;
  }
  if (ref.kind === "observation") {
    return `Событие ${ref.id.slice(0, 8)}`;
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
    return "Закономерность";
  }
  if (key === "focus") {
    return "Фокус";
  }
  if (key === "stress") {
    return "Напряжение";
  }
  if (key === "life_event") {
    return "События дня";
  }
  if (!key) {
    return "";
  }
  return category.trim();
}

/** Sample baseline / pattern Insight (mirrors `focus_vs_recent_baseline_v1` copy). */
const MOCK_PATTERN_INSIGHT: InsightDto = {
  id: "01900000-0000-7000-8000-000000000003",
  title: "Фокус на фоне вашего недавнего среднего",
  description: "Фокус выше вашего недавнего среднего по второй половине дня.",
  category: "pattern",
  evidenceList: [{ kind: "feature", id: "FocusScore" }],
  actionRecommendation:
    "Фокус держится лучше, чем в недавние дни после полудня. Если так удобно, оставьте ту же обстановку.",
};

/** Sample `life_event_before_after_v1` Insight (mirrors Core copy). */
const MOCK_LIFE_EVENT_INSIGHT: InsightDto = {
  id: "01900000-0000-7000-8000-000000000004",
  title: "Around your walks",
  description:
    "Across 2 recent logged walks, Focus averaged 68 after vs 59 before (higher by 9 points). Compared: 45 min before vs 15–60 min after. A personal pattern, not a rule. Confidence: medium.",
  category: "life_event",
  evidenceList: [
    { kind: "feature", id: "FocusScore" },
    { kind: "observation", id: "0190f2a1-7c00-7000-8000-00000000a001" },
    { kind: "observation", id: "0190f2a1-9d00-7000-8000-00000000a002" },
  ],
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
      MOCK_LIFE_EVENT_INSIGHT,
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
