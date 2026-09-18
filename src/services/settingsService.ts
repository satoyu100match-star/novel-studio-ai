import { getSetting, listSettings, setSetting } from "@/types/tauriCommands";
import { isTauriRuntime } from "@/services/runtime";
import { logger } from "@/utils/logger";

/**
 * Application Services layer for `app_settings` (see docs/ARCHITECTURE.md).
 * Features call these functions, never the raw Tauri command wrappers.
 */
export async function readSetting(key: string): Promise<string | null> {
  if (!isTauriRuntime()) return null;
  try {
    return await getSetting(key);
  } catch (err) {
    logger.error("failed to read setting", { key, err: String(err) });
    return null;
  }
}

export async function writeSetting(key: string, value: string): Promise<boolean> {
  if (!isTauriRuntime()) return false;
  try {
    await setSetting(key, value);
    return true;
  } catch (err) {
    logger.error("failed to write setting", { key, err: String(err) });
    return false;
  }
}

export async function readAllSettings(): Promise<[string, string][]> {
  if (!isTauriRuntime()) return [];
  try {
    return await listSettings();
  } catch (err) {
    logger.error("failed to list settings", { err: String(err) });
    return [];
  }
}
