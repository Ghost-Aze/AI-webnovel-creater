use rusqlite::{types::Type, OptionalExtension};

use crate::{
    db::SharedConnection,
    domain::user::{
        UpdateUserPreferencesInput, UpdateUserProfileInput, UserPreferences, UserProfile,
        LOCAL_USER_ID,
    },
    error::{AppError, AppResult},
};

#[derive(Clone)]
pub struct UserRepository {
    connection: SharedConnection,
}

impl UserRepository {
    pub fn new(connection: SharedConnection) -> Self {
        Self { connection }
    }

    pub fn get_profile(&self) -> AppResult<UserProfile> {
        let connection = self.connection.lock()?;
        connection
            .query_row(
                "SELECT id, display_name, preferred_language, created_at, updated_at, revision
                 FROM user_profile WHERE id = ?1",
                [LOCAL_USER_ID],
                map_profile,
            )
            .optional()?
            .ok_or(AppError::Storage)
    }

    pub fn update_profile(
        &self,
        input: &UpdateUserProfileInput,
        expected_revision: u64,
        updated_at: &str,
    ) -> AppResult<UserProfile> {
        let connection = self.connection.lock()?;
        let transaction = connection.unchecked_transaction()?;
        let next_revision = next_revision(expected_revision)?;
        let exists: Option<i64> = transaction
            .query_row(
                "SELECT 1 FROM user_profile WHERE id = ?1",
                [LOCAL_USER_ID],
                |row| row.get(0),
            )
            .optional()?;
        if exists.is_none() {
            return Err(AppError::Storage);
        }
        let changed = transaction.execute(
            "UPDATE user_profile
             SET display_name = ?1, preferred_language = ?2, updated_at = ?3, revision = ?4
             WHERE id = ?5 AND revision = ?6",
            rusqlite::params![
                input.display_name,
                input.preferred_language,
                updated_at,
                next_revision,
                LOCAL_USER_ID,
                revision_value(expected_revision)?,
            ],
        )?;
        if changed != 1 {
            return Err(AppError::Conflict);
        }
        let profile = transaction.query_row(
            "SELECT id, display_name, preferred_language, created_at, updated_at, revision
             FROM user_profile WHERE id = ?1",
            [LOCAL_USER_ID],
            map_profile,
        )?;
        transaction.commit()?;
        Ok(profile)
    }

    pub fn get_preferences(&self) -> AppResult<UserPreferences> {
        let connection = self.connection.lock()?;
        connection
            .query_row(
                "SELECT id, preferred_narrator, preferred_pov, chapter_length, scene_length,
                        dialogue_density, prose_level, pacing, avoid_repetition,
                        created_at, updated_at, revision
                 FROM user_preferences WHERE id = ?1",
                [LOCAL_USER_ID],
                map_preferences,
            )
            .optional()?
            .ok_or(AppError::Storage)
    }

    pub fn update_preferences(
        &self,
        input: &UpdateUserPreferencesInput,
        expected_revision: u64,
        updated_at: &str,
    ) -> AppResult<UserPreferences> {
        let connection = self.connection.lock()?;
        let transaction = connection.unchecked_transaction()?;
        let next_revision = next_revision(expected_revision)?;
        let exists: Option<i64> = transaction
            .query_row(
                "SELECT 1 FROM user_preferences WHERE id = ?1",
                [LOCAL_USER_ID],
                |row| row.get(0),
            )
            .optional()?;
        if exists.is_none() {
            return Err(AppError::Storage);
        }
        let changed = transaction.execute(
            "UPDATE user_preferences
             SET preferred_narrator = ?1, preferred_pov = ?2, chapter_length = ?3,
                 scene_length = ?4, dialogue_density = ?5, prose_level = ?6,
                 pacing = ?7, avoid_repetition = ?8, updated_at = ?9, revision = ?10
             WHERE id = ?11 AND revision = ?12",
            rusqlite::params![
                input.preferred_narrator,
                input.preferred_pov,
                i64::from(input.chapter_length),
                i64::from(input.scene_length),
                i64::from(input.dialogue_density),
                input.prose_level,
                input.pacing,
                if input.avoid_repetition { 1_i64 } else { 0_i64 },
                updated_at,
                next_revision,
                LOCAL_USER_ID,
                revision_value(expected_revision)?,
            ],
        )?;
        if changed != 1 {
            return Err(AppError::Conflict);
        }
        let preferences = transaction.query_row(
            "SELECT id, preferred_narrator, preferred_pov, chapter_length, scene_length,
                    dialogue_density, prose_level, pacing, avoid_repetition,
                    created_at, updated_at, revision
             FROM user_preferences WHERE id = ?1",
            [LOCAL_USER_ID],
            map_preferences,
        )?;
        transaction.commit()?;
        Ok(preferences)
    }
}

fn revision_value(revision: u64) -> AppResult<i64> {
    i64::try_from(revision).map_err(|_| AppError::Validation {
        message: "Revision is out of range.".to_string(),
    })
}

fn next_revision(revision: u64) -> AppResult<i64> {
    revision_value(revision.checked_add(1).ok_or(AppError::Conflict)?)
}

fn map_profile(row: &rusqlite::Row<'_>) -> rusqlite::Result<UserProfile> {
    Ok(UserProfile {
        id: row.get(0)?,
        display_name: row.get(1)?,
        preferred_language: row.get(2)?,
        created_at: row.get(3)?,
        updated_at: row.get(4)?,
        revision: map_revision(row.get(5)?, 5)?,
    })
}

fn map_preferences(row: &rusqlite::Row<'_>) -> rusqlite::Result<UserPreferences> {
    let chapter_length: i64 = row.get(3)?;
    let scene_length: i64 = row.get(4)?;
    let dialogue_density: i64 = row.get(5)?;
    let avoid_repetition: i64 = row.get(8)?;
    Ok(UserPreferences {
        id: row.get(0)?,
        preferred_narrator: row.get(1)?,
        preferred_pov: row.get(2)?,
        chapter_length: map_u32(chapter_length, 3)?,
        scene_length: map_u32(scene_length, 4)?,
        dialogue_density: map_u8(dialogue_density, 5)?,
        prose_level: row.get(6)?,
        pacing: row.get(7)?,
        avoid_repetition: map_bool(avoid_repetition, 8)?,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
        revision: map_revision(row.get(11)?, 11)?,
    })
}

fn map_revision(value: i64, column: usize) -> rusqlite::Result<u64> {
    u64::try_from(value).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(column, Type::Integer, Box::new(error))
    })
}

fn map_u32(value: i64, column: usize) -> rusqlite::Result<u32> {
    u32::try_from(value).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(column, Type::Integer, Box::new(error))
    })
}

fn map_u8(value: i64, column: usize) -> rusqlite::Result<u8> {
    u8::try_from(value).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(column, Type::Integer, Box::new(error))
    })
}

fn map_bool(value: i64, column: usize) -> rusqlite::Result<bool> {
    match value {
        0 => Ok(false),
        1 => Ok(true),
        other => Err(rusqlite::Error::FromSqlConversionFailure(
            column,
            Type::Integer,
            format!("invalid boolean value: {other}").into(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    #[test]
    fn repository_round_trips_boolean_preference_values() {
        let repository = UserRepository::new(db::in_memory().unwrap());
        let input = UpdateUserPreferencesInput {
            preferred_narrator: "first_person".to_string(),
            preferred_pov: "close".to_string(),
            chapter_length: 1800,
            scene_length: 500,
            dialogue_density: 60,
            prose_level: "lyrical".to_string(),
            pacing: "brisk".to_string(),
            avoid_repetition: false,
        };
        let updated = repository
            .update_preferences(&input, 1, "2026-10-02T00:00:00Z")
            .unwrap();
        assert!(!updated.avoid_repetition);
        assert!(!repository.get_preferences().unwrap().avoid_repetition);

        let input = UpdateUserPreferencesInput {
            avoid_repetition: true,
            ..input
        };
        let updated = repository
            .update_preferences(&input, 2, "2026-10-02T00:00:01Z")
            .unwrap();
        assert!(updated.avoid_repetition);
        assert!(repository.get_preferences().unwrap().avoid_repetition);
    }

    #[test]
    fn missing_singletons_map_to_storage_errors() {
        let connection = db::in_memory().unwrap();
        let repository = UserRepository::new(connection.clone());
        connection
            .lock()
            .unwrap()
            .execute("DELETE FROM user_profile", [])
            .unwrap();
        assert_eq!(repository.get_profile(), Err(AppError::Storage));
        assert_eq!(
            repository.update_profile(
                &UpdateUserProfileInput {
                    display_name: "Mira".to_string(),
                    preferred_language: "tr".to_string(),
                },
                1,
                "2026-10-02T00:00:00Z",
            ),
            Err(AppError::Storage)
        );

        connection
            .lock()
            .unwrap()
            .execute("DELETE FROM user_preferences", [])
            .unwrap();
        assert_eq!(repository.get_preferences(), Err(AppError::Storage));
    }
}
