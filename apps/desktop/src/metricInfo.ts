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
    name: "Фокус",
    what: "Насколько ровной выглядела работа за последние 15 минут.",
    from: "Одно и то же приложение (главный вес), ровный набор текста (тихое чтение — нейтрально) и вариабельность пульса, если часы подключены.",
    direction: "higher_better",
    unit: "0–100",
  },
  ContextSwitchRate: {
    name: "Переключения",
    what: "Как часто менялось переднее приложение за последние 15 минут, в пересчёте на минуту.",
    from: "История активного окна на этом Mac. 1,0 — около 15 переключений за 15 минут.",
    direction: "neutral",
    unit: "в минуту",
  },
  CognitiveLoad: {
    name: "Общая нагрузка",
    what: "Сколько одновременно тянуло внимание: встречи, переключения приложений и уведомления.",
    from: "Занятость календаря, переключения и число уведомлений — что из этого включено. Если источник один, цифра в основном про него.",
    direction: "lower_better",
    unit: "0–100",
  },
  DeepWorkScore: {
    name: "Глубокая работа",
    what: "Фокус, который держался ровно и без частых прыжков между приложениями.",
    from: "Фокус и ровность переключений.",
    direction: "higher_better",
    unit: "0–100",
  },
  AttentionStability: {
    name: "Устойчивость внимания",
    what: "Насколько мало фокус качался вверх и вниз внутри окна.",
    from: "Изменения фокуса за окно и ровность переключений.",
    direction: "higher_better",
    unit: "0–100",
  },
  DistractionScore: {
    name: "Смесь сайтов",
    what: "Насколько просмотр в браузере был раздроблен по типам сайтов.",
    from: "Грубые категории браузера (работа, общение, развлечения и так далее) — без адресов и заголовков — плюс переключения приложений.",
    direction: "lower_better",
    unit: "0–100",
  },
  StressIndex: {
    name: "Индекс напряжения",
    what: "Оценка напряжения по вариабельности пульса. Нужны подключённые часы или телефон.",
    from: "Показатели HRV (RMSSD или SDNN). Это не медицинское измерение.",
    direction: "lower_better",
    unit: "0–100",
  },
  FatigueIndex: {
    name: "Усталость",
    what: "Насколько вымотанным выглядит день к этому моменту.",
    from: "Тренд фокуса, время активности сегодня и сдвиг пульса, если он есть.",
    direction: "lower_better",
    unit: "0–100",
  },
  RecoveryScore: {
    name: "Восстановление",
    what: "Насколько восстановленным выглядит тело сейчас.",
    from: "HRV и пульс покоя с подключённого телефона.",
    direction: "higher_better",
    unit: "0–100",
  },
  MeetingDensity: {
    name: "Доля встреч",
    what: "Какая доля последних 15 минут была занята событиями календаря.",
    from: "Занятые слоты календаря (только время, без названий).",
    direction: "neutral",
    unit: "0–1",
  },
  RecoveryBetweenMeetings: {
    name: "Паузы между встречами",
    what: "Сколько воздуха было между встречами подряд.",
    from: "Промежутки между занятыми слотами календаря.",
    direction: "higher_better",
    unit: "0–100",
  },
  NotificationPressure: {
    name: "Уведомления",
    what: "Сколько уведомлений пришло недавно.",
    from: "Только число уведомлений, никогда не их текст.",
    direction: "lower_better",
    unit: "0–100",
  },
  GitActivityRate: {
    name: "Активность Git",
    what: "Сколько было коммитов, переключений и синхронизаций в выбранных репозиториях.",
    from: "События Git в папках из блока «Папки Git».",
    direction: "neutral",
  },
  ActivityBalance: {
    name: "Движение",
    what: "Насколько много вы двигались недавно.",
    from: "Шаги и записанные тренировки с телефона.",
    direction: "higher_better",
    unit: "0–100",
  },
  EnergyScore: {
    name: "Энергия",
    what: "Грубая оценка энергии по активности, пульсу и отдыху.",
    from: "Активная энергия, пульс и сон с телефона.",
    direction: "higher_better",
    unit: "0–100",
  },
  SleepDebt: {
    name: "Недосып",
    what: "Насколько недавний сон короче обычной ночи.",
    from: "Интервалы сна с телефона.",
    direction: "lower_better",
    unit: "0–100",
  },
  DeskAwayPresence: {
    name: "Вдали от стола",
    what: "Насколько вероятно, что вас не было за столом. Показывается только при явных следах, например шагах или прогулке.",
    from: "Тихая клавиатура и приложения плюс шаги или записанная прогулка.",
    direction: "neutral",
    unit: "0–100",
  },
  CircadianOffset: {
    name: "Совпадение с ритмом",
    what: "Насколько время работы совпадает с ритмом сна.",
    from: "Середина сна против времени, когда вы активны.",
    direction: "higher_better",
    unit: "0–100",
  },
  SustainedLoadIndicator: {
    name: "Длительная нагрузка",
    what: "Держалось ли напряжение долго, а не только короткий всплеск.",
    from: "Напряжение, усталость и доля встреч на более длинном окне.",
    direction: "lower_better",
    unit: "0–100",
  },
  AmbientMediaShare: {
    name: "Медиа",
    what: "Какая доля окна прошла с музыкой или подкастом.",
    from: "Только тип Now Playing, без названий треков.",
    direction: "neutral",
    unit: "%",
  },
  AmbientLightShare: {
    name: "Освещение",
    what: "Какая доля окна прошла при более ярком свете.",
    from: "Грубый уровень света: темно, тускло, умеренно, ярко.",
    direction: "neutral",
    unit: "%",
  },
  RestingHeartRate: {
    name: "Пульс покоя",
    what: "Последний пульс покоя рядом с вашей недавней медианой, когда дней уже достаточно.",
    from: "Сэмплы пульса покоя с телефона. Для сравнения нужны пять предыдущих дней.",
    direction: "neutral",
  },
  HrvVsBaseline: {
    name: "Вариабельность пульса",
    what: "Как сегодняшняя вариабельность выглядит на фоне ваших недавних дней. SDNN и RMSSD не смешиваются.",
    from: "Только один метод и только после пяти предыдущих дней этого же метода.",
    direction: "neutral",
  },
  SleepStages: {
    name: "Стадии сна",
    what: "Время сна и доли глубокого, REM и основного сна, если источник записал стадии. Иначе только общий сон.",
    from: "Интервалы сна с телефона.",
    direction: "neutral",
  },
  NightSpO2: {
    name: "Кислород во сне",
    what: "Среднее и самое низкое насыщение кислородом за ночь, рядом с вашими недавними ночами, когда их уже достаточно.",
    from: "Сэмплы насыщения кислородом. Только для контекста.",
    direction: "neutral",
  },
  RespiratoryRate: {
    name: "Частота дыхания",
    what: "Средняя и самая низкая частота дыхания рядом с вашими недавними ночами, когда их уже достаточно.",
    from: "Сэмплы частоты дыхания. Только для контекста.",
    direction: "neutral",
  },
  WristTemperature: {
    name: "Температура запястья",
    what: "Насколько температура запястья за ночь отличается от ваших недавних ночей.",
    from: "Изменение температуры запястья с часов. Только для контекста.",
    direction: "neutral",
  },
};

export function metricInfo(featureId: string): MetricInfo {
  return (
    METRIC_INFO[featureId] ?? {
      name: featureId,
      what: "Экспериментальная метрика.",
      from: "Локальные сигналы на этом Mac.",
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

/** A metric whose window ended longer ago than this is shown as stale. */
export const STALE_AFTER_SECS = 18 * 60 * 60;

export function isStaleFeature(windowEndUnix: number, nowUnix = Date.now() / 1000): boolean {
  return Number.isFinite(windowEndUnix) && nowUnix - windowEndUnix > STALE_AFTER_SECS;
}

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
      return "Грубая оценка — в этом окне пока мало данных.";
    case "single_input":
      return `Только один источник (${factors?.[0]?.label ?? "один сигнал"}) — это подсказка, не вывод.`;
    case "ok":
      return null;
  }
}

export function directionHint(d: MetricDirection): string {
  switch (d) {
    case "higher_better":
      return "Выше — спокойнее.";
    case "lower_better":
      return "Ниже — спокойнее.";
    case "neutral":
      return "Ни хорошо, ни плохо — только контекст.";
  }
}

export type StatTone = "ok" | "elevated" | "rough" | "neutral";

/** Calm tone + word for a headline stat card. */
export function statTone(
  f: { featureId: string; value: unknown; confidence?: number; factors?: { id: string }[] },
): { tone: StatTone; word: string } {
  if (reliabilityOf(f) !== "ok") {
    return { tone: "rough", word: "грубая оценка" };
  }
  const v = typeof f.value === "number" ? f.value : NaN;
  const dir = metricInfo(f.featureId).direction;
  if (!Number.isFinite(v) || dir === "neutral") {
    if (f.featureId === "ContextSwitchRate" && Number.isFinite(v)) {
      return v >= 2 ? { tone: "elevated", word: "много" } : { tone: "ok", word: "ровно" };
    }
    return { tone: "neutral", word: "контекст" };
  }
  if (dir === "higher_better") {
    return v >= 55 ? { tone: "ok", word: "ровно" } : { tone: "elevated", word: "ниже" };
  }
  return v <= 55 ? { tone: "ok", word: "спокойно" } : { tone: "elevated", word: "выше" };
}

/** Keep only the newest window per metric (snapshot lists every sliding window). */
export function latestPerFeature<T extends { featureId: string; timeWindow: { end: number } }>(
  features: T[],
): T[] {
  const best = new Map<string, T>();
  for (const f of features) {
    const cur = best.get(f.featureId);
    if (!cur || f.timeWindow.end > cur.timeWindow.end) {
      best.set(f.featureId, f);
    }
  }
  return [...best.values()];
}

/** Headline metrics, in display order; first three always, fourth = first available. */
export const HEADLINE_IDS = ["FocusScore", "CognitiveLoad", "ContextSwitchRate"];
export const HEADLINE_FOURTH = ["RecoveryScore", "StressIndex", "DeepWorkScore"];
