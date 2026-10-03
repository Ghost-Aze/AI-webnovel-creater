use serde_json::json;
use webnovel_ai_studio_lib::{
    characters::{repository::CharacterRepository, service::CharacterService},
    db,
    domain::{
        character::{CreateCharacterInput, UpdateCharacterInput},
        project::CreateProjectInput,
        revision::{ActorType, CanonStatus, MemoryEntityType, ProposalStatus, RevisionOperation},
    },
    error::AppError,
    project_memory::repository::ProjectMemoryRepository,
    projects::{repository::ProjectRepository, service::ProjectService},
    revisions::{
        repository::RevisionRepository,
        service::{ProposalService, RevisionService},
    },
};

struct Services {
    projects: ProjectService,
    characters: CharacterService,
    revisions: RevisionService,
    proposals: ProposalService,
}

fn services() -> Services {
    let connection = db::in_memory().expect("in-memory database should initialize");
    let character_repository = CharacterRepository::new(connection.clone());
    Services {
        projects: ProjectService::new(ProjectRepository::new(connection.clone())),
        characters: CharacterService::new(character_repository.clone()),
        revisions: RevisionService::new(
            RevisionRepository::new(connection.clone()),
            character_repository.clone(),
            ProjectMemoryRepository::new(connection.clone()),
        ),
        proposals: ProposalService::new(
            RevisionRepository::new(connection.clone()),
            character_repository,
        ),
    }
}

fn character(services: &Services) -> webnovel_ai_studio_lib::domain::character::Character {
    let project = services
        .projects
        .create(CreateProjectInput {
            name: "Proposal story".to_string(),
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

fn update_payload(name: &str) -> serde_json::Value {
    json!({"name": name, "summary": "A changed courier", "role": "protagonist"})
}

fn proposal_input(
    project_id: String,
    entity_id: String,
    payload: serde_json::Value,
    base_revision: u64,
) -> webnovel_ai_studio_lib::domain::revision::CreateProposalInput {
    webnovel_ai_studio_lib::domain::revision::CreateProposalInput {
        project_id,
        entity_type: MemoryEntityType::Character,
        entity_id: Some(entity_id),
        operation: RevisionOperation::Update,
        payload,
        base_revision,
        actor_type: ActorType::Ai,
        actor_id: Some("planner".to_string()),
    }
}

#[test]
fn proposal_create_is_non_canonical() {
    let services = services();
    let character = character(&services);
    let proposal = services
        .proposals
        .create(proposal_input(
            character.project_id.clone(),
            character.id.clone(),
            update_payload("Draft Mira"),
            character.revision,
        ))
        .unwrap();
    assert_eq!(proposal.status, ProposalStatus::Draft);
    assert_eq!(services.characters.get(&character.id).unwrap().name, "Mira");
    assert_eq!(
        services
            .proposals
            .list(&character.project_id, None)
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn proposal_promotion_checks_base_revision() {
    let services = services();
    let character = character(&services);
    let proposal = services
        .proposals
        .create(proposal_input(
            character.project_id.clone(),
            character.id.clone(),
            update_payload("Stale proposal"),
            character.revision,
        ))
        .unwrap();
    services
        .characters
        .update_with_revision(
            &character.id,
            UpdateCharacterInput {
                name: "Current".to_string(),
                summary: Some("Current summary".to_string()),
                role: Some("protagonist".to_string()),
            },
            character.revision,
        )
        .unwrap();

    assert_eq!(
        services.proposals.promote(&proposal.id, 2).unwrap_err(),
        AppError::Conflict
    );
    assert_eq!(
        services.characters.get(&character.id).unwrap().name,
        "Current"
    );
    assert_eq!(
        services.proposals.get(&proposal.id).unwrap().status,
        ProposalStatus::Draft
    );
}

#[test]
fn proposal_reject_is_idempotent() {
    let services = services();
    let character = character(&services);
    let proposal = services
        .proposals
        .create(proposal_input(
            character.project_id.clone(),
            character.id.clone(),
            update_payload("Rejected"),
            character.revision,
        ))
        .unwrap();
    let rejected = services.proposals.reject(&proposal.id).unwrap();
    assert_eq!(rejected.status, ProposalStatus::Rejected);
    assert_eq!(services.proposals.reject(&proposal.id).unwrap(), rejected);
    assert_eq!(
        services
            .proposals
            .promote(&proposal.id, character.revision)
            .unwrap_err(),
        AppError::InvalidProposal
    );
}

#[test]
fn locked_proposal_cannot_promote() {
    let services = services();
    let character = character(&services);
    let proposal = services
        .proposals
        .create(proposal_input(
            character.project_id.clone(),
            character.id.clone(),
            update_payload("Locked proposal"),
            character.revision,
        ))
        .unwrap();
    services
        .revisions
        .set_canon_status(
            MemoryEntityType::Character,
            &character.id,
            CanonStatus::LockedCanon,
            character.revision,
        )
        .unwrap();
    assert_eq!(
        services.proposals.promote(&proposal.id, 2).unwrap_err(),
        AppError::LockedCanon
    );
    assert_eq!(
        services.proposals.get(&proposal.id).unwrap().status,
        ProposalStatus::Draft
    );
}

#[test]
fn archived_project_rejects_proposal_promotion() {
    let services = services();
    let character = character(&services);
    let proposal = services
        .proposals
        .create(proposal_input(
            character.project_id.clone(),
            character.id.clone(),
            update_payload("Archived project proposal"),
            character.revision,
        ))
        .unwrap();
    services.projects.archive(&character.project_id).unwrap();

    assert_eq!(
        services.proposals.promote(&proposal.id, character.revision),
        Err(AppError::ArchivedProject)
    );
    assert_eq!(services.characters.get(&character.id).unwrap().name, "Mira");
    assert_eq!(
        services.proposals.get(&proposal.id).unwrap().status,
        ProposalStatus::Draft
    );
}
