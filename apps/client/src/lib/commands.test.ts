import { describe, expect, it, vi } from "vitest";

import { CommandError } from "./command-error";

const invokeMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
}));

import {
  appendConversationMessage,
  archiveCanonRule,
  archiveStoryFact,
  chapterChatSend,
  loadChatRuntime,
  createCanonRule,
  createCharacter,
  createConversation,
  createChapter,
  createManuscriptProposal,
  createProject,
  deleteProject,
  createStoryFact,
  compileContext,
  configureProvider,
  generateProvider,
  getCredentialStoreStatus,
  getCharacterState,
  getConversation,
  getCanonRule,
  getManuscript,
  getProject,
  getProviderSettings,
  getStoryFact,
  getUserPreferences,
  getUserProfile,
  listConversationMessages,
  listConversations,
  listChapters,
  listManuscriptProposals,
  listManuscriptRevisions,
  listModels,
  listMemoryHistory,
  listCanonRules,
  listProviders,
  listStoryFacts,
  removeProvider,
  proposeMemoryTool,
  promoteManuscriptProposal,
  rejectManuscriptProposal,
  routeModel,
  restoreManuscript,
  saveManuscript,
  runOrchestrator,
  sendDeveloperChat,
  sendChat,
  restoreMemory,
  setMemoryCanonStatus,
  updateCanonRule,
  updateCharacter,
  updateCharacterState,
  updateStoryFact,
  updateUserPreferences,
  updateUserProfile,
  updateProvider,
  testProvider,
  updateChatRuntime,
} from "./commands";
import { normalizeCommandError } from "./command-error";
import type { ContextCompileRequest } from "../types/context";
import type {
  ChatRuntimeLoadRequest,
  ChatRuntimeSettingsInput,
  ChatSendRequest,
  CreateConversationInput,
  DeveloperChatSendRequest,
  MemoryToolRequest,
} from "../types/conversation";
import type { OrchestrationRequest } from "../types/orchestration";
import type {
  GenerateRequest,
  ProviderConfigureInput,
  ProviderUpdateInput,
  RoutingRequest,
} from "../types/provider";
import type {
  UpdateUserPreferencesInput,
  UpdateUserProfileInput,
} from "../types/user";

describe("typed project commands", () => {
  it("returns the exact project shape from the command boundary", async () => {
    const project = {
      id: "project-id",
      name: "A project",
      description: "",
      status: "active",
      created_at: "2026-10-01T00:00:00Z",
      updated_at: "2026-10-01T00:00:00Z",
    };
    invokeMock.mockResolvedValueOnce(project);

    await expect(createProject({ name: "A project" })).resolves.toEqual(
      project,
    );
    expect(invokeMock).toHaveBeenCalledWith("project_create", {
      input: { name: "A project" },
    });
  });

  it("deletes a project through the typed command boundary", async () => {
    invokeMock.mockResolvedValueOnce(undefined);

    await expect(deleteProject("project-id")).resolves.toBeUndefined();
    expect(invokeMock).toHaveBeenCalledWith("project_delete", {
      id: "project-id",
    });
  });

  it("uses typed local user profile and preference payloads", async () => {
    const profile = {
      id: "local_user",
      display_name: "Writer",
      preferred_language: "en",
      created_at: "2026-10-02T00:00:00Z",
      updated_at: "2026-10-02T00:00:00Z",
      revision: 1,
    };
    invokeMock.mockResolvedValueOnce(profile);
    await expect(getUserProfile()).resolves.toEqual(profile);
    expect(invokeMock).toHaveBeenCalledWith("user_profile_get", {});

    const profileInput: UpdateUserProfileInput = {
      display_name: "Mira",
      preferred_language: "tr",
    };
    invokeMock.mockResolvedValueOnce({ ...profile, ...profileInput, revision: 2 });
    await updateUserProfile(profileInput, 1);
    expect(invokeMock).toHaveBeenCalledWith("user_profile_update", {
      input: profileInput,
      expected_revision: 1,
    });

    const preferences = {
      id: "local_user",
      preferred_narrator: "third_person",
      preferred_pov: "limited",
      chapter_length: 2000,
      scene_length: 600,
      dialogue_density: 40,
      prose_level: "standard",
      pacing: "balanced",
      avoid_repetition: true,
      created_at: "2026-10-02T00:00:00Z",
      updated_at: "2026-10-02T00:00:00Z",
      revision: 1,
    };
    invokeMock.mockResolvedValueOnce(preferences);
    await expect(getUserPreferences()).resolves.toEqual(preferences);
    expect(invokeMock).toHaveBeenCalledWith("user_preferences_get", {});

    const preferencesInput: UpdateUserPreferencesInput = {
      preferred_narrator: "first_person",
      preferred_pov: "close",
      chapter_length: 1800,
      scene_length: 500,
      dialogue_density: 60,
      prose_level: "lyrical",
      pacing: "brisk",
      avoid_repetition: false,
    };
    invokeMock.mockResolvedValueOnce({
      ...preferences,
      ...preferencesInput,
      revision: 2,
    });
    await updateUserPreferences(preferencesInput, 1);
    expect(invokeMock).toHaveBeenCalledWith("user_preferences_update", {
      input: preferencesInput,
      expected_revision: 1,
    });
  });

  it("maps backend errors to safe user messages", async () => {
    invokeMock.mockRejectedValueOnce({
      code: "Storage",
      details: { raw: "near SELECT: secret" },
    });

    const error = await getProject("missing").catch((value: unknown) => value);
    expect(error).toBeInstanceOf(CommandError);
    expect(error).toMatchObject({
      code: "storage",
      message: "The project could not be saved. Try again.",
    });
    expect((error as Error).message).not.toContain("secret");
  });

  it("uses the Rust command argument names for character calls", async () => {
    invokeMock.mockResolvedValueOnce({ id: "character-id" });
    await createCharacter("project-id", { name: "Mara Voss" });
    expect(invokeMock).toHaveBeenCalledWith("character_create", {
      project_id: "project-id",
      input: { name: "Mara Voss" },
    });

    invokeMock.mockResolvedValueOnce({ character_id: "character-id" });
    await getCharacterState("character-id");
    expect(invokeMock).toHaveBeenCalledWith("character_state_get", {
      character_id: "character-id",
    });
  });

  it("uses typed project memory commands and expected revisions", async () => {
    const factInput = { title: "Dawn", content: "The bells ring." };
    const fact = {
      id: "fact-id",
      project_id: "project-id",
      ...factInput,
      status: "active" as const,
      canon_status: "canon" as const,
      revision: 1,
      created_at: "2026-10-03T00:00:00Z",
      updated_at: "2026-10-03T00:00:00Z",
    };
    invokeMock.mockResolvedValueOnce(fact);
    await expect(createStoryFact("project-id", factInput)).resolves.toEqual(fact);
    expect(invokeMock).toHaveBeenCalledWith("story_fact_create", {
      project_id: "project-id",
      input: factInput,
    });

    invokeMock.mockResolvedValueOnce([fact]);
    await listStoryFacts("project-id", { include_archived: true });
    expect(invokeMock).toHaveBeenCalledWith("story_fact_list", {
      project_id: "project-id",
      filter: { include_archived: true },
    });

    invokeMock.mockResolvedValueOnce(fact);
    await getStoryFact("fact-id");
    expect(invokeMock).toHaveBeenCalledWith("story_fact_get", { id: "fact-id" });

    invokeMock.mockResolvedValueOnce({ ...fact, revision: 2 });
    await updateStoryFact("fact-id", factInput, 1);
    expect(invokeMock).toHaveBeenCalledWith("story_fact_update", {
      id: "fact-id",
      input: factInput,
      expected_revision: 1,
    });

    invokeMock.mockResolvedValueOnce({ ...fact, status: "archived", revision: 3 });
    await archiveStoryFact("fact-id", 2);
    expect(invokeMock).toHaveBeenCalledWith("story_fact_archive", {
      id: "fact-id",
      expected_revision: 2,
    });

    const ruleInput = { title: "Time", rule: "Days pass.", scope: "World" };
    invokeMock.mockResolvedValueOnce({ id: "rule-id" });
    await createCanonRule("project-id", ruleInput);
    expect(invokeMock).toHaveBeenCalledWith("canon_rule_create", {
      project_id: "project-id",
      input: ruleInput,
    });
    invokeMock.mockResolvedValueOnce([]);
    await listCanonRules("project-id");
    expect(invokeMock).toHaveBeenCalledWith("canon_rule_list", {
      project_id: "project-id",
      filter: { include_archived: false },
    });
    invokeMock.mockResolvedValueOnce({ id: "rule-id" });
    await getCanonRule("rule-id");
    expect(invokeMock).toHaveBeenCalledWith("canon_rule_get", { id: "rule-id" });
    invokeMock.mockResolvedValueOnce({ id: "rule-id", revision: 2 });
    await updateCanonRule("rule-id", ruleInput, 1);
    expect(invokeMock).toHaveBeenCalledWith("canon_rule_update", {
      id: "rule-id",
      input: ruleInput,
      expected_revision: 1,
    });
    invokeMock.mockResolvedValueOnce({ id: "rule-id", revision: 3 });
    await archiveCanonRule("rule-id", 2);
    expect(invokeMock).toHaveBeenCalledWith("canon_rule_archive", {
      id: "rule-id",
      expected_revision: 2,
    });
  });

  it("uses snake_case arguments for revision commands", async () => {
    invokeMock.mockResolvedValueOnce([]);
    await listMemoryHistory("character", "character-id");
    expect(invokeMock).toHaveBeenCalledWith("memory_history_list", {
      entity_type: "character",
      entity_id: "character-id",
    });

    invokeMock.mockResolvedValueOnce({ revision: 3 });
    await restoreMemory("character", "character-id", 1, 2);
    expect(invokeMock).toHaveBeenCalledWith("memory_restore", {
      entity_type: "character",
      entity_id: "character-id",
      revision: 1,
      expected_revision: 2,
    });

    invokeMock.mockResolvedValueOnce({ revision: 3 });
    await setMemoryCanonStatus("character", "character-id", "locked_canon", 2);
    expect(invokeMock).toHaveBeenCalledWith("memory_set_canon_status", {
      entity_type: "character",
      entity_id: "character-id",
      status: "locked_canon",
      expected_revision: 2,
    });
  });

  it("uses typed Developer Chat and memory-tool command payloads", async () => {
    const input: CreateConversationInput = {
      project_id: "project-id",
      chapter_id: null,
      kind: "developer_chat",
      title: "Developer Chat",
    };
    invokeMock.mockResolvedValueOnce({ id: "conversation-id" });
    await createConversation(input);
    expect(invokeMock).toHaveBeenCalledWith("conversation_create", { input });

    invokeMock.mockResolvedValueOnce([]);
    await listConversations("project-id");
    expect(invokeMock).toHaveBeenCalledWith("conversation_list", {
      project_id: "project-id",
      filter: { kind: null },
    });

    invokeMock.mockResolvedValueOnce({ id: "conversation-id" });
    await getConversation("conversation-id");
    expect(invokeMock).toHaveBeenCalledWith("conversation_get", {
      id: "conversation-id",
    });

    invokeMock.mockResolvedValueOnce([]);
    await listConversationMessages("conversation-id", 20);
    expect(invokeMock).toHaveBeenCalledWith("conversation_message_list", {
      conversation_id: "conversation-id",
      limit: 20,
    });

    const appendInput = {
      role: "user" as const,
      content: "Update the character.",
      model: null,
    };
    invokeMock.mockResolvedValueOnce({ id: "message-id" });
    await appendConversationMessage("conversation-id", appendInput);
    expect(invokeMock).toHaveBeenCalledWith("conversation_message_append", {
      conversation_id: "conversation-id",
      input: appendInput,
    });

    const chatRequest: DeveloperChatSendRequest = {
      conversation_id: "conversation-id",
      task: "main_writing",
      quality: "balanced",
      preferred_model: null,
      required_capabilities: {
        streaming: false,
        embeddings: false,
        tools: false,
        vision: false,
        structured_output: false,
        prompt_caching: false,
      },
      minimum_context_window_tokens: null,
      system_instructions: "Stay within canon.",
      character_ids: [],
      include_character_states: false,
      context_budget: {
        output_reserve_tokens: null,
        safety_margin_tokens: 0,
      },
      message: "Draft the next scene.",
      temperature: 0.4,
      retry_attempt: false,
    };
    invokeMock.mockResolvedValueOnce({});
    await sendDeveloperChat(chatRequest);
    expect(invokeMock).toHaveBeenCalledWith("developer_chat_send", {
      request: chatRequest,
    });

    invokeMock.mockResolvedValueOnce({});
    await chapterChatSend("chapter-id", chatRequest);
    expect(invokeMock).toHaveBeenCalledWith("chapter_chat_send", {
      chapter_id: "chapter-id",
      request: chatRequest,
    });

    const memoryRequest: MemoryToolRequest = {
      project_id: "project-id",
      character_id: "character-id",
      base_revision: 2,
      actor_id: "developer-chat",
      action: {
        action: "update_character",
        input: { name: "Mira Vale", summary: "Scout", role: null },
      },
    };
    invokeMock.mockResolvedValueOnce({ id: "proposal-id" });
    await proposeMemoryTool(memoryRequest);
    expect(invokeMock).toHaveBeenCalledWith("memory_tool_propose", {
      request: memoryRequest,
    });
  });

  it("uses exact snake_case payloads for the shared chat runtime", async () => {
    invokeMock.mockReset();
    try {
      const loadRequest: ChatRuntimeLoadRequest = {
      project_id: "project-id",
      chapter_id: null,
      conversation_id: null,
      kind: "developer_chat",
      };
      const runtimeSettings: ChatRuntimeSettingsInput = {
      assistant_id: "writing-coach",
      provider_id: "mock",
      model_id: "writer",
      quality: "deep",
      temperature: 0.7,
      };
      const sendRequest: ChatSendRequest = {
      conversation_id: "conversation-id",
      task: "developer_chat",
      runtime: runtimeSettings,
      system_instructions: "Stay within canon.",
      character_ids: [],
      include_character_states: false,
      context_budget: { output_reserve_tokens: null, safety_margin_tokens: 0 },
      message: "Draft the next scene.",
      retry_attempt: false,
      };

      invokeMock.mockResolvedValue({});
      await loadChatRuntime(loadRequest);
      expect(invokeMock).toHaveBeenCalledWith("chat_runtime_load", {
        request: loadRequest,
      });

      await updateChatRuntime("conversation-id", runtimeSettings);
      expect(invokeMock).toHaveBeenCalledWith("chat_runtime_update", {
        conversation_id: "conversation-id",
        input: runtimeSettings,
      });

      await sendChat(sendRequest);
      expect(invokeMock).toHaveBeenCalledWith("chat_send", {
        request: sendRequest,
      });
    } finally {
      invokeMock.mockReset();
    }
  });

  it("uses typed chapter and manuscript command payloads", async () => {
    const chapterInput = { number: 1, title: "Opening", synopsis: "Arrival" };
    invokeMock.mockResolvedValueOnce({ id: "chapter-id" });
    await createChapter("project-id", chapterInput);
    expect(invokeMock).toHaveBeenCalledWith("chapter_create", {
      project_id: "project-id",
      input: chapterInput,
    });

    invokeMock.mockResolvedValueOnce([]);
    await listChapters("project-id");
    expect(invokeMock).toHaveBeenCalledWith("chapter_list", {
      project_id: "project-id",
      filter: { include_archived: false },
    });

    invokeMock.mockResolvedValueOnce({ id: "manuscript-id", revision: 2 });
    await saveManuscript("chapter-id", {
      content: "The gate opened.",
      label: "Draft 2",
      expected_revision: 1,
    });
    expect(invokeMock).toHaveBeenCalledWith("manuscript_save", {
      chapter_id: "chapter-id",
      input: {
        content: "The gate opened.",
        label: "Draft 2",
        expected_revision: 1,
      },
    });

    invokeMock.mockResolvedValueOnce({ id: "manuscript-id" });
    await getManuscript("chapter-id");
    expect(invokeMock).toHaveBeenCalledWith("manuscript_get", {
      chapter_id: "chapter-id",
    });

    invokeMock.mockResolvedValueOnce([]);
    await listManuscriptRevisions("chapter-id");
    expect(invokeMock).toHaveBeenCalledWith("manuscript_revision_list", {
      chapter_id: "chapter-id",
    });

    invokeMock.mockResolvedValueOnce({ revision: 3 });
    await restoreManuscript("chapter-id", 1, 2);
    expect(invokeMock).toHaveBeenCalledWith("manuscript_restore", {
      chapter_id: "chapter-id",
      revision: 1,
      expected_revision: 2,
    });

    const proposalInput = {
      chapter_id: "chapter-id",
      base_revision: 2,
      proposed_content: "A revised scene.",
      content_format: "plain_text" as const,
      rationale: "Tighten pacing",
    };
    invokeMock.mockResolvedValueOnce({ id: "proposal-id" });
    await createManuscriptProposal("project-id", proposalInput);
    expect(invokeMock).toHaveBeenCalledWith("manuscript_proposal_create", {
      project_id: "project-id",
      input: proposalInput,
    });

    invokeMock.mockResolvedValueOnce([]);
    await listManuscriptProposals("chapter-id", "draft");
    expect(invokeMock).toHaveBeenCalledWith("manuscript_proposal_list", {
      chapter_id: "chapter-id",
      status: "draft",
    });

    invokeMock.mockResolvedValueOnce({ revision: 3 });
    await promoteManuscriptProposal("proposal-id", 2);
    expect(invokeMock).toHaveBeenCalledWith("manuscript_proposal_promote", {
      id: "proposal-id",
      expected_revision: 2,
    });

    invokeMock.mockResolvedValueOnce({ status: "rejected" });
    await rejectManuscriptProposal("proposal-id");
    expect(invokeMock).toHaveBeenCalledWith("manuscript_proposal_reject", {
      id: "proposal-id",
    });
  });

  it("maps conflict and locked canon errors without backend details", () => {
    const conflict = normalizeCommandError({
      code: "conflict",
      details: { sql: "secret" },
    });
    expect(conflict).toMatchObject({
      code: "conflict",
      message: "This record changed. Reload and try again.",
    });
    expect(conflict.message).not.toContain("secret");

    const locked = normalizeCommandError({ code: "locked_canon" });
    expect(locked).toMatchObject({
      code: "locked_canon",
      message: "Locked canon cannot be changed.",
    });

    const archivedMemory = normalizeCommandError({ code: "archived_memory" });
    expect(archivedMemory).toMatchObject({
      code: "archived_memory",
      message: "Archived project memory cannot be edited.",
    });
  });

  it("passes expected revisions for character mutations", async () => {
    invokeMock.mockClear();
    invokeMock.mockResolvedValueOnce({ id: "character-id", revision: 2 });
    await updateCharacter("character-id", { name: "Mira" }, 1);
    expect(invokeMock).toHaveBeenCalledWith("character_update", {
      id: "character-id",
      input: { name: "Mira" },
      expected_revision: 1,
    });

    invokeMock.mockResolvedValueOnce({
      character_id: "character-id",
      revision: 2,
    });
    await updateCharacterState("character-id", { goals: "Find the key" }, 1);
    expect(invokeMock).toHaveBeenCalledWith("character_state_update", {
      character_id: "character-id",
      input: { goals: "Find the key" },
      expected_revision: 1,
    });
  });

  it("uses typed provider and context command payloads", async () => {
    invokeMock.mockResolvedValueOnce([]);
    await listProviders();
    expect(invokeMock).toHaveBeenCalledWith("provider_list", {});

    invokeMock.mockResolvedValueOnce([]);
    await listModels();
    expect(invokeMock).toHaveBeenCalledWith("model_list", {
      provider_id: null,
    });

    const routingRequest: RoutingRequest = {
      task: "main_writing",
      quality: "balanced",
      preferred_model: null,
      required_capabilities: {
        streaming: true,
        embeddings: false,
        tools: false,
        vision: false,
        structured_output: true,
        prompt_caching: false,
      },
      minimum_context_window_tokens: 8_192,
    };
    const route = {
      model: { provider_id: "mock", model_id: "mock-small" },
      profile: {
        provider_id: "mock",
        model_id: "mock-small",
        display_name: "Mock Small",
        context_window_tokens: 16_384,
        default_output_tokens: 1_024,
        strengths: ["prose"],
        weaknesses: [],
        strategy: [],
        tier: "small",
        capabilities: routingRequest.required_capabilities,
      },
      quality: "balanced",
      reason: "policy",
    } as const;
    invokeMock.mockResolvedValueOnce(route);
    await expect(routeModel(routingRequest)).resolves.toEqual(route);
    expect(invokeMock).toHaveBeenCalledWith("model_route", {
      request: routingRequest,
    });

    const orchestrationRequest: OrchestrationRequest = {
      project_id: "project-id",
      task: "main_writing",
      quality: "fast",
      preferred_model: null,
      required_capabilities: routingRequest.required_capabilities,
      minimum_context_window_tokens: null,
      system_instructions: "Stay within canon.",
      character_ids: [],
      include_character_states: false,
      working_memory: [],
      context_budget: {
        output_reserve_tokens: null,
        safety_margin_tokens: 0,
      },
      user_prompt: "Draft the next scene.",
      temperature: 0.4,
    };
    const orchestrationResult = {
      plan: { task: "main_writing", quality: "fast", steps: [] },
      steps: [],
    } as const;
    invokeMock.mockResolvedValueOnce(orchestrationResult);
    await expect(runOrchestrator(orchestrationRequest)).resolves.toEqual(
      orchestrationResult,
    );
    expect(invokeMock).toHaveBeenCalledWith("orchestrator_run", {
      request: orchestrationRequest,
    });

    invokeMock.mockResolvedValueOnce({ blocks: [] });
    const request: ContextCompileRequest = {
      project_id: "project-id",
      task: "writing",
      model: { provider_id: "mock", model_id: "mock-small" },
      system_instructions: "Write",
      project_memory_refs: [],
      character_ids: [],
      include_character_states: false,
      working_memory: [],
      budget: { output_reserve_tokens: null, safety_margin_tokens: 0 },
    };
    await compileContext(request);
    expect(invokeMock).toHaveBeenCalledWith("context_compile", { request });
  });

  it("uses the typed provider generate command payload", async () => {
    const request: GenerateRequest = {
      model: { provider_id: "mock", model_id: "mock-small" },
      messages: [{ role: "user", content: "Write a scene" }],
      max_output_tokens: 64,
      temperature: 0.7,
    };
    const response = {
      model: request.model,
      text: "A scene",
      usage: { input_tokens: 10, output_tokens: 3 },
    };
    invokeMock.mockResolvedValueOnce(response);

    await expect(generateProvider(request)).resolves.toEqual(response);
    expect(invokeMock).toHaveBeenCalledWith("provider_generate", { request });
  });

  it("uses snake_case provider runtime setup payloads", async () => {
    const input: ProviderConfigureInput = {
      descriptor: {
        id: "openai-compatible",
        display_name: "OpenAI-compatible",
      },
      base_url: "https://example.test/v1",
      models: [],
      credential_id: "session-key",
      credential_value: "secret-token",
    };
    const configured = { descriptor: input.descriptor, models: [] };
    invokeMock.mockResolvedValueOnce(configured);
    await expect(configureProvider(input)).resolves.toEqual(configured);
    expect(invokeMock).toHaveBeenCalledWith("provider_configure", { input });

    invokeMock.mockResolvedValueOnce(input.descriptor);
    await expect(removeProvider(input.descriptor.id)).resolves.toEqual(
      input.descriptor,
    );
    expect(invokeMock).toHaveBeenCalledWith("provider_remove", {
      provider_id: input.descriptor.id,
    });

    const status = {
      kind: "ephemeral",
      persistent: false,
      available: true,
    } as const;
    invokeMock.mockResolvedValueOnce(status);
    await expect(getCredentialStoreStatus()).resolves.toEqual(status);
    expect(invokeMock).toHaveBeenLastCalledWith(
      "provider_credential_status",
      {},
    );
  });

  it("uses typed provider editing, settings and connection-test payloads", async () => {
    const settings = {
      descriptor: inputDescriptor(),
      base_url: "https://example.test/v1",
      models: [],
      credential_id: "session-key",
    };
    invokeMock.mockResolvedValueOnce(settings);
    await expect(getProviderSettings("openai-compatible")).resolves.toEqual(
      settings,
    );
    expect(invokeMock).toHaveBeenCalledWith("provider_get", {
      provider_id: "openai-compatible",
    });

    const update: ProviderUpdateInput = {
      descriptor: inputDescriptor(),
      base_url: "https://updated.example.test/v1",
      models: [],
      credential_id: "session-key",
      credential_value: null,
    };
    invokeMock.mockResolvedValueOnce({ descriptor: update.descriptor, models: [] });
    await updateProvider(update);
    expect(invokeMock).toHaveBeenCalledWith("provider_update", { input: update });

    const testResult = {
      provider_id: "openai-compatible",
      model_id: "mock-small",
      message: "Connection successful.",
    };
    invokeMock.mockResolvedValueOnce(testResult);
    await expect(testProvider("openai-compatible")).resolves.toEqual(testResult);
    expect(invokeMock).toHaveBeenCalledWith("provider_test", {
      provider_id: "openai-compatible",
    });
  });
});

function inputDescriptor() {
  return {
    id: "openai-compatible",
    display_name: "OpenAI-compatible",
  };
}
