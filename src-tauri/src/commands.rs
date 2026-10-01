use crate::{
    characters::service::CharacterService,
    domain::character::{
        Character, CharacterListFilter, CharacterState, CreateCharacterInput, UpdateCharacterInput,
        UpdateCharacterStateInput,
    },
    domain::project::{CreateProjectInput, Project, ProjectListFilter, UpdateProjectInput},
    domain::revision::{
        CanonStatus, CreateProposalInput, MemoryEntityType, MemoryProposal, MemoryRevision,
        ProposalStatus,
    },
    error::AppResult,
    revisions::service::ProposalService,
    revisions::service::RevisionService,
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

pub fn archive_character(service: &CharacterService, id: String) -> AppResult<Character> {
    report("character_archive", service.archive(&id))
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
    ) -> AppResult<Character> {
        update_character(&state.character_service, id, input)
    }

    #[tauri::command]
    pub fn character_archive(state: State<'_, AppState>, id: String) -> AppResult<Character> {
        archive_character(&state.character_service, id)
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
    ) -> AppResult<CharacterState> {
        update_character_state(&state.character_service, character_id, input)
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
}

#[cfg(feature = "tauri-app")]
pub use tauri_commands::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        db,
        domain::{
            character::CreateCharacterInput,
            project::{ProjectStatus, UpdateProjectInput},
        },
        error::AppError,
        projects::{repository::ProjectRepository, service::ProjectService},
        revisions::{repository::RevisionRepository, service::RevisionService},
    };

    fn service() -> ProjectService {
        ProjectService::new(ProjectRepository::new(db::in_memory().unwrap()))
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
    }
}
