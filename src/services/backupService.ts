import * as api from "@/types/tauriCommands";
import type { BackupInfo } from "@/types/tauriCommands";
import { logger } from "@/utils/logger";

export async function wasUncleanShutdown(): Promise<boolean> {
  try {
    return await api.wasUncleanShutdown();
  } catch (err) {
    logger.error("wasUncleanShutdown failed", { err: String(err) });
    return false;
  }
}

export async function listBackups(): Promise<BackupInfo[]> {
  try {
    return await api.listBackups();
  } catch (err) {
    logger.error("listBackups failed", { err: String(err) });
    return [];
  }
}

export const backupNow = api.backupNow;
export const restoreBackup = api.restoreBackup;
