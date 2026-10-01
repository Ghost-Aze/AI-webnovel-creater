use webnovel_ai_studio_lib::{
    characters::{repository::CharacterRepository, service::CharacterService},
    context::{
        ContextBudget, ContextCompileRequest, ContextCompiler, ContextTask, ServiceContextSource,
    },
    db,
    domain::{
        character::{CreateCharacterInput, UpdateCharacterStateInput},
        project::CreateProjectInput,
    },
    error::AppError,
    projects::{repository::ProjectRepository, service::ProjectService},
    provider::{ModelProfile, ModelRef, ModelTier, ProviderCapabilities},
};

fn services() -> (ProjectService, CharacterService) {
    let connection = db::in_memory().expect("in-memory database should initialize");
    (
        ProjectService::new(ProjectRepository::new(connection.clone())),
        CharacterService::new(CharacterRepository::new(connection)),
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
        include_character_states: true,
        working_memory: Vec::new(),
        budget: ContextBudget::default(),
    }
}

#[test]
fn compiles_selected_character_and_state() {
    let (projects, characters) = services();
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

    let source = ServiceContextSource::new(projects, characters);
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
    let (projects, characters) = services();
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
    let source = ServiceContextSource::new(projects, characters);
    let result = ContextCompiler::default().compile(
        request(&first.id, vec![foreign.id]),
        &profile(),
        &source,
    );
    assert_eq!(result, Err(AppError::NotFound));
}

#[test]
fn does_not_load_unselected_characters() {
    let (projects, characters) = services();
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
    let source = ServiceContextSource::new(projects, characters);
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
    let (projects, characters) = services();
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
    let source = ServiceContextSource::new(projects, characters.clone());
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
