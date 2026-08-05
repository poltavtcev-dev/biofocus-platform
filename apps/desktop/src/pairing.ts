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
