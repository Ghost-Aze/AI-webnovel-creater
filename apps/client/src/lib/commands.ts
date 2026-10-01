import { invoke } from "@tauri-apps/api/core";

import { normalizeCommandError } from "./command-error";
import type {
  Character,
  CharacterListFilter,
  CharacterState,
  CreateCharacterInput,
  UpdateCharacterInput,
  UpdateCharacterStateInput,
} from "../types/character";
import type {
  CreateProjectInput,
  Project,
  ProjectListFilter,
  UpdateProjectInput,
} from "../types/project";

async function call<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    throw normalizeCommandError(error);
  }
}

export function createProject(input: CreateProjectInput): Promise<Project> {
  return call<Project>("project_create", { input });
}

export function listProjects(
  filter: ProjectListFilter = { include_archived: false },
): Promise<Project[]> {
  return call<Project[]>("project_list", { filter });
}

export function getProject(id: string): Promise<Project> {
  return call<Project>("project_get", { id });
}

export function updateProject(
  id: string,
  input: UpdateProjectInput,
): Promise<Project> {
  return call<Project>("project_update", { id, input });
}

export function archiveProject(id: string): Promise<Project> {
  return call<Project>("project_archive", { id });
}

export function createCharacter(
  projectId: string,
  input: CreateCharacterInput,
): Promise<Character> {
  return call<Character>("character_create", { project_id: projectId, input });
}

export function listCharacters(
  projectId: string,
  filter: CharacterListFilter = { include_archived: false },
): Promise<Character[]> {
  return call<Character[]>("character_list", { project_id: projectId, filter });
}

export function getCharacter(id: string): Promise<Character> {
  return call<Character>("character_get", { id });
}

export function updateCharacter(
  id: string,
  input: UpdateCharacterInput,
): Promise<Character> {
  return call<Character>("character_update", { id, input });
}

export function archiveCharacter(id: string): Promise<Character> {
  return call<Character>("character_archive", { id });
}

export function getCharacterState(
  characterId: string,
): Promise<CharacterState> {
  return call<CharacterState>("character_state_get", {
    character_id: characterId,
  });
}

export function updateCharacterState(
  characterId: string,
  input: UpdateCharacterStateInput,
): Promise<CharacterState> {
  return call<CharacterState>("character_state_update", {
    character_id: characterId,
    input,
  });
}
