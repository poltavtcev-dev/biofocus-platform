import { invoke } from "@tauri-apps/api/core";

/** Wire payload from `get_local_llm_status` (`docs/09-api.md`). */
export type LocalLlmProviderStatus = "disabled" | "ready" | "error" | string;

export type LocalLlmProviderDto = {
  status: LocalLlmProviderStatus;
  detail: string;
  model?: string;
  packId: string;
  packVersion: string;
};

export type LlmProviderSource = "get_local_llm_status" | "mock";

export type LlmProviderView = {
  status: LocalLlmProviderStatus;
  label: string;
  detail: string;
  source: LlmProviderSource;
  model?: string;
  packId: string;
  packVersion: string;
};

const DEFAULT_PACK_ID = "biofocus.default";
const DEFAULT_PACK_VERSION = "1";

const LABELS: Record<"disabled" | "ready" | "error", string> = {
  disabled: "Локальный ИИ · выключен",
  ready: "Локальный ИИ · готов",
  error: "Локальный ИИ · сбой",
};

function labelFor(status: LocalLlmProviderStatus): string {
  if (status === "disabled" || status === "ready" || status === "error") {
    return LABELS[status];
  }
  return `Локальный ИИ · ${status}`;
}

export function llmProviderView(
  dto: LocalLlmProviderDto,
  source: LlmProviderSource,
): LlmProviderView {
  return {
    status: dto.status,
    label: labelFor(dto.status),
    detail: dto.detail,
    source,
    ...(dto.model ? { model: dto.model } : {}),
    packId: dto.packId,
    packVersion: dto.packVersion,
  };
}

export function loadingLlmProviderView(): LlmProviderView {
  return {
    status: "disabled",
    label: "Локальный ИИ",
    detail: "Проверяем локальный ИИ…",
    source: "get_local_llm_status",
    packId: DEFAULT_PACK_ID,
    packVersion: DEFAULT_PACK_VERSION,
  };
}

/** Dot class for calm traffic-light (reuses Core status-dot styles). */
export function llmProviderDotKind(
  status: LocalLlmProviderStatus,
): "idle" | "ready" | "error" {
  if (status === "ready") {
    return "ready";
  }
  if (status === "error") {
    return "error";
  }
  return "idle";
}

/** QA: `?mockLlmStatus=disabled|ready|error` forces provider status without Core. */
export function mockLlmProviderFromLocation(
  search: string = typeof window !== "undefined" ? window.location.search : "",
): LlmProviderView | null {
  const raw = new URLSearchParams(search).get("mockLlmStatus");
  if (raw === "disabled") {
    return llmProviderView(
      {
        status: "disabled",
        detail: "Локальный ИИ необязателен и сейчас выключен.",
        packId: DEFAULT_PACK_ID,
        packVersion: DEFAULT_PACK_VERSION,
      },
      "mock",
    );
  }
  if (raw === "ready") {
    return llmProviderView(
      {
        status: "ready",
        detail:
          "Локальный ИИ настроен. Пояснение появляется только когда вы собираете отчёт.",
        model: "llama3.2",
        packId: DEFAULT_PACK_ID,
        packVersion: DEFAULT_PACK_VERSION,
      },
      "mock",
    );
  }
  if (raw === "error") {
    return llmProviderView(
      {
        status: "error",
        detail: "Локальный ИИ включён, но адрес выглядит непригодным.",
        packId: DEFAULT_PACK_ID,
        packVersion: DEFAULT_PACK_VERSION,
      },
      "mock",
    );
  }
  return null;
}

type WireStatus = {
  status?: unknown;
  detail?: unknown;
  model?: unknown;
  packId?: unknown;
  pack_id?: unknown;
  packVersion?: unknown;
  pack_version?: unknown;
};

function parseStatus(payload: WireStatus): LocalLlmProviderDto | null {
  const status = payload.status;
  const detail = payload.detail;
  const packId = payload.packId ?? payload.pack_id;
  const packVersion = payload.packVersion ?? payload.pack_version;
  if (
    typeof status !== "string" ||
    typeof detail !== "string" ||
    typeof packId !== "string" ||
    typeof packVersion !== "string"
  ) {
    return null;
  }
  const model = payload.model;
  return {
    status,
    detail,
    packId,
    packVersion,
    ...(typeof model === "string" ? { model } : {}),
  };
}

/**
 * Config-only provider status via IPC.
 * Safe on Dashboard mount — never calls `generate_report` / interpret.
 * UI never opens SQLite; no secrets/tokens.
 */
export async function fetchLocalLlmStatus(): Promise<LlmProviderView> {
  const mocked = mockLlmProviderFromLocation();
  if (mocked) {
    return mocked;
  }

  try {
    const payload = await invoke<WireStatus>("get_local_llm_status");
    const dto = parseStatus(payload ?? {});
    if (!dto) {
      return llmProviderView(
        {
          status: "error",
          detail: "Не удалось прочитать статус локального ИИ из ядра.",
          packId: DEFAULT_PACK_ID,
          packVersion: DEFAULT_PACK_VERSION,
        },
        "get_local_llm_status",
      );
    }
    return llmProviderView(dto, "get_local_llm_status");
  } catch {
    return llmProviderView(
      {
        status: "error",
        detail: "Не удалось получить статус локального ИИ из ядра.",
        packId: DEFAULT_PACK_ID,
        packVersion: DEFAULT_PACK_VERSION,
      },
      "get_local_llm_status",
    );
  }
}
