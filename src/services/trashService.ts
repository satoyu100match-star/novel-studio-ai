import * as api from "@/types/tauriCommands";
import type { TrashItem } from "@/types/tauriCommands";
import { logger } from "@/utils/logger";

export async function listTrash(projectId: string): Promise<TrashItem[]> {
  try {
    return await api.listTrash(projectId);
  } catch (err) {
    logger.error("listTrash failed", { err: String(err) });
    return [];
  }
}

export const restoreTrashItem = api.restoreTrashItem;
export const purgeTrashItem = api.purgeTrashItem;
export const purgeAllTrash = api.purgeAllTrash;
