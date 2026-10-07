/**
 * Plain-language explanations of each metric for the Dashboard, plus a simple
 * reliability rule so thin estimates are clearly marked.
 *
 * Keep copy honest: these are self-tracking estimates from local signals, not
 * medical or performance judgements.
 */

export type MetricDirection = "higher_better" | "lower_better" | "neutral";

export type MetricInfo = {
  name: string;
  /** One sentence: what the number means. */
  what: string;
  /** What it is computed from. */
  from: string;
  direction: MetricDirection;
  /** Optional range/unit hint. */
  unit?: string;
};

export const METRIC_INFO: Record<string, MetricInfo> = {
  FocusScore: {
    name: "Focus",
    what: "How settled your work looked in the last 15 minutes.",
    from: "Staying in the same app (most weight), steady typing (quiet reading counts as neutral), and heart-rate variability if a watch is paired.",
    direction: "higher_better",
    unit: "0–100",
  },
  ContextSwitchRate: {
    name: "App switches",
    what: "How often you changed the front app, per minute, over the last 15 minutes.",
    from: "The active-window history on this Mac. 1.0 means about 15 switches in 15 minutes.",
    direction: "neutral",
    unit: "per minute",
  },
  CognitiveLoad: {
    name: "Combined demand",
    what: "How much was pulling at your attention: meetings, app switching and notifications.",
    from: "Calendar busy time, app switches and notification counts — whichever are turned on. With only one source it mostly reflects that source.",
    direction: "lower_better",
    unit: "0–100",
  },
  DeepWorkScore: {
    name: "Deep work",
    what: "Focus that also held steady without much app hopping.",
    from: "Focus plus app-switch steadiness.",
    direction: "higher_better",
    unit: "0–100",
  },
  AttentionStability: {
    name: "Attention steadiness",
    what: "How little your Focus swung up and down within the window.",
    from: "Focus changes over the window plus app-switch steadiness.",
    direction: "higher_better",
    unit: "0–100",
  },
  DistractionScore: {
    name: "Browser mix",
    what: "How fragmented your browsing was across kinds of sites.",
    from: "Coarse browser categories (work, communication, entertainment, …) — no URLs or titles — plus app switches.",
    direction: "lower_better",
    unit: "0–100",
  },
  StressIndex: {
    name: "Stress index",
    what: "A body-strain estimate from heart-rate variability. Needs a paired watch/phone.",
    from: "HRV readings (RMSSD/SDNN). Not a medical measure.",
    direction: "lower_better",
    unit: "0–100",
  },
  FatigueIndex: {
    name: "Fatigue index",
    what: "How worn-down the day looks so far.",
    from: "Focus trend, time active today, and heart-rate drift if available.",
    direction: "lower_better",
    unit: "0–100",
  },
  RecoveryScore: {
    name: "Recovery",
    what: "How recovered your body looks right now.",
    from: "HRV and resting heart rate from the paired phone.",
    direction: "higher_better",
    unit: "0–100",
  },
  MeetingDensity: {
    name: "Meeting share",
    what: "Share of the last 15 minutes that was blocked by calendar events.",
    from: "Busy calendar events (times only, no titles).",
    direction: "neutral",
    unit: "0–1",
  },
  RecoveryBetweenMeetings: {
    name: "Breaks between meetings",
    what: "How much breathing room you had between back-to-back meetings.",
    from: "Gaps between busy calendar events.",
    direction: "higher_better",
    unit: "0–100",
  },
  NotificationPressure: {
    name: "Notifications",
    what: "How many notifications arrived recently.",
    from: "Notification counts only — never content.",
    direction: "lower_better",
    unit: "0–100",
  },
  GitActivityRate: {
    name: "Git activity",
    what: "How much commit/checkout/sync activity happened in your watched repos.",
    from: "Git events in folders you added under “Git folders”.",
    direction: "neutral",
  },
  ActivityBalance: {
    name: "Movement",
    what: "How much you moved recently.",
    from: "Step counts and logged workouts from the phone.",
    direction: "higher_better",
    unit: "0–100",
  },
  EnergyScore: {
    name: "Energy",
    what: "A rough energy estimate from activity, heart rate and rest.",
    from: "Active energy, heart rate and sleep from the phone.",
    direction: "higher_better",
    unit: "0–100",
  },
  SleepDebt: {
    name: "Sleep shortfall",
    what: "How far recent sleep fell short of a typical night.",
    from: "Sleep intervals from the phone.",
    direction: "lower_better",
    unit: "0–100",
  },
  DeskAwayPresence: {
    name: "Away from desk",
    what: "How likely you were away from the desk (only shown with real evidence like a walk or steps).",
    from: "Quiet keyboard/apps plus steps or a logged walk.",
    direction: "neutral",
    unit: "0–100",
  },
  CircadianOffset: {
    name: "Rhythm alignment",
    what: "How well work timing lines up with your sleep rhythm.",
    from: "Sleep midpoint vs. when you are active.",
    direction: "higher_better",
    unit: "0–100",
  },
  SustainedLoadIndicator: {
    name: "Prolonged load",
    what: "Whether strain has stayed high for a long stretch, not just a moment.",
    from: "Stress, fatigue and meeting share over a longer lookback.",
    direction: "lower_better",
    unit: "0–100",
  },
  AmbientMediaShare: {
    name: "Media playing",
    what: "Share of the window with music/podcasts playing.",
    from: "Now Playing kind only (no track names).",
    direction: "neutral",
    unit: "%",
  },
  AmbientLightShare: {
    name: "Lighting",
    what: "Share of the window in brighter light.",
    from: "Coarse ambient-light level (dark/dim/moderate/bright).",
    direction: "neutral",
    unit: "%",
  },
};

export function metricInfo(featureId: string): MetricInfo {
  return (
    METRIC_INFO[featureId] ?? {
      name: featureId,
      what: "An experimental metric.",
      from: "Local signals on this Mac.",
      direction: "neutral",
    }
  );
}

export type Reliability = "ok" | "low" | "single_input";

/** Below this confidence a value is shown as a rough estimate. */
export const LOW_CONFIDENCE = 0.5;

/** Composite metrics where a single input means the score is just that input. */
const COMPOSITES = new Set([
  "FocusScore",
  "CognitiveLoad",
  "DeepWorkScore",
  "AttentionStability",
  "FatigueIndex",
  "SustainedLoadIndicator",
  "EnergyScore",
]);

export function reliabilityOf(f: {
  featureId: string;
  confidence?: number;
  factors?: { id: string }[];
}): Reliability {
  if (typeof f.confidence === "number" && f.confidence < LOW_CONFIDENCE) {
    return "low";
  }
  if (COMPOSITES.has(f.featureId) && (f.factors?.length ?? 0) === 1) {
    return "single_input";
  }
  return "ok";
}

export function reliabilityText(r: Reliability, factors?: { label: string }[]): string | null {
  switch (r) {
    case "low":
      return "Rough estimate — not enough data in this window yet.";
    case "single_input":
      return `Based on one source only (${factors?.[0]?.label ?? "one signal"}) — treat as a hint.`;
    case "ok":
      return null;
  }
}

export function directionHint(d: MetricDirection): string {
  switch (d) {
    case "higher_better":
      return "Higher is calmer / better.";
    case "lower_better":
      return "Lower is calmer.";
    case "neutral":
      return "Neither good nor bad — context only.";
  }
}
