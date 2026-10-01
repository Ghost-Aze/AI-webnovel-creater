import { invoke } from "@tauri-apps/api/core";

import { CommandError, normalizeCommandError } from "./command-error";
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
    if (error instanceof CommandError) {
      throw error;
    }
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
