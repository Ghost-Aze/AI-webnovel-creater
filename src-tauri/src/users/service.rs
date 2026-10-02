use crate::{
    domain::{
        project::now_utc,
        user::{
            normalize_preferences_input, normalize_profile_input, UpdateUserPreferencesInput,
            UpdateUserProfileInput, UserPreferences, UserProfile,
        },
    },
    error::AppResult,
};

use super::repository::UserRepository;

#[derive(Clone)]
pub struct UserService {
    repository: UserRepository,
}

impl UserService {
    pub fn new(repository: UserRepository) -> Self {
        Self { repository }
    }

    pub fn get_profile(&self) -> AppResult<UserProfile> {
        self.repository.get_profile()
    }

    pub fn update_profile(
        &self,
        input: UpdateUserProfileInput,
        expected_revision: u64,
    ) -> AppResult<UserProfile> {
        let input = normalize_profile_input(input)?;
        self.repository
            .update_profile(&input, expected_revision, &now_utc())
    }

    pub fn get_preferences(&self) -> AppResult<UserPreferences> {
        self.repository.get_preferences()
    }

    pub fn update_preferences(
        &self,
        input: UpdateUserPreferencesInput,
        expected_revision: u64,
    ) -> AppResult<UserPreferences> {
        let input = normalize_preferences_input(input)?;
        self.repository
            .update_preferences(&input, expected_revision, &now_utc())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{db, error::AppError};

    fn service() -> UserService {
        UserService::new(UserRepository::new(db::in_memory().unwrap()))
    }

    fn preferences() -> UpdateUserPreferencesInput {
        UpdateUserPreferencesInput {
            preferred_narrator: "first_person".to_string(),
            preferred_pov: "close".to_string(),
            chapter_length: 1800,
            scene_length: 500,
            dialogue_density: 60,
            prose_level: "lyrical".to_string(),
            pacing: "brisk".to_string(),
            avoid_repetition: false,
        }
    }

    #[test]
    fn service_reads_defaults_and_updates_both_singletons() {
        let service = service();
        let profile = service.get_profile().unwrap();
        assert_eq!(profile.id, "local_user");
        assert_eq!(profile.display_name, "Writer");
        assert_eq!(profile.preferred_language, "en");
        assert_eq!(profile.revision, 1);

        let defaults = service.get_preferences().unwrap();
        assert_eq!(defaults.preferred_narrator, "third_person");
        assert_eq!(defaults.preferred_pov, "limited");
        assert_eq!(defaults.chapter_length, 2000);
        assert_eq!(defaults.scene_length, 600);
        assert_eq!(defaults.dialogue_density, 40);
        assert_eq!(defaults.prose_level, "standard");
        assert_eq!(defaults.pacing, "balanced");
        assert!(defaults.avoid_repetition);

        let profile = service
            .update_profile(
                UpdateUserProfileInput {
                    display_name: "  Mira  ".to_string(),
                    preferred_language: "  tr  ".to_string(),
                },
                1,
            )
            .unwrap();
        assert_eq!(profile.display_name, "Mira");
        assert_eq!(profile.preferred_language, "tr");
        assert_eq!(profile.revision, 2);

        let preferences = service.update_preferences(preferences(), 1).unwrap();
        assert_eq!(preferences.revision, 2);
        assert!(!preferences.avoid_repetition);
    }

    #[test]
    fn service_rejects_stale_updates_without_changing_rows() {
        let service = service();
        service
            .update_profile(
                UpdateUserProfileInput {
                    display_name: "Mira".to_string(),
                    preferred_language: "tr".to_string(),
                },
                1,
            )
            .unwrap();
        let error = service
            .update_profile(
                UpdateUserProfileInput {
                    display_name: "Changed".to_string(),
                    preferred_language: "de".to_string(),
                },
                1,
            )
            .unwrap_err();
        assert_eq!(error, AppError::Conflict);
        assert_eq!(service.get_profile().unwrap().display_name, "Mira");
        assert_eq!(service.get_profile().unwrap().revision, 2);

        service.update_preferences(preferences(), 1).unwrap();
        let error = service.update_preferences(preferences(), 1).unwrap_err();
        assert_eq!(error, AppError::Conflict);
        assert_eq!(service.get_preferences().unwrap().revision, 2);
    }
}
