import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

/** QA: `?mockDataSources=ready|empty` paints the tab without Core. */
const MOCK_SAMPLE = 1_791_455_400;
const MOCK_RECEIVED = 1_791_455_520;

export type TypeCount = {
  dataType: string;
  last24h: number;
  last7d: number;
  last30d: number;
};

export type SourceCard = {
  kind: string;
  app: string;
  deviceModel: string;
  lastSampleUnix: number;
  lastReceivedUnix: number;
  counts: TypeCount[];
  coverage: number;
  priorityRank: number;
  active: boolean;
};

export type CompanionSync = {
  phase: string;
  typesOk: number;
  typesEmpty: number;
  typesTotal: number;
  pending: number;
  receivedAt: number;
};

export type DataSourcesDto = {
  sources: SourceCard[];
  priority: string[];
  hints: string[];
  companion: CompanionSync | null;
  timezone: string;
};

type DataSourcesView =
  | { kind: "loading" }
  | { kind: "ready"; data: DataSourcesDto }
  | { kind: "empty"; data: DataSourcesDto }
  | { kind: "error"; detail: string };

const KIND_LABEL: Record<string, string> = {
  apple_watch: "Apple Watch",
  iphone: "iPhone",
  xiaomi_mi_fitness: "Xiaomi · Mi Fitness",
  zepp_life: "Zepp Life",
  other_app: "Другое приложение",
  manual: "Вручную",
};

const TYPE_LABEL: Record<string, string> = {
  heart_rate: "Пульс",
  resting_heart_rate: "Пульс покоя",
  walking_heart_rate_average: "Пульс при ходьбе",
  hrv: "HRV",
  respiratory_rate: "Дыхание",
  sleeping_wrist_temperature: "Температура запястья",
  vo2_max: "VO₂max",
  oxygen_saturation: "Кислород",
  sleep_interval: "Сон",
  workout: "Тренировки",
  exercise_time: "Упражнения",
  stand_time: "Стоя",
  stand_hour: "Часы стоя",
  mindful_session: "Осознанность",
  distance_walking_running: "Дистанция",
  basal_energy: "Базовая энергия",
  step_count: "Шаги",
  active_energy: "Активная энергия",
};

const PHASE_LABEL: Record<string, string> = {
  idle: "ожидание",
  recent: "сначала последние 30 дней",
  history: "история",
  done: "окно дочитано",
};

/** Clock time in Europe/Belgrade, independent of the Mac zone. */
export function formatBelgrade(unixSecs: number): string {
  return new Intl.DateTimeFormat("ru-RU", {
    timeZone: "Europe/Belgrade",
    day: "2-digit",
    month: "short",
    year: "numeric",
    hour: "2-digit",
    minute: "2-digit",
    hourCycle: "h23",
  }).format(new Date(unixSecs * 1000));
}

function kindLabel(kind: string): string {
  return KIND_LABEL[kind] ?? kind;
}

function typeLabel(dataType: string): string {
  return TYPE_LABEL[dataType] ?? dataType;
}

export function mockDataSourcesFromLocation(
  search: string = typeof window !== "undefined" ? window.location.search : "",
): DataSourcesDto | null {
  const raw = new URLSearchParams(search).get("mockDataSources");
  if (raw === "empty") {
    return {
      sources: [],
      priority: [
        "apple_watch",
        "xiaomi_mi_fitness",
        "zepp_life",
        "iphone",
        "other_app",
        "manual",
      ],
      hints: [],
      companion: null,
      timezone: "Europe/Belgrade",
    };
  }
  if (raw !== "ready") {
    return null;
  }
  return {
    timezone: "Europe/Belgrade",
    priority: [
      "apple_watch",
      "xiaomi_mi_fitness",
      "zepp_life",
      "iphone",
      "other_app",
      "manual",
    ],
    companion: {
      phase: "recent",
      typesOk: 6,
      typesEmpty: 2,
      typesTotal: 18,
      pending: 3,
      receivedAt: MOCK_RECEIVED,
    },
    hints: [
      "HRV: не видно ни от одного источника — Xiaomi через Zepp Life его не передаёт; нужен Apple Watch.",
    ],
    sources: [
      {
        kind: "apple_watch",
        app: "Health",
        deviceModel: "Watch",
        lastSampleUnix: MOCK_SAMPLE,
        lastReceivedUnix: MOCK_RECEIVED,
        coverage: 8 / 30,
        priorityRank: 0,
        active: true,
        counts: [
          { dataType: "heart_rate", last24h: 12, last7d: 80, last30d: 240 },
          { dataType: "sleep_interval", last24h: 1, last7d: 6, last30d: 20 },
        ],
      },
      {
        kind: "zepp_life",
        app: "Zepp Life",
        deviceModel: "Band",
        lastSampleUnix: MOCK_SAMPLE - 3600,
        lastReceivedUnix: MOCK_RECEIVED - 600,
        coverage: 4 / 30,
        priorityRank: 2,
        active: true,
        counts: [{ dataType: "step_count", last24h: 1, last7d: 7, last30d: 22 }],
      },
    ],
  };
}

function viewFrom(data: DataSourcesDto): DataSourcesView {
  return data.sources.length === 0
    ? { kind: "empty", data }
    : { kind: "ready", data };
}

async function fetchDataSources(): Promise<DataSourcesView> {
  const mocked = mockDataSourcesFromLocation();
  if (mocked) {
    return viewFrom(mocked);
  }
  try {
    const payload = await invoke<DataSourcesDto>("get_data_sources");
    if (!payload || !Array.isArray(payload.sources)) {
      return { kind: "error", detail: "Источник ответа не распознан." };
    }
    return viewFrom({
      ...payload,
      hints: payload.hints ?? [],
      priority: payload.priority ?? [],
      companion: payload.companion ?? null,
      timezone: payload.timezone || "Europe/Belgrade",
    });
  } catch {
    return { kind: "error", detail: "Не удалось прочитать источники." };
  }
}

async function saveOrder(order: string[]): Promise<DataSourcesView> {
  const mocked = mockDataSourcesFromLocation();
  if (mocked) {
    return viewFrom({ ...mocked, priority: order });
  }
  try {
    const payload = await invoke<DataSourcesDto>("set_source_priority", { order });
    return viewFrom(payload);
  } catch {
    return { kind: "error", detail: "Не удалось сохранить порядок." };
  }
}

function move(order: string[], kind: string, direction: -1 | 1): string[] {
  const index = order.indexOf(kind);
  const next = index + direction;
  if (index < 0 || next < 0 || next >= order.length) {
    return order;
  }
  const copy = order.slice();
  const [item] = copy.splice(index, 1);
  copy.splice(next, 0, item);
  return copy;
}

export function DataSourcesPanel() {
  const [view, setView] = useState<DataSourcesView>({ kind: "loading" });

  useEffect(() => {
    let cancelled = false;
    const load = () => {
      void fetchDataSources().then((next) => {
        if (!cancelled) {
          setView(next);
        }
      });
    };
    load();
    const timer = window.setInterval(load, 30_000);
    return () => {
      cancelled = true;
      window.clearInterval(timer);
    };
  }, []);

  const data = view.kind === "ready" || view.kind === "empty" ? view.data : null;

  const onMove = (kind: string, direction: -1 | 1) => {
    if (!data) {
      return;
    }
    const order = move(data.priority, kind, direction);
    void saveOrder(order).then(setView);
  };

  return (
    <section className="sources-panel" aria-label="Источники данных" data-timezone="Europe/Belgrade">
      <p className="chart-slot-title">Источники данных</p>
      <p className="status-meta">
        Время сэмпла и получения — Europe/Belgrade. Это наблюдение о том, что пришло, не диагноз.
      </p>

      {view.kind === "loading" && <p className="status-detail">Загрузка…</p>}
      {view.kind === "error" && <p className="status-detail">{view.detail}</p>}

      {data?.companion && (
        <p className="sources-companion">
          Компаньон: {PHASE_LABEL[data.companion.phase] ?? data.companion.phase}. Готово{" "}
          {data.companion.typesOk} из {data.companion.typesTotal}, пустых{" "}
          {data.companion.typesEmpty}, в очереди {data.companion.pending}. Обновлено{" "}
          {formatBelgrade(data.companion.receivedAt)}.
        </p>
      )}

      {data && data.hints.length > 0 && (
        <ul className="sources-hints">
          {data.hints.map((hint) => (
            <li key={hint}>{hint}</li>
          ))}
        </ul>
      )}

      {data && (
        <ol className="sources-priority">
          {data.priority.map((kind, index) => (
            <li key={kind}>
              <span>
                {index + 1}. {kindLabel(kind)}
              </span>
              <span className="sources-priority-actions">
                <button type="button" onClick={() => onMove(kind, -1)} disabled={index === 0}>
                  Выше
                </button>
                <button
                  type="button"
                  onClick={() => onMove(kind, 1)}
                  disabled={index === data.priority.length - 1}
                >
                  Ниже
                </button>
              </span>
            </li>
          ))}
        </ol>
      )}

      {view.kind === "empty" && (
        <p className="status-detail">Пока нет наблюдений с часов за 30 дней.</p>
      )}

      {view.kind === "ready" && data && (
        <ul className="sources-cards">
          {data.sources.map((source) => (
            <li key={`${source.kind}-${source.app}-${source.deviceModel}`} className="sources-card">
              <p className="sources-card-title">
                {kindLabel(source.kind)}
                {source.app ? ` · ${source.app}` : ""}
                {source.deviceModel ? ` · ${source.deviceModel}` : ""}
              </p>
              <p className="status-meta">
                {source.active ? "Сейчас берётся для своих типов" : "Есть данные, приоритет ниже"}
                {" · "}
                покрытие {Math.round(source.coverage * 100)}% дней
              </p>
              <p className="status-meta">
                Последний сэмпл {formatBelgrade(source.lastSampleUnix)} · получено{" "}
                {formatBelgrade(source.lastReceivedUnix)}
              </p>
              <ul className="sources-counts">
                {source.counts.map((count) => (
                  <li key={count.dataType}>
                    {typeLabel(count.dataType)}: {count.last24h} / {count.last7d} / {count.last30d}
                    <span className="status-meta"> за 24 ч / 7 д / 30 д</span>
                  </li>
                ))}
              </ul>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
