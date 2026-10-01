use crate::{
    domain::project::{CreateProjectInput, Project, ProjectListFilter, UpdateProjectInput},
    error::AppResult,
};

#[cfg(feature = "tauri-app")]
use crate::AppState;

pub fn create_project(
    service: &crate::projects::service::ProjectService,
    input: CreateProjectInput,
) -> AppResult<Project> {
    service.create(input)
}

pub fn list_projects(
    service: &crate::projects::service::ProjectService,
    filter: ProjectListFilter,
) -> AppResult<Vec<Project>> {
    service.list(filter)
}

pub fn get_project(
    service: &crate::projects::service::ProjectService,
    id: String,
) -> AppResult<Project> {
    service.get(&id)
}

pub fn update_project(
    service: &crate::projects::service::ProjectService,
    id: String,
    input: UpdateProjectInput,
) -> AppResult<Project> {
    service.update(&id, input)
}

pub fn archive_project(
    service: &crate::projects::service::ProjectService,
    id: String,
) -> AppResult<Project> {
    service.archive(&id)
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
