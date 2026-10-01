use crate::{
    domain::project::{
        new_id, normalize_description, normalize_name, now_utc, CreateProjectInput, Project,
        ProjectListFilter, ProjectStatus, UpdateProjectInput,
    },
    error::AppResult,
};

use super::repository::ProjectRepository;

#[derive(Clone)]
pub struct ProjectService {
    repository: ProjectRepository,
}

impl ProjectService {
    pub fn new(repository: ProjectRepository) -> Self {
        Self { repository }
    }

    pub fn create(&self, input: CreateProjectInput) -> AppResult<Project> {
        let timestamp = now_utc();
        let project = Project {
            id: new_id(),
            name: normalize_name(&input.name)?,
            description: normalize_description(input.description.as_deref()),
            status: ProjectStatus::Active,
            created_at: timestamp.clone(),
            updated_at: timestamp,
        };
        self.repository.create(&project)
    }

    pub fn get(&self, id: &str) -> AppResult<Project> {
        self.repository.get(id)
    }

    pub fn list(&self, filter: ProjectListFilter) -> AppResult<Vec<Project>> {
        self.repository.list(filter)
    }

    pub fn update(&self, id: &str, input: UpdateProjectInput) -> AppResult<Project> {
        let name = normalize_name(&input.name)?;
        let description = normalize_description(input.description.as_deref());
        self.repository.update(id, &name, &description, &now_utc())
    }

    pub fn archive(&self, id: &str) -> AppResult<Project> {
        self.repository.archive(id, &now_utc())
    }
}
