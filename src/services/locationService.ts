import * as api from "@/types/tauriCommands";
import type { Location, LocationInput } from "@/types/tauriCommands";
import { logger } from "@/utils/logger";

export async function listLocations(projectId: string): Promise<Location[]> {
  try {
    return await api.listLocations(projectId);
  } catch (err) {
    logger.error("listLocations failed", { err: String(err) });
    return [];
  }
}
export const createLocation = api.createLocation;
export const updateLocation = api.updateLocation;
export const deleteLocation = api.deleteLocation;

export function emptyLocationInput(): LocationInput {
  return { ...api.EMPTY_LOCATION_INPUT };
}
