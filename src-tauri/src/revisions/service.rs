use crate::{
    characters::repository::CharacterRepository,
    domain::{
        character::{Character, CharacterState},
        revision::{CanonStatus, MemoryEntityType, MemoryRevision},
    },
    error::{AppError, AppResult},
};

use super::repository::RevisionRepository;

#[derive(Clone)]
pub struct RevisionService {
    repository: RevisionRepository,
    characters: CharacterRepository,
}

impl RevisionService {
    pub fn new(repository: RevisionRepository, characters: CharacterRepository) -> Self {
        Self {
            repository,
            characters,
        }
    }

    pub fn history(
        &self,
        entity_type: MemoryEntityType,
        entity_id: &str,
    ) -> AppResult<Vec<MemoryRevision>> {
        self.repository.list(entity_type, entity_id)
    }

    pub fn restore(
        &self,
        entity_type: MemoryEntityType,
        entity_id: &str,
        target_revision: u64,
        expected_revision: u64,
    ) -> AppResult<MemoryRevision> {
        let target = self
            .repository
            .get(entity_type, entity_id, target_revision)?;
        match entity_type {
            MemoryEntityType::Character => {
                let snapshot: Character = serde_json::from_value(target.new_value.clone())
                    .map_err(|_| AppError::InvalidProposal)?;
                self.characters
                    .restore_character(&snapshot, expected_revision, target_revision)
            }
            MemoryEntityType::CharacterState => {
                let snapshot: CharacterState = serde_json::from_value(target.new_value.clone())
                    .map_err(|_| AppError::InvalidProposal)?;
                self.characters
                    .restore_state(&snapshot, expected_revision, target_revision)
            }
        }
    }

    pub fn set_canon_status(
        &self,
        entity_type: MemoryEntityType,
        entity_id: &str,
        status: CanonStatus,
        expected_revision: u64,
    ) -> AppResult<MemoryRevision> {
        self.characters
            .set_canon_status(entity_type, entity_id, status, expected_revision)
    }
}
