import * as api from "@/types/tauriCommands";
import type { AiAnalysisReport, ContextBlock, ReadabilityStats } from "@/types/tauriCommands";
import { logger } from "@/utils/logger";

export async function listAiAnalysisTypes(): Promise<string[]> {
  try {
    return await api.listAiAnalysisTypes();
  } catch (err) {
    logger.error("listAiAnalysisTypes failed", { err: String(err) });
    return [];
  }
}

export async function buildAiAnalysisContext(
  projectId: string,
  analysisType: string,
  characterId: string | null,
): Promise<ContextBlock[]> {
  try {
    return await api.buildAiAnalysisContext(projectId, analysisType, characterId);
  } catch (err) {
    logger.error("buildAiAnalysisContext failed", { err: String(err) });
    return [];
  }
}

export async function computeReadabilityStats(projectId: string): Promise<ReadabilityStats | null> {
  try {
    return await api.computeReadabilityStats(projectId);
  } catch (err) {
    logger.error("computeReadabilityStats failed", { err: String(err) });
    return null;
  }
}

export const runAiAnalysis = api.runAiAnalysis;

export async function listAiAnalysisReports(
  projectId: string,
  analysisType: string | null,
): Promise<AiAnalysisReport[]> {
  try {
    return await api.listAiAnalysisReports(projectId, analysisType);
  } catch (err) {
    logger.error("listAiAnalysisReports failed", { err: String(err) });
    return [];
  }
}

export const deleteAiAnalysisReport = api.deleteAiAnalysisReport;
