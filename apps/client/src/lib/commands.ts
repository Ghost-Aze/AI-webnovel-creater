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
  CanonRule,
  CreateCanonRuleInput,
  CreateStoryFactInput,
  ProjectMemoryListFilter,
  StoryFact,
  UpdateCanonRuleInput,
  UpdateStoryFactInput,
} from "../types/project-memory";
import type {
  AppendMessageInput,
  Conversation,
  ConversationListFilter,
  ConversationMessage,
  CreateConversationInput,
  DeveloperChatSendRequest,
  DeveloperChatSendResult,
  MemoryToolRequest,
} from "../types/conversation";
import type {
  Chapter,
  ChapterListFilter,
  CreateManuscriptProposalInput,
  CreateChapterInput,
  Manuscript,
  ManuscriptProposal,
  ManuscriptProposalStatus,
  ManuscriptRevision,
  SaveManuscriptInput,
  UpdateChapterInput,
} from "../types/manuscript";
import type {
  OrchestrationRequest,
  OrchestrationResult,
} from "../types/orchestration";
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
import type {
  UpdateUserPreferencesInput,
  UpdateUserProfileInput,
  UserPreferences,
  UserProfile,
} from "../types/user";

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

export function getUserProfile(): Promise<UserProfile> {
  return call<UserProfile>("user_profile_get", {});
}

export function updateUserProfile(
  input: UpdateUserProfileInput,
  expectedRevision: number,
): Promise<UserProfile> {
  return call<UserProfile>("user_profile_update", {
    input,
    expected_revision: expectedRevision,
  });
}

export function getUserPreferences(): Promise<UserPreferences> {
  return call<UserPreferences>("user_preferences_get", {});
}

export function updateUserPreferences(
  input: UpdateUserPreferencesInput,
  expectedRevision: number,
): Promise<UserPreferences> {
  return call<UserPreferences>("user_preferences_update", {
    input,
    expected_revision: expectedRevision,
  });
}

export function createChapter(
  projectId: string,
  input: CreateChapterInput,
): Promise<Chapter> {
  return call<Chapter>("chapter_create", { project_id: projectId, input });
}

export function listChapters(
  projectId: string,
  filter: ChapterListFilter = { include_archived: false },
): Promise<Chapter[]> {
  return call<Chapter[]>("chapter_list", {
    project_id: projectId,
    filter,
  });
}

export function getChapter(id: string): Promise<Chapter> {
  return call<Chapter>("chapter_get", { id });
}

export function updateChapter(
  id: string,
  input: UpdateChapterInput,
  expectedRevision: number,
): Promise<Chapter> {
  return call<Chapter>("chapter_update", {
    id,
    input,
    expected_revision: expectedRevision,
  });
}

export function archiveChapter(
  id: string,
  expectedRevision: number,
): Promise<Chapter> {
  return call<Chapter>("chapter_archive", {
    id,
    expected_revision: expectedRevision,
  });
}

export function getManuscript(chapterId: string): Promise<Manuscript> {
  return call<Manuscript>("manuscript_get", { chapter_id: chapterId });
}

export function saveManuscript(
  chapterId: string,
  input: SaveManuscriptInput,
): Promise<Manuscript> {
  return call<Manuscript>("manuscript_save", {
    chapter_id: chapterId,
    input,
  });
}

export function listManuscriptRevisions(
  chapterId: string,
): Promise<ManuscriptRevision[]> {
  return call<ManuscriptRevision[]>("manuscript_revision_list", {
    chapter_id: chapterId,
  });
}

export function restoreManuscript(
  chapterId: string,
  revision: number,
  expectedRevision: number,
): Promise<Manuscript> {
  return call<Manuscript>("manuscript_restore", {
    chapter_id: chapterId,
    revision,
    expected_revision: expectedRevision,
  });
}

export function createManuscriptProposal(
  projectId: string,
  input: CreateManuscriptProposalInput,
): Promise<ManuscriptProposal> {
  return call<ManuscriptProposal>("manuscript_proposal_create", {
    project_id: projectId,
    input,
  });
}

export function listManuscriptProposals(
  chapterId: string,
  status?: ManuscriptProposalStatus,
): Promise<ManuscriptProposal[]> {
  return call<ManuscriptProposal[]>("manuscript_proposal_list", {
    chapter_id: chapterId,
    status: status ?? null,
  });
}

export function getManuscriptProposal(id: string): Promise<ManuscriptProposal> {
  return call<ManuscriptProposal>("manuscript_proposal_get", { id });
}

export function promoteManuscriptProposal(
  id: string,
  expectedRevision: number,
): Promise<Manuscript> {
  return call<Manuscript>("manuscript_proposal_promote", {
    id,
    expected_revision: expectedRevision,
  });
}

export function rejectManuscriptProposal(
  id: string,
): Promise<ManuscriptProposal> {
  return call<ManuscriptProposal>("manuscript_proposal_reject", { id });
}

export function createConversation(
  input: CreateConversationInput,
): Promise<Conversation> {
  return call<Conversation>("conversation_create", { input });
}

export function listConversations(
  projectId: string,
  filter: ConversationListFilter = { kind: null },
): Promise<Conversation[]> {
  return call<Conversation[]>("conversation_list", {
    project_id: projectId,
    filter,
  });
}

export function getConversation(id: string): Promise<Conversation> {
  return call<Conversation>("conversation_get", { id });
}

export function listConversationMessages(
  conversationId: string,
  limit: number | null = null,
): Promise<ConversationMessage[]> {
  return call<ConversationMessage[]>("conversation_message_list", {
    conversation_id: conversationId,
    limit,
  });
}

export function appendConversationMessage(
  conversationId: string,
  input: AppendMessageInput,
): Promise<ConversationMessage> {
  return call<ConversationMessage>("conversation_message_append", {
    conversation_id: conversationId,
    input,
  });
}

export function chapterChatSend(
  chapterId: string,
  request: DeveloperChatSendRequest,
): Promise<DeveloperChatSendResult> {
  return call<DeveloperChatSendResult>("chapter_chat_send", {
    chapter_id: chapterId,
    request,
  });
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

export function createStoryFact(
  projectId: string,
  input: CreateStoryFactInput,
): Promise<StoryFact> {
  return call<StoryFact>("story_fact_create", { project_id: projectId, input });
}

export function listStoryFacts(
  projectId: string,
  filter: ProjectMemoryListFilter = { include_archived: false },
): Promise<StoryFact[]> {
  return call<StoryFact[]>("story_fact_list", {
    project_id: projectId,
    filter,
  });
}

export function getStoryFact(id: string): Promise<StoryFact> {
  return call<StoryFact>("story_fact_get", { id });
}

export function updateStoryFact(
  id: string,
  input: UpdateStoryFactInput,
  expectedRevision: number,
): Promise<StoryFact> {
  return call<StoryFact>("story_fact_update", {
    id,
    input,
    expected_revision: expectedRevision,
  });
}

export function archiveStoryFact(
  id: string,
  expectedRevision: number,
): Promise<StoryFact> {
  return call<StoryFact>("story_fact_archive", {
    id,
    expected_revision: expectedRevision,
  });
}

export function createCanonRule(
  projectId: string,
  input: CreateCanonRuleInput,
): Promise<CanonRule> {
  return call<CanonRule>("canon_rule_create", { project_id: projectId, input });
}

export function listCanonRules(
  projectId: string,
  filter: ProjectMemoryListFilter = { include_archived: false },
): Promise<CanonRule[]> {
  return call<CanonRule[]>("canon_rule_list", {
    project_id: projectId,
    filter,
  });
}

export function getCanonRule(id: string): Promise<CanonRule> {
  return call<CanonRule>("canon_rule_get", { id });
}

export function updateCanonRule(
  id: string,
  input: UpdateCanonRuleInput,
  expectedRevision: number,
): Promise<CanonRule> {
  return call<CanonRule>("canon_rule_update", {
    id,
    input,
    expected_revision: expectedRevision,
  });
}

export function archiveCanonRule(
  id: string,
  expectedRevision: number,
): Promise<CanonRule> {
  return call<CanonRule>("canon_rule_archive", {
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

export function runOrchestrator(
  request: OrchestrationRequest,
): Promise<OrchestrationResult> {
  return call<OrchestrationResult>("orchestrator_run", { request });
}

export function sendDeveloperChat(
  request: DeveloperChatSendRequest,
): Promise<DeveloperChatSendResult> {
  return call<DeveloperChatSendResult>("developer_chat_send", { request });
}

export function proposeMemoryTool(
  request: MemoryToolRequest,
): Promise<Proposal> {
  return call<Proposal>("memory_tool_propose", { request });
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
