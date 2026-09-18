/**
 * Application Services for the Navigator tree (Part > Chapter > Scene)
 * and the manuscript body underneath a chapter/scene. Features call
 * these, never `src/types/tauriCommands.ts` directly (see
 * docs/ARCHITECTURE.md #2).
 */
import * as api from "@/types/tauriCommands";
import type {
  Chapter,
  ChapterStatus,
  ManuscriptDocument,
  Part,
  Scene,
  SearchHit,
} from "@/types/tauriCommands";
import { logger } from "@/utils/logger";

export async function listParts(projectId: string): Promise<Part[]> {
  try {
    return await api.listParts(projectId);
  } catch (err) {
    logger.error("listParts failed", { err: String(err) });
    return [];
  }
}
export const createPart = api.createPart;
export const renamePart = api.renamePart;
export const deletePart = api.deletePart;

export async function listChapters(projectId: string): Promise<Chapter[]> {
  try {
    return await api.listChapters(projectId);
  } catch (err) {
    logger.error("listChapters failed", { err: String(err) });
    return [];
  }
}
export const createChapter = api.createChapter;
export const getChapter = api.getChapter;
export function updateChapterMeta(id: string, title: string, status: ChapterStatus): Promise<void> {
  return api.updateChapterMeta(id, title, status);
}
export const deleteChapter = api.deleteChapter;

export async function listScenes(chapterId: string): Promise<Scene[]> {
  try {
    return await api.listScenes(chapterId);
  } catch (err) {
    logger.error("listScenes failed", { err: String(err) });
    return [];
  }
}
export const createScene = api.createScene;
export const getScene = api.getScene;
export const renameScene = api.renameScene;
export const deleteScene = api.deleteScene;

export async function getDocument(
  ownerType: "chapter" | "scene",
  ownerId: string,
): Promise<ManuscriptDocument | null> {
  try {
    return await api.getDocument(ownerType, ownerId);
  } catch (err) {
    logger.error("getDocument failed", { err: String(err) });
    return null;
  }
}

export async function saveDocumentBody(
  ownerType: "chapter" | "scene",
  ownerId: string,
  body: string,
): Promise<ManuscriptDocument | null> {
  try {
    return await api.saveDocumentBody(ownerType, ownerId, body);
  } catch (err) {
    logger.error("saveDocumentBody failed", { err: String(err) });
    return null;
  }
}

export async function projectCharCount(projectId: string): Promise<number> {
  try {
    return await api.projectCharCount(projectId);
  } catch (err) {
    logger.error("projectCharCount failed", { err: String(err) });
    return 0;
  }
}

export async function searchProject(projectId: string, query: string): Promise<SearchHit[]> {
  try {
    return await api.searchProject(projectId, query);
  } catch (err) {
    logger.error("searchProject failed", { err: String(err) });
    return [];
  }
}

/** 400字詰め換算枚数。実レイアウト枚数(Phase 2)とは区別する -- docs/MANUSCRIPT.md 4章。 */
export function toManuscriptPages(charCount: number): number {
  return charCount / 400;
}
