import * as api from "@/types/tauriCommands";
import type { ManuscriptDocument, Revision } from "@/types/tauriCommands";
import { logger } from "@/utils/logger";

export async function listRevisions(ownerType: "chapter" | "scene", ownerId: string): Promise<Revision[]> {
  try {
    return await api.listRevisions(ownerType, ownerId);
  } catch (err) {
    logger.error("listRevisions failed", { err: String(err) });
    return [];
  }
}

export const createManualRevision = api.createManualRevision;

export async function restoreRevision(id: string): Promise<ManuscriptDocument> {
  return api.restoreRevision(id);
}
