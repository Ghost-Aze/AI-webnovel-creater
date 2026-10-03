use std::sync::Arc;

use futures::executor::block_on;
use webnovel_ai_studio_lib::{
    characters::{repository::CharacterRepository, service::CharacterService},
    context::{ContextBudget, ContextCompiler, ServiceContextSource},
    conversations::{
        repository::ConversationRepository,
        runtime::{ChatRuntimeLoadRequest, ChatRuntimeService, ChatSendRequest},
        service::ConversationService,
    },
    db,
    domain::{
        conversation::{ChatRuntimeSettingsInput, ConversationKind, MessageRole},
        manuscript::CreateChapterInput,
        project::CreateProjectInput,
    },
    error::AppError,
    manuscripts::{repository::ManuscriptRepository, service::ManuscriptService},
    project_memory::{repository::ProjectMemoryRepository, service::ProjectMemoryService},
    projects::{repository::ProjectRepository, service::ProjectService},
    provider::{
        MockProvider, ModelProfile, ModelTask, ModelTier, ProviderCapabilities, ProviderDescriptor,
        ProviderRegistry, QualityMode,
    },
};

struct Fixture {
    projects: ProjectService,
    manuscripts: ManuscriptService,
    conversations: ConversationService,
    registry: ProviderRegistry,
    runtime: ChatRuntimeService,
    project_id: String,
    conversation_id: String,
}

fn profile() -> ModelProfile {
    ModelProfile {
        provider_id: "mock".into(),
        model_id: "writer".into(),
        display_name: "Writer".into(),
        context_window_tokens: 8192,
        default_output_tokens: 512,
        strengths: vec!["writing".into(), "chat".into()],
        weaknesses: Vec::new(),
        strategy: Vec::new(),
        tier: ModelTier::Medium,
        capabilities: ProviderCapabilities::default(),
    }
}

fn fixture() -> Fixture {
    let connection = db::in_memory().unwrap();
    let project_repository = ProjectRepository::new(connection.clone());
    let projects = ProjectService::new(project_repository.clone());
    let characters = CharacterService::new(CharacterRepository::new(connection.clone()));
    let project_memory = ProjectMemoryService::new(
        ProjectMemoryRepository::new(connection.clone()),
        project_repository.clone(),
    );
    let conversations = ConversationService::new(
        ConversationRepository::new(connection.clone()),
        project_repository.clone(),
    );
    let manuscripts =
        ManuscriptService::new(ManuscriptRepository::new(connection), project_repository);
    let project = projects
        .create(CreateProjectInput {
            name: "Runtime project".into(),
            description: Some("A runtime fixture".into()),
        })
        .unwrap();
    let conversation = conversations
        .create(
            webnovel_ai_studio_lib::domain::conversation::CreateConversationInput {
                project_id: project.id.clone(),
                chapter_id: None,
                kind: ConversationKind::DeveloperChat,
                title: "Runtime chat".into(),
            },
        )
        .unwrap();
    let registry = ProviderRegistry::new();
    let source = ServiceContextSource::new(projects.clone(), characters, project_memory);
    let runtime = ChatRuntimeService::new(
        conversations.clone(),
        manuscripts.clone(),
        registry.clone(),
        ContextCompiler::default(),
        source,
    );
    Fixture {
        projects,
        manuscripts,
        conversations,
        registry,
        runtime,
        project_id: project.id,
        conversation_id: conversation.id,
    }
}

fn register_mock(registry: &ProviderRegistry) -> Arc<MockProvider> {
    let provider = Arc::new(MockProvider::new(
        ProviderDescriptor {
            id: "mock".into(),
            display_name: "Mock".into(),
        },
        vec![profile()],
    ));
    registry.register(provider.clone()).unwrap();
    provider
}

fn runtime_settings() -> ChatRuntimeSettingsInput {
    ChatRuntimeSettingsInput {
        assistant_id: "general-assistant".into(),
        provider_id: None,
        model_id: None,
        quality: QualityMode::Fast,
        temperature: None,
    }
}

fn request(conversation_id: &str, message: &str, retry_attempt: bool) -> ChatSendRequest {
    ChatSendRequest {
        conversation_id: conversation_id.into(),
        task: ModelTask::DeveloperChat,
        runtime: runtime_settings(),
        system_instructions: "Stay within canon.".into(),
        character_ids: Vec::new(),
        include_character_states: false,
        context_budget: ContextBudget::default(),
        message: message.into(),
        retry_attempt,
    }
}

#[test]
fn runtime_load_returns_defaults_catalog_and_empty_scope() {
    let fixture = fixture();
    let empty_project = fixture
        .projects
        .create(CreateProjectInput {
            name: "Empty runtime project".into(),
            description: None,
        })
        .unwrap();
    register_mock(&fixture.registry);

    let snapshot = fixture
        .runtime
        .load(ChatRuntimeLoadRequest {
            project_id: empty_project.id,
            chapter_id: None,
            conversation_id: None,
            kind: ConversationKind::DeveloperChat,
        })
        .unwrap();

    assert!(snapshot.conversation.is_none());
    assert!(snapshot.messages.is_empty());
    assert_eq!(snapshot.settings.assistant_id, "general-assistant");
    assert_eq!(snapshot.settings.quality, QualityMode::Balanced);
    assert_eq!(snapshot.models[0].model_id, "writer");
    assert_eq!(
        snapshot
            .assistants
            .iter()
            .map(|assistant| assistant.id.as_str())
            .collect::<Vec<_>>(),
        vec![
            "general-assistant",
            "world-builder",
            "continuity-reviewer",
            "writing-coach"
        ]
    );
}

#[test]
fn runtime_settings_reject_unknown_provider_and_model() {
    let fixture = fixture();
    register_mock(&fixture.registry);

    assert_eq!(
        fixture
            .runtime
            .update_settings(
                &fixture.conversation_id,
                ChatRuntimeSettingsInput {
                    provider_id: Some("missing".into()),
                    model_id: Some("writer".into()),
                    ..runtime_settings()
                },
            )
            .unwrap_err(),
        AppError::ProviderNotFound
    );
    assert_eq!(
        fixture
            .runtime
            .update_settings(
                &fixture.conversation_id,
                ChatRuntimeSettingsInput {
                    provider_id: Some("mock".into()),
                    model_id: Some("missing".into()),
                    ..runtime_settings()
                },
            )
            .unwrap_err(),
        AppError::ModelNotFound
    );
}

#[test]
fn runtime_send_persists_one_user_and_one_assistant_turn() {
    let fixture = fixture();
    register_mock(&fixture.registry);

    let result = block_on(fixture.runtime.send(request(
        &fixture.conversation_id,
        "Draft the next scene.",
        false,
    )))
    .unwrap();

    assert_eq!(result.user_message.role, MessageRole::User);
    assert_eq!(result.assistant_message.role, MessageRole::Assistant);
    assert_eq!(result.orchestration.steps.len(), 1);
    assert_eq!(
        fixture
            .conversations
            .list_messages(&fixture.conversation_id, None)
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn runtime_retry_reuses_failed_user_turn_without_duplicate_rows() {
    let fixture = fixture();
    let initial = request(&fixture.conversation_id, "Retry this request.", false);
    assert_eq!(
        block_on(fixture.runtime.send(initial)).unwrap_err(),
        AppError::NoSuitableModel
    );
    assert_eq!(
        fixture
            .conversations
            .list_messages(&fixture.conversation_id, None)
            .unwrap()
            .len(),
        1
    );

    register_mock(&fixture.registry);
    let result = block_on(fixture.runtime.send(request(
        &fixture.conversation_id,
        "Retry this request.",
        true,
    )))
    .unwrap();
    assert_eq!(result.user_message.sequence, 1);
    assert_eq!(
        fixture
            .conversations
            .list_messages(&fixture.conversation_id, None)
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn runtime_rejects_archived_project_and_chapter_before_provider_execution() {
    let project_fixture = fixture();
    let provider = register_mock(&project_fixture.registry);
    project_fixture
        .projects
        .archive(&project_fixture.project_id)
        .unwrap();
    assert_eq!(
        block_on(project_fixture.runtime.send(request(
            &project_fixture.conversation_id,
            "Do not run.",
            false,
        )))
        .unwrap_err(),
        AppError::ArchivedProject
    );
    assert_eq!(provider.generate_calls(), 0);

    let chapter_fixture = fixture();
    let chapter = chapter_fixture
        .manuscripts
        .create_chapter(
            chapter_fixture.project_id.clone(),
            CreateChapterInput {
                number: 1,
                title: "Opening".into(),
                synopsis: None,
            },
        )
        .unwrap();
    let conversation = chapter_fixture
        .conversations
        .create(
            webnovel_ai_studio_lib::domain::conversation::CreateConversationInput {
                project_id: chapter_fixture.project_id.clone(),
                chapter_id: Some(chapter.id.clone()),
                kind: ConversationKind::ChapterChat,
                title: "Chapter runtime".into(),
            },
        )
        .unwrap();
    chapter_fixture
        .manuscripts
        .archive_chapter(&chapter.id, chapter.revision)
        .unwrap();
    let provider = register_mock(&chapter_fixture.registry);
    assert_eq!(
        block_on(chapter_fixture.runtime.send(request(
            &conversation.id,
            "Do not run chapter.",
            false,
        )))
        .unwrap_err(),
        AppError::ArchivedChapter
    );
    assert_eq!(provider.generate_calls(), 0);
}
