use crate::{
    characters::service::CharacterService,
    domain::character::{
        Character, CharacterListFilter, CharacterState, CreateCharacterInput, UpdateCharacterInput,
        UpdateCharacterStateInput,
    },
    domain::project::{CreateProjectInput, Project, ProjectListFilter, UpdateProjectInput},
    error::AppResult,
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
}

#[cfg(feature = "tauri-app")]
pub use tauri_commands::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        db,
        domain::project::{ProjectStatus, UpdateProjectInput},
        error::AppError,
        projects::{repository::ProjectRepository, service::ProjectService},
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
}
