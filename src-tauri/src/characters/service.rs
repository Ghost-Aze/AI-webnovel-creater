use crate::{
    domain::character::{
        build_character, update_character_fields, Character, CharacterListFilter, CharacterState,
        CreateCharacterInput, UpdateCharacterInput, UpdateCharacterStateInput,
    },
    domain::project::now_utc,
    error::AppResult,
};

use super::repository::CharacterRepository;

#[derive(Clone)]
pub struct CharacterService {
    repository: CharacterRepository,
}

impl CharacterService {
    pub fn new(repository: CharacterRepository) -> Self {
        Self { repository }
    }

    pub fn create(&self, project_id: String, input: CreateCharacterInput) -> AppResult<Character> {
        self.repository.project_is_active(&project_id)?;
        self.repository.create(&build_character(project_id, input)?)
    }

    pub fn list(&self, project_id: &str, filter: CharacterListFilter) -> AppResult<Vec<Character>> {
        match self.repository.project_is_active(project_id) {
            Ok(()) | Err(crate::error::AppError::ArchivedProject) => {
                self.repository.list(project_id, filter)
            }
            Err(error) => Err(error),
        }
    }

    pub fn get(&self, id: &str) -> AppResult<Character> {
        self.repository.get(id)
    }

    pub fn update(&self, id: &str, input: UpdateCharacterInput) -> AppResult<Character> {
        let existing = self.repository.get(id)?;
        self.repository.project_is_active(&existing.project_id)?;
        let (name, summary, role) = update_character_fields(input)?;
        self.repository
            .update(id, &name, &summary, &role, &now_utc())
    }

    pub fn archive(&self, id: &str) -> AppResult<Character> {
        let existing = self.repository.get(id)?;
        self.repository.project_is_active(&existing.project_id)?;
        self.repository.archive(id, &now_utc())
    }

    pub fn get_state(&self, character_id: &str) -> AppResult<CharacterState> {
        let character = self.repository.get(character_id)?;
        Ok(self
            .repository
            .get_state(character_id)?
            .unwrap_or_else(|| CharacterState::empty(character.id)))
    }

    pub fn update_state(
        &self,
        character_id: &str,
        input: UpdateCharacterStateInput,
    ) -> AppResult<CharacterState> {
        let character = self.repository.get(character_id)?;
        self.repository.project_is_active(&character.project_id)?;
        if character.status == crate::domain::character::CharacterStatus::Archived {
            return Err(crate::error::AppError::ArchivedCharacter);
        }
        let mut state = self
            .repository
            .get_state(character_id)?
            .unwrap_or_else(|| CharacterState::empty(character_id.to_string()));
        state.apply(input);
        self.repository.upsert_state(&state)
    }
}
