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
};

type PairingPayload = {
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

/** Calm warning when physical phone cannot use the shown base URL. */
export function companionLoopbackWarning(info: PairingTokenInfo): string | null {
  if (info.bindMode === "loopback") {
    return "Physical iPhone needs LAN — enable LAN below, restart BioFocus, then copy the LAN Base URL (not 127.0.0.1).";
  }
  return null;
}

/** Error when LAN is on but no usable address was discovered. */
export function companionLanAddressError(info: PairingTokenInfo): string | null {
  if (!needsLanHintFallback(info)) {
    return null;
  }
  return "LAN bind is on, but no usable network address was found. Set this Mac’s LAN IPv4 (Companion → enable LAN, or BIOFOCUS_INGEST_BIND_HOST), restart, then Reload.";
}

/** Whether the primary base URL is safe to copy for a physical phone. */
export function isPrimaryUrlCopyable(info: PairingTokenInfo): boolean {
  if (info.bindMode === "loopback") {
    return false;
  }
  return !needsLanHintFallback(info);
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
  if (needsLanHintFallback(info)) {
    return "LAN bind is on, but no usable network address was found. Showing loopback for Simulator. Restart Desktop with BIOFOCUS_INGEST_BIND_HOST set to this Mac’s LAN IPv4, then reload pairing.";
  }
  if (info.bindMode === "lan") {
    return "LAN reachability is opt-in on this Mac. Use this URL on a phone on the same Wi‑Fi — local network only.";
  }
  return "Same-machine / Simulator. For a physical phone, enable LAN bind on Desktop, restart, then reload pairing.";
}

export async function fetchPairingToken(): Promise<PairingView> {
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

export async function copyText(value: string): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(value);
    return true;
  } catch {
    return false;
  }
}
