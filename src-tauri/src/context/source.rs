use crate::{
    characters::service::CharacterService,
    context::compiler::ContextSource,
    domain::{
        character::{Character, CharacterState},
        project::Project,
    },
    error::{AppError, AppResult},
    projects::service::ProjectService,
};

#[derive(Clone)]
pub struct ServiceContextSource {
    projects: ProjectService,
    characters: CharacterService,
}

impl ServiceContextSource {
    pub fn new(projects: ProjectService, characters: CharacterService) -> Self {
        Self {
            projects,
            characters,
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
}
