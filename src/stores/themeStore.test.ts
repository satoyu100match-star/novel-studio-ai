import { describe, expect, it, beforeEach } from "vitest";
import { nextTheme, THEME_LABELS, THEME_OPTIONS, useThemeStore } from "@/stores/themeStore";

describe("nextTheme", () => {
  it("toggles light to dark and back", () => {
    expect(nextTheme("light")).toBe("dark");
    expect(nextTheme("dark")).toBe("light");
  });

  // Phase13: かわいい/かっこいい/パンクからトグル操作をした場合、未定義の
  // 遷移を作らずlightへ戻す(安全策)。
  it("falls back to light from any non-light theme", () => {
    expect(nextTheme("cute")).toBe("light");
    expect(nextTheme("cool")).toBe("light");
    expect(nextTheme("punk")).toBe("light");
  });
});

describe("THEME_OPTIONS / THEME_LABELS (Phase13)", () => {
  it("declares a Japanese label for every selectable theme", () => {
    for (const theme of THEME_OPTIONS) {
      expect(THEME_LABELS[theme]).toBeTruthy();
    }
  });

  it("includes the three new mood themes alongside light/dark", () => {
    expect(THEME_OPTIONS).toEqual(["light", "dark", "cute", "cool", "punk"]);
  });
});

describe("useThemeStore", () => {
  beforeEach(() => {
    useThemeStore.setState({ theme: "light", hydrated: false });
  });

  it("defaults to light theme", () => {
    expect(useThemeStore.getState().theme).toBe("light");
  });

  it("toggleTheme flips the theme synchronously in state", () => {
    // Outside of a real Tauri runtime, persistence is a no-op (see
    // services/settingsService.ts), but the in-memory state change must
    // still happen immediately so the UI updates without waiting on IPC.
    useThemeStore.getState().toggleTheme();
    expect(useThemeStore.getState().theme).toBe("dark");

    useThemeStore.getState().toggleTheme();
    expect(useThemeStore.getState().theme).toBe("light");
  });

  it("setTheme sets an explicit value", () => {
    useThemeStore.getState().setTheme("dark");
    expect(useThemeStore.getState().theme).toBe("dark");
  });

  it("setTheme accepts the Phase13 mood themes too", () => {
    useThemeStore.getState().setTheme("punk");
    expect(useThemeStore.getState().theme).toBe("punk");
  });
});
