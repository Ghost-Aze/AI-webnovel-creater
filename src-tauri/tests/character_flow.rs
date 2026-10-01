use webnovel_ai_studio_lib::{
    characters::{repository::CharacterRepository, service::CharacterService},
    db,
    domain::{
        character::{
            CharacterListFilter, CharacterStatus, CreateCharacterInput, UpdateCharacterInput,
            UpdateCharacterStateInput,
        },
        project::{CreateProjectInput, ProjectStatus},
    },
    error::AppError,
    projects::{repository::ProjectRepository, service::ProjectService},
};

fn services() -> (ProjectService, CharacterService) {
    let connection = db::in_memory().expect("in-memory database should initialize");
    let projects = ProjectService::new(ProjectRepository::new(connection.clone()));
    let characters = CharacterService::new(CharacterRepository::new(connection));
    (projects, characters)
}

#[test]
fn character_migration_is_idempotent_and_state_defaults_are_structured() {
    let (projects, characters) = services();
    let project = projects
        .create(CreateProjectInput {
            name: "Story".to_string(),
            description: None,
        })
        .unwrap();
    let character = characters
        .create(
            project.id.clone(),
            CreateCharacterInput {
                name: "  Ji-an  ".to_string(),
                summary: Some("  Courier  ".to_string()),
                role: Some("protagonist".to_string()),
            },
        )
        .unwrap();
    assert_eq!(character.name, "Ji-an");
    assert_eq!(character.status, CharacterStatus::Active);

    let state = characters.get_state(&character.id).unwrap();
    assert_eq!(state.character_id, character.id);
    assert_eq!(state.current_location, "");
    assert_eq!(state.goals, "");
}

#[test]
fn character_lifecycle_rejects_duplicates_and_archived_updates() {
    let (projects, characters) = services();
    let project = projects
        .create(CreateProjectInput {
            name: "Story".to_string(),
            description: None,
        })
        .unwrap();
    let created = characters
        .create(
            project.id.clone(),
            CreateCharacterInput {
                name: "Mira".to_string(),
                summary: None,
                role: None,
            },
        )
        .unwrap();
    assert!(matches!(
        characters.create(
            project.id.clone(),
            CreateCharacterInput {
                name: "mira".to_string(),
                summary: None,
                role: None,
            },
        ),
        Err(AppError::DuplicateName)
    ));

    let archived = characters.archive(&created.id).unwrap();
    assert_eq!(archived.status, CharacterStatus::Archived);
    assert_eq!(
        characters
            .list(&project.id, CharacterListFilter::default())
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        characters
            .list(
                &project.id,
                CharacterListFilter {
                    include_archived: true
                }
            )
            .unwrap()
            .len(),
        1
    );
    assert!(matches!(
        characters.update(
            &created.id,
            UpdateCharacterInput {
                name: "Cannot change".to_string(),
                summary: None,
                role: None,
            },
        ),
        Err(AppError::ArchivedCharacter)
    ));
}

#[test]
fn character_state_update_is_an_upsert_and_preserves_untouched_fields() {
    let (projects, characters) = services();
    let project = projects
        .create(CreateProjectInput {
            name: "Story".to_string(),
            description: None,
        })
        .unwrap();
    let character = characters
        .create(
            project.id,
            CreateCharacterInput {
                name: "Mira".to_string(),
                summary: None,
                role: None,
            },
        )
        .unwrap();

    let first = characters
        .update_state(
            &character.id,
            UpdateCharacterStateInput {
                current_location: Some("North gate".to_string()),
                emotional_state: Some("Guarded".to_string()),
                ..Default::default()
            },
        )
        .unwrap();
    let second = characters
        .update_state(
            &character.id,
            UpdateCharacterStateInput {
                goals: Some("Find the key".to_string()),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(first.current_location, "North gate");
    assert_eq!(second.current_location, "North gate");
    assert_eq!(second.emotional_state, "Guarded");
    assert_eq!(second.goals, "Find the key");
}

#[test]
fn archived_projects_reject_character_mutations_but_keep_records() {
    let (projects, characters) = services();
    let project = projects
        .create(CreateProjectInput {
            name: "Story".to_string(),
            description: None,
        })
        .unwrap();
    let character = characters
        .create(
            project.id.clone(),
            CreateCharacterInput {
                name: "Mira".to_string(),
                summary: None,
                role: None,
            },
        )
        .unwrap();
    let archived_project = projects.archive(&project.id).unwrap();
    assert_eq!(archived_project.status, ProjectStatus::Archived);
    assert!(matches!(
        characters.update_state(
            &character.id,
            UpdateCharacterStateInput {
                goals: Some("No change".to_string()),
                ..Default::default()
            },
        ),
        Err(AppError::ArchivedProject)
    ));
    assert_eq!(characters.get(&character.id).unwrap().name, "Mira");
}
