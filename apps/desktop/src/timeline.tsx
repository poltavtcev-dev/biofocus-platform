import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

/** «Динамика» (timeline slice A): coverage calendar first, then week / month cards. */

export type CoverageDay = { date: string; value: number | null; srcKind: string | null };
export type CoverageMonth = {
  year: number;
  month: number;
  daysWithData: number;
  daysInRange: number;
  median: number | null;
};
export type TimelineWindow = {
  firstDate: string;
  lastDate: string;
  daysWithData: number;
  median: number | null;
  mainSrcKind: string | null;
};
export type TimelineComparison = {
  scale: "week" | "month";
  current: TimelineWindow;
  previous: TimelineWindow;
  delta: number | null;
  direction: "similar" | "higher" | "lower" | null;
  sourcesDiffer: boolean;
  text: string;
};
export type TimelineDto = {
  metric: string;
  timezone: string;
  today: string;
  days: CoverageDay[];
  months: CoverageMonth[];
  comparisons: TimelineComparison[];
};

type Metric = "sleep" | "in_bed" | "steps" | "focus";

const METRICS: { id: Metric; label: string }[] = [
  { id: "sleep", label: "Сон" },
  { id: "in_bed", label: "Время в постели" },
  { id: "steps", label: "Шаги" },
  { id: "focus", label: "Focus" },
];

const MONTHS = [
  "янв", "фев", "мар", "апр", "май", "июн", "июл", "авг", "сен", "окт", "ноя", "дек",
];

const SCALE_TITLE: Record<string, string> = {
  week: "7 дней к 7 дням до этого",
  month: "30 дней к 30 дням до этого",
};

function formatValue(metric: Metric, value: number | null): string {
  if (value === null) {
    return "нет данных";
  }
  if (metric === "sleep" || metric === "in_bed") {
    const total = Math.round(value);
    return `${Math.floor(total / 60)} ч ${total % 60} мин`;
  }
  if (metric === "steps") {
    return `${Math.round(value)} шагов`;
  }
  return `${Math.round(value)} пунктов`;
}

type View =
  | { kind: "loading" }
  | { kind: "ready"; data: TimelineDto }
  | { kind: "error"; detail: string };

async function fetchTimeline(metric: Metric): Promise<View> {
  try {
    const data = await invoke<TimelineDto>("get_timeline", { metric });
    return { kind: "ready", data };
  } catch (err) {
    return { kind: "error", detail: `Не удалось загрузить динамику: ${String(err)}` };
  }
}

function CoverageGrid({ data, metric }: { data: TimelineDto; metric: Metric }) {
  const byMonth = new Map<string, CoverageDay[]>();
  for (const day of data.days) {
    const key = day.date.slice(0, 7);
    const list = byMonth.get(key) ?? [];
    list.push(day);
    byMonth.set(key, list);
  }
  return (
    <div className="timeline-calendar">
      {data.months.map((month) => {
        const key = `${month.year}-${String(month.month).padStart(2, "0")}`;
        const days = byMonth.get(key) ?? [];
        return (
          <div key={key} className="timeline-month">
            <p className="timeline-month-title">
              {MONTHS[month.month - 1]} {month.year}
              <span className="status-meta">
                {" "}
                {month.daysWithData > 0
                  ? `${month.daysWithData} из ${month.daysInRange} дн. · медиана ${formatValue(metric, month.median)}`
                  : "нет данных"}
              </span>
            </p>
            <div className="timeline-days">
              {days.map((day) => (
                <span
                  key={day.date}
                  className={day.value === null ? "timeline-day" : "timeline-day timeline-day--on"}
                  title={`${day.date}: ${formatValue(metric, day.value)}`}
                />
              ))}
            </div>
          </div>
        );
      })}
    </div>
  );
}

export function TimelinePanel() {
  const [metric, setMetric] = useState<Metric>("sleep");
  const [view, setView] = useState<View>({ kind: "loading" });

  useEffect(() => {
    let cancelled = false;
    setView({ kind: "loading" });
    void fetchTimeline(metric).then((next) => {
      if (!cancelled) {
        setView(next);
      }
    });
    return () => {
      cancelled = true;
    };
  }, [metric]);

  const data = view.kind === "ready" ? view.data : null;
  const hasAny = data ? data.days.some((d) => d.value !== null) : false;

  return (
    <section className="timeline-panel" aria-label="Динамика" data-timezone="Europe/Belgrade">
      <p className="chart-slot-title">Динамика</p>
      <p className="status-meta">
        Вы относительно себя. Дни — Europe/Belgrade. Это наблюдение о данных, без выводов о самочувствии.
      </p>

      <nav className="dash-tabs" aria-label="Показатель">
        {METRICS.map((m) => (
          <button
            key={m.id}
            type="button"
            className={metric === m.id ? "dash-tab dash-tab--on" : "dash-tab"}
            aria-pressed={metric === m.id}
            onClick={() => setMetric(m.id)}
          >
            {m.label}
          </button>
        ))}
      </nav>

      {view.kind === "loading" && <p className="status-detail">Загрузка…</p>}
      {view.kind === "error" && <p className="status-detail">{view.detail}</p>}

      {data && (
        <>
          <p className="chart-slot-title">Какие дни есть</p>
          {!hasAny && <p className="status-detail">За последние полгода данных по этому показателю нет.</p>}
          <CoverageGrid data={data} metric={metric} />

          <p className="chart-slot-title">Сравнения</p>
          <ul className="timeline-cards">
            {data.comparisons.map((c) => (
              <li key={c.scale} className="sources-card">
                <p className="sources-card-title">{SCALE_TITLE[c.scale] ?? c.scale}</p>
                <p className="status-detail">{c.text}</p>
                {c.delta !== null && (
                  <p className="status-meta">
                    {formatValue(metric, c.current.median)} против {formatValue(metric, c.previous.median)}
                  </p>
                )}
              </li>
            ))}
          </ul>
        </>
      )}
    </section>
  );
}
