use crate::{
    characters::service::CharacterService,
    context::compiler::ContextSource,
    domain::{
        character::{Character, CharacterState},
        project::Project,
        project_memory::{CanonRule, StoryFact},
    },
    error::{AppError, AppResult},
    project_memory::service::ProjectMemoryService,
    projects::service::ProjectService,
};

#[derive(Clone)]
pub struct ServiceContextSource {
    projects: ProjectService,
    characters: CharacterService,
    project_memory: ProjectMemoryService,
}

impl ServiceContextSource {
    pub fn new(
        projects: ProjectService,
        characters: CharacterService,
        project_memory: ProjectMemoryService,
    ) -> Self {
        Self {
            projects,
            characters,
            project_memory,
        }
    }
}

impl ContextSource for ServiceContextSource {
    fn load_project(&self, project_id: &str) -> AppResult<Project> {
        self.projects.get(project_id)
    }

    fn load_character(&self, project_id: &str, character_id: &str) -> AppResult<Character> {
        let character = self.characters.get(character_id)?;
        if character.project_id != project_id {
            return Err(AppError::NotFound);
        }
        Ok(character)
    }

    fn load_character_state(&self, character_id: &str) -> AppResult<CharacterState> {
        self.characters.get_state(character_id)
    }

    fn load_story_fact(&self, project_id: &str, entity_id: &str) -> AppResult<StoryFact> {
        let fact = self.project_memory.get_story_fact(entity_id)?;
        if fact.project_id != project_id {
            return Err(AppError::NotFound);
        }
        Ok(fact)
    }

    fn load_canon_rule(&self, project_id: &str, entity_id: &str) -> AppResult<CanonRule> {
        let rule = self.project_memory.get_canon_rule(entity_id)?;
        if rule.project_id != project_id {
            return Err(AppError::NotFound);
        }
        Ok(rule)
    }
}
