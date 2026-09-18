import * as api from "@/types/tauriCommands";
import type { WorldCategory, WorldEntry, WorldEntryInput } from "@/types/tauriCommands";
import { logger } from "@/utils/logger";

export async function listWorldCategories(projectId: string): Promise<WorldCategory[]> {
  try {
    return await api.listWorldCategories(projectId);
  } catch (err) {
    logger.error("listWorldCategories failed", { err: String(err) });
    return [];
  }
}
export const createWorldCategory = api.createWorldCategory;

export async function listWorldEntries(projectId: string): Promise<WorldEntry[]> {
  try {
    return await api.listWorldEntries(projectId);
  } catch (err) {
    logger.error("listWorldEntries failed", { err: String(err) });
    return [];
  }
}
export const createWorldEntry = api.createWorldEntry;
export const updateWorldEntry = api.updateWorldEntry;
export const deleteWorldEntry = api.deleteWorldEntry;
export const setWorldEntryTags = api.setWorldEntryTags;

export async function getWorldEntryTags(entryId: string): Promise<string[]> {
  try {
    return await api.getWorldEntryTags(entryId);
  } catch (err) {
    logger.error("getWorldEntryTags failed", { err: String(err) });
    return [];
  }
}

export function emptyWorldEntryInput(): WorldEntryInput {
  return { ...api.EMPTY_WORLD_ENTRY_INPUT };
}
