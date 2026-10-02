use serde::{Deserialize, Serialize};

use super::{
    manuscript::ManuscriptContentFormat,
    project::{new_id, now_utc},
    revision::{ActorType, ProposalStatus},
};
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManuscriptProposal {
    pub id: String,
    pub project_id: String,
    pub chapter_id: String,
    pub base_revision: u64,
    pub proposed_content: String,
    pub content_format: ManuscriptContentFormat,
    pub rationale: String,
    pub status: ProposalStatus,
    pub actor_type: ActorType,
    pub actor_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateManuscriptProposalInput {
    pub chapter_id: String,
    pub base_revision: u64,
    pub proposed_content: String,
    pub content_format: ManuscriptContentFormat,
    #[serde(default)]
    pub rationale: Option<String>,
    #[serde(default)]
    pub actor_type: Option<ActorType>,
    #[serde(default)]
    pub actor_id: Option<String>,
}

pub fn build_proposal(
    project_id: String,
    input: CreateManuscriptProposalInput,
) -> AppResult<ManuscriptProposal> {
    if input.chapter_id.trim().is_empty() || input.base_revision == 0 {
        return Err(AppError::Validation {
            message: "A chapter and positive base revision are required.".into(),
        });
    }
    if input.proposed_content.trim().is_empty() {
        return Err(AppError::Validation {
            message: "Proposed manuscript content is required.".into(),
        });
    }
    let timestamp = now_utc();
    Ok(ManuscriptProposal {
        id: new_id(),
        project_id,
        chapter_id: input.chapter_id,
        base_revision: input.base_revision,
        proposed_content: input.proposed_content,
        content_format: input.content_format,
        rationale: input.rationale.unwrap_or_default().trim().to_string(),
        status: ProposalStatus::Draft,
        actor_type: input.actor_type.unwrap_or(ActorType::Ai),
        actor_id: input.actor_id,
        created_at: timestamp.clone(),
        updated_at: timestamp,
    })
}
