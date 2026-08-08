import { useEffect, useState } from "react";
import {
  CartesianGrid,
  Legend,
  Line,
  LineChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";
import {
  buildChartPoints,
  CHART_SERIES_META,
  formatChartTime,
  presentSeriesIds,
  SCORE_SERIES_IDS,
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

/** Soft refresh — idle-safe; no busy-loop. Does not regenerate reports. */
const SNAPSHOT_POLL_MS = 30_000;

function ChartSlot({
  empty,
  features,
}: {
  empty: boolean;
  features: FeatureDto[];
}) {
  const points = empty ? [] : buildChartPoints(features);
  const series = presentSeriesIds(points);
  const scoreSeries = series.filter((id) => SCORE_SERIES_IDS.includes(id));
  const showCsr = series.includes("ContextSwitchRate");

  return (
    <section className="chart-slot" aria-label="Feature charts">
      <p className="chart-slot-title">Features</p>
      <div
        className={`chart-slot-body${points.length > 0 ? " chart-slot-body--chart" : ""}`}
      >
        {points.length === 0 ? (
          <p className="chart-slot-placeholder">
            {empty
              ? "No Feature series yet."
              : "No chartable Feature values in this snapshot."}
          </p>
        ) : (
          <FeatureSeriesChart
            points={points}
            scoreSeries={scoreSeries}
            showCsr={showCsr}
          />
        )}
      </div>
      {points.length > 0 && (
        <p className="chart-slot-units" aria-hidden>
          Scores 0–100
          {showCsr ? " · Context switches per window minute (right)" : ""}
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
  return (
    <ResponsiveContainer width="100%" height={220}>
      <LineChart
        data={points}
        margin={{ top: 8, right: showCsr ? 12 : 4, left: 0, bottom: 0 }}
      >
        <CartesianGrid stroke="var(--bf-bg-accent)" strokeDasharray="3 3" />
        <XAxis
          dataKey="t"
          tickFormatter={formatChartTime}
          tick={{ fill: "var(--bf-meta)", fontSize: 11 }}
          axisLine={{ stroke: "var(--bf-retry-border)" }}
          tickLine={false}
          minTickGap={28}
        />
        <YAxis
          yAxisId="score"
          domain={[0, 100]}
          width={36}
          tick={{ fill: "var(--bf-meta)", fontSize: 11 }}
          axisLine={false}
          tickLine={false}
        />
        {showCsr && (
          <YAxis
            yAxisId="csr"
            orientation="right"
            width={36}
            tick={{ fill: "var(--bf-meta)", fontSize: 11 }}
            axisLine={false}
            tickLine={false}
            allowDecimals
          />
        )}
        <Tooltip
          contentStyle={{
            background: "var(--bf-bg)",
            border: "1px solid var(--bf-retry-border)",
            borderRadius: 8,
            fontSize: 12,
          }}
          labelFormatter={(label) =>
            typeof label === "number" ? formatChartTime(label) : String(label)
          }
          formatter={(value: number, name: string) => {
            const id = name as ChartFeatureId;
            const meta = CHART_SERIES_META[id];
            const display =
              typeof value === "number"
                ? Number.isInteger(value)
                  ? String(value)
                  : value.toFixed(1)
                : String(value);
            return [display, meta?.label ?? name];
          }}
        />
        <Legend
          formatter={(value) => {
            const id = value as ChartFeatureId;
            return CHART_SERIES_META[id]?.label ?? value;
          }}
          wrapperStyle={{ fontSize: 12, color: "var(--bf-muted)" }}
        />
        {scoreSeries.map((id) => (
          <Line
            key={id}
            yAxisId="score"
            type="monotone"
            dataKey={id}
            name={id}
            stroke={CHART_SERIES_META[id].color}
            strokeWidth={2}
            dot={{ r: 2.5, strokeWidth: 0 }}
            activeDot={{ r: 4 }}
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
            stroke={CHART_SERIES_META.ContextSwitchRate.color}
            strokeWidth={2}
            strokeDasharray="4 3"
            dot={{ r: 2.5, strokeWidth: 0 }}
            activeDot={{ r: 4 }}
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
              <li key={item.id} className="insight-row">
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
}: {
  view: ReportView;
  busy: boolean;
  onGenerate: () => void;
}) {
  const report = view.report;
  const showBody = view.kind === "ready" && report;

  return (
    <section className="report-slot" aria-label="Report" aria-live="polite">
      <p className="chart-slot-title">Report</p>
      <p className="status-meta">
        {view.kind === "loading" ? view.detail : view.label}
      </p>
      {view.kind !== "loading" && (
        <p className="status-meta">{view.detail}</p>
      )}
      <p className="report-ai-note">
        AI interpretation is local and optional — never sent automatically.
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
  const [insightsView, setInsightsView] = useState<InsightsView>(() =>
    loadingInsightsView(),
  );
  const [recommendationsView, setRecommendationsView] =
    useState<RecommendationsView>(() => loadingRecommendationsView());
  const [reportView, setReportView] = useState<ReportView>(
    () => mockReportFromLocation() ?? idleReportView(),
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
      // Snapshot / Insights / Recommendations only — never auto-invoke report / LLM.
      const [next, nextInsights, nextRecommendations] = await Promise.all([
        fetchFeatureSnapshot(),
        fetchInsights(),
        fetchRecommendations(),
      ]);
      if (cancelled) {
        return;
      }
      setView(next);
      setInsightsView(nextInsights);
      setRecommendationsView(nextRecommendations);
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

  const onRetry = () => {
    setBusy(true);
    setView(loadingView());
    setInsightsView(loadingInsightsView());
    setRecommendationsView(loadingRecommendationsView());
    void Promise.all([
      fetchFeatureSnapshot(),
      fetchInsights(),
      fetchRecommendations(),
    ]).then(([next, nextInsights, nextRecommendations]) => {
      setView(next);
      setInsightsView(nextInsights);
      setRecommendationsView(nextRecommendations);
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

  const features = view.snapshot?.features ?? [];
  const signals = view.snapshot?.signals ?? [];
  const isEmpty = view.kind === "empty";
  const isReady = view.kind === "ready";
  const chartEmpty =
    isEmpty || view.kind === "loading" || view.kind === "error";

  return (
    <main className="shell shell--dashboard" data-snapshot={view.kind}>
      <header className="brand brand--dashboard">
        <h1>BioFocus</h1>
        <p className="brand-sub">Dashboard</p>
      </header>

      <section className="status-block" aria-live="polite">
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

      <ChartSlot empty={chartEmpty} features={features} />

      {isReady && features.length > 0 && (
        <section className="feature-list" aria-label="Feature snapshot">
          <p className="chart-slot-title">Snapshot</p>
          <ul className="feature-rows">
            {features.map((f) => (
              <li key={`${f.featureId}-${f.timeWindow.end}`} className="feature-row">
                <span className="feature-id">{f.featureId}</span>
                <span className="feature-value">{formatFeatureValue(f.value)}</span>
              </li>
            ))}
          </ul>
          {signals.length > 0 && (
            <p className="status-meta">
              Signals: {signals.map((s) => s.type).join(", ")}
            </p>
          )}
        </section>
      )}

      <InsightsSlot view={insightsView} />

      <RecommendationsSlot view={recommendationsView} />

      <ReportSlot
        view={reportView}
        busy={reportBusy}
        onGenerate={onGenerateReport}
      />
    </main>
  );
}
