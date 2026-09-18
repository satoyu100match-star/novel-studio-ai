import * as api from "@/types/tauriCommands";
import type { TimelineEvent } from "@/types/tauriCommands";
import { logger } from "@/utils/logger";

export async function listTimelineEvents(projectId: string): Promise<TimelineEvent[]> {
  try {
    return await api.listTimelineEvents(projectId);
  } catch (err) {
    logger.error("listTimelineEvents failed", { err: String(err) });
    return [];
  }
}
export const createTimelineEvent = api.createTimelineEvent;
export const updateTimelineEvent = api.updateTimelineEvent;
export const deleteTimelineEvent = api.deleteTimelineEvent;
