import * as api from "@/types/tauriCommands";
import type { PlotCard, PlotLane } from "@/types/tauriCommands";
import { logger } from "@/utils/logger";

export async function listPlotLanes(projectId: string): Promise<PlotLane[]> {
  try {
    return await api.listPlotLanes(projectId);
  } catch (err) {
    logger.error("listPlotLanes failed", { err: String(err) });
    return [];
  }
}
export const createPlotLane = api.createPlotLane;
export const renamePlotLane = api.renamePlotLane;
export const deletePlotLane = api.deletePlotLane;

export async function listPlotCards(projectId: string): Promise<PlotCard[]> {
  try {
    return await api.listPlotCards(projectId);
  } catch (err) {
    logger.error("listPlotCards failed", { err: String(err) });
    return [];
  }
}
export const createPlotCard = api.createPlotCard;
export const updatePlotCard = api.updatePlotCard;
export const movePlotCard = api.movePlotCard;
export const deletePlotCard = api.deletePlotCard;
