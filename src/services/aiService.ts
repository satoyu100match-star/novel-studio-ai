import * as api from "@/types/tauriCommands";
import type { AiChatMessage, AiSendResponseDto, AiUsageSummary, ContextBlock } from "@/types/tauriCommands";
import { logger } from "@/utils/logger";

export async function listAiProviderKinds(): Promise<string[]> {
  try {
    return await api.listAiProviderKinds();
  } catch (err) {
    logger.error("listAiProviderKinds failed", { err: String(err) });
    return [];
  }
}
export const setAiApiKey = api.setAiApiKey;
export const hasAiApiKey = api.hasAiApiKey;
export const clearAiApiKey = api.clearAiApiKey;

export async function buildAiContext(
  projectId: string,
  chapterId: string | null,
  sceneId: string | null,
  userQuestion: string | null,
): Promise<ContextBlock[]> {
  try {
    return await api.buildAiContext(projectId, chapterId, sceneId, userQuestion);
  } catch (err) {
    logger.error("buildAiContext failed", { err: String(err) });
    return [];
  }
}

export async function listAiChatMessages(projectId: string): Promise<AiChatMessage[]> {
  try {
    return await api.listAiChatMessages(projectId);
  } catch (err) {
    logger.error("listAiChatMessages failed", { err: String(err) });
    return [];
  }
}
export const clearAiChat = api.clearAiChat;
export const sendAiChatMessage = api.sendAiChatMessage;

export async function getAiUsageSummary(projectId: string | null): Promise<AiUsageSummary | null> {
  try {
    return await api.getAiUsageSummary(projectId);
  } catch (err) {
    logger.error("getAiUsageSummary failed", { err: String(err) });
    return null;
  }
}

export async function listAiWritingFeatures(): Promise<string[]> {
  try {
    return await api.listAiWritingFeatures();
  } catch (err) {
    logger.error("listAiWritingFeatures failed", { err: String(err) });
    return [];
  }
}

export async function runAiWritingFeature(
  projectId: string,
  feature: string,
  providerKind: string,
  model: string,
  baseUrl: string | null,
  temperature: number | null,
  maxOutputTokens: number | null,
  selectedContext: ContextBlock[],
  inputText: string | null,
): Promise<AiSendResponseDto> {
  return api.runAiWritingFeature(
    projectId,
    feature,
    providerKind,
    model,
    baseUrl,
    temperature,
    maxOutputTokens,
    selectedContext,
    inputText,
  );
}
