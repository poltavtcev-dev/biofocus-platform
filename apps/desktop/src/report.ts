import { invoke } from "@tauri-apps/api/core";

/** Wire payload from `generate_report` (`docs/09-api.md`). */
export type ReportDto = {
  markdown: string;
  llmPrompt: string;
  interpretation?: string;
  llmStatus: "disabled" | "ok" | "error" | "timeout" | string;
  llmError?: string;
};

export type ReportSource = "generate_report" | "mock";

/** Calm Report panel states — idle until the user generates. */
export type ReportViewKind = "idle" | "loading" | "ready" | "error";

export type ReportView = {
  kind: ReportViewKind;
  label: string;
  detail: string;
  source: ReportSource;
  report?: ReportDto;
};

const COPY: Record<
  Exclude<ReportViewKind, "loading">,
  { label: string; detail: string }
> = {
  idle: {
    label: "Report",
    detail:
      "Generate a calm offline summary from the latest Features and Insights. Local AI is optional and off by default.",
  },
  ready: {
    label: "Report ready",
    detail: "Offline summary from Core. Local AI is optional.",
  },
  error: {
    label: "Could not generate report",
    detail: "Could not reach the report builder from Core.",
  },
};

export function reportView(
  kind: Exclude<ReportViewKind, "loading">,
  source: ReportSource,
  report?: ReportDto,
): ReportView {
  const copy = COPY[kind];
  return {
    kind,
    label: copy.label,
    detail: copy.detail,
    source,
    report,
  };
}

export function idleReportView(): ReportView {
  return reportView("idle", "generate_report");
}

export function loadingReportView(): ReportView {
  return {
    kind: "loading",
    label: "Generating",
    detail: "Building offline report…",
    source: "generate_report",
  };
}

/** Short calm line for LLM status under the offline report. */
export function llmStatusDetail(report: ReportDto): string {
  switch (report.llmStatus) {
    case "disabled":
      return "Offline report only. Local AI is optional and currently off.";
    case "ok":
      return "Local AI interpretation (optional).";
    case "timeout":
      return report.llmError ?? "Local AI did not respond in time.";
    case "error":
      return report.llmError ?? "Local AI could not complete this time.";
    default:
      return report.llmError ?? `Local AI status: ${report.llmStatus}`;
  }
}

/** QA: `?mockReport=idle|ready|disabled|error` forces Report state without Core. */
export function mockReportFromLocation(
  search: string = typeof window !== "undefined" ? window.location.search : "",
): ReportView | null {
  const raw = new URLSearchParams(search).get("mockReport");
  if (raw === "idle") {
    return reportView("idle", "mock");
  }
  if (raw === "error") {
    return reportView("error", "mock");
  }
  if (raw === "disabled" || raw === "ready") {
    return reportView("ready", "mock", {
      markdown:
        "# BioFocus report\n\nNothing to summarize yet.\n\nNot a medical assessment.",
      llmPrompt:
        "Interpret the following BioFocus offline report calmly.\n\n# BioFocus report\n\nNothing to summarize yet.",
      llmStatus: "disabled",
    });
  }
  if (raw === "ok") {
    return reportView("ready", "mock", {
      markdown:
        "# BioFocus report\n\n## Features\n\n- FocusScore: 72.5000\n\nNot a medical assessment.",
      llmPrompt: "Interpret calmly…\n\n# BioFocus report\n\n## Features\n\n- FocusScore: 72.5000",
      interpretation:
        "FocusScore sits in a moderate range for this window. No clinical claim — just a calm restatement of the offline facts.",
      llmStatus: "ok",
    });
  }
  return null;
}

type WireReport = {
  markdown?: unknown;
  llmPrompt?: unknown;
  llm_prompt?: unknown;
  interpretation?: unknown;
  llmStatus?: unknown;
  llm_status?: unknown;
  llmError?: unknown;
  llm_error?: unknown;
};

function parseReport(payload: WireReport): ReportDto | null {
  const markdown = payload.markdown;
  const llmPrompt = payload.llmPrompt ?? payload.llm_prompt;
  const llmStatus = payload.llmStatus ?? payload.llm_status;
  if (
    typeof markdown !== "string" ||
    typeof llmPrompt !== "string" ||
    typeof llmStatus !== "string"
  ) {
    return null;
  }
  const interpretation = payload.interpretation;
  const llmError = payload.llmError ?? payload.llm_error;
  return {
    markdown,
    llmPrompt,
    llmStatus,
    ...(typeof interpretation === "string" ? { interpretation } : {}),
    ...(typeof llmError === "string" ? { llmError } : {}),
  };
}

/**
 * Explicit user action → `generate_report` IPC.
 * Never call on Dashboard mount / poll. UI never opens SQLite.
 */
export async function generateReport(): Promise<ReportView> {
  const mocked = mockReportFromLocation();
  if (mocked && mocked.kind !== "idle") {
    return mocked;
  }
  // `mockReport=idle` still allows a real invoke when the user clicks — skip.

  try {
    const payload = await invoke<WireReport>("generate_report");
    const report = parseReport(payload ?? {});
    if (!report) {
      return reportView("error", "generate_report");
    }
    return reportView("ready", "generate_report", report);
  } catch {
    return reportView("error", "generate_report");
  }
}
