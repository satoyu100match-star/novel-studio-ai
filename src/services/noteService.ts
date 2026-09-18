import * as api from "@/types/tauriCommands";
import type { Note } from "@/types/tauriCommands";
import { logger } from "@/utils/logger";

export async function listNotes(projectId: string): Promise<Note[]> {
  try {
    return await api.listNotes(projectId);
  } catch (err) {
    logger.error("listNotes failed", { err: String(err) });
    return [];
  }
}
export const createNote = api.createNote;
export const saveNote = api.saveNote;
export const deleteNote = api.deleteNote;
