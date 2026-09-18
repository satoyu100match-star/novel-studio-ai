import { describe, expect, it } from "vitest";
import { readSetting, writeSetting, readAllSettings } from "@/services/settingsService";

// These run under jsdom (no Tauri backend attached), which exercises the
// "not running in Tauri" fallback path -- the same path a future web
// preview build would hit. It must never throw.
describe("settingsService outside Tauri runtime", () => {
  it("readSetting resolves to null instead of throwing", async () => {
    await expect(readSetting("ui.theme")).resolves.toBeNull();
  });

  it("writeSetting resolves to false instead of throwing", async () => {
    await expect(writeSetting("ui.theme", "dark")).resolves.toBe(false);
  });

  it("readAllSettings resolves to an empty list", async () => {
    await expect(readAllSettings()).resolves.toEqual([]);
  });
});
