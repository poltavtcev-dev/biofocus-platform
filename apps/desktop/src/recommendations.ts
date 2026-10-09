import { invoke } from "@tauri-apps/api/core";
import { formatEvidenceRef, type EvidenceRefDto } from "./insights";

export type { EvidenceRefDto };

/** Wire Recommendation from `get_recommendations` (`docs/09-api.md`). */
export type RecommendationDto = {
  id: string;
  title: string;
  suggestion: string;
  category: string;
  evidenceList: EvidenceRefDto[];
};

export type RecommendationsDto = {
  recommendations: RecommendationDto[];
};

export type RecommendationsSource = "get_recommendations" | "mock";

/** Calm Recommendations panel states (non-clinical, optional hints). */
export type RecommendationsViewKind = "loading" | "empty" | "ready" | "error";

export type RecommendationsView = {
  kind: RecommendationsViewKind;
  label: string;
  detail: string;
  source: RecommendationsSource;
  recommendations: RecommendationDto[];
};

const COPY: Record<
  Exclude<RecommendationsViewKind, "loading">,
  { label: string; detail: string }
> = {
  empty: {
    label: "Подсказок пока нет",
    detail: "Необязательные подсказки о темпе появятся здесь, когда для них хватит закономерностей.",
  },
  ready: {
    label: "Подсказки",
    detail: "Необязательные личные подсказки из ядра. Это не медицинский совет.",
  },
  error: {
    label: "Не удалось загрузить подсказки",
    detail: "Не удалось получить подсказки из ядра.",
  },
};

export function recommendationsView(
  kind: Exclude<RecommendationsViewKind, "loading">,
  source: RecommendationsSource,
  recommendations: RecommendationDto[] = [],
): RecommendationsView {
  const copy = COPY[kind];
  return {
    kind,
    label: copy.label,
    detail: copy.detail,
    source,
    recommendations,
  };
}

export function loadingRecommendationsView(): RecommendationsView {
  return {
    kind: "loading",
    label: "Загрузка",
    detail: "Загружаем подсказки…",
    source: "get_recommendations",
    recommendations: [],
  };
}

/** Calm category label for list rows (Core `category` as returned). */
export function formatRecommendationCategory(category: string): string {
  const key = category.trim().toLowerCase();
  if (key === "pace") {
    return "Темп";
  }
  if (key === "focus") {
    return "Фокус";
  }
  if (!key) {
    return "";
  }
  return category.trim();
}

export { formatEvidenceRef };

/** Sample pace Recommendation (mirrors `focus_dip_pace_hint_v1` copy). */
const MOCK_PACE_RECOMMENDATION: RecommendationDto = {
  id: "01900000-0000-7000-8000-0000000000a1",
  title: "Можно чуть сбавить темп",
  suggestion:
    "Если это вписывается в день, короткая пауза или чуть более спокойный темп могут помочь, когда фокус ниже вашего недавнего среднего.",
  category: "pace",
  evidenceList: [
    { kind: "feature", id: "FocusScore" },
    { kind: "insight", id: "01900000-0000-7000-8000-000000000003" },
  ],
};

/** QA: `?mockRecommendations=empty|ready|pace|error` forces state without Core. */
export function mockRecommendationsFromLocation(
  search: string = typeof window !== "undefined" ? window.location.search : "",
): RecommendationsView | null {
  const raw = new URLSearchParams(search).get("mockRecommendations");
  if (raw === "empty") {
    return recommendationsView("empty", "mock", []);
  }
  if (raw === "error") {
    return recommendationsView("error", "mock");
  }
  if (raw === "pace" || raw === "ready") {
    return recommendationsView("ready", "mock", [MOCK_PACE_RECOMMENDATION]);
  }
  return null;
}

/**
 * Loads Recommendations via IPC only (`get_recommendations`).
 * Evaluated on the host after Insights — UI never opens SQLite.
 */
export async function fetchRecommendations(): Promise<RecommendationsView> {
  const mocked = mockRecommendationsFromLocation();
  if (mocked) {
    return mocked;
  }

  try {
    const payload = await invoke<RecommendationsDto>("get_recommendations");
    const list = Array.isArray(payload?.recommendations)
      ? payload.recommendations
      : [];
    if (list.length === 0) {
      return recommendationsView("empty", "get_recommendations", []);
    }
    return recommendationsView("ready", "get_recommendations", list);
  } catch {
    return recommendationsView("error", "get_recommendations");
  }
}
