import * as api from "@/types/tauriCommands";
import type { Character, CharacterInput, CharacterRelation } from "@/types/tauriCommands";
import { logger } from "@/utils/logger";

export async function listCharacters(projectId: string): Promise<Character[]> {
  try {
    return await api.listCharacters(projectId);
  } catch (err) {
    logger.error("listCharacters failed", { err: String(err) });
    return [];
  }
}
export const createCharacter = api.createCharacter;
export const updateCharacter = api.updateCharacter;
export const deleteCharacter = api.deleteCharacter;
export const setCharacterTags = api.setCharacterTags;

export async function getCharacterTags(characterId: string): Promise<string[]> {
  try {
    return await api.getCharacterTags(characterId);
  } catch (err) {
    logger.error("getCharacterTags failed", { err: String(err) });
    return [];
  }
}

export async function listCharacterRelations(projectId: string): Promise<CharacterRelation[]> {
  try {
    return await api.listCharacterRelations(projectId);
  } catch (err) {
    logger.error("listCharacterRelations failed", { err: String(err) });
    return [];
  }
}
export const createCharacterRelation = api.createCharacterRelation;
export const deleteCharacterRelation = api.deleteCharacterRelation;

export function emptyCharacterInput(): CharacterInput {
  return { ...api.EMPTY_CHARACTER_INPUT };
}
