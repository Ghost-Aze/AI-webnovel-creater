use webnovel_ai_studio_lib::{
    characters::{repository::CharacterRepository, service::CharacterService},
    context::{
        ContextBlockKind, ContextBudget, ContextCompileRequest, ContextCompiler, ContextTask,
        ProjectMemoryRef, ServiceContextSource,
    },
    db,
    domain::{
        character::{CreateCharacterInput, UpdateCharacterStateInput},
        project::CreateProjectInput,
        project_memory::{CreateCanonRuleInput, CreateStoryFactInput},
        revision::MemoryEntityType,
    },
    error::AppError,
    project_memory::{repository::ProjectMemoryRepository, service::ProjectMemoryService},
    projects::{repository::ProjectRepository, service::ProjectService},
    provider::{ModelProfile, ModelRef, ModelTier, ProviderCapabilities},
};

fn services() -> (ProjectService, CharacterService, ProjectMemoryService) {
    let connection = db::in_memory().expect("in-memory database should initialize");
    (
        ProjectService::new(ProjectRepository::new(connection.clone())),
        CharacterService::new(CharacterRepository::new(connection.clone())),
        ProjectMemoryService::new(
            ProjectMemoryRepository::new(connection.clone()),
            ProjectRepository::new(connection),
        ),
    )
}

fn profile() -> ModelProfile {
    ModelProfile {
        provider_id: "mock".into(),
        model_id: "mock-small".into(),
        display_name: "Mock Small".into(),
        context_window_tokens: 4096,
        default_output_tokens: 512,
        strengths: Vec::new(),
        weaknesses: Vec::new(),
        strategy: Vec::new(),
        tier: ModelTier::Medium,
        capabilities: ProviderCapabilities::default(),
    }
}

fn request(project_id: &str, character_ids: Vec<String>) -> ContextCompileRequest {
    ContextCompileRequest {
        project_id: project_id.into(),
        task: ContextTask::Writing,
        model: ModelRef {
            provider_id: "mock".into(),
            model_id: "mock-small".into(),
        },
        system_instructions: "Write the scene".into(),
        character_ids,
        project_memory_refs: Vec::new(),
        include_character_states: true,
        working_memory: Vec::new(),
        budget: ContextBudget::default(),
    }
}

#[test]
fn compiles_selected_character_and_state() {
    let (projects, characters, memory) = services();
    let project = projects
        .create(CreateProjectInput {
            name: "Story".into(),
            description: Some("A premise".into()),
        })
        .unwrap();
    let character = characters
        .create(
            project.id.clone(),
            CreateCharacterInput {
                name: "Ji-an".into(),
                summary: Some("Courier".into()),
                role: Some("Lead".into()),
            },
        )
        .unwrap();
    characters
        .update_state(
            &character.id,
            UpdateCharacterStateInput {
                current_location: Some("North gate".into()),
                ..Default::default()
            },
        )
        .unwrap();

    let source = ServiceContextSource::new(projects, characters, memory);
    let result = ContextCompiler::default()
        .compile(
            request(&project.id, vec![character.id.clone()]),
            &profile(),
            &source,
        )
        .unwrap();
    assert!(result
        .blocks
        .iter()
        .any(|block| block.content.contains("Ji-an")));
    assert!(result
        .blocks
        .iter()
        .any(|block| block.content.contains("North gate")));
}

#[test]
fn rejects_character_from_another_project() {
    let (projects, characters, memory) = services();
    let first = projects
        .create(CreateProjectInput {
            name: "First".into(),
            description: None,
        })
        .unwrap();
    let second = projects
        .create(CreateProjectInput {
            name: "Second".into(),
            description: None,
        })
        .unwrap();
    let foreign = characters
        .create(
            second.id,
            CreateCharacterInput {
                name: "Foreign".into(),
                summary: None,
                role: None,
            },
        )
        .unwrap();
    let source = ServiceContextSource::new(projects, characters, memory);
    let result = ContextCompiler::default().compile(
        request(&first.id, vec![foreign.id]),
        &profile(),
        &source,
    );
    assert_eq!(result, Err(AppError::NotFound));
}

#[test]
fn does_not_load_unselected_characters() {
    let (projects, characters, memory) = services();
    let project = projects
        .create(CreateProjectInput {
            name: "Story".into(),
            description: None,
        })
        .unwrap();
    let selected = characters
        .create(
            project.id.clone(),
            CreateCharacterInput {
                name: "Selected".into(),
                summary: None,
                role: None,
            },
        )
        .unwrap();
    let unselected = characters
        .create(
            project.id.clone(),
            CreateCharacterInput {
                name: "Unselected".into(),
                summary: None,
                role: None,
            },
        )
        .unwrap();
    let source = ServiceContextSource::new(projects, characters, memory);
    let result = ContextCompiler::default()
        .compile(request(&project.id, vec![selected.id]), &profile(), &source)
        .unwrap();
    let content = result
        .blocks
        .iter()
        .map(|block| block.content.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(content.contains("Selected"));
    assert!(!content.contains(&unselected.id));
    assert!(!content.contains("Unselected"));
}

#[test]
fn does_not_persist_or_mutate_memory() {
    let (projects, characters, memory) = services();
    let project = projects
        .create(CreateProjectInput {
            name: "Story".into(),
            description: None,
        })
        .unwrap();
    let character = characters
        .create(
            project.id.clone(),
            CreateCharacterInput {
                name: "Stable".into(),
                summary: Some("Before".into()),
                role: None,
            },
        )
        .unwrap();
    let before = characters.get(&character.id).unwrap();
    let source = ServiceContextSource::new(projects, characters.clone(), memory);
    ContextCompiler::default()
        .compile(
            request(&project.id, vec![character.id.clone()]),
            &profile(),
            &source,
        )
        .unwrap();
    assert_eq!(characters.get(&character.id).unwrap(), before);
    assert_eq!(characters.get_state(&character.id).unwrap().revision, 0);
}

#[test]
fn compiles_selected_project_memory_in_request_order_and_rejects_foreign_refs() {
    let (projects, characters, memory) = services();
    let project = projects
        .create(CreateProjectInput {
            name: "Story".into(),
            description: None,
        })
        .unwrap();
    let fact = memory
        .create_story_fact(
            project.id.clone(),
            CreateStoryFactInput {
                title: "Dawn".into(),
                content: "The bells ring.".into(),
            },
        )
        .unwrap();
    let rule = memory
        .create_canon_rule(
            project.id.clone(),
            CreateCanonRuleInput {
                title: "Time".into(),
                rule: "Days pass normally.".into(),
                scope: "World".into(),
            },
        )
        .unwrap();

    let source = ServiceContextSource::new(projects.clone(), characters, memory.clone());
    let mut input = request(&project.id, Vec::new());
    input.project_memory_refs = vec![
        ProjectMemoryRef {
            entity_type: MemoryEntityType::CanonRule,
            entity_id: rule.id.clone(),
        },
        ProjectMemoryRef {
            entity_type: MemoryEntityType::StoryFact,
            entity_id: fact.id.clone(),
        },
    ];
    let result = ContextCompiler::default()
        .compile(input, &profile(), &source)
        .unwrap();
    let project_memory_blocks = result
        .blocks
        .iter()
        .filter(|block| {
            matches!(
                block.kind,
                ContextBlockKind::CanonRule | ContextBlockKind::StoryFact
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(project_memory_blocks.len(), 2);
    assert_eq!(
        project_memory_blocks[0].source_id.as_deref(),
        Some(rule.id.as_str())
    );
    assert_eq!(
        project_memory_blocks[1].source_id.as_deref(),
        Some(fact.id.as_str())
    );
    assert!(project_memory_blocks[0]
        .content
        .contains("Days pass normally."));
    assert!(project_memory_blocks[1].content.contains("The bells ring."));

    let foreign = projects
        .create(CreateProjectInput {
            name: "Foreign".into(),
            description: None,
        })
        .unwrap();
    let foreign_fact = memory
        .create_story_fact(
            foreign.id,
            CreateStoryFactInput {
                title: "Foreign fact".into(),
                content: "Should not load.".into(),
            },
        )
        .unwrap();
    let mut foreign_request = request(&project.id, Vec::new());
    foreign_request.project_memory_refs = vec![ProjectMemoryRef {
        entity_type: MemoryEntityType::StoryFact,
        entity_id: foreign_fact.id,
    }];
    assert_eq!(
        ContextCompiler::default().compile(foreign_request, &profile(), &source),
        Err(AppError::NotFound)
    );
}
