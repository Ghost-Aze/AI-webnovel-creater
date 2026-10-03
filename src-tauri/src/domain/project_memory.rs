use serde::{Deserialize, Serialize};

use super::{
    project::{new_id, now_utc},
    revision::CanonStatus,
};
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProjectMemoryStatus {
    Active,
    Archived,
}

impl ProjectMemoryStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Archived => "archived",
        }
    }
}

impl TryFrom<&str> for ProjectMemoryStatus {
    type Error = AppError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "active" => Ok(Self::Active),
            "archived" => Ok(Self::Archived),
            _ => Err(AppError::InvalidStatus),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize)]
pub struct ProjectMemoryListFilter {
    #[serde(default)]
    pub include_archived: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoryFact {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub content: String,
    pub status: ProjectMemoryStatus,
    pub canon_status: CanonStatus,
    pub revision: u64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonRule {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub rule: String,
    pub scope: String,
    pub status: ProjectMemoryStatus,
    pub canon_status: CanonStatus,
    pub revision: u64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateStoryFactInput {
    pub title: String,
    pub content: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpdateStoryFactInput {
    pub title: String,
    pub content: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateCanonRuleInput {
    pub title: String,
    pub rule: String,
    pub scope: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpdateCanonRuleInput {
    pub title: String,
    pub rule: String,
    pub scope: String,
}

pub fn build_story_fact(project_id: String, input: CreateStoryFactInput) -> AppResult<StoryFact> {
    let title = required_text(input.title, "Story fact title")?;
    let content = required_text(input.content, "Story fact content")?;
    let timestamp = now_utc();
    Ok(StoryFact {
        id: new_id(),
        project_id,
        title,
        content,
        status: ProjectMemoryStatus::Active,
        canon_status: CanonStatus::Canon,
        revision: 1,
        created_at: timestamp.clone(),
        updated_at: timestamp,
    })
}

pub fn update_story_fact_fields(input: UpdateStoryFactInput) -> AppResult<(String, String)> {
    Ok((
        required_text(input.title, "Story fact title")?,
        required_text(input.content, "Story fact content")?,
    ))
}

pub fn build_canon_rule(project_id: String, input: CreateCanonRuleInput) -> AppResult<CanonRule> {
    let title = required_text(input.title, "Canon rule title")?;
    let rule = required_text(input.rule, "Canon rule text")?;
    let scope = required_text(input.scope, "Canon rule scope")?;
    let timestamp = now_utc();
    Ok(CanonRule {
        id: new_id(),
        project_id,
        title,
        rule,
        scope,
        status: ProjectMemoryStatus::Active,
        canon_status: CanonStatus::Canon,
        revision: 1,
        created_at: timestamp.clone(),
        updated_at: timestamp,
    })
}

pub fn update_canon_rule_fields(
    input: UpdateCanonRuleInput,
) -> AppResult<(String, String, String)> {
    Ok((
        required_text(input.title, "Canon rule title")?,
        required_text(input.rule, "Canon rule text")?,
        required_text(input.scope, "Canon rule scope")?,
    ))
}

fn required_text(value: String, label: &str) -> AppResult<String> {
    let value = value.trim();
    if value.is_empty() {
        return Err(AppError::Validation {
            message: format!("{label} cannot be empty."),
        });
    }
    Ok(value.to_string())
}

#[cfg(test)]
mod tests {
    use crate::{domain::revision::MemoryEntityType, error::AppError};

    use super::*;

    #[test]
    fn story_fact_and_rule_inputs_are_normalized_and_defaulted() {
        let fact = build_story_fact(
            "project".into(),
            CreateStoryFactInput {
                title: "  The gate  ".into(),
                content: "  Opens at dawn. ".into(),
            },
        )
        .unwrap();
        assert_eq!(fact.title, "The gate");
        assert_eq!(fact.content, "Opens at dawn.");
        assert_eq!(fact.revision, 1);
        assert_eq!(fact.status, ProjectMemoryStatus::Active);
        assert_eq!(fact.canon_status, CanonStatus::Canon);

        let rule = build_canon_rule(
            "project".into(),
            CreateCanonRuleInput {
                title: "  Magic  ".into(),
                rule: "  It costs memory. ".into(),
                scope: "  world  ".into(),
            },
        )
        .unwrap();
        assert_eq!(rule.title, "Magic");
        assert_eq!(rule.rule, "It costs memory.");
        assert_eq!(rule.scope, "world");
    }

    #[test]
    fn blank_project_memory_fields_are_rejected() {
        assert!(build_story_fact(
            "project".into(),
            CreateStoryFactInput {
                title: " ".into(),
                content: "fact".into(),
            },
        )
        .is_err());
        assert!(build_canon_rule(
            "project".into(),
            CreateCanonRuleInput {
                title: "rule".into(),
                rule: "rule".into(),
                scope: " ".into(),
            },
        )
        .is_err());
    }

    #[test]
    fn project_memory_wire_values_round_trip() {
        assert_eq!(
            serde_json::to_string(&MemoryEntityType::StoryFact).unwrap(),
            "\"story_fact\""
        );
        assert_eq!(
            serde_json::to_string(&MemoryEntityType::CanonRule).unwrap(),
            "\"canon_rule\""
        );
        assert_eq!(
            serde_json::to_string(&AppError::ArchivedMemory).unwrap(),
            "{\"code\":\"archived_memory\"}"
        );
    }
}
