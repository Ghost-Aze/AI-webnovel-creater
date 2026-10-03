use crate::manuscripts::service::ManuscriptService;
use crate::{
    characters::service::CharacterService,
    context::{
        CompiledContext, ContextCompileRequest, ContextCompiler, ContextSource,
        ServiceContextSource,
    },
    conversations::{
        chat::{
            ChapterChatService, DeveloperChatSendRequest, DeveloperChatSendResult,
            DeveloperChatService,
        },
        service::ConversationService,
    },
    domain::character::{
        Character, CharacterListFilter, CharacterState, CreateCharacterInput, UpdateCharacterInput,
        UpdateCharacterStateInput,
    },
    domain::conversation::{
        AppendMessageInput, Conversation, ConversationListFilter, ConversationMessage,
        CreateConversationInput,
    },
    domain::manuscript::{
        Chapter, ChapterListFilter, CreateChapterInput, Manuscript, ManuscriptRevision,
        SaveManuscriptInput, UpdateChapterInput,
    },
    domain::manuscript_proposal::{CreateManuscriptProposalInput, ManuscriptProposal},
    domain::project::{CreateProjectInput, Project, ProjectListFilter, UpdateProjectInput},
    domain::revision::{
        CanonStatus, CreateProposalInput, MemoryEntityType, MemoryProposal, MemoryRevision,
        ProposalStatus,
    },
    domain::user::{
        UpdateUserPreferencesInput, UpdateUserProfileInput, UserPreferences, UserProfile,
    },
    error::AppResult,
    memory_tools::{MemoryToolRequest, MemoryToolService},
    orchestration::{NarrativeOrchestrator, OrchestrationRequest, OrchestrationResult},
    provider::{
        CredentialStoreStatus, GenerateRequest, GenerateResponse, ModelProfile, ModelRouter,
        ProviderConfigureInput, ProviderConfigureResult, ProviderDescriptor, ProviderRegistry,
        ProviderRuntime, RouteDecision, RoutingRequest,
    },
    revisions::service::ProposalService,
    revisions::service::RevisionService,
    users::service::UserService,
};

#[cfg(feature = "tauri-app")]
use crate::AppState;

pub fn create_project(
    service: &crate::projects::service::ProjectService,
    input: CreateProjectInput,
) -> AppResult<Project> {
    report("project_create", service.create(input))
}

pub fn list_projects(
    service: &crate::projects::service::ProjectService,
    filter: ProjectListFilter,
) -> AppResult<Vec<Project>> {
    report("project_list", service.list(filter))
}

pub fn get_project(
    service: &crate::projects::service::ProjectService,
    id: String,
) -> AppResult<Project> {
    report("project_get", service.get(&id))
}

pub fn update_project(
    service: &crate::projects::service::ProjectService,
    id: String,
    input: UpdateProjectInput,
) -> AppResult<Project> {
    report("project_update", service.update(&id, input))
}

pub fn archive_project(
    service: &crate::projects::service::ProjectService,
    id: String,
) -> AppResult<Project> {
    report("project_archive", service.archive(&id))
}

pub fn get_user_profile(service: &UserService) -> AppResult<UserProfile> {
    report("user_profile_get", service.get_profile())
}

pub fn update_user_profile(
    service: &UserService,
    input: UpdateUserProfileInput,
    expected_revision: u64,
) -> AppResult<UserProfile> {
    report(
        "user_profile_update",
        service.update_profile(input, expected_revision),
    )
}

pub fn get_user_preferences(service: &UserService) -> AppResult<UserPreferences> {
    report("user_preferences_get", service.get_preferences())
}

pub fn update_user_preferences(
    service: &UserService,
    input: UpdateUserPreferencesInput,
    expected_revision: u64,
) -> AppResult<UserPreferences> {
    report(
        "user_preferences_update",
        service.update_preferences(input, expected_revision),
    )
}

pub fn create_conversation(
    service: &ConversationService,
    input: CreateConversationInput,
) -> AppResult<Conversation> {
    report("conversation_create", service.create(input))
}

pub fn list_conversations(
    service: &ConversationService,
    project_id: String,
    filter: ConversationListFilter,
) -> AppResult<Vec<Conversation>> {
    report("conversation_list", service.list(&project_id, filter))
}

pub fn get_conversation(service: &ConversationService, id: String) -> AppResult<Conversation> {
    report("conversation_get", service.get(&id))
}

pub fn list_conversation_messages(
    service: &ConversationService,
    conversation_id: String,
    limit: Option<u32>,
) -> AppResult<Vec<ConversationMessage>> {
    report(
        "conversation_message_list",
        service.list_messages(&conversation_id, limit),
    )
}

pub fn append_conversation_message(
    service: &ConversationService,
    conversation_id: String,
    input: AppendMessageInput,
) -> AppResult<ConversationMessage> {
    report(
        "conversation_message_append",
        service.append_message(&conversation_id, input),
    )
}

pub fn create_chapter(
    service: &ManuscriptService,
    project_id: String,
    input: CreateChapterInput,
) -> AppResult<Chapter> {
    report("chapter_create", service.create_chapter(project_id, input))
}

pub fn list_chapters(
    service: &ManuscriptService,
    project_id: String,
    filter: ChapterListFilter,
) -> AppResult<Vec<Chapter>> {
    report("chapter_list", service.list_chapters(&project_id, filter))
}

pub fn get_chapter(service: &ManuscriptService, id: String) -> AppResult<Chapter> {
    report("chapter_get", service.get_chapter(&id))
}

pub fn update_chapter(
    service: &ManuscriptService,
    id: String,
    input: UpdateChapterInput,
    expected_revision: u64,
) -> AppResult<Chapter> {
    report(
        "chapter_update",
        service.update_chapter(&id, input, expected_revision),
    )
}

pub fn archive_chapter(
    service: &ManuscriptService,
    id: String,
    expected_revision: u64,
) -> AppResult<Chapter> {
    report(
        "chapter_archive",
        service.archive_chapter(&id, expected_revision),
    )
}

pub fn get_manuscript(service: &ManuscriptService, chapter_id: String) -> AppResult<Manuscript> {
    report("manuscript_get", service.get_manuscript(&chapter_id))
}

pub fn save_manuscript(
    service: &ManuscriptService,
    chapter_id: String,
    input: SaveManuscriptInput,
) -> AppResult<Manuscript> {
    report(
        "manuscript_save",
        service.save_manuscript(&chapter_id, input),
    )
}

pub fn list_manuscript_revisions(
    service: &ManuscriptService,
    chapter_id: String,
) -> AppResult<Vec<ManuscriptRevision>> {
    report(
        "manuscript_revision_list",
        service.list_revisions(&chapter_id),
    )
}

pub fn restore_manuscript(
    service: &ManuscriptService,
    chapter_id: String,
    revision: u64,
    expected_revision: u64,
) -> AppResult<Manuscript> {
    report(
        "manuscript_restore",
        service.restore_manuscript(&chapter_id, revision, expected_revision),
    )
}

pub fn create_manuscript_proposal(
    service: &crate::manuscript_proposals::service::ManuscriptProposalService,
    project_id: String,
    input: CreateManuscriptProposalInput,
) -> AppResult<ManuscriptProposal> {
    report(
        "manuscript_proposal_create",
        service.create(project_id, input),
    )
}

pub fn list_manuscript_proposals(
    service: &crate::manuscript_proposals::service::ManuscriptProposalService,
    chapter_id: String,
    status: Option<ProposalStatus>,
) -> AppResult<Vec<ManuscriptProposal>> {
    report(
        "manuscript_proposal_list",
        service.list(&chapter_id, status),
    )
}

pub fn get_manuscript_proposal(
    service: &crate::manuscript_proposals::service::ManuscriptProposalService,
    id: String,
) -> AppResult<ManuscriptProposal> {
    report("manuscript_proposal_get", service.get(&id))
}

pub fn promote_manuscript_proposal(
    service: &crate::manuscript_proposals::service::ManuscriptProposalService,
    id: String,
    expected_revision: u64,
) -> AppResult<Manuscript> {
    report(
        "manuscript_proposal_promote",
        service.promote(&id, expected_revision),
    )
}

pub fn reject_manuscript_proposal(
    service: &crate::manuscript_proposals::service::ManuscriptProposalService,
    id: String,
) -> AppResult<ManuscriptProposal> {
    report("manuscript_proposal_reject", service.reject(&id))
}

pub fn create_character(
    service: &CharacterService,
    project_id: String,
    input: CreateCharacterInput,
) -> AppResult<Character> {
    report("character_create", service.create(project_id, input))
}

pub fn list_characters(
    service: &CharacterService,
    project_id: String,
    filter: CharacterListFilter,
) -> AppResult<Vec<Character>> {
    report("character_list", service.list(&project_id, filter))
}

pub fn get_character(service: &CharacterService, id: String) -> AppResult<Character> {
    report("character_get", service.get(&id))
}

pub fn update_character(
    service: &CharacterService,
    id: String,
    input: UpdateCharacterInput,
) -> AppResult<Character> {
    report("character_update", service.update(&id, input))
}

pub fn update_character_with_revision(
    service: &CharacterService,
    id: String,
    input: UpdateCharacterInput,
    expected_revision: u64,
) -> AppResult<Character> {
    report(
        "character_update",
        service.update_with_revision(&id, input, expected_revision),
    )
}

pub fn archive_character(service: &CharacterService, id: String) -> AppResult<Character> {
    report("character_archive", service.archive(&id))
}

pub fn archive_character_with_revision(
    service: &CharacterService,
    id: String,
    expected_revision: u64,
) -> AppResult<Character> {
    report(
        "character_archive",
        service.archive_with_revision(&id, expected_revision),
    )
}

pub fn get_character_state(
    service: &CharacterService,
    character_id: String,
) -> AppResult<CharacterState> {
    report("character_state_get", service.get_state(&character_id))
}

pub fn update_character_state(
    service: &CharacterService,
    character_id: String,
    input: UpdateCharacterStateInput,
) -> AppResult<CharacterState> {
    report(
        "character_state_update",
        service.update_state(&character_id, input),
    )
}

pub fn update_character_state_with_revision(
    service: &CharacterService,
    character_id: String,
    input: UpdateCharacterStateInput,
    expected_revision: u64,
) -> AppResult<CharacterState> {
    report(
        "character_state_update",
        service.update_state_with_revision(&character_id, input, expected_revision),
    )
}

pub fn list_memory_history(
    service: &RevisionService,
    entity_type: MemoryEntityType,
    entity_id: String,
) -> AppResult<Vec<crate::domain::revision::MemoryRevision>> {
    report(
        "memory_history_list",
        service.history(entity_type, &entity_id),
    )
}

pub fn restore_memory(
    service: &RevisionService,
    entity_type: MemoryEntityType,
    entity_id: String,
    revision: u64,
    expected_revision: u64,
) -> AppResult<crate::domain::revision::MemoryRevision> {
    report(
        "memory_restore",
        service.restore(entity_type, &entity_id, revision, expected_revision),
    )
}

pub fn create_memory_proposal(
    service: &ProposalService,
    input: CreateProposalInput,
) -> AppResult<MemoryProposal> {
    report("memory_proposal_create", service.create(input))
}

pub fn list_memory_proposals(
    service: &ProposalService,
    project_id: String,
    status: Option<ProposalStatus>,
) -> AppResult<Vec<MemoryProposal>> {
    report("memory_proposal_list", service.list(&project_id, status))
}

pub fn promote_memory_proposal(
    service: &ProposalService,
    id: String,
    expected_revision: u64,
) -> AppResult<MemoryRevision> {
    report(
        "memory_proposal_promote",
        service.promote(&id, expected_revision),
    )
}

pub fn reject_memory_proposal(service: &ProposalService, id: String) -> AppResult<MemoryProposal> {
    report("memory_proposal_reject", service.reject(&id))
}

pub fn set_memory_canon_status(
    service: &RevisionService,
    entity_type: MemoryEntityType,
    entity_id: String,
    status: CanonStatus,
    expected_revision: u64,
) -> AppResult<crate::domain::revision::MemoryRevision> {
    report(
        "memory_set_canon_status",
        service.set_canon_status(entity_type, &entity_id, status, expected_revision),
    )
}

pub fn list_providers(registry: &ProviderRegistry) -> AppResult<Vec<ProviderDescriptor>> {
    report(
        "provider_list",
        registry.list_providers().map_err(Into::into),
    )
}

pub fn list_models(
    registry: &ProviderRegistry,
    provider_id: Option<String>,
) -> AppResult<Vec<ModelProfile>> {
    report(
        "model_list",
        registry
            .list_models(provider_id.as_deref())
            .map_err(Into::into),
    )
}

pub fn route_model(
    registry: &ProviderRegistry,
    request: RoutingRequest,
) -> AppResult<RouteDecision> {
    report(
        "model_route",
        ModelRouter.route(registry, request).map_err(Into::into),
    )
}

pub async fn orchestrate(
    registry: &ProviderRegistry,
    compiler: &ContextCompiler,
    source: &dyn ContextSource,
    request: OrchestrationRequest,
) -> AppResult<OrchestrationResult> {
    report(
        "orchestrator_run",
        NarrativeOrchestrator
            .run(registry, compiler, source, request)
            .await,
    )
}

pub async fn send_developer_chat(
    service: &ConversationService,
    registry: &ProviderRegistry,
    compiler: &ContextCompiler,
    source: &ServiceContextSource,
    request: DeveloperChatSendRequest,
) -> AppResult<DeveloperChatSendResult> {
    report(
        "developer_chat_send",
        DeveloperChatService
            .send(service, registry, compiler, source, request)
            .await,
    )
}

pub async fn send_chapter_chat(
    conversations: &ConversationService,
    manuscripts: &crate::manuscripts::service::ManuscriptService,
    registry: &ProviderRegistry,
    compiler: &ContextCompiler,
    source: &ServiceContextSource,
    chapter_id: String,
    request: DeveloperChatSendRequest,
) -> AppResult<DeveloperChatSendResult> {
    report(
        "chapter_chat_send",
        ChapterChatService::send(
            conversations,
            manuscripts,
            registry,
            compiler,
            source,
            &chapter_id,
            request,
        )
        .await,
    )
}

pub fn propose_memory_tool(
    service: &MemoryToolService,
    request: MemoryToolRequest,
) -> AppResult<crate::domain::revision::MemoryProposal> {
    report("memory_tool_propose", service.propose(request))
}

pub fn compile_context(
    registry: &ProviderRegistry,
    compiler: &ContextCompiler,
    source: &ServiceContextSource,
    request: ContextCompileRequest,
) -> AppResult<CompiledContext> {
    let resolved = registry.resolve(&request.model.provider_id, &request.model.model_id)?;
    report(
        "context_compile",
        compiler.compile(request, &resolved.profile, source),
    )
}

pub async fn generate(
    registry: &ProviderRegistry,
    request: GenerateRequest,
) -> AppResult<GenerateResponse> {
    let resolved = registry.resolve(&request.model.provider_id, &request.model.model_id)?;
    report(
        "provider_generate",
        resolved
            .provider
            .generate(request)
            .await
            .map_err(Into::into),
    )
}

pub fn configure_provider(
    runtime: &ProviderRuntime,
    input: ProviderConfigureInput,
) -> AppResult<ProviderConfigureResult> {
    report(
        "provider_configure",
        runtime.configure(input).map_err(Into::into),
    )
}

pub fn remove_provider(
    runtime: &ProviderRuntime,
    provider_id: String,
) -> AppResult<ProviderDescriptor> {
    report(
        "provider_remove",
        runtime.remove(&provider_id).map_err(Into::into),
    )
}

pub fn credential_store_status(runtime: &ProviderRuntime) -> CredentialStoreStatus {
    runtime.credential_store_status()
}

fn report<T>(command: &'static str, result: AppResult<T>) -> AppResult<T> {
    match &result {
        Ok(_) => tracing::debug!(command, "project command completed"),
        Err(error) => tracing::warn!(command, code = error.code(), "project command failed"),
    }
    result
}

#[cfg(feature = "tauri-app")]
mod tauri_commands {
    use tauri::State;

    use super::*;

    #[tauri::command]
    pub fn project_create(
        state: State<'_, AppState>,
        input: CreateProjectInput,
    ) -> AppResult<Project> {
        create_project(&state.project_service, input)
    }

    #[tauri::command]
    pub fn project_list(
        state: State<'_, AppState>,
        filter: ProjectListFilter,
    ) -> AppResult<Vec<Project>> {
        list_projects(&state.project_service, filter)
    }

    #[tauri::command]
    pub fn project_get(state: State<'_, AppState>, id: String) -> AppResult<Project> {
        get_project(&state.project_service, id)
    }

    #[tauri::command]
    pub fn project_update(
        state: State<'_, AppState>,
        id: String,
        input: UpdateProjectInput,
    ) -> AppResult<Project> {
        update_project(&state.project_service, id, input)
    }

    #[tauri::command]
    pub fn project_archive(state: State<'_, AppState>, id: String) -> AppResult<Project> {
        archive_project(&state.project_service, id)
    }

    #[tauri::command(rename_all = "snake_case")]
    pub fn user_profile_get(state: State<'_, AppState>) -> AppResult<UserProfile> {
        super::get_user_profile(&state.user_service)
    }

    #[tauri::command(rename_all = "snake_case")]
    pub fn user_profile_update(
        state: State<'_, AppState>,
        input: UpdateUserProfileInput,
        expected_revision: u64,
    ) -> AppResult<UserProfile> {
        super::update_user_profile(&state.user_service, input, expected_revision)
    }

    #[tauri::command(rename_all = "snake_case")]
    pub fn user_preferences_get(state: State<'_, AppState>) -> AppResult<UserPreferences> {
        super::get_user_preferences(&state.user_service)
    }

    #[tauri::command(rename_all = "snake_case")]
    pub fn user_preferences_update(
        state: State<'_, AppState>,
        input: UpdateUserPreferencesInput,
        expected_revision: u64,
    ) -> AppResult<UserPreferences> {
        super::update_user_preferences(&state.user_service, input, expected_revision)
    }

    #[tauri::command(rename_all = "snake_case")]
    pub fn conversation_create(
        state: State<'_, AppState>,
        input: CreateConversationInput,
    ) -> AppResult<Conversation> {
        create_conversation(&state.conversation_service, input)
    }

    #[tauri::command(rename_all = "snake_case")]
    pub fn conversation_list(
        state: State<'_, AppState>,
        project_id: String,
        filter: ConversationListFilter,
    ) -> AppResult<Vec<Conversation>> {
        list_conversations(&state.conversation_service, project_id, filter)
    }

    #[tauri::command(rename_all = "snake_case")]
    pub fn conversation_get(state: State<'_, AppState>, id: String) -> AppResult<Conversation> {
        get_conversation(&state.conversation_service, id)
    }

    #[tauri::command(rename_all = "snake_case")]
    pub fn conversation_message_list(
        state: State<'_, AppState>,
        conversation_id: String,
        limit: Option<u32>,
    ) -> AppResult<Vec<ConversationMessage>> {
        list_conversation_messages(&state.conversation_service, conversation_id, limit)
    }

    #[tauri::command(rename_all = "snake_case")]
    pub fn conversation_message_append(
        state: State<'_, AppState>,
        conversation_id: String,
        input: AppendMessageInput,
    ) -> AppResult<ConversationMessage> {
        append_conversation_message(&state.conversation_service, conversation_id, input)
    }

    #[tauri::command(rename_all = "snake_case")]
    pub async fn developer_chat_send(
        state: State<'_, AppState>,
        request: DeveloperChatSendRequest,
    ) -> AppResult<DeveloperChatSendResult> {
        super::send_developer_chat(
            &state.conversation_service,
            &state.provider_registry,
            &state.context_compiler,
            &state.context_source,
            request,
        )
        .await
    }

    #[tauri::command(rename_all = "snake_case")]
    pub async fn chapter_chat_send(
        state: State<'_, AppState>,
        chapter_id: String,
        request: DeveloperChatSendRequest,
    ) -> AppResult<DeveloperChatSendResult> {
        super::send_chapter_chat(
            &state.conversation_service,
            &state.manuscript_service,
            &state.provider_registry,
            &state.context_compiler,
            &state.context_source,
            chapter_id,
            request,
        )
        .await
    }

    #[tauri::command(rename_all = "snake_case")]
    pub fn memory_tool_propose(
        state: State<'_, AppState>,
        request: MemoryToolRequest,
    ) -> AppResult<crate::domain::revision::MemoryProposal> {
        super::propose_memory_tool(&state.memory_tool_service, request)
    }

    #[tauri::command(rename_all = "snake_case")]
    pub fn chapter_create(
        state: State<'_, AppState>,
        project_id: String,
        input: CreateChapterInput,
    ) -> AppResult<Chapter> {
        create_chapter(&state.manuscript_service, project_id, input)
    }

    #[tauri::command(rename_all = "snake_case")]
    pub fn chapter_list(
        state: State<'_, AppState>,
        project_id: String,
        filter: ChapterListFilter,
    ) -> AppResult<Vec<Chapter>> {
        list_chapters(&state.manuscript_service, project_id, filter)
    }

    #[tauri::command(rename_all = "snake_case")]
    pub fn chapter_get(state: State<'_, AppState>, id: String) -> AppResult<Chapter> {
        get_chapter(&state.manuscript_service, id)
    }

    #[tauri::command(rename_all = "snake_case")]
    pub fn chapter_update(
        state: State<'_, AppState>,
        id: String,
        input: UpdateChapterInput,
        expected_revision: u64,
    ) -> AppResult<Chapter> {
        update_chapter(&state.manuscript_service, id, input, expected_revision)
    }

    #[tauri::command(rename_all = "snake_case")]
    pub fn chapter_archive(
        state: State<'_, AppState>,
        id: String,
        expected_revision: u64,
    ) -> AppResult<Chapter> {
        archive_chapter(&state.manuscript_service, id, expected_revision)
    }

    #[tauri::command(rename_all = "snake_case")]
    pub fn manuscript_get(state: State<'_, AppState>, chapter_id: String) -> AppResult<Manuscript> {
        get_manuscript(&state.manuscript_service, chapter_id)
    }

    #[tauri::command(rename_all = "snake_case")]
    pub fn manuscript_save(
        state: State<'_, AppState>,
        chapter_id: String,
        input: SaveManuscriptInput,
    ) -> AppResult<Manuscript> {
        save_manuscript(&state.manuscript_service, chapter_id, input)
    }

    #[tauri::command(rename_all = "snake_case")]
    pub fn manuscript_revision_list(
        state: State<'_, AppState>,
        chapter_id: String,
    ) -> AppResult<Vec<ManuscriptRevision>> {
        list_manuscript_revisions(&state.manuscript_service, chapter_id)
    }

    #[tauri::command(rename_all = "snake_case")]
    pub fn manuscript_restore(
        state: State<'_, AppState>,
        chapter_id: String,
        revision: u64,
        expected_revision: u64,
    ) -> AppResult<Manuscript> {
        restore_manuscript(
            &state.manuscript_service,
            chapter_id,
            revision,
            expected_revision,
        )
    }

    #[tauri::command(rename_all = "snake_case")]
    pub fn manuscript_proposal_create(
        state: State<'_, AppState>,
        project_id: String,
        input: CreateManuscriptProposalInput,
    ) -> AppResult<ManuscriptProposal> {
        create_manuscript_proposal(&state.manuscript_proposal_service, project_id, input)
    }

    #[tauri::command(rename_all = "snake_case")]
    pub fn manuscript_proposal_list(
        state: State<'_, AppState>,
        chapter_id: String,
        status: Option<ProposalStatus>,
    ) -> AppResult<Vec<ManuscriptProposal>> {
        list_manuscript_proposals(&state.manuscript_proposal_service, chapter_id, status)
    }

    #[tauri::command(rename_all = "snake_case")]
    pub fn manuscript_proposal_get(
        state: State<'_, AppState>,
        id: String,
    ) -> AppResult<ManuscriptProposal> {
        get_manuscript_proposal(&state.manuscript_proposal_service, id)
    }

    #[tauri::command(rename_all = "snake_case")]
    pub fn manuscript_proposal_promote(
        state: State<'_, AppState>,
        id: String,
        expected_revision: u64,
    ) -> AppResult<Manuscript> {
        promote_manuscript_proposal(&state.manuscript_proposal_service, id, expected_revision)
    }

    #[tauri::command(rename_all = "snake_case")]
    pub fn manuscript_proposal_reject(
        state: State<'_, AppState>,
        id: String,
    ) -> AppResult<ManuscriptProposal> {
        reject_manuscript_proposal(&state.manuscript_proposal_service, id)
    }

    #[tauri::command]
    pub fn character_create(
        state: State<'_, AppState>,
        project_id: String,
        input: CreateCharacterInput,
    ) -> AppResult<Character> {
        create_character(&state.character_service, project_id, input)
    }

    #[tauri::command]
    pub fn character_list(
        state: State<'_, AppState>,
        project_id: String,
        filter: CharacterListFilter,
    ) -> AppResult<Vec<Character>> {
        list_characters(&state.character_service, project_id, filter)
    }

    #[tauri::command]
    pub fn character_get(state: State<'_, AppState>, id: String) -> AppResult<Character> {
        get_character(&state.character_service, id)
    }

    #[tauri::command]
    pub fn character_update(
        state: State<'_, AppState>,
        id: String,
        input: UpdateCharacterInput,
        expected_revision: u64,
    ) -> AppResult<Character> {
        update_character_with_revision(&state.character_service, id, input, expected_revision)
    }

    #[tauri::command]
    pub fn character_archive(
        state: State<'_, AppState>,
        id: String,
        expected_revision: u64,
    ) -> AppResult<Character> {
        archive_character_with_revision(&state.character_service, id, expected_revision)
    }

    #[tauri::command]
    pub fn character_state_get(
        state: State<'_, AppState>,
        character_id: String,
    ) -> AppResult<CharacterState> {
        get_character_state(&state.character_service, character_id)
    }

    #[tauri::command]
    pub fn character_state_update(
        state: State<'_, AppState>,
        character_id: String,
        input: UpdateCharacterStateInput,
        expected_revision: u64,
    ) -> AppResult<CharacterState> {
        update_character_state_with_revision(
            &state.character_service,
            character_id,
            input,
            expected_revision,
        )
    }

    #[tauri::command]
    pub fn memory_history_list(
        state: State<'_, AppState>,
        entity_type: MemoryEntityType,
        entity_id: String,
    ) -> AppResult<Vec<MemoryRevision>> {
        list_memory_history(&state.revision_service, entity_type, entity_id)
    }

    #[tauri::command]
    pub fn memory_restore(
        state: State<'_, AppState>,
        entity_type: MemoryEntityType,
        entity_id: String,
        revision: u64,
        expected_revision: u64,
    ) -> AppResult<MemoryRevision> {
        restore_memory(
            &state.revision_service,
            entity_type,
            entity_id,
            revision,
            expected_revision,
        )
    }

    #[tauri::command]
    pub fn memory_set_canon_status(
        state: State<'_, AppState>,
        entity_type: MemoryEntityType,
        entity_id: String,
        status: CanonStatus,
        expected_revision: u64,
    ) -> AppResult<MemoryRevision> {
        set_memory_canon_status(
            &state.revision_service,
            entity_type,
            entity_id,
            status,
            expected_revision,
        )
    }

    #[tauri::command]
    pub fn memory_proposal_create(
        state: State<'_, AppState>,
        input: CreateProposalInput,
    ) -> AppResult<MemoryProposal> {
        create_memory_proposal(&state.proposal_service, input)
    }

    #[tauri::command]
    pub fn memory_proposal_list(
        state: State<'_, AppState>,
        project_id: String,
        status: Option<ProposalStatus>,
    ) -> AppResult<Vec<MemoryProposal>> {
        list_memory_proposals(&state.proposal_service, project_id, status)
    }

    #[tauri::command]
    pub fn memory_proposal_promote(
        state: State<'_, AppState>,
        id: String,
        expected_revision: u64,
    ) -> AppResult<MemoryRevision> {
        promote_memory_proposal(&state.proposal_service, id, expected_revision)
    }

    #[tauri::command]
    pub fn memory_proposal_reject(
        state: State<'_, AppState>,
        id: String,
    ) -> AppResult<MemoryProposal> {
        reject_memory_proposal(&state.proposal_service, id)
    }

    #[tauri::command]
    pub fn provider_list(state: State<'_, AppState>) -> AppResult<Vec<ProviderDescriptor>> {
        list_providers(&state.provider_registry)
    }

    #[tauri::command(rename_all = "snake_case")]
    pub fn model_list(
        state: State<'_, AppState>,
        provider_id: Option<String>,
    ) -> AppResult<Vec<ModelProfile>> {
        list_models(&state.provider_registry, provider_id)
    }

    #[tauri::command(rename_all = "snake_case")]
    pub fn model_route(
        state: State<'_, AppState>,
        request: RoutingRequest,
    ) -> AppResult<RouteDecision> {
        route_model(&state.provider_registry, request)
    }

    #[tauri::command(rename_all = "snake_case")]
    pub async fn orchestrator_run(
        state: State<'_, AppState>,
        request: OrchestrationRequest,
    ) -> AppResult<OrchestrationResult> {
        orchestrate(
            &state.provider_registry,
            &state.context_compiler,
            &state.context_source,
            request,
        )
        .await
    }

    #[tauri::command]
    pub fn context_compile(
        state: State<'_, AppState>,
        request: ContextCompileRequest,
    ) -> AppResult<CompiledContext> {
        compile_context(
            &state.provider_registry,
            &state.context_compiler,
            &state.context_source,
            request,
        )
    }

    #[tauri::command(rename_all = "snake_case")]
    pub async fn provider_generate(
        state: State<'_, AppState>,
        request: GenerateRequest,
    ) -> AppResult<GenerateResponse> {
        generate(&state.provider_registry, request).await
    }

    #[tauri::command(rename_all = "snake_case")]
    pub fn provider_configure(
        state: State<'_, AppState>,
        input: ProviderConfigureInput,
    ) -> AppResult<ProviderConfigureResult> {
        configure_provider(&state.provider_runtime, input)
    }

    #[tauri::command(rename_all = "snake_case")]
    pub fn provider_remove(
        state: State<'_, AppState>,
        provider_id: String,
    ) -> AppResult<ProviderDescriptor> {
        remove_provider(&state.provider_runtime, provider_id)
    }

    #[tauri::command]
    pub fn provider_credential_status(state: State<'_, AppState>) -> CredentialStoreStatus {
        credential_store_status(&state.provider_runtime)
    }
}

#[cfg(feature = "tauri-app")]
pub use tauri_commands::*;

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use futures::executor::block_on;

    use super::*;
    use crate::{
        context::{ContextBudget, ContextTask},
        db,
        domain::{
            character::CreateCharacterInput,
            project::{ProjectStatus, UpdateProjectInput},
        },
        error::AppError,
        orchestration::OrchestrationRequest,
        projects::{repository::ProjectRepository, service::ProjectService},
        provider::{
            EphemeralCredentialStore, GenerateRequest, MockProvider, ModelProfile, ModelRef,
            ModelTask, ModelTier, PromptMessage, PromptRole, ProviderCapabilities,
            ProviderConfigureInput, ProviderDescriptor, ProviderRegistry, ProviderRuntime,
            QualityMode, RouteSelectionReason, RoutingRequest,
        },
        revisions::{repository::RevisionRepository, service::RevisionService},
        users::{repository::UserRepository, service::UserService},
    };

    fn service() -> ProjectService {
        ProjectService::new(ProjectRepository::new(db::in_memory().unwrap()))
    }

    fn user_service() -> UserService {
        UserService::new(UserRepository::new(db::in_memory().unwrap()))
    }

    #[test]
    fn user_command_helpers_delegate_typed_profile_and_preferences() {
        let service = user_service();
        let profile = get_user_profile(&service).unwrap();
        assert_eq!(profile.display_name, "Writer");
        let updated = update_user_profile(
            &service,
            UpdateUserProfileInput {
                display_name: "Mira".to_string(),
                preferred_language: "tr".to_string(),
            },
            1,
        )
        .unwrap();
        assert_eq!(updated.revision, 2);

        let preferences = get_user_preferences(&service).unwrap();
        assert!(preferences.avoid_repetition);
        let updated = update_user_preferences(
            &service,
            UpdateUserPreferencesInput {
                preferred_narrator: "first_person".to_string(),
                preferred_pov: "close".to_string(),
                chapter_length: 1800,
                scene_length: 500,
                dialogue_density: 60,
                prose_level: "lyrical".to_string(),
                pacing: "brisk".to_string(),
                avoid_repetition: false,
            },
            1,
        )
        .unwrap();
        assert_eq!(updated.revision, 2);
        assert!(!updated.avoid_repetition);

        let error = update_user_profile(
            &service,
            UpdateUserProfileInput {
                display_name: "Stale".to_string(),
                preferred_language: "en".to_string(),
            },
            1,
        )
        .unwrap_err();
        assert_eq!(error, crate::error::AppError::Conflict);
        assert!(!serde_json::to_string(&error).unwrap().contains("SELECT"));
    }

    #[test]
    fn command_helpers_return_project_shape_and_safe_errors() {
        let service = service();
        let project = create_project(
            &service,
            CreateProjectInput {
                name: "Command contract".to_string(),
                description: None,
            },
        )
        .unwrap();
        assert_eq!(project.status, ProjectStatus::Active);
        assert_eq!(project.description, "");

        let missing = get_project(&service, "missing".to_string()).unwrap_err();
        assert_eq!(missing, AppError::NotFound);
        let serialized = serde_json::to_string(&missing).unwrap();
        assert!(serialized.contains("not_found"));
        assert!(!serialized.contains("SELECT"));

        let updated = update_project(
            &service,
            project.id.clone(),
            UpdateProjectInput {
                name: "Updated".to_string(),
                description: None,
            },
        )
        .unwrap();
        assert_eq!(updated.name, "Updated");
    }

    #[test]
    fn revision_command_helpers_delegate_typed_arguments() {
        let connection = db::in_memory().unwrap();
        let project_service = ProjectService::new(ProjectRepository::new(connection.clone()));
        let character_repository =
            crate::characters::repository::CharacterRepository::new(connection.clone());
        let revision_service = RevisionService::new(
            RevisionRepository::new(connection.clone()),
            character_repository.clone(),
            crate::project_memory::repository::ProjectMemoryRepository::new(connection.clone()),
        );
        let character_service =
            crate::characters::service::CharacterService::new(character_repository);
        let project = project_service
            .create(CreateProjectInput {
                name: "Revision commands".to_string(),
                description: None,
            })
            .unwrap();
        let character = character_service
            .create(
                project.id,
                CreateCharacterInput {
                    name: "Mira".to_string(),
                    summary: None,
                    role: None,
                },
            )
            .unwrap();
        let history = list_memory_history(
            &revision_service,
            MemoryEntityType::Character,
            character.id.clone(),
        )
        .unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].revision, 1);
        let updated = update_character_with_revision(
            &character_service,
            character.id.clone(),
            UpdateCharacterInput {
                name: "Mira Vale".to_string(),
                summary: None,
                role: None,
            },
            1,
        )
        .unwrap();
        assert_eq!(updated.revision, 2);
    }

    #[test]
    fn provider_and_context_helpers_use_safe_typed_contracts() {
        let registry = ProviderRegistry::new();
        assert!(list_providers(&registry).unwrap().is_empty());
        assert!(list_models(&registry, None).unwrap().is_empty());

        let connection = db::in_memory().unwrap();
        let projects = ProjectService::new(ProjectRepository::new(connection.clone()));
        let characters = crate::characters::service::CharacterService::new(
            crate::characters::repository::CharacterRepository::new(connection),
        );
        let source = crate::context::ServiceContextSource::new(projects, characters);
        let request = ContextCompileRequest {
            project_id: "project".into(),
            task: ContextTask::DeveloperChat,
            model: ModelRef {
                provider_id: "missing".into(),
                model_id: "model".into(),
            },
            system_instructions: "Instructions".into(),
            character_ids: Vec::new(),
            include_character_states: false,
            working_memory: Vec::new(),
            budget: ContextBudget::default(),
        };
        assert_eq!(
            compile_context(&registry, &ContextCompiler::default(), &source, request),
            Err(AppError::ProviderNotFound)
        );
    }

    #[test]
    fn provider_generate_helper_resolves_registry_and_returns_typed_response() {
        let registry = ProviderRegistry::new();
        registry
            .register(Arc::new(MockProvider::new(
                ProviderDescriptor {
                    id: "mock".into(),
                    display_name: "Mock".into(),
                },
                vec![ModelProfile {
                    provider_id: "mock".into(),
                    model_id: "writer".into(),
                    display_name: "Writer".into(),
                    context_window_tokens: 4096,
                    default_output_tokens: 512,
                    strengths: Vec::new(),
                    weaknesses: Vec::new(),
                    strategy: Vec::new(),
                    tier: crate::provider::ModelTier::Medium,
                    capabilities: ProviderCapabilities::default(),
                }],
            )))
            .unwrap();
        let response = block_on(generate(
            &registry,
            GenerateRequest {
                model: ModelRef {
                    provider_id: "mock".into(),
                    model_id: "writer".into(),
                },
                messages: vec![PromptMessage {
                    role: PromptRole::User,
                    content: "Hello".into(),
                }],
                max_output_tokens: 32,
                temperature: None,
            },
        ))
        .unwrap();
        assert_eq!(response.text, "mock response: Hello");
    }

    #[test]
    fn orchestrator_helper_runs_typed_pipeline_without_memory_writes() {
        let registry = ProviderRegistry::new();
        registry
            .register(Arc::new(MockProvider::new(
                ProviderDescriptor {
                    id: "mock".into(),
                    display_name: "Mock".into(),
                },
                vec![ModelProfile {
                    provider_id: "mock".into(),
                    model_id: "writer".into(),
                    display_name: "Writer".into(),
                    context_window_tokens: 4096,
                    default_output_tokens: 512,
                    strengths: vec!["planning".into(), "prose".into()],
                    weaknesses: Vec::new(),
                    strategy: Vec::new(),
                    tier: crate::provider::ModelTier::Medium,
                    capabilities: ProviderCapabilities::default(),
                }],
            )))
            .unwrap();
        let connection = db::in_memory().unwrap();
        let projects = ProjectService::new(ProjectRepository::new(connection.clone()));
        let characters = crate::characters::service::CharacterService::new(
            crate::characters::repository::CharacterRepository::new(connection),
        );
        let project = projects
            .create(CreateProjectInput {
                name: "Orchestrated project".into(),
                description: Some("A premise".into()),
            })
            .unwrap();
        let source = ServiceContextSource::new(projects, characters);
        let result = block_on(orchestrate(
            &registry,
            &ContextCompiler::default(),
            &source,
            OrchestrationRequest {
                project_id: project.id,
                task: ModelTask::MainWriting,
                quality: QualityMode::Fast,
                preferred_model: None,
                required_capabilities: ProviderCapabilities::default(),
                minimum_context_window_tokens: None,
                system_instructions: "Stay within canon.".into(),
                character_ids: Vec::new(),
                include_character_states: false,
                working_memory: Vec::new(),
                context_budget: ContextBudget::default(),
                user_prompt: "Draft a scene.".into(),
                temperature: None,
            },
        ))
        .unwrap();
        assert_eq!(result.plan.steps.len(), 3);
        assert_eq!(result.steps.len(), 3);
    }

    #[test]
    fn model_route_helper_returns_typed_policy_decision_and_safe_errors() {
        let registry = ProviderRegistry::new();
        registry
            .register(Arc::new(MockProvider::new(
                ProviderDescriptor {
                    id: "mock".into(),
                    display_name: "Mock".into(),
                },
                vec![ModelProfile {
                    provider_id: "mock".into(),
                    model_id: "writer".into(),
                    display_name: "Writer".into(),
                    context_window_tokens: 4096,
                    default_output_tokens: 512,
                    strengths: vec!["prose".into()],
                    weaknesses: Vec::new(),
                    strategy: Vec::new(),
                    tier: ModelTier::Medium,
                    capabilities: ProviderCapabilities::default(),
                }],
            )))
            .unwrap();

        let decision = route_model(
            &registry,
            RoutingRequest {
                task: ModelTask::MainWriting,
                quality: QualityMode::Balanced,
                preferred_model: None,
                required_capabilities: ProviderCapabilities::default(),
                minimum_context_window_tokens: Some(2048),
            },
        )
        .unwrap();
        assert_eq!(decision.model.model_id, "writer");
        assert_eq!(decision.reason, RouteSelectionReason::Policy);

        let error = route_model(
            &registry,
            RoutingRequest {
                task: ModelTask::MainWriting,
                quality: QualityMode::Balanced,
                preferred_model: None,
                required_capabilities: ProviderCapabilities {
                    tools: true,
                    ..ProviderCapabilities::default()
                },
                minimum_context_window_tokens: None,
            },
        )
        .unwrap_err();
        assert_eq!(error, AppError::NoSuitableModel);
    }

    #[test]
    fn provider_runtime_commands_configure_and_remove_shared_registry() {
        let runtime = ProviderRuntime::new(Arc::new(EphemeralCredentialStore::new()));
        let input = ProviderConfigureInput {
            descriptor: ProviderDescriptor {
                id: "configured".into(),
                display_name: "Configured".into(),
            },
            base_url: "https://example.test/v1".into(),
            models: vec![ModelProfile {
                provider_id: "configured".into(),
                model_id: "writer".into(),
                display_name: "Writer".into(),
                context_window_tokens: 4096,
                default_output_tokens: 512,
                strengths: Vec::new(),
                weaknesses: Vec::new(),
                strategy: Vec::new(),
                tier: crate::provider::ModelTier::Medium,
                capabilities: ProviderCapabilities::default(),
            }],
            credential_id: "session-credential".into(),
            credential_value: "session-secret".into(),
        };
        let configured = configure_provider(&runtime, input).unwrap();
        assert_eq!(configured.descriptor.id, "configured");
        assert_eq!(runtime.registry().list_models(None).unwrap().len(), 1);
        assert_eq!(
            remove_provider(&runtime, "configured".into()).unwrap().id,
            "configured"
        );
        assert!(runtime.registry().list_providers().unwrap().is_empty());
    }

    #[test]
    fn credential_store_status_command_does_not_expose_secret_data() {
        let runtime = ProviderRuntime::new(Arc::new(EphemeralCredentialStore::new()));
        let status = credential_store_status(&runtime);
        assert_eq!(status.kind, crate::provider::CredentialStoreKind::Ephemeral);
        assert!(!status.persistent);
        assert!(status.available);
        assert!(!format!("{status:?}").contains("secret"));
    }

    #[test]
    fn model_list_command_declares_snake_case_arguments() {
        let source = include_str!("commands.rs");
        let attribute = ["#[tauri::command(", "rename_all = \"snake_case\")]"].concat();
        assert!(source.contains(&attribute));
        assert!(source.contains("pub fn user_profile_update"));
        assert!(source.contains("pub fn user_preferences_update"));
        assert!(source.contains("expected_revision: u64"));
    }
}
