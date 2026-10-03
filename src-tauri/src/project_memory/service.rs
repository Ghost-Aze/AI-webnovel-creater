use crate::{
    domain::{
        project::ProjectStatus,
        project_memory::{
            build_canon_rule, build_story_fact, update_canon_rule_fields,
            update_story_fact_fields, CanonRule, CreateCanonRuleInput, CreateStoryFactInput,
            ProjectMemoryListFilter, StoryFact, UpdateCanonRuleInput, UpdateStoryFactInput,
        },
    },
    error::{AppError, AppResult},
    projects::repository::ProjectRepository,
};

use super::repository::ProjectMemoryRepository;

#[derive(Clone)]
pub struct ProjectMemoryService {
    repository: ProjectMemoryRepository,
    projects: ProjectRepository,
}

impl ProjectMemoryService {
    pub fn new(repository: ProjectMemoryRepository, projects: ProjectRepository) -> Self {
        Self {
            repository,
            projects,
        }
    }

    pub fn create_story_fact(
        &self,
        project_id: String,
        input: CreateStoryFactInput,
    ) -> AppResult<StoryFact> {
        self.ensure_project_active(&project_id)?;
        self.repository
            .create_story_fact(&build_story_fact(project_id, input)?)
    }

    pub fn list_story_facts(
        &self,
        project_id: &str,
        filter: ProjectMemoryListFilter,
    ) -> AppResult<Vec<StoryFact>> {
        self.projects.get(project_id)?;
        self.repository.list_story_facts(project_id, filter)
    }

    pub fn get_story_fact(&self, id: &str) -> AppResult<StoryFact> {
        self.repository.get_story_fact(id)
    }

    pub fn update_story_fact(
        &self,
        id: &str,
        input: UpdateStoryFactInput,
        expected_revision: u64,
    ) -> AppResult<StoryFact> {
        let existing = self.repository.get_story_fact(id)?;
        self.ensure_project_active(&existing.project_id)?;
        let (title, content) = update_story_fact_fields(input)?;
        self.repository.update_story_fact(
            id,
            &title,
            &content,
            &crate::domain::project::now_utc(),
            expected_revision,
        )
    }

    pub fn archive_story_fact(&self, id: &str, expected_revision: u64) -> AppResult<StoryFact> {
        let existing = self.repository.get_story_fact(id)?;
        self.ensure_project_active(&existing.project_id)?;
        self.repository.archive_story_fact(
            id,
            &crate::domain::project::now_utc(),
            expected_revision,
        )
    }

    pub fn create_canon_rule(
        &self,
        project_id: String,
        input: CreateCanonRuleInput,
    ) -> AppResult<CanonRule> {
        self.ensure_project_active(&project_id)?;
        self.repository
            .create_canon_rule(&build_canon_rule(project_id, input)?)
    }

    pub fn list_canon_rules(
        &self,
        project_id: &str,
        filter: ProjectMemoryListFilter,
    ) -> AppResult<Vec<CanonRule>> {
        self.projects.get(project_id)?;
        self.repository.list_canon_rules(project_id, filter)
    }

    pub fn get_canon_rule(&self, id: &str) -> AppResult<CanonRule> {
        self.repository.get_canon_rule(id)
    }

    pub fn update_canon_rule(
        &self,
        id: &str,
        input: UpdateCanonRuleInput,
        expected_revision: u64,
    ) -> AppResult<CanonRule> {
        let existing = self.repository.get_canon_rule(id)?;
        self.ensure_project_active(&existing.project_id)?;
        let (title, rule, scope) = update_canon_rule_fields(input)?;
        self.repository.update_canon_rule(
            id,
            &title,
            &rule,
            &scope,
            &crate::domain::project::now_utc(),
            expected_revision,
        )
    }

    pub fn archive_canon_rule(&self, id: &str, expected_revision: u64) -> AppResult<CanonRule> {
        let existing = self.repository.get_canon_rule(id)?;
        self.ensure_project_active(&existing.project_id)?;
        self.repository.archive_canon_rule(
            id,
            &crate::domain::project::now_utc(),
            expected_revision,
        )
    }

    fn ensure_project_active(&self, project_id: &str) -> AppResult<()> {
        let project = self.projects.get(project_id)?;
        if project.status == ProjectStatus::Archived {
            return Err(AppError::ArchivedProject);
        }
        Ok(())
    }
}
