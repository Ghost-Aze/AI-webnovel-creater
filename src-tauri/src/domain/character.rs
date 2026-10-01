use serde::{Deserialize, Serialize};

use super::project::{new_id, normalize_name, now_utc};
use super::revision::CanonStatus;
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CharacterStatus {
    Active,
    Archived,
}

impl CharacterStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Archived => "archived",
        }
    }
}

impl TryFrom<&str> for CharacterStatus {
    type Error = AppError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "active" => Ok(Self::Active),
            "archived" => Ok(Self::Archived),
            _ => Err(AppError::InvalidStatus),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Character {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub summary: String,
    pub role: String,
    pub status: CharacterStatus,
    pub revision: u64,
    pub canon_status: CanonStatus,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateCharacterInput {
    pub name: String,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub role: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpdateCharacterInput {
    pub name: String,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub role: Option<String>,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize)]
pub struct CharacterListFilter {
    #[serde(default)]
    pub include_archived: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CharacterState {
    pub character_id: String,
    pub current_location: String,
    pub physical_condition: String,
    pub injuries: String,
    pub emotional_state: String,
    pub goals: String,
    pub beliefs: String,
    pub knowledge: String,
    pub secrets_known: String,
    pub current_conflicts: String,
    pub possessions: String,
    pub promises: String,
    pub last_appearance: String,
    pub current_arc_role: String,
    pub revision: u64,
    pub canon_status: CanonStatus,
    pub updated_at: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct UpdateCharacterStateInput {
    pub current_location: Option<String>,
    pub physical_condition: Option<String>,
    pub injuries: Option<String>,
    pub emotional_state: Option<String>,
    pub goals: Option<String>,
    pub beliefs: Option<String>,
    pub knowledge: Option<String>,
    pub secrets_known: Option<String>,
    pub current_conflicts: Option<String>,
    pub possessions: Option<String>,
    pub promises: Option<String>,
    pub last_appearance: Option<String>,
    pub current_arc_role: Option<String>,
}

impl CharacterState {
    pub fn empty(character_id: String) -> Self {
        Self {
            character_id,
            current_location: String::new(),
            physical_condition: String::new(),
            injuries: String::new(),
            emotional_state: String::new(),
            goals: String::new(),
            beliefs: String::new(),
            knowledge: String::new(),
            secrets_known: String::new(),
            current_conflicts: String::new(),
            possessions: String::new(),
            promises: String::new(),
            last_appearance: String::new(),
            current_arc_role: String::new(),
            revision: 0,
            canon_status: CanonStatus::Canon,
            updated_at: String::new(),
        }
    }

    pub fn apply(&mut self, input: UpdateCharacterStateInput) {
        apply_field(&mut self.current_location, input.current_location);
        apply_field(&mut self.physical_condition, input.physical_condition);
        apply_field(&mut self.injuries, input.injuries);
        apply_field(&mut self.emotional_state, input.emotional_state);
        apply_field(&mut self.goals, input.goals);
        apply_field(&mut self.beliefs, input.beliefs);
        apply_field(&mut self.knowledge, input.knowledge);
        apply_field(&mut self.secrets_known, input.secrets_known);
        apply_field(&mut self.current_conflicts, input.current_conflicts);
        apply_field(&mut self.possessions, input.possessions);
        apply_field(&mut self.promises, input.promises);
        apply_field(&mut self.last_appearance, input.last_appearance);
        apply_field(&mut self.current_arc_role, input.current_arc_role);
        self.updated_at = now_utc();
    }
}

fn apply_field(target: &mut String, value: Option<String>) {
    if let Some(value) = value {
        *target = value.trim().to_string();
    }
}

pub fn build_character(project_id: String, input: CreateCharacterInput) -> AppResult<Character> {
    let timestamp = now_utc();
    Ok(Character {
        id: new_id(),
        project_id,
        name: normalize_name(&input.name)?,
        summary: input
            .summary
            .as_deref()
            .unwrap_or_default()
            .trim()
            .to_string(),
        role: input.role.as_deref().unwrap_or_default().trim().to_string(),
        status: CharacterStatus::Active,
        revision: 0,
        canon_status: CanonStatus::Canon,
        created_at: timestamp.clone(),
        updated_at: timestamp,
    })
}

pub fn update_character_fields(input: UpdateCharacterInput) -> AppResult<(String, String, String)> {
    Ok((
        normalize_name(&input.name)?,
        input
            .summary
            .as_deref()
            .unwrap_or_default()
            .trim()
            .to_string(),
        input.role.as_deref().unwrap_or_default().trim().to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn character_input_is_normalized_and_state_defaults_are_empty() {
        let character = build_character(
            "project".to_string(),
            CreateCharacterInput {
                name: "  Ji-an  ".to_string(),
                summary: Some("  A courier. ".to_string()),
                role: None,
            },
        )
        .unwrap();
        assert_eq!(character.name, "Ji-an");
        assert_eq!(character.summary, "A courier.");
        assert_eq!(CharacterState::empty(character.id).goals, "");
    }

    #[test]
    fn character_state_applies_partial_updates() {
        let mut state = CharacterState::empty("character".to_string());
        state.apply(UpdateCharacterStateInput {
            emotional_state: Some("  guarded  ".to_string()),
            goals: Some("Find the gate".to_string()),
            ..Default::default()
        });
        assert_eq!(state.emotional_state, "guarded");
        assert_eq!(state.goals, "Find the gate");
        assert_eq!(state.beliefs, "");
    }

    #[test]
    fn character_state_defaults_to_revision_zero_and_canon() {
        let state = CharacterState::empty("character".to_string());
        assert_eq!(state.revision, 0);
        assert_eq!(state.canon_status, CanonStatus::Canon);
    }
}
