import * as api from "@/types/tauriCommands";
import type { GlossaryEntry } from "@/types/tauriCommands";
import { logger } from "@/utils/logger";

export async function listGlossaryEntries(projectId: string): Promise<GlossaryEntry[]> {
  try {
    return await api.listGlossaryEntries(projectId);
  } catch (err) {
    logger.error("listGlossaryEntries failed", { err: String(err) });
    return [];
  }
}
export const createGlossaryEntry = api.createGlossaryEntry;
export const updateGlossaryEntry = api.updateGlossaryEntry;
export const deleteGlossaryEntry = api.deleteGlossaryEntry;
