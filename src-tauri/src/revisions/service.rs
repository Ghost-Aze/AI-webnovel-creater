use crate::{
    characters::repository::CharacterRepository,
    domain::{
        character::{Character, CharacterState, UpdateCharacterInput},
        project_memory::{
            CanonRule, CreateCanonRuleInput, CreateStoryFactInput, StoryFact, UpdateCanonRuleInput,
            UpdateStoryFactInput,
        },
        revision::{
            CanonStatus, CreateProposalInput, MemoryEntityType, MemoryProposal, MemoryRevision,
            ProposalStatus, RevisionOperation,
        },
    },
    error::{AppError, AppResult},
};

use super::repository::RevisionRepository;
use crate::project_memory::repository::ProjectMemoryRepository;

#[derive(Clone)]
pub struct RevisionService {
    repository: RevisionRepository,
    characters: CharacterRepository,
    project_memory: ProjectMemoryRepository,
}

#[derive(Clone)]
pub struct ProposalService {
    repository: RevisionRepository,
    characters: CharacterRepository,
    project_memory: ProjectMemoryRepository,
}

impl ProposalService {
    pub fn new(
        repository: RevisionRepository,
        characters: CharacterRepository,
        project_memory: ProjectMemoryRepository,
    ) -> Self {
        Self {
            repository,
            characters,
            project_memory,
        }
    }

    pub fn create(&self, input: CreateProposalInput) -> AppResult<MemoryProposal> {
        self.characters.project_is_active(&input.project_id)?;
        if !input.payload.is_object() {
            return Err(AppError::InvalidProposal);
        }
        match input.entity_type {
            MemoryEntityType::Character
                if input.operation == RevisionOperation::Update && input.entity_id.is_some() =>
            {
                let entity_id = input
                    .entity_id
                    .as_deref()
                    .ok_or(AppError::InvalidProposal)?;
                let character = self.characters.get(entity_id)?;
                if character.project_id != input.project_id {
                    return Err(AppError::NotFound);
                }
                serde_json::from_value::<UpdateCharacterInput>(input.payload.clone())
                    .map_err(|_| AppError::InvalidProposal)?;
            }
            MemoryEntityType::StoryFact => {
                validate_story_fact_proposal(&self.project_memory, &input)?;
            }
            MemoryEntityType::CanonRule => {
                validate_canon_rule_proposal(&self.project_memory, &input)?;
            }
            MemoryEntityType::CharacterState => return Err(AppError::InvalidProposal),
            _ => return Err(AppError::InvalidProposal),
        }
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
        if proposal.status != ProposalStatus::Draft {
            return Err(AppError::InvalidProposal);
        }
        if matches!(
            proposal.entity_type,
            MemoryEntityType::StoryFact | MemoryEntityType::CanonRule
        ) {
            return RevisionService::promote_project_memory_proposal(
                &self.project_memory,
                &proposal,
                expected_revision,
            );
        }
        if proposal.entity_type != MemoryEntityType::Character
            || proposal.operation != RevisionOperation::Update
        {
            return Err(AppError::InvalidProposal);
        }
        let entity_id = proposal
            .entity_id
            .as_deref()
            .ok_or(AppError::InvalidProposal)?;
        let current = self.characters.get(entity_id)?;
        if current.canon_status == CanonStatus::LockedCanon {
            return Err(AppError::LockedCanon);
        }
        if proposal.base_revision != expected_revision {
            return Err(AppError::Conflict);
        }
        let input: UpdateCharacterInput = serde_json::from_value(proposal.payload.clone())
            .map_err(|_| AppError::InvalidProposal)?;
        self.characters.promote_character_update(
            entity_id,
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
    pub fn new(
        repository: RevisionRepository,
        characters: CharacterRepository,
        project_memory: ProjectMemoryRepository,
    ) -> Self {
        Self {
            repository,
            characters,
            project_memory,
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
            MemoryEntityType::StoryFact | MemoryEntityType::CanonRule => match entity_type {
                MemoryEntityType::StoryFact => {
                    let snapshot: StoryFact = serde_json::from_value(target.new_value.clone())
                        .map_err(|_| AppError::InvalidProposal)?;
                    self.project_memory.restore_story_fact(
                        &snapshot,
                        expected_revision,
                        target_revision,
                    )
                }
                MemoryEntityType::CanonRule => {
                    let snapshot: CanonRule = serde_json::from_value(target.new_value.clone())
                        .map_err(|_| AppError::InvalidProposal)?;
                    self.project_memory.restore_canon_rule(
                        &snapshot,
                        expected_revision,
                        target_revision,
                    )
                }
                _ => Err(AppError::InvalidProposal),
            },
        }
    }

    pub fn set_canon_status(
        &self,
        entity_type: MemoryEntityType,
        entity_id: &str,
        status: CanonStatus,
        expected_revision: u64,
    ) -> AppResult<MemoryRevision> {
        match entity_type {
            MemoryEntityType::Character | MemoryEntityType::CharacterState => self
                .characters
                .set_canon_status(entity_type, entity_id, status, expected_revision),
            MemoryEntityType::StoryFact => self.project_memory.set_story_fact_canon_status(
                entity_id,
                status,
                expected_revision,
            ),
            MemoryEntityType::CanonRule => self.project_memory.set_canon_rule_canon_status(
                entity_id,
                status,
                expected_revision,
            ),
        }
    }

    fn promote_project_memory_proposal(
        project_memory: &ProjectMemoryRepository,
        proposal: &MemoryProposal,
        expected_revision: u64,
    ) -> AppResult<MemoryRevision> {
        match (
            proposal.entity_type,
            proposal.operation,
            proposal.entity_id.as_deref(),
        ) {
            (MemoryEntityType::StoryFact, RevisionOperation::Create, None) => {
                let input: CreateStoryFactInput = serde_json::from_value(proposal.payload.clone())
                    .map_err(|_| AppError::InvalidProposal)?;
                let fact = crate::domain::project_memory::build_story_fact(
                    proposal.project_id.clone(),
                    input,
                )?;
                if expected_revision != 0 || proposal.base_revision != 0 {
                    return Err(AppError::Conflict);
                }
                project_memory.promote_story_fact_create(&fact, proposal)
            }
            (MemoryEntityType::StoryFact, RevisionOperation::Update, Some(entity_id)) => {
                let input: UpdateStoryFactInput = serde_json::from_value(proposal.payload.clone())
                    .map_err(|_| AppError::InvalidProposal)?;
                if proposal.base_revision != expected_revision {
                    return Err(AppError::Conflict);
                }
                let (title, content) =
                    crate::domain::project_memory::update_story_fact_fields(input)?;
                project_memory.promote_story_fact_update(
                    entity_id,
                    &title,
                    &content,
                    expected_revision,
                    proposal,
                )
            }
            (MemoryEntityType::CanonRule, RevisionOperation::Create, None) => {
                let input: CreateCanonRuleInput = serde_json::from_value(proposal.payload.clone())
                    .map_err(|_| AppError::InvalidProposal)?;
                let rule = crate::domain::project_memory::build_canon_rule(
                    proposal.project_id.clone(),
                    input,
                )?;
                if expected_revision != 0 || proposal.base_revision != 0 {
                    return Err(AppError::Conflict);
                }
                project_memory.promote_canon_rule_create(&rule, proposal)
            }
            (MemoryEntityType::CanonRule, RevisionOperation::Update, Some(entity_id)) => {
                let input: UpdateCanonRuleInput = serde_json::from_value(proposal.payload.clone())
                    .map_err(|_| AppError::InvalidProposal)?;
                if proposal.base_revision != expected_revision {
                    return Err(AppError::Conflict);
                }
                let (title, rule, scope) =
                    crate::domain::project_memory::update_canon_rule_fields(input)?;
                project_memory.promote_canon_rule_update(
                    entity_id,
                    &title,
                    &rule,
                    &scope,
                    expected_revision,
                    proposal,
                )
            }
            _ => Err(AppError::InvalidProposal),
        }
    }
}

fn validate_story_fact_proposal(
    project_memory: &ProjectMemoryRepository,
    input: &CreateProposalInput,
) -> AppResult<()> {
    match (
        input.operation,
        input.entity_id.as_deref(),
        input.base_revision,
    ) {
        (RevisionOperation::Create, None, 0) => {
            serde_json::from_value::<CreateStoryFactInput>(input.payload.clone())
                .map_err(|_| AppError::InvalidProposal)?;
        }
        (RevisionOperation::Update, Some(entity_id), base_revision) if base_revision > 0 => {
            let fact = project_memory.get_story_fact(entity_id)?;
            if fact.project_id != input.project_id {
                return Err(AppError::NotFound);
            }
            serde_json::from_value::<UpdateStoryFactInput>(input.payload.clone())
                .map_err(|_| AppError::InvalidProposal)?;
        }
        _ => return Err(AppError::InvalidProposal),
    }
    Ok(())
}

fn validate_canon_rule_proposal(
    project_memory: &ProjectMemoryRepository,
    input: &CreateProposalInput,
) -> AppResult<()> {
    match (
        input.operation,
        input.entity_id.as_deref(),
        input.base_revision,
    ) {
        (RevisionOperation::Create, None, 0) => {
            serde_json::from_value::<CreateCanonRuleInput>(input.payload.clone())
                .map_err(|_| AppError::InvalidProposal)?;
        }
        (RevisionOperation::Update, Some(entity_id), base_revision) if base_revision > 0 => {
            let rule = project_memory.get_canon_rule(entity_id)?;
            if rule.project_id != input.project_id {
                return Err(AppError::NotFound);
            }
            serde_json::from_value::<UpdateCanonRuleInput>(input.payload.clone())
                .map_err(|_| AppError::InvalidProposal)?;
        }
        _ => return Err(AppError::InvalidProposal),
    }
    Ok(())
}
