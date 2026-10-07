import { useEffect, useState } from "react";
import {
  CartesianGrid,
  Line,
  LineChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";
import {
  buildChartPoints,
  defaultSelectedSeries,
  formatChartTime,
  presentSeriesIds,
  SCORE_SERIES_IDS,
  seriesColor,
  type ChartFeatureId,
  type ChartPoint,
} from "./featureChart";
import {
  fetchFeatureSnapshot,
  formatFeatureValue,
  loadingView,
  type FeatureDto,
  type SnapshotView,
} from "./featureSnapshot";
import {
  directionHint,
  HEADLINE_FOURTH,
  HEADLINE_IDS,
  latestPerFeature,
  metricInfo,
  reliabilityOf,
  reliabilityText,
  statTone,
} from "./metricInfo";
import {
  CHART_RANGES,
  fetchFeatureSeries,
  loadingSeriesView,
  rangeLabel,
  type ChartRange,
  type SeriesView,
} from "./featureSeries";
import {
  fetchInsights,
  formatEvidenceRef,
  formatInsightCategory,
  loadingInsightsView,
  type InsightsView,
} from "./insights";
import {
  fetchRecommendations,
  formatRecommendationCategory,
  loadingRecommendationsView,
  type RecommendationsView,
} from "./recommendations";
import {
  generateReport,
  idleReportView,
  llmStatusDetail,
  loadingReportView,
  mockReportFromLocation,
  type ReportView,
} from "./report";
import {
  fetchLocalLlmStatus,
  loadingLlmProviderView,
  llmProviderDotKind,
  mockLlmProviderFromLocation,
  type LlmProviderView,
} from "./llmProvider";

/** Soft refresh — idle-safe; no busy-loop. Does not regenerate reports. */
const SNAPSHOT_POLL_MS = 30_000;

const SERIES_PREF_KEY = "biofocus.chartSeries.v1";

function loadSeriesPref(): ChartFeatureId[] | null {
  try {
    const raw = window.localStorage.getItem(SERIES_PREF_KEY);
    const parsed: unknown = raw ? JSON.parse(raw) : null;
    return Array.isArray(parsed) ? (parsed.filter((x) => typeof x === "string") as ChartFeatureId[]) : null;
  } catch {
    return null;
  }
}

function saveSeriesPref(ids: ChartFeatureId[]) {
  try {
    window.localStorage.setItem(SERIES_PREF_KEY, JSON.stringify(ids));
  } catch {
    // Storage unavailable — selection just won't persist.
  }
}

function ChartSlot({
  seriesView,
  range,
  onRangeChange,
}: {
  seriesView: SeriesView;
  range: ChartRange;
  onRangeChange: (next: ChartRange) => void;
}) {
  const features = seriesView.series?.features ?? [];
  const empty =
    seriesView.kind === "loading" ||
    seriesView.kind === "error" ||
    seriesView.kind === "empty";
  const points = empty ? [] : buildChartPoints(features);
  const present = presentSeriesIds(points);
  const [picked, setPicked] = useState<ChartFeatureId[] | null>(() => loadSeriesPref());
  const selected = (picked ?? defaultSelectedSeries(present)).filter((id) =>
    present.includes(id),
  );
  const effective = selected.length > 0 ? selected : defaultSelectedSeries(present);
  const scoreSeries = effective.filter((id) => SCORE_SERIES_IDS.includes(id));
  const showCsr = effective.includes("ContextSwitchRate");

  const toggle = (id: ChartFeatureId) => {
    const next = effective.includes(id)
      ? effective.filter((x) => x !== id)
      : [...effective, id];
    if (next.length === 0) {
      return; // keep at least one line
    }
    setPicked(next);
    saveSeriesPref(next);
  };

  let placeholder = "No data to chart yet.";
  if (seriesView.kind === "loading") {
    placeholder = "Loading…";
  } else if (seriesView.kind === "error") {
    placeholder = seriesView.detail;
  } else if (seriesView.kind === "empty") {
    placeholder = "No data for this range yet. Keep BioFocus running for a few minutes.";
  }

  return (
    <section className="chart-slot" aria-label="Trends">
      <div className="chart-slot-header">
        <p className="chart-slot-title">Trends</p>
        <div className="range-picker" role="group" aria-label="Chart range">
          {CHART_RANGES.map((r) => (
            <button
              key={r}
              type="button"
              className={`range-picker-btn${r === range ? " range-picker-btn--active" : ""}`}
              aria-pressed={r === range}
              onClick={() => onRangeChange(r)}
            >
              {rangeLabel(r)}
            </button>
          ))}
        </div>
      </div>
      {present.length > 0 && (
        <div className="series-chips" role="group" aria-label="Lines to show">
          {present.map((id) => {
            const on = effective.includes(id);
            return (
              <button
                key={id}
                type="button"
                className={`series-chip${on ? " series-chip--on" : ""}`}
                aria-pressed={on}
                onClick={() => toggle(id)}
                title={metricInfo(id).what}
              >
                <span
                  className="series-swatch"
                  style={{
                    background: on ? seriesColor(id) : "transparent",
                    borderColor: seriesColor(id),
                    borderStyle: id === "ContextSwitchRate" ? "dashed" : "solid",
                  }}
                  aria-hidden
                />
                {metricInfo(id).name}
              </button>
            );
          })}
        </div>
      )}
      <div
        className={`chart-slot-body${points.length > 0 ? " chart-slot-body--chart" : ""}`}
      >
        {points.length === 0 ? (
          <p className="chart-slot-placeholder">{placeholder}</p>
        ) : (
          <FeatureSeriesChart points={points} scoreSeries={scoreSeries} showCsr={showCsr} />
        )}
      </div>
      {points.length > 0 && (
        <p className="chart-slot-units" aria-hidden>
          Left axis: score 0–100
          {showCsr ? " · Right axis (dashed): app switches per minute" : ""} · Tap a
          name above to show or hide it.
        </p>
      )}
    </section>
  );
}

function FeatureSeriesChart({
  points,
  scoreSeries,
  showCsr,
}: {
  points: ChartPoint[];
  scoreSeries: ChartFeatureId[];
  showCsr: boolean;
}) {
  const axisTick = { fill: "var(--c-fg-3)", fontSize: 11 };
  return (
    <ResponsiveContainer width="100%" height={240}>
      <LineChart data={points} margin={{ top: 8, right: showCsr ? 4 : 12, left: -4, bottom: 0 }}>
        <CartesianGrid stroke="var(--c-border)" vertical={false} />
        <XAxis
          dataKey="t"
          tickFormatter={formatChartTime}
          tick={axisTick}
          axisLine={{ stroke: "var(--c-border-strong)" }}
          tickLine={false}
          minTickGap={48}
        />
        <YAxis
          yAxisId="score"
          domain={[0, 100]}
          ticks={[0, 25, 50, 75, 100]}
          width={34}
          tick={axisTick}
          axisLine={false}
          tickLine={false}
        />
        {showCsr && (
          <YAxis
            yAxisId="csr"
            orientation="right"
            width={34}
            tick={axisTick}
            axisLine={false}
            tickLine={false}
            allowDecimals
            tickFormatter={(v: number) => `${v}/m`}
          />
        )}
        <Tooltip
          contentStyle={{
            background: "var(--c-surface-2)",
            border: "1px solid var(--c-border-strong)",
            borderRadius: 8,
            fontSize: 12,
            color: "var(--c-fg)",
          }}
          labelStyle={{ color: "var(--c-fg-3)", marginBottom: 4 }}
          labelFormatter={(label) =>
            typeof label === "number" ? formatChartTime(label) : String(label)
          }
          formatter={(value: number, name: string) => {
            const id = name as ChartFeatureId;
            const display =
              typeof value === "number"
                ? Number.isInteger(value)
                  ? String(value)
                  : value.toFixed(1)
                : String(value);
            return [display, metricInfo(id).name];
          }}
        />
        {scoreSeries.map((id) => (
          <Line
            key={id}
            yAxisId="score"
            type="monotone"
            dataKey={id}
            name={id}
            stroke={seriesColor(id)}
            strokeWidth={2.25}
            dot={false}
            activeDot={{ r: 4, strokeWidth: 0 }}
            connectNulls
            isAnimationActive={false}
          />
        ))}
        {showCsr && (
          <Line
            yAxisId="csr"
            type="monotone"
            dataKey="ContextSwitchRate"
            name="ContextSwitchRate"
            stroke={seriesColor("ContextSwitchRate")}
            strokeWidth={1.75}
            strokeDasharray="5 4"
            dot={false}
            activeDot={{ r: 4, strokeWidth: 0 }}
            connectNulls
            isAnimationActive={false}
          />
        )}
      </LineChart>
    </ResponsiveContainer>
  );
}

function InsightsSlot({ view }: { view: InsightsView }) {
  const showList = view.kind === "ready" && view.insights.length > 0;

  return (
    <section className="insights-slot" aria-label="Insights" aria-live="polite">
      <p className="chart-slot-title">Insights</p>
      {showList ? (
        <ul className="insight-rows">
          {view.insights.map((insight) => {
            const categoryLabel = formatInsightCategory(insight.category);
            return (
              <li key={insight.id} className="insight-row">
                {categoryLabel && (
                  <p className="insight-category">{categoryLabel}</p>
                )}
                <p className="insight-title">{insight.title}</p>
                <p className="insight-description">{insight.description}</p>
                {insight.evidenceList.length > 0 && (
                  <p className="insight-evidence">
                    Evidence:{" "}
                    {insight.evidenceList.map(formatEvidenceRef).join(" · ")}
                  </p>
                )}
                {insight.actionRecommendation && (
                  <p className="insight-action">{insight.actionRecommendation}</p>
                )}
              </li>
            );
          })}
        </ul>
      ) : (
        <>
          <p className="status-meta">
            {view.kind === "loading" ? view.detail : view.label}
          </p>
          {view.kind !== "loading" && (
            <p className="status-meta">{view.detail}</p>
          )}
        </>
      )}
    </section>
  );
}

function RecommendationsSlot({ view }: { view: RecommendationsView }) {
  const showList = view.kind === "ready" && view.recommendations.length > 0;

  return (
    <section
      className="recommendations-slot"
      aria-label="Suggestions"
      aria-live="polite"
    >
      <p className="chart-slot-title">Suggestions</p>
      {showList ? (
        <ul className="insight-rows">
          {view.recommendations.map((item) => {
            const categoryLabel = formatRecommendationCategory(item.category);
            return (
              <li key={item.id} className="insight-row insight-row--suggestion">
                {categoryLabel && (
                  <p className="insight-category">{categoryLabel}</p>
                )}
                <p className="insight-title">{item.title}</p>
                <p className="insight-description">{item.suggestion}</p>
                {item.evidenceList.length > 0 && (
                  <p className="insight-evidence">
                    Evidence:{" "}
                    {item.evidenceList.map(formatEvidenceRef).join(" · ")}
                  </p>
                )}
              </li>
            );
          })}
        </ul>
      ) : (
        <>
          <p className="status-meta">
            {view.kind === "loading" ? view.detail : view.label}
          </p>
          {view.kind !== "loading" && (
            <p className="status-meta">{view.detail}</p>
          )}
        </>
      )}
    </section>
  );
}

function ReportSlot({
  view,
  busy,
  onGenerate,
  llmProvider,
}: {
  view: ReportView;
  busy: boolean;
  onGenerate: () => void;
  llmProvider: LlmProviderView;
}) {
  const report = view.report;
  const showBody = view.kind === "ready" && report;
  const packMeta = `${llmProvider.packId} @ ${llmProvider.packVersion}`;

  return (
    <section className="report-slot" aria-label="Report" aria-live="polite">
      <p className="chart-slot-title">Report</p>
      <div
        className="llm-provider-row"
        aria-label="Local AI provider status"
      >
        <span
          className={`status-dot status-dot--${llmProviderDotKind(llmProvider.status)}`}
          aria-hidden
        />
        <div className="llm-provider-copy">
          <p className="status-label">{llmProvider.label}</p>
          <p className="status-meta">{llmProvider.detail}</p>
          <p className="status-meta">
            Report pack: {packMeta}
            {llmProvider.model ? ` · model ${llmProvider.model}` : ""}
          </p>
        </div>
      </div>
      <p className="status-meta">
        {view.kind === "loading" ? view.detail : view.label}
      </p>
      {view.kind !== "loading" && (
        <p className="status-meta">{view.detail}</p>
      )}
      <p className="report-ai-note">
        Local AI is optional and never runs automatically. Offline markdown is
        always available — interpretation only after you generate a report.
      </p>
      <button
        type="button"
        className="retry"
        onClick={onGenerate}
        disabled={busy || view.kind === "loading"}
      >
        {view.kind === "loading" ? "Generating…" : "Generate report"}
      </button>

      {showBody && (
        <div className="report-body">
          <pre className="report-markdown">{report.markdown}</pre>
          <details className="report-prompt">
            <summary>Prompt for local AI</summary>
            <pre className="report-markdown report-markdown--prompt">
              {report.llmPrompt}
            </pre>
          </details>
          <p className="status-meta">{llmStatusDetail(report)}</p>
          {report.interpretation && (
            <div className="report-interpretation">
              <p className="chart-slot-title">Local AI (optional)</p>
              <pre className="report-markdown">{report.interpretation}</pre>
            </div>
          )}
        </div>
      )}
    </section>
  );
}

export function Dashboard() {
  const [view, setView] = useState<SnapshotView>(() => loadingView());
  const [seriesView, setSeriesView] = useState<SeriesView>(() =>
    loadingSeriesView(),
  );
  const [chartRange, setChartRange] = useState<ChartRange>("1d");
  const [insightsView, setInsightsView] = useState<InsightsView>(() =>
    loadingInsightsView(),
  );
  const [recommendationsView, setRecommendationsView] =
    useState<RecommendationsView>(() => loadingRecommendationsView());
  const [reportView, setReportView] = useState<ReportView>(
    () => mockReportFromLocation() ?? idleReportView(),
  );
  const [llmProvider, setLlmProvider] = useState<LlmProviderView>(
    () => mockLlmProviderFromLocation() ?? loadingLlmProviderView(),
  );
  const [busy, setBusy] = useState(true);
  const [reportBusy, setReportBusy] = useState(false);

  useEffect(() => {
    let cancelled = false;

    const load = async (isFirst: boolean) => {
      if (isFirst) {
        setBusy(true);
        setView(loadingView());
        setInsightsView(loadingInsightsView());
        setRecommendationsView(loadingRecommendationsView());
      }
      // Snapshot / Insights / Recommendations / LLM status only —
      // never auto-invoke report / interpret.
      const [next, nextInsights, nextRecommendations, nextLlm] =
        await Promise.all([
          fetchFeatureSnapshot(),
          fetchInsights(),
          fetchRecommendations(),
          fetchLocalLlmStatus(),
        ]);
      if (cancelled) {
        return;
      }
      setView(next);
      setInsightsView(nextInsights);
      setRecommendationsView(nextRecommendations);
      setLlmProvider(nextLlm);
      if (isFirst) {
        setBusy(false);
      }
    };

    void load(true);
    const timer = window.setInterval(() => {
      void load(false);
    }, SNAPSHOT_POLL_MS);

    return () => {
      cancelled = true;
      window.clearInterval(timer);
    };
  }, []);

  useEffect(() => {
    let cancelled = false;
    setSeriesView(loadingSeriesView());
    void fetchFeatureSeries(chartRange).then((next) => {
      if (!cancelled) {
        setSeriesView(next);
      }
    });
    const timer = window.setInterval(() => {
      void fetchFeatureSeries(chartRange).then((next) => {
        if (!cancelled) {
          setSeriesView(next);
        }
      });
    }, SNAPSHOT_POLL_MS);
    return () => {
      cancelled = true;
      window.clearInterval(timer);
    };
  }, [chartRange]);

  const onRetry = () => {
    setBusy(true);
    setView(loadingView());
    setSeriesView(loadingSeriesView());
    setInsightsView(loadingInsightsView());
    setRecommendationsView(loadingRecommendationsView());
    void Promise.all([
      fetchFeatureSnapshot(),
      fetchFeatureSeries(chartRange),
      fetchInsights(),
      fetchRecommendations(),
      fetchLocalLlmStatus(),
    ]).then(([next, nextSeries, nextInsights, nextRecommendations, nextLlm]) => {
      setView(next);
      setSeriesView(nextSeries);
      setInsightsView(nextInsights);
      setRecommendationsView(nextRecommendations);
      setLlmProvider(nextLlm);
      setBusy(false);
    });
  };

  const onGenerateReport = () => {
    setReportBusy(true);
    setReportView(loadingReportView());
    void generateReport().then((next) => {
      setReportView(next);
      setReportBusy(false);
    });
  };

  const onRangeChange = (next: ChartRange) => {
    setChartRange(next);
  };

  const features = latestPerFeature(view.snapshot?.features ?? []);
  const signals = view.snapshot?.signals ?? [];
  const isReady = view.kind === "ready";

  return (
    <main className="shell shell--dashboard" data-snapshot={view.kind}>
      <header className="dash-header">
        <div className="brand brand--dashboard">
          <h1>BioFocus</h1>
          <p className="brand-sub">Last 15 minutes · everything stays on this Mac</p>
        </div>
      <section className="status-block status-block--inline" aria-live="polite">
        <div className="status-row">
          <span
            className={`status-dot status-dot--${
              view.kind === "error"
                ? "error"
                : view.kind === "loading"
                  ? "idle"
                  : "ready"
            }`}
            aria-hidden
          />
          <p className="status-label">
            {busy && view.kind === "loading" ? "Loading" : view.label}
          </p>
        </div>
        <p className="status-detail">{view.detail}</p>
      </section>
      </header>

      {isReady && features.length > 0 && <StatCards features={features} />}

      {view.kind === "error" && (
        <button
          type="button"
          className="retry"
          onClick={onRetry}
          disabled={busy}
        >
          Try again
        </button>
      )}

      <ChartSlot
        seriesView={seriesView}
        range={chartRange}
        onRangeChange={onRangeChange}
      />

      {isReady && features.length > 0 && (
        <section className="feature-list" aria-label="Feature snapshot">
          <p className="chart-slot-title">Snapshot</p>
          <p className="status-meta">
            Latest 15-minute window. Click a metric to see what it means. Values
            marked “rough” or “one source” are hints, not conclusions.
          </p>
          <ul className="feature-rows">
            {[...features]
              .sort((a, b) => metricInfo(a.featureId).name.localeCompare(metricInfo(b.featureId).name))
              .map((f) => (
              <FeatureRow key={`${f.featureId}-${f.timeWindow.end}`} f={f} />
            ))}
          </ul>
          {signals.length > 0 && (
            <p className="status-meta">
              Signals: {signals.map((s) => s.type).join(", ")}
            </p>
          )}
        </section>
      )}

      <div className="dash-two-col">
        <InsightsSlot view={insightsView} />
        <RecommendationsSlot view={recommendationsView} />
      </div>

      <ReportSlot
        view={reportView}
        busy={reportBusy}
        onGenerate={onGenerateReport}
        llmProvider={llmProvider}
      />
    </main>
  );
}

function FeatureRow({ f }: { f: FeatureDto }) {
  const info = metricInfo(f.featureId);
  const rel = reliabilityOf(f);
  const relText = reliabilityText(rel, f.factors);
  return (
    <li
      className={`feature-row${rel === "ok" ? "" : " feature-row--weak"}`}
      title={info.what}
    >
      <details className="feature-details">
        <summary>
          <span className="feature-id">
            {info.name}
            {rel === "low" && <span className="feature-badge">rough</span>}
            {rel === "single_input" && (
              <span className="feature-badge">one source</span>
            )}
          </span>
          <span className="feature-value">
            {formatFeatureValue(f.value)}
            {info.unit ? <span className="feature-unit"> {info.unit}</span> : null}
          </span>
        </summary>
        <div className="feature-explain">
          <p>{info.what}</p>
          <p className="status-meta">From: {info.from}</p>
          <p className="status-meta">{directionHint(info.direction)}</p>
          {f.factors && f.factors.length > 0 && (
            <p className="status-meta">
              This window:{" "}
              {f.factors
                .map((x) => `${x.label} ${Math.round(x.share * 100)}%`)
                .join(" · ")}
            </p>
          )}
          {typeof f.confidence === "number" && (
            <p className="status-meta">
              Data coverage: {Math.round(f.confidence * 100)}%
            </p>
          )}
          {relText && <p className="status-meta">{relText}</p>}
          <p className="status-meta feature-tech-id">Metric id: {f.featureId}</p>
        </div>
      </details>
    </li>
  );
}

function StatCards({ features }: { features: FeatureDto[] }) {
  const byId = new Map(features.map((f) => [f.featureId, f]));
  const fourth = HEADLINE_FOURTH.find((id) => byId.has(id));
  const ids = [...HEADLINE_IDS, ...(fourth ? [fourth] : [])];
  return (
    <section className="stat-cards" aria-label="Key metrics">
      {ids.map((id) => {
        const f = byId.get(id);
        const info = metricInfo(id);
        if (!f) {
          return (
            <div key={id} className="stat-card stat-card--empty">
              <p className="stat-name">{info.name}</p>
              <p className="stat-value">—</p>
              <p className="stat-foot">No data yet</p>
            </div>
          );
        }
        const { tone, word } = statTone(f);
        return (
          <div key={id} className={`stat-card stat-card--${tone}`} title={info.what}>
            <p className="stat-name">{info.name}</p>
            <p className="stat-value">
              {formatFeatureValue(f.value)}
              {info.unit && <span className="stat-unit">{info.unit === "0–100" ? "/100" : info.unit}</span>}
            </p>
            <p className="stat-foot">
              <span className={`tone-chip tone-chip--${tone}`}>{word}</span>
            </p>
          </div>
        );
      })}
    </section>
  );
}
