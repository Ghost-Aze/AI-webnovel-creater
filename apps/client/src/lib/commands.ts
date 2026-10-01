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
  CreateProposalInput,
  MemoryEntityType,
  Proposal,
  ProposalStatus,
  Revision,
} from "../types/revision";
import type {
  CreateProjectInput,
  Project,
  ProjectListFilter,
  UpdateProjectInput,
} from "../types/project";
import type { CompiledContext, ContextCompileRequest } from "../types/context";
import type {
  GenerateRequest,
  GenerateResponse,
  CredentialStoreStatus,
  ModelProfile,
  ProviderConfigureInput,
  ProviderConfigureResult,
  ProviderDescriptor,
  RouteDecision,
  RoutingRequest,
} from "../types/provider";

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
  expectedRevision: number,
): Promise<Character> {
  return call<Character>("character_update", {
    id,
    input,
    expected_revision: expectedRevision,
  });
}

export function archiveCharacter(
  id: string,
  expectedRevision: number,
): Promise<Character> {
  return call<Character>("character_archive", {
    id,
    expected_revision: expectedRevision,
  });
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
  expectedRevision: number,
): Promise<CharacterState> {
  return call<CharacterState>("character_state_update", {
    character_id: characterId,
    input,
    expected_revision: expectedRevision,
  });
}

export function listMemoryHistory(
  entityType: MemoryEntityType,
  entityId: string,
): Promise<Revision[]> {
  return call<Revision[]>("memory_history_list", {
    entity_type: entityType,
    entity_id: entityId,
  });
}

export function restoreMemory(
  entityType: MemoryEntityType,
  entityId: string,
  revision: number,
  expectedRevision: number,
): Promise<Revision> {
  return call<Revision>("memory_restore", {
    entity_type: entityType,
    entity_id: entityId,
    revision,
    expected_revision: expectedRevision,
  });
}

export function setMemoryCanonStatus(
  entityType: MemoryEntityType,
  entityId: string,
  status: "canon" | "locked_canon",
  expectedRevision: number,
): Promise<Revision> {
  return call<Revision>("memory_set_canon_status", {
    entity_type: entityType,
    entity_id: entityId,
    status,
    expected_revision: expectedRevision,
  });
}

export function createMemoryProposal(
  input: CreateProposalInput,
): Promise<Proposal> {
  return call<Proposal>("memory_proposal_create", { input });
}

export function listMemoryProposals(
  projectId: string,
  status?: ProposalStatus,
): Promise<Proposal[]> {
  return call<Proposal[]>("memory_proposal_list", {
    project_id: projectId,
    status: status ?? null,
  });
}

export function promoteMemoryProposal(
  id: string,
  expectedRevision: number,
): Promise<Revision> {
  return call<Revision>("memory_proposal_promote", {
    id,
    expected_revision: expectedRevision,
  });
}

export function rejectMemoryProposal(id: string): Promise<Proposal> {
  return call<Proposal>("memory_proposal_reject", { id });
}

export function listProviders(): Promise<ProviderDescriptor[]> {
  return call<ProviderDescriptor[]>("provider_list", {});
}

export function listModels(providerId?: string): Promise<ModelProfile[]> {
  return call<ModelProfile[]>("model_list", {
    provider_id: providerId ?? null,
  });
}

export function routeModel(request: RoutingRequest): Promise<RouteDecision> {
  return call<RouteDecision>("model_route", { request });
}

export function generateProvider(
  request: GenerateRequest,
): Promise<GenerateResponse> {
  return call<GenerateResponse>("provider_generate", { request });
}

export function configureProvider(
  input: ProviderConfigureInput,
): Promise<ProviderConfigureResult> {
  return call<ProviderConfigureResult>("provider_configure", { input });
}

export function removeProvider(
  providerId: string,
): Promise<ProviderDescriptor> {
  return call<ProviderDescriptor>("provider_remove", {
    provider_id: providerId,
  });
}

export function getCredentialStoreStatus(): Promise<CredentialStoreStatus> {
  return call<CredentialStoreStatus>("provider_credential_status", {});
}

export function compileContext(
  input: ContextCompileRequest,
): Promise<CompiledContext> {
  return call<CompiledContext>("context_compile", { request: input });
}
