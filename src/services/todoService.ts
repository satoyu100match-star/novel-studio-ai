import * as api from "@/types/tauriCommands";
import type { Todo } from "@/types/tauriCommands";
import { logger } from "@/utils/logger";

export async function listTodos(projectId: string): Promise<Todo[]> {
  try {
    return await api.listTodos(projectId);
  } catch (err) {
    logger.error("listTodos failed", { err: String(err) });
    return [];
  }
}
export const createTodo = api.createTodo;
export const updateTodo = api.updateTodo;
export const setTodoDone = api.setTodoDone;
export const deleteTodo = api.deleteTodo;
