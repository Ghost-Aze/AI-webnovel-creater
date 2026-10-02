use serde::{Deserialize, Serialize};

use crate::{
    domain::{
        character::{update_character_fields, UpdateCharacterInput},
        revision::{ActorType, MemoryProposal},
    },
    error::AppResult,
    revisions::service::ProposalService,
};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum MemoryToolAction {
    UpdateCharacter { input: UpdateCharacterInput },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MemoryToolRequest {
    pub project_id: String,
    pub character_id: String,
    pub base_revision: u64,
    pub actor_id: Option<String>,
    pub action: MemoryToolAction,
}

#[derive(Clone)]
pub struct MemoryToolService {
    proposals: ProposalService,
}

impl MemoryToolService {
    pub fn new(proposals: ProposalService) -> Self {
        Self { proposals }
    }

    pub fn propose(&self, request: MemoryToolRequest) -> AppResult<MemoryProposal> {
        match request.action {
            MemoryToolAction::UpdateCharacter { input } => {
                update_character_fields(input.clone())?;
                self.proposals
                    .create(crate::domain::revision::CreateProposalInput {
                        project_id: request.project_id,
                        entity_type: crate::domain::revision::MemoryEntityType::Character,
                        entity_id: Some(request.character_id),
                        operation: crate::domain::revision::RevisionOperation::Update,
                        payload: serde_json::to_value(input)
                            .map_err(|_| crate::error::AppError::Internal)?,
                        base_revision: request.base_revision,
                        actor_type: ActorType::Ai,
                        actor_id: request.actor_id,
                    })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        characters::{repository::CharacterRepository, service::CharacterService},
        db,
        domain::{
            character::CreateCharacterInput, project::CreateProjectInput, revision::ProposalStatus,
        },
        projects::{repository::ProjectRepository, service::ProjectService},
        revisions::{repository::RevisionRepository, service::ProposalService},
    };

    #[test]
    fn character_tool_creates_draft_without_mutating_canonical_character() {
        let connection = db::in_memory().unwrap();
        let projects = ProjectService::new(ProjectRepository::new(connection.clone()));
        let characters = CharacterService::new(CharacterRepository::new(connection.clone()));
        let revisions = RevisionRepository::new(connection.clone());
        let proposals = ProposalService::new(revisions, CharacterRepository::new(connection));
        let project = projects
            .create(CreateProjectInput {
                name: "Tool project".into(),
                description: None,
            })
            .unwrap();
        let character = characters
            .create(
                project.id.clone(),
                CreateCharacterInput {
                    name: "Mira".into(),
                    summary: Some("Scout".into()),
                    role: None,
                },
            )
            .unwrap();
        let service = MemoryToolService::new(proposals.clone());
        let proposal = service
            .propose(MemoryToolRequest {
                project_id: project.id.clone(),
                character_id: character.id.clone(),
                base_revision: character.revision,
                actor_id: Some("developer-chat".into()),
                action: MemoryToolAction::UpdateCharacter {
                    input: UpdateCharacterInput {
                        name: "Mira Vale".into(),
                        summary: Some("Scout captain".into()),
                        role: None,
                    },
                },
            })
            .unwrap();
        assert_eq!(proposal.status, ProposalStatus::Draft);
        assert_eq!(proposal.actor_type, ActorType::Ai);
        assert_eq!(characters.get(&character.id).unwrap().name, "Mira");
    }
}
