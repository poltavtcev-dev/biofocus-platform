import { WebviewWindow } from "@tauri-apps/api/webviewWindow";

/**
 * Whether this webview should render the Dashboard shell.
 * Prefer `?view=dashboard` (Tauri window URL + Vite preview); fall back to label.
 */
export function isDashboardSurface(
  search: string = typeof window !== "undefined" ? window.location.search : "",
): boolean {
  if (new URLSearchParams(search).get("view") === "dashboard") {
    return true;
  }
  try {
    return WebviewWindow.getCurrent().label === "dashboard";
  } catch {
    return false;
  }
}

/**
 * Shows the preconfigured `dashboard` window via IPC (hide-on-close).
 * Safe no-op outside Tauri (Vite preview).
 */
export async function openDashboardWindow(): Promise<boolean> {
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    await invoke("open_dashboard");
    return true;
  } catch {
    // Browser preview: navigate same tab for layout QA.
    if (typeof window !== "undefined" && !("__TAURI_INTERNALS__" in window)) {
      const url = new URL(window.location.href);
      url.searchParams.set("view", "dashboard");
      window.location.assign(url.toString());
      return true;
    }
    return false;
  }
}
