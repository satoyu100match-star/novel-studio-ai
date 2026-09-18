/**
 * Detects whether the app is running inside the Tauri webview (real
 * desktop app) versus a plain browser (e.g. `vitest` + jsdom, or a future
 * `pnpm dev` preview opened directly in a browser tab). Features that
 * touch the backend should degrade gracefully rather than throw when this
 * is false, so the UI never crashes just because it's under test.
 */
export function isTauriRuntime(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}
