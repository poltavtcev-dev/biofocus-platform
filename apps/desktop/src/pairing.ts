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
        title: "Phone sync is not running",
        detail:
          info.ingestError ??
          "The local sync server did not start. Restart BioFocus; if it persists, report it (see TESTING.md).",
      };
    case "restart_needed":
      return {
        tone: "warn",
        title: "Restart BioFocus to apply",
        detail: info.lanConfigured
          ? "LAN is enabled in settings, but this Mac is still listening only on itself. Quit and reopen BioFocus, then press Reload."
          : "LAN was turned off in settings, but this Mac is still reachable on the network until you quit and reopen BioFocus.",
      };
    case "lan_off":
      return {
        tone: "info",
        title: "LAN is off — this Mac only",
        detail:
          "The iOS Simulator on this Mac can connect. To pair a real iPhone, tick “Enable LAN” below, restart BioFocus, then press Reload.",
      };
    case "lan_no_address":
      return {
        tone: "error",
        title: "LAN is on, but no network address was found",
        detail:
          "Check that Wi‑Fi is connected, then press Reload. If it still fails, set BIOFOCUS_INGEST_BIND_HOST to this Mac’s Wi‑Fi IPv4 (System Settings → Wi‑Fi → Details) and restart.",
      };
    case "lan_ready":
      return {
        tone: "ok",
        title: "Ready to pair",
        detail:
          "Your iPhone must be on the same Wi‑Fi. The address is https. Scan the QR: it carries the token and the certificate fingerprint. A different certificate is rejected.",
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
      return `${primaryBaseUrl(info)} (this Mac / Simulator only)`;
    case "restart_needed":
      return "Restart BioFocus, then Reload.";
    case "lan_no_address":
      return "No network address found — check Wi‑Fi, then Reload.";
    case "ingest_off":
      return "Phone sync is not running.";
  }
}

/** Short reachability label under the base URL. */
export function networkModeLabel(info: PairingTokenInfo): string {
  if (info.bindMode !== "lan") {
    return "This Mac";
  }
  if (needsLanHintFallback(info)) {
    return "Local network (address unavailable)";
  }
  return "Local network";
}

/** Calm, non-evaluative network copy (LAN opt-in / local only). */
export function networkModeDetail(info: PairingTokenInfo): string {
  if (info.bindMode === "lan") {
    return "Local network only, over TLS. Nothing leaves your Wi‑Fi. Every request still needs the token, and the phone pins the certificate fingerprint in the QR.";
  }
  return "Nothing is reachable from other devices while LAN is off.";
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
    ingestError: raw === "ingest_off" ? "Phone sync is off: port 8787 is already in use." : null,
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
      return { kind: "error", detail: "Pairing data was incomplete." };
    }
    return { kind: "ready", info };
  } catch (err) {
    const text = err instanceof Error ? err.message : String(err);
    return {
      kind: "error",
      detail: text.trim() || "Could not load pairing token.",
    };
  }
}

/** Replaces the pairing token. The QR changes; the phone must pair again. */
export async function rotatePairingToken(): Promise<PairingView> {
  try {
    const payload = await invoke<PairingPayload>("rotate_pairing_token");
    const info = normalize(payload);
    if (!info) {
      return { kind: "error", detail: "Pairing data was incomplete." };
    }
    return { kind: "ready", info };
  } catch (err) {
    const text = err instanceof Error ? err.message : String(err);
    return {
      kind: "error",
      detail: text.trim() || "Could not rotate the pairing token.",
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
