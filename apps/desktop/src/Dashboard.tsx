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

/** Soft refresh — idle-safe; no busy-loop. */
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

function InsightsSlot() {
  return (
    <section className="insights-slot" aria-label="Insights">
      <p className="chart-slot-title">Insights</p>
      <p className="status-meta">Insights will appear here later.</p>
    </section>
  );
}

export function Dashboard() {
  const [view, setView] = useState<SnapshotView>(() => loadingView());
  const [busy, setBusy] = useState(true);

  useEffect(() => {
    let cancelled = false;

    const load = async (isFirst: boolean) => {
      if (isFirst) {
        setBusy(true);
        setView(loadingView());
      }
      const next = await fetchFeatureSnapshot();
      if (cancelled) {
        return;
      }
      setView(next);
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
    void fetchFeatureSnapshot().then((next) => {
      setView(next);
      setBusy(false);
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

      <InsightsSlot />
    </main>
  );
}
