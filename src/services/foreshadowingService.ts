import * as api from "@/types/tauriCommands";
import type { Foreshadowing } from "@/types/tauriCommands";
import { logger } from "@/utils/logger";

export async function listForeshadowings(projectId: string): Promise<Foreshadowing[]> {
  try {
    return await api.listForeshadowings(projectId);
  } catch (err) {
    logger.error("listForeshadowings failed", { err: String(err) });
    return [];
  }
}
export const createForeshadowing = api.createForeshadowing;
export const updateForeshadowing = api.updateForeshadowing;
export const deleteForeshadowing = api.deleteForeshadowing;
