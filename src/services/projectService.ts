import * as api from "@/types/tauriCommands";
import type { Project, ProjectInput } from "@/types/tauriCommands";
import { logger } from "@/utils/logger";

export async function listProjects(): Promise<Project[]> {
  try {
    return await api.listProjects();
  } catch (err) {
    logger.error("listProjects failed", { err: String(err) });
    return [];
  }
}

export async function getProject(id: string): Promise<Project | null> {
  try {
    return await api.getProject(id);
  } catch (err) {
    logger.error("getProject failed", { id, err: String(err) });
    return null;
  }
}

export async function createProject(input: ProjectInput): Promise<Project> {
  return api.createProject(input);
}

export async function createSampleProject(): Promise<Project> {
  return api.createSampleProject();
}

export async function updateProject(id: string, input: ProjectInput): Promise<Project> {
  return api.updateProject(id, input);
}

export async function deleteProject(id: string): Promise<void> {
  await api.deleteProject(id);
}
