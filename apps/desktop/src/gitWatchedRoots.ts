import { invoke } from "@tauri-apps/api/core";

/** Mirrors desktop `get_git_watched_roots` / `set_git_watched_roots` IPC. */
export type GitWatchedRootsInfo = {
  roots: string[];
  source: string;
  version: number | null;
};

type GitWatchedRootsPayload = {
  roots?: string[];
  source?: string;
  version?: number | null;
};

export type GitWatchedRootsView =
  | { kind: "idle" }
  | { kind: "loading" }
  | { kind: "ready"; info: GitWatchedRootsInfo }
  | { kind: "error"; detail: string };

export type GitWatchedRootsSaveView =
  | { kind: "idle" }
  | { kind: "saving" }
  | { kind: "ok"; info: GitWatchedRootsInfo; message: string }
  | { kind: "error"; detail: string };

function normalize(payload: GitWatchedRootsPayload): GitWatchedRootsInfo | null {
  if (!payload || !Array.isArray(payload.roots)) {
    return null;
  }
  const roots = payload.roots
    .map((r) => (typeof r === "string" ? r.trim() : ""))
    .filter((r) => r.length > 0);
  const source =
    typeof payload.source === "string" && payload.source.trim()
      ? payload.source.trim()
      : roots.length > 0
        ? "file"
        : "empty";
  const version =
    typeof payload.version === "number" && Number.isFinite(payload.version)
      ? payload.version
      : null;
  return { roots, source, version };
}

function mockFromQuery(): GitWatchedRootsInfo | null {
  if (typeof window === "undefined") {
    return null;
  }
  const raw = new URLSearchParams(window.location.search).get("mockGitRoots");
  if (!raw) {
    return null;
  }
  if (raw === "empty") {
    return { roots: [], source: "empty", version: 1 };
  }
  if (raw === "ready") {
    return {
      roots: [
        "/Users/you/Developer/AI Project/BioFocus",
        "/Users/you/Developer/personal-notes",
      ],
      source: "file",
      version: 1,
    };
  }
  if (raw === "error") {
    return null;
  }
  return null;
}

function mockErrorFromQuery(): boolean {
  if (typeof window === "undefined") {
    return false;
  }
  return new URLSearchParams(window.location.search).get("mockGitRoots") === "error";
}

/** Load watched folders via IPC (or QA mock). No busy-loop. */
export async function fetchGitWatchedRoots(): Promise<GitWatchedRootsView> {
  if (mockErrorFromQuery()) {
    return { kind: "error", detail: "Could not load watched folders." };
  }
  const mocked = mockFromQuery();
  if (mocked) {
    return { kind: "ready", info: mocked };
  }

  try {
    const payload = await invoke<GitWatchedRootsPayload>("get_git_watched_roots");
    const info = normalize(payload);
    if (!info) {
      return { kind: "error", detail: "Could not load watched folders." };
    }
    return { kind: "ready", info };
  } catch (err) {
    const detail =
      typeof err === "string" && err.trim()
        ? err.trim()
        : "Could not load watched folders.";
    return { kind: "error", detail };
  }
}

/** Save watched folders via IPC. Mock mode updates in-memory only for the call. */
export async function saveGitWatchedRoots(
  roots: string[],
): Promise<GitWatchedRootsSaveView> {
  if (mockErrorFromQuery()) {
    return { kind: "error", detail: "Could not save watched folders." };
  }
  const mocked = mockFromQuery();
  if (mocked) {
    const info: GitWatchedRootsInfo = {
      roots: roots.map((r) => r.trim()).filter(Boolean),
      source: roots.some((r) => r.trim()) ? "file" : "empty",
      version: 1,
    };
    return {
      kind: "ok",
      info,
      message:
        info.roots.length === 0
          ? "Watched folders cleared — Git activity stays idle."
          : "Watched folders saved.",
    };
  }

  try {
    const payload = await invoke<GitWatchedRootsPayload>("set_git_watched_roots", {
      roots,
    });
    const info = normalize(payload);
    if (!info) {
      return { kind: "error", detail: "Could not save watched folders." };
    }
    return {
      kind: "ok",
      info,
      message:
        info.roots.length === 0
          ? "Watched folders cleared — Git activity stays idle."
          : "Watched folders saved.",
    };
  } catch (err) {
    const detail =
      typeof err === "string" && err.trim()
        ? err.trim()
        : "Could not save watched folders.";
    return { kind: "error", detail };
  }
}

/** Menubar label: home as `~`, shortened only on a `/` so names stay whole. */
export function shortenRootPath(path: string, max = 42): string {
  let shown = path.trim().replace(/\/+$/, "");
  if (!shown) {
    return path.trim();
  }
  shown = shown.replace(/^\/Users\/[^/]+/, "~").replace(/^\/home\/[^/]+/, "~");
  if (shown.length <= max) {
    return shown;
  }
  const tail = shown.slice(-(max - 1));
  const slash = tail.indexOf("/");
  const cut = slash >= 0 ? tail.slice(slash) : `/${tail}`;
  return `…${cut}`;
}
