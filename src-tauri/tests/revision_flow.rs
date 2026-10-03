use serde_json::Value;
use webnovel_ai_studio_lib::{
    characters::{repository::CharacterRepository, service::CharacterService},
    db,
    domain::{
        character::{CreateCharacterInput, UpdateCharacterInput, UpdateCharacterStateInput},
        project::CreateProjectInput,
        project_memory::{CreateStoryFactInput, UpdateStoryFactInput},
        revision::{CanonStatus, MemoryEntityType, RevisionOperation},
    },
    error::AppError,
    project_memory::{repository::ProjectMemoryRepository, service::ProjectMemoryService},
    projects::{repository::ProjectRepository, service::ProjectService},
    revisions::{repository::RevisionRepository, service::RevisionService},
};

struct Services {
    projects: ProjectService,
    characters: CharacterService,
    memory: ProjectMemoryService,
    revisions: RevisionService,
}

fn services() -> Services {
    let connection = db::in_memory().expect("in-memory database should initialize");
    let projects_repository = ProjectRepository::new(connection.clone());
    let characters_repository = CharacterRepository::new(connection.clone());
    Services {
        projects: ProjectService::new(projects_repository.clone()),
        characters: CharacterService::new(characters_repository.clone()),
        memory: ProjectMemoryService::new(
            ProjectMemoryRepository::new(connection.clone()),
            projects_repository,
        ),
        revisions: RevisionService::new(
            RevisionRepository::new(connection.clone()),
            characters_repository,
            ProjectMemoryRepository::new(connection),
        ),
    }
}

fn character(services: &Services) -> webnovel_ai_studio_lib::domain::character::Character {
    let project = services
        .projects
        .create(CreateProjectInput {
            name: "Revision story".to_string(),
            description: None,
        })
        .unwrap();
    services
        .characters
        .create(
            project.id,
            CreateCharacterInput {
                name: "Mira".to_string(),
                summary: Some("A courier".to_string()),
                role: Some("protagonist".to_string()),
            },
        )
        .unwrap()
}

#[test]
fn character_create_and_update_append_ordered_revisions() {
    let services = services();
    let created = character(&services);
    assert_eq!(created.revision, 1);

    let updated = services
        .characters
        .update_with_revision(
            &created.id,
            UpdateCharacterInput {
                name: "Mira Vale".to_string(),
                summary: Some("A guarded courier".to_string()),
                role: Some("protagonist".to_string()),
            },
            created.revision,
        )
        .unwrap();
    assert_eq!(updated.revision, 2);

    let history = services
        .revisions
        .history(MemoryEntityType::Character, &created.id)
        .unwrap();
    assert_eq!(history.len(), 2);
    assert_eq!(history[0].revision, 1);
    assert_eq!(history[0].base_revision, 0);
    assert_eq!(history[0].operation, RevisionOperation::Create);
    assert_eq!(history[1].revision, 2);
    assert_eq!(history[1].base_revision, 1);
    assert_eq!(history[1].operation, RevisionOperation::Update);
    assert_eq!(history[1].previous_value.as_ref().unwrap()["name"], "Mira");
    assert_eq!(history[1].new_value["name"], "Mira Vale");
}

#[test]
fn stale_character_update_does_not_write_revision() {
    let services = services();
    let created = character(&services);
    let current = services
        .characters
        .update_with_revision(
            &created.id,
            UpdateCharacterInput {
                name: "Current".to_string(),
                summary: None,
                role: None,
            },
            1,
        )
        .unwrap();

    let error = services
        .characters
        .update_with_revision(
            &created.id,
            UpdateCharacterInput {
                name: "Stale write".to_string(),
                summary: None,
                role: None,
            },
            1,
        )
        .unwrap_err();
    assert_eq!(error, AppError::Conflict);
    assert_eq!(services.characters.get(&created.id).unwrap(), current);
    assert_eq!(
        services
            .revisions
            .history(MemoryEntityType::Character, &created.id)
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn state_upsert_creates_first_revision_and_preserves_fields() {
    let services = services();
    let created = character(&services);
    let first = services
        .characters
        .update_state_with_revision(
            &created.id,
            UpdateCharacterStateInput {
                current_location: Some("North gate".to_string()),
                emotional_state: Some("Guarded".to_string()),
                ..Default::default()
            },
            0,
        )
        .unwrap();
    assert_eq!(first.revision, 1);
    let second = services
        .characters
        .update_state_with_revision(
            &created.id,
            UpdateCharacterStateInput {
                goals: Some("Find the key".to_string()),
                ..Default::default()
            },
            first.revision,
        )
        .unwrap();
    assert_eq!(second.revision, 2);
    assert_eq!(second.current_location, "North gate");
    assert_eq!(second.emotional_state, "Guarded");

    let history = services
        .revisions
        .history(MemoryEntityType::CharacterState, &created.id)
        .unwrap();
    assert_eq!(history.len(), 2);
    assert_eq!(history[0].operation, RevisionOperation::Create);
    assert_eq!(history[0].base_revision, 0);
    assert_eq!(
        history[1].previous_value.as_ref().unwrap()["current_location"],
        "North gate"
    );
}

#[test]
fn locked_character_rejects_mutation() {
    let services = services();
    let created = character(&services);
    let locked = services
        .revisions
        .set_canon_status(
            MemoryEntityType::Character,
            &created.id,
            CanonStatus::LockedCanon,
            created.revision,
        )
        .unwrap();
    assert_eq!(locked.revision, 2);
    let error = services
        .characters
        .update_with_revision(
            &created.id,
            UpdateCharacterInput {
                name: "Blocked".to_string(),
                summary: None,
                role: None,
            },
            locked.revision,
        )
        .unwrap_err();
    assert_eq!(error, AppError::LockedCanon);
}

#[test]
fn restore_creates_new_revision_without_mutating_history() {
    let services = services();
    let created = character(&services);
    let updated = services
        .characters
        .update_with_revision(
            &created.id,
            UpdateCharacterInput {
                name: "Mira Vale".to_string(),
                summary: None,
                role: None,
            },
            created.revision,
        )
        .unwrap();
    let restored = services
        .revisions
        .restore(
            MemoryEntityType::Character,
            &created.id,
            1,
            updated.revision,
        )
        .unwrap();
    assert_eq!(restored.revision, 3);
    assert_eq!(restored.operation, RevisionOperation::Restore);
    assert_eq!(restored.source_type.as_deref(), Some("restore"));
    assert_eq!(
        restored.new_value["name"],
        Value::String("Mira".to_string())
    );
    assert_eq!(services.characters.get(&created.id).unwrap().name, "Mira");
    let history = services
        .revisions
        .history(MemoryEntityType::Character, &created.id)
        .unwrap();
    assert_eq!(history.len(), 3);
    assert_eq!(history[1].new_value["name"], "Mira Vale");
}

#[test]
fn archived_character_rejects_restore_and_canon_status_changes() {
    let services = services();
    let created = character(&services);
    let archived = services
        .characters
        .archive_with_revision(&created.id, created.revision)
        .unwrap();

    assert_eq!(
        services.revisions.restore(
            MemoryEntityType::Character,
            &created.id,
            created.revision,
            archived.revision,
        ),
        Err(AppError::ArchivedCharacter)
    );
    assert_eq!(
        services.revisions.set_canon_status(
            MemoryEntityType::Character,
            &created.id,
            CanonStatus::Canon,
            archived.revision,
        ),
        Err(AppError::ArchivedCharacter)
    );
}

#[test]
fn project_memory_restore_and_canon_status_are_forward_revision_safe() {
    let services = services();
    let project = services
        .projects
        .create(CreateProjectInput {
            name: "Memory revision story".to_string(),
            description: None,
        })
        .unwrap();
    let created = services
        .memory
        .create_story_fact(
            project.id,
            CreateStoryFactInput {
                title: "Gate".into(),
                content: "Opens at dawn.".into(),
            },
        )
        .unwrap();
    let updated = services
        .memory
        .update_story_fact(
            &created.id,
            UpdateStoryFactInput {
                title: "Gate".into(),
                content: "Opens at dusk.".into(),
            },
            created.revision,
        )
        .unwrap();

    let restored = services
        .revisions
        .restore(
            MemoryEntityType::StoryFact,
            &created.id,
            created.revision,
            updated.revision,
        )
        .unwrap();
    assert_eq!(restored.revision, 3);
    assert_eq!(restored.operation, RevisionOperation::Restore);
    assert_eq!(restored.new_value["content"], "Opens at dawn.");
    assert_eq!(
        services
            .memory
            .get_story_fact(&created.id)
            .unwrap()
            .revision,
        3
    );
    assert_eq!(
        services
            .revisions
            .history(MemoryEntityType::StoryFact, &created.id)
            .unwrap()
            .len(),
        3
    );

    let locked = services
        .revisions
        .set_canon_status(
            MemoryEntityType::StoryFact,
            &created.id,
            CanonStatus::LockedCanon,
            restored.revision,
        )
        .unwrap();
    assert_eq!(locked.revision, 4);
    assert_eq!(
        services
            .memory
            .update_story_fact(
                &created.id,
                UpdateStoryFactInput {
                    title: "Blocked".into(),
                    content: "Blocked".into(),
                },
                locked.revision,
            )
            .unwrap_err(),
        AppError::LockedCanon
    );
}
