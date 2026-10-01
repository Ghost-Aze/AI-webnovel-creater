use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CanonStatus {
    Canon,
    LockedCanon,
}

impl CanonStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Canon => "canon",
            Self::LockedCanon => "locked_canon",
        }
    }
}

impl TryFrom<&str> for CanonStatus {
    type Error = crate::error::AppError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "canon" => Ok(Self::Canon),
            "locked_canon" => Ok(Self::LockedCanon),
            _ => Err(crate::error::AppError::InvalidStatus),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryEntityType {
    Character,
    CharacterState,
}

impl MemoryEntityType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Character => "character",
            Self::CharacterState => "character_state",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RevisionOperation {
    Create,
    Update,
    Archive,
    Restore,
    CanonStatus,
    Promote,
}

impl RevisionOperation {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Update => "update",
            Self::Archive => "archive",
            Self::Restore => "restore",
            Self::CanonStatus => "canon_status",
            Self::Promote => "promote",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ActorType {
    User,
    Ai,
    System,
}

impl ActorType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Ai => "ai",
            Self::System => "system",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProposalStatus {
    Draft,
    Accepted,
    Rejected,
}

impl ProposalStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryRevision {
    pub id: String,
    pub project_id: String,
    pub entity_type: MemoryEntityType,
    pub entity_id: String,
    pub revision: u64,
    pub operation: RevisionOperation,
    pub actor_type: ActorType,
    pub actor_id: Option<String>,
    pub base_revision: u64,
    pub previous_value: Option<Value>,
    pub new_value: Value,
    pub source_type: Option<String>,
    pub source_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryProposal {
    pub id: String,
    pub project_id: String,
    pub entity_type: MemoryEntityType,
    pub entity_id: String,
    pub operation: RevisionOperation,
    pub payload: Value,
    pub base_revision: u64,
    pub status: ProposalStatus,
    pub actor_type: ActorType,
    pub actor_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn revision_enums_round_trip_as_lowercase_values() {
        assert_eq!(
            serde_json::to_string(&CanonStatus::LockedCanon).unwrap(),
            "\"locked_canon\""
        );
        assert_eq!(
            serde_json::from_str::<CanonStatus>("\"canon\"").unwrap(),
            CanonStatus::Canon
        );
        assert_eq!(
            serde_json::to_string(&RevisionOperation::CanonStatus).unwrap(),
            "\"canon_status\""
        );
        assert_eq!(serde_json::to_string(&ActorType::Ai).unwrap(), "\"ai\"");
        assert_eq!(
            serde_json::to_string(&ProposalStatus::Draft).unwrap(),
            "\"draft\""
        );
    }

    #[test]
    fn invalid_revision_enum_value_is_rejected() {
        assert!(serde_json::from_str::<CanonStatus>("\"draft\"").is_err());
        assert!(serde_json::from_str::<MemoryEntityType>("\"project\"").is_err());
    }
}
