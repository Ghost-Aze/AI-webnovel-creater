use crate::{
    characters::repository::CharacterRepository,
    domain::{
        character::{Character, CharacterState, UpdateCharacterInput},
        revision::{
            CanonStatus, CreateProposalInput, MemoryEntityType, MemoryProposal, MemoryRevision,
            ProposalStatus, RevisionOperation,
        },
    },
    error::{AppError, AppResult},
};

use super::repository::RevisionRepository;

#[derive(Clone)]
pub struct RevisionService {
    repository: RevisionRepository,
    characters: CharacterRepository,
}

#[derive(Clone)]
pub struct ProposalService {
    repository: RevisionRepository,
    characters: CharacterRepository,
}

impl ProposalService {
    pub fn new(repository: RevisionRepository, characters: CharacterRepository) -> Self {
        Self {
            repository,
            characters,
        }
    }

    pub fn create(&self, input: CreateProposalInput) -> AppResult<MemoryProposal> {
        self.characters.project_is_active(&input.project_id)?;
        if let Some(entity_id) = &input.entity_id {
            let character = self.characters.get(entity_id)?;
            if character.project_id != input.project_id {
                return Err(AppError::NotFound);
            }
        }
        if !input.payload.is_object() {
            return Err(AppError::InvalidProposal);
        }
        if input.operation != RevisionOperation::Update
            || input.entity_type != MemoryEntityType::Character
            || input.entity_id.is_none()
        {
            return Err(AppError::InvalidProposal);
        }
        serde_json::from_value::<UpdateCharacterInput>(input.payload.clone())
            .map_err(|_| AppError::InvalidProposal)?;
        self.repository.create_proposal(input)
    }

    pub fn get(&self, id: &str) -> AppResult<MemoryProposal> {
        self.repository.get_proposal(id)
    }

    pub fn list(
        &self,
        project_id: &str,
        status: Option<ProposalStatus>,
    ) -> AppResult<Vec<MemoryProposal>> {
        self.repository.list_proposals(project_id, status)
    }

    pub fn promote(&self, id: &str, expected_revision: u64) -> AppResult<MemoryRevision> {
        let proposal = self.repository.get_proposal(id)?;
        if proposal.status != ProposalStatus::Draft
            || proposal.entity_type != MemoryEntityType::Character
            || proposal.entity_id.is_empty()
            || proposal.operation != RevisionOperation::Update
        {
            return Err(AppError::InvalidProposal);
        }
        let current = self.characters.get(&proposal.entity_id)?;
        if current.canon_status == CanonStatus::LockedCanon {
            return Err(AppError::LockedCanon);
        }
        if proposal.base_revision != expected_revision {
            return Err(AppError::Conflict);
        }
        let input: UpdateCharacterInput = serde_json::from_value(proposal.payload.clone())
            .map_err(|_| AppError::InvalidProposal)?;
        self.characters.promote_character_update(
            &proposal.entity_id,
            &input.name,
            input.summary.as_deref().unwrap_or_default(),
            input.role.as_deref().unwrap_or_default(),
            expected_revision,
            &proposal.id,
        )
    }

    pub fn reject(&self, id: &str) -> AppResult<MemoryProposal> {
        let proposal = self.repository.get_proposal(id)?;
        match proposal.status {
            ProposalStatus::Rejected => Ok(proposal),
            ProposalStatus::Accepted => Err(AppError::InvalidProposal),
            ProposalStatus::Draft => self
                .repository
                .set_proposal_status(id, ProposalStatus::Rejected),
        }
    }
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
