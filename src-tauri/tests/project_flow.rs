use std::thread::sleep;
use std::time::Duration;

use webnovel_ai_studio_lib::{
    db,
    domain::project::{CreateProjectInput, ProjectListFilter, ProjectStatus, UpdateProjectInput},
    error::AppError,
    projects::{repository::ProjectRepository, service::ProjectService},
};

fn service() -> ProjectService {
    let connection = db::in_memory().expect("in-memory database should initialize");
    ProjectService::new(ProjectRepository::new(connection))
}

#[test]
fn migration_is_idempotent_and_preserves_data() {
    let connection = db::in_memory().expect("database should initialize");
    let project_service = ProjectService::new(ProjectRepository::new(connection.clone()));
    let created = project_service
        .create(CreateProjectInput {
            name: "Persistent project".to_string(),
            description: None,
        })
        .unwrap();

    db::run_migrations(&connection.lock().unwrap()).unwrap();
    let loaded = project_service.get(&created.id).unwrap();
    assert_eq!(loaded, created);
    assert_eq!(
        connection
            .lock()
            .unwrap()
            .query_row::<i64, _, _>("SELECT COUNT(*) FROM _migrations", [], |row| row.get(0),)
            .unwrap(),
        1
    );
}

#[test]
fn project_lifecycle_supports_create_get_update_and_archive() {
    let project_service = service();
    let created = project_service
        .create(CreateProjectInput {
            name: "  The Long Night  ".to_string(),
            description: None,
        })
        .unwrap();
    assert_eq!(created.name, "The Long Night");
    assert_eq!(created.description, "");
    assert_eq!(created.status, ProjectStatus::Active);

    sleep(Duration::from_millis(2));
    let updated = project_service
        .update(
            &created.id,
            UpdateProjectInput {
                name: "The Longer Night".to_string(),
                description: Some("A city waits for dawn.".to_string()),
            },
        )
        .unwrap();
    assert_eq!(updated.name, "The Longer Night");
    assert_eq!(updated.description, "A city waits for dawn.");
    assert_ne!(updated.updated_at, created.updated_at);

    let unchanged = project_service
        .update(
            &created.id,
            UpdateProjectInput {
                name: updated.name.clone(),
                description: Some(updated.description.clone()),
            },
        )
        .unwrap();
    assert_eq!(unchanged.name, updated.name);

    let archived = project_service.archive(&created.id).unwrap();
    assert_eq!(archived.status, ProjectStatus::Archived);
    assert_eq!(project_service.get(&created.id).unwrap(), archived);
    assert_eq!(project_service.archive(&created.id).unwrap(), archived);
}

#[test]
fn active_listing_hides_archived_projects_and_archived_updates_are_rejected() {
    let project_service = service();
    let active = project_service
        .create(CreateProjectInput {
            name: "Active".to_string(),
            description: None,
        })
        .unwrap();
    let archived = project_service
        .create(CreateProjectInput {
            name: "Archived".to_string(),
            description: None,
        })
        .unwrap();
    project_service.archive(&archived.id).unwrap();

    let active_projects = project_service.list(ProjectListFilter::default()).unwrap();
    assert_eq!(
        active_projects.iter().map(|p| &p.id).collect::<Vec<_>>(),
        vec![&active.id]
    );

    let all_projects = project_service
        .list(ProjectListFilter {
            include_archived: true,
        })
        .unwrap();
    assert_eq!(all_projects.len(), 2);
    assert!(matches!(
        project_service.update(
            &archived.id,
            UpdateProjectInput {
                name: "Cannot change".to_string(),
                description: None,
            },
        ),
        Err(AppError::ArchivedProject)
    ));
}

#[test]
fn invalid_and_missing_projects_return_safe_errors() {
    let project_service = service();
    assert!(matches!(
        project_service.create(CreateProjectInput {
            name: "  ".to_string(),
            description: None,
        }),
        Err(AppError::Validation { .. })
    ));
    assert_eq!(project_service.get("missing-id"), Err(AppError::NotFound));
}
