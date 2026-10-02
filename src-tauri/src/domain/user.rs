use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

pub const LOCAL_USER_ID: &str = "local_user";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: String,
    pub display_name: String,
    pub preferred_language: String,
    pub created_at: String,
    pub updated_at: String,
    pub revision: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateUserProfileInput {
    pub display_name: String,
    pub preferred_language: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserPreferences {
    pub id: String,
    pub preferred_narrator: String,
    pub preferred_pov: String,
    pub chapter_length: u32,
    pub scene_length: u32,
    pub dialogue_density: u8,
    pub prose_level: String,
    pub pacing: String,
    pub avoid_repetition: bool,
    pub created_at: String,
    pub updated_at: String,
    pub revision: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateUserPreferencesInput {
    pub preferred_narrator: String,
    pub preferred_pov: String,
    pub chapter_length: u32,
    pub scene_length: u32,
    pub dialogue_density: u8,
    pub prose_level: String,
    pub pacing: String,
    pub avoid_repetition: bool,
}

pub fn normalize_profile_input(input: UpdateUserProfileInput) -> AppResult<UpdateUserProfileInput> {
    Ok(UpdateUserProfileInput {
        display_name: normalize_required(input.display_name, "Display name cannot be empty.")?,
        preferred_language: normalize_required(
            input.preferred_language,
            "Preferred language cannot be empty.",
        )?,
    })
}

pub fn normalize_preferences_input(
    input: UpdateUserPreferencesInput,
) -> AppResult<UpdateUserPreferencesInput> {
    if input.chapter_length == 0 {
        return Err(AppError::Validation {
            message: "Chapter length must be positive.".to_string(),
        });
    }
    if input.scene_length == 0 {
        return Err(AppError::Validation {
            message: "Scene length must be positive.".to_string(),
        });
    }
    if input.dialogue_density > 100 {
        return Err(AppError::Validation {
            message: "Dialogue density must be between 0 and 100.".to_string(),
        });
    }

    Ok(UpdateUserPreferencesInput {
        preferred_narrator: normalize_required(
            input.preferred_narrator,
            "Preferred narrator cannot be empty.",
        )?,
        preferred_pov: normalize_required(input.preferred_pov, "Preferred POV cannot be empty.")?,
        chapter_length: input.chapter_length,
        scene_length: input.scene_length,
        dialogue_density: input.dialogue_density,
        prose_level: normalize_required(input.prose_level, "Prose level cannot be empty.")?,
        pacing: normalize_required(input.pacing, "Pacing cannot be empty.")?,
        avoid_repetition: input.avoid_repetition,
    })
}

fn normalize_required(value: String, message: &str) -> AppResult<String> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(AppError::Validation {
            message: message.to_string(),
        });
    }
    Ok(normalized.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile_input() -> UpdateUserProfileInput {
        UpdateUserProfileInput {
            display_name: "  Mira  ".to_string(),
            preferred_language: "  tr  ".to_string(),
        }
    }

    fn preferences_input() -> UpdateUserPreferencesInput {
        UpdateUserPreferencesInput {
            preferred_narrator: "  first_person  ".to_string(),
            preferred_pov: "  close  ".to_string(),
            chapter_length: 1800,
            scene_length: 500,
            dialogue_density: 60,
            prose_level: "  lyrical  ".to_string(),
            pacing: "  brisk  ".to_string(),
            avoid_repetition: false,
        }
    }

    #[test]
    fn profile_input_trims_text_and_rejects_blank_values() {
        let normalized = normalize_profile_input(profile_input()).unwrap();
        assert_eq!(normalized.display_name, "Mira");
        assert_eq!(normalized.preferred_language, "tr");

        let error = normalize_profile_input(UpdateUserProfileInput {
            display_name: "  ".to_string(),
            preferred_language: "en".to_string(),
        })
        .unwrap_err();
        assert!(matches!(error, AppError::Validation { .. }));
    }

    #[test]
    fn preferences_input_trims_text_preserves_boolean_and_validates_ranges() {
        let normalized = normalize_preferences_input(preferences_input()).unwrap();
        assert_eq!(normalized.preferred_narrator, "first_person");
        assert_eq!(normalized.preferred_pov, "close");
        assert_eq!(normalized.prose_level, "lyrical");
        assert_eq!(normalized.pacing, "brisk");
        assert!(!normalized.avoid_repetition);

        for invalid in [
            UpdateUserPreferencesInput {
                chapter_length: 0,
                ..preferences_input()
            },
            UpdateUserPreferencesInput {
                scene_length: 0,
                ..preferences_input()
            },
            UpdateUserPreferencesInput {
                dialogue_density: 101,
                ..preferences_input()
            },
            UpdateUserPreferencesInput {
                preferred_pov: " ".to_string(),
                ..preferences_input()
            },
        ] {
            assert!(matches!(
                normalize_preferences_input(invalid),
                Err(AppError::Validation { .. })
            ));
        }
    }
}
