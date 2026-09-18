import * as api from "@/types/tauriCommands";
import type { Comment } from "@/types/tauriCommands";
import { logger } from "@/utils/logger";

export async function listComments(ownerType: "chapter" | "scene", ownerId: string): Promise<Comment[]> {
  try {
    return await api.listComments(ownerType, ownerId);
  } catch (err) {
    logger.error("listComments failed", { err: String(err) });
    return [];
  }
}
export const createComment = api.createComment;
export const setCommentResolved = api.setCommentResolved;
export const deleteComment = api.deleteComment;
