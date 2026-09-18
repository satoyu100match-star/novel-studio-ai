import { create } from "zustand";
import type { Project, ProjectInput } from "@/types/tauriCommands";
import * as projectService from "@/services/projectService";

interface ProjectListState {
  projects: Project[];
  loading: boolean;
  load: () => Promise<void>;
  create: (input: ProjectInput) => Promise<Project>;
  createSample: () => Promise<Project>;
  remove: (id: string) => Promise<void>;
}

export const useProjectListStore = create<ProjectListState>((set, get) => ({
  projects: [],
  loading: false,

  load: async () => {
    set({ loading: true });
    const projects = await projectService.listProjects();
    set({ projects, loading: false });
  },

  create: async (input: ProjectInput) => {
    const project = await projectService.createProject(input);
    set({ projects: [project, ...get().projects] });
    return project;
  },

  createSample: async () => {
    const project = await projectService.createSampleProject();
    set({ projects: [project, ...get().projects] });
    return project;
  },

  remove: async (id: string) => {
    await projectService.deleteProject(id);
    set({ projects: get().projects.filter((p) => p.id !== id) });
  },
}));
