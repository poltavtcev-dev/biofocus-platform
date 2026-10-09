import { invoke } from "@tauri-apps/api/core";

/** IPC payload for ingest LAN preference (P28-E1-T1 / ADR-029). */
export type IngestLanPreference = {
  persisted: boolean;
  fromEnv: boolean;
  effectiveLan: boolean;
  needsRestart: boolean;
};

type WireLanPreference = {
  persisted?: boolean;
  fromEnv?: boolean;
  from_env?: boolean;
  effectiveLan?: boolean;
  effective_lan?: boolean;
  needsRestart?: boolean;
  needs_restart?: boolean;
};

function normalizeLanPreference(payload: WireLanPreference): IngestLanPreference | null {
  if (typeof payload.persisted !== "boolean") {
    return null;
  }
  return {
    persisted: payload.persisted,
    fromEnv: Boolean(payload.fromEnv ?? payload.from_env),
    effectiveLan: Boolean(payload.effectiveLan ?? payload.effective_lan),
    needsRestart: Boolean(payload.needsRestart ?? payload.needs_restart),
  };
}

export async function fetchIngestLanPreference(): Promise<IngestLanPreference | null> {
  try {
    const payload = await invoke<WireLanPreference>("get_ingest_lan_preference");
    return normalizeLanPreference(payload ?? {});
  } catch {
    return null;
  }
}

export async function setIngestLanPreference(
  enabled: boolean,
): Promise<{ ok: true; pref: IngestLanPreference } | { ok: false; detail: string }> {
  try {
    const payload = await invoke<WireLanPreference>("set_ingest_lan_preference", {
      enabled,
    });
    const pref = normalizeLanPreference(payload ?? {});
    if (!pref) {
      return { ok: false, detail: "Не удалось изменить настройку LAN." };
    }
    return { ok: true, pref };
  } catch (err) {
    const text = err instanceof Error ? err.message : String(err);
    return { ok: false, detail: text.trim() || "Не удалось изменить настройку LAN." };
  }
}
