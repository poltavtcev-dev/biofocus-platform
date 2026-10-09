import { invoke } from "@tauri-apps/api/core";

/** Mirrors desktop `get_pairing_token` IPC (`docs/09-api.md`). */
export type PairingTokenInfo = {
  token: string;
  ingestBaseUrl: string;
  /** `"loopback"` | `"lan"` — from host advertise (P5-E1-T2). */
  bindMode: string;
  /** Usable base URLs; primary is also `ingestBaseUrl`. */
  baseUrlHints: string[];
  fromEnv: boolean;
  qrSvg: string;
  /** Saved settings ask for LAN (may need a restart to take effect). */
  lanConfigured: boolean;
  /** Saved settings differ from the running listener — restart BioFocus. */
  restartRequired: boolean;
  /** Ingest listener is up. */
  ingestRunning: boolean;
  /** Why ingest is not running (UI-safe). */
  ingestError: string | null;
  /** SHA-256 of the LAN certificate. Empty on loopback. */
  certFingerprint: string | null;
};

type PairingPayload = {
  lanConfigured?: boolean;
  restartRequired?: boolean;
  ingestRunning?: boolean;
  ingestError?: string | null;
  token?: string;
  ingestBaseUrl?: string;
  ingest_base_url?: string;
  bindMode?: string;
  bind_mode?: string;
  baseUrlHints?: string[];
  base_url_hints?: string[];
  fromEnv?: boolean;
  from_env?: boolean;
  qrSvg?: string;
  qr_svg?: string;
  certFingerprint?: string | null;
  cert_fingerprint?: string | null;
};

export type PairingView =
  | { kind: "idle" }
  | { kind: "loading" }
  | { kind: "ready"; info: PairingTokenInfo }
  | { kind: "error"; detail: string };

function normalize(payload: PairingPayload): PairingTokenInfo | null {
  const token = payload.token?.trim();
  const ingestBaseUrl = (
    payload.ingestBaseUrl ??
    payload.ingest_base_url ??
    ""
  ).trim();
  const qrSvg = (payload.qrSvg ?? payload.qr_svg ?? "").trim();
  if (!token || !ingestBaseUrl || !qrSvg) {
    return null;
  }
  const bindMode = (
    payload.bindMode ??
    payload.bind_mode ??
    "loopback"
  ).trim();
  const baseUrlHints = (
    payload.baseUrlHints ??
    payload.base_url_hints ??
    [ingestBaseUrl]
  )
    .map((u) => u.trim())
    .filter(Boolean);
  return {
    token,
    ingestBaseUrl,
    bindMode: bindMode || "loopback",
    baseUrlHints: baseUrlHints.length > 0 ? baseUrlHints : [ingestBaseUrl],
    fromEnv: Boolean(payload.fromEnv ?? payload.from_env),
    qrSvg,
    lanConfigured: Boolean(payload.lanConfigured ?? bindMode === "lan"),
    restartRequired: Boolean(payload.restartRequired),
    ingestRunning: payload.ingestRunning ?? true,
    ingestError: payload.ingestError?.trim() || null,
    certFingerprint:
      (payload.certFingerprint ?? payload.cert_fingerprint ?? "").trim() || null,
  };
}

/** Masks all but the last 4 characters for calm default display. */
export function maskToken(token: string): string {
  if (token.length <= 4) {
    return "••••";
  }
  return `${"•".repeat(Math.min(token.length - 4, 12))}${token.slice(-4)}`;
}

/** True when URL is loopback (Simulator / same-machine). */
export function isLoopbackBaseUrl(url: string): boolean {
  const trimmed = url.trim();
  try {
    const host = new URL(trimmed).hostname.toLowerCase();
    return host === "127.0.0.1" || host === "localhost" || host === "::1";
  } catch {
    return (
      trimmed.includes("127.0.0.1") ||
      trimmed.includes("localhost") ||
      trimmed.includes("[::1]")
    );
  }
}

/**
 * Primary companion base URL from IPC — already prefers LAN hint when present.
 * Do not re-derive LAN IP in the UI.
 */
export function primaryBaseUrl(info: PairingTokenInfo): string {
  return info.ingestBaseUrl;
}

/** LAN opt-in but no usable non-loopback hint (discovery empty / offline). */
export function needsLanHintFallback(info: PairingTokenInfo): boolean {
  if (info.bindMode !== "lan") {
    return false;
  }
  if (info.baseUrlHints.length === 0) {
    return true;
  }
  return isLoopbackBaseUrl(info.ingestBaseUrl);
}

/** One clear phone-connection state for the Companion panel. */
export type CompanionNetworkStatus =
  | "ingest_off"
  | "restart_needed"
  | "lan_off"
  | "lan_no_address"
  | "lan_ready";

export function companionNetworkStatus(info: PairingTokenInfo): CompanionNetworkStatus {
  if (!info.ingestRunning) {
    return "ingest_off";
  }
  if (info.restartRequired) {
    return "restart_needed";
  }
  if (info.bindMode !== "lan") {
    return "lan_off";
  }
  if (needsLanHintFallback(info)) {
    return "lan_no_address";
  }
  return "lan_ready";
}

/** Plain-language headline + next step for each state. */
export function companionStatusCopy(info: PairingTokenInfo): {
  tone: "ok" | "info" | "warn" | "error";
  title: string;
  detail: string;
} {
  switch (companionNetworkStatus(info)) {
    case "ingest_off":
      return {
        tone: "error",
        title: "Синхронизация с телефоном не запущена",
        detail:
          info.ingestError ??
          "Локальный сервер синхронизации не стартовал. Перезапустите BioFocus. Если повторится, это стоит сообщить.",
      };
    case "restart_needed":
      return {
        tone: "warn",
        title: "Перезапустите BioFocus",
        detail: info.lanConfigured
          ? "LAN включён в настройках, но этот Mac всё ещё слушает только себя. Закройте и снова откройте BioFocus, затем нажмите «Обновить»."
          : "LAN выключен в настройках, но Mac останется доступен в сети, пока вы не закроете и не откроете BioFocus снова.",
      };
    case "lan_off":
      return {
        tone: "info",
        title: "LAN выключен — только этот Mac",
        detail:
          "Симулятор iOS на этом Mac подключится. Для настоящего iPhone включите «LAN для iPhone» ниже, перезапустите BioFocus и нажмите «Обновить».",
      };
    case "lan_no_address":
      return {
        tone: "error",
        title: "LAN включён, но адрес сети не найден",
        detail:
          "Проверьте Wi‑Fi и нажмите «Обновить». Если не поможет, задайте BIOFOCUS_INGEST_BIND_HOST равным IPv4 этого Mac (Системные настройки → Wi‑Fi → Подробнее) и перезапустите.",
      };
    case "lan_ready":
      return {
        tone: "ok",
        title: "Можно подключать телефон",
        detail:
          "iPhone должен быть в той же сети Wi‑Fi. Адрес — https. В QR токен и отпечаток сертификата. Чужой сертификат телефон отклонит.",
      };
  }
}

/** @deprecated Use {@link companionStatusCopy}. Kept for older imports. */
export function companionLoopbackWarning(info: PairingTokenInfo): string | null {
  return companionNetworkStatus(info) === "lan_off" ? companionStatusCopy(info).detail : null;
}

/** @deprecated Use {@link companionStatusCopy}. */
export function companionLanAddressError(info: PairingTokenInfo): string | null {
  return companionNetworkStatus(info) === "lan_no_address"
    ? companionStatusCopy(info).detail
    : null;
}

/** Whether the primary base URL is safe to copy for a physical phone. */
export function isPrimaryUrlCopyable(info: PairingTokenInfo): boolean {
  return companionNetworkStatus(info) === "lan_ready";
}

/** Text shown in place of the Base URL when it is not usable for a phone. */
export function baseUrlPlaceholder(info: PairingTokenInfo): string {
  switch (companionNetworkStatus(info)) {
    case "lan_ready":
      return primaryBaseUrl(info);
    case "lan_off":
      return `${primaryBaseUrl(info)} (только этот Mac / симулятор)`;
    case "restart_needed":
      return "Перезапустите BioFocus, затем «Обновить».";
    case "lan_no_address":
      return "Адрес сети не найден — проверьте Wi‑Fi и нажмите «Обновить».";
    case "ingest_off":
      return "Синхронизация с телефоном не запущена.";
  }
}

/** Short reachability label under the base URL. */
export function networkModeLabel(info: PairingTokenInfo): string {
  if (info.bindMode !== "lan") {
    return "Этот Mac";
  }
  if (needsLanHintFallback(info)) {
    return "Локальная сеть (адрес недоступен)";
  }
  return "Локальная сеть";
}

/** Calm, non-evaluative network copy (LAN opt-in / local only). */
export function networkModeDetail(info: PairingTokenInfo): string {
  if (info.bindMode === "lan") {
    return "Только локальная сеть, по TLS. За пределы Wi‑Fi ничего не уходит. Каждый запрос всё равно с токеном, а телефон сверяет отпечаток сертификата из QR.";
  }
  return "Пока LAN выключен, другие устройства достучаться не могут.";
}

/** QA: `?mockPairing=ready|lan_off|restart|no_address|ingest_off` (layout / screenshots). */
export function mockPairingFromLocation(
  search: string = typeof window !== "undefined" ? window.location.search : "",
): PairingView | null {
  const raw = new URLSearchParams(search).get("mockPairing");
  if (!raw) {
    return null;
  }
  const lan = raw !== "lan_off";
  const info: PairingTokenInfo = {
    token: "0000000000000000000000000000000000000000000000000000000000mock",
    ingestBaseUrl: lan && raw !== "no_address" ? "https://192.168.0.37:8787" : "http://127.0.0.1:8787",
    bindMode: raw === "lan_off" || raw === "restart" ? "loopback" : "lan",
    baseUrlHints: raw === "no_address" ? [] : [lan ? "https://192.168.0.37:8787" : "http://127.0.0.1:8787"],
    fromEnv: false,
    qrSvg: "<svg xmlns='http://www.w3.org/2000/svg' width='168' height='168'/>",
    lanConfigured: lan,
    restartRequired: raw === "restart",
    ingestRunning: raw !== "ingest_off",
    ingestError: raw === "ingest_off" ? "Синхронизация с телефоном выключена: порт 8787 уже занят." : null,
    certFingerprint:
      lan && raw !== "no_address" && raw !== "lan_off" && raw !== "restart"
        ? "ab".repeat(32)
        : null,
  };
  return { kind: "ready", info };
}

export async function fetchPairingToken(): Promise<PairingView> {
  const mocked = mockPairingFromLocation();
  if (mocked) {
    return mocked;
  }
  try {
    const payload = await invoke<PairingPayload>("get_pairing_token");
    const info = normalize(payload);
    if (!info) {
      return { kind: "error", detail: "Данные подключения неполные." };
    }
    return { kind: "ready", info };
  } catch (err) {
    const text = err instanceof Error ? err.message : String(err);
    return {
      kind: "error",
      detail: text.trim() || "Не удалось загрузить токен.",
    };
  }
}

/** Replaces the pairing token. The QR changes; the phone must pair again. */
export async function rotatePairingToken(): Promise<PairingView> {
  try {
    const payload = await invoke<PairingPayload>("rotate_pairing_token");
    const info = normalize(payload);
    if (!info) {
      return { kind: "error", detail: "Данные подключения неполные." };
    }
    return { kind: "ready", info };
  } catch (err) {
    const text = err instanceof Error ? err.message : String(err);
    return {
      kind: "error",
      detail: text.trim() || "Не удалось сменить токен.",
    };
  }
}

export async function copyText(value: string): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(value);
    return true;
  } catch {
    return false;
  }
}
