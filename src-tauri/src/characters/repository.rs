use crate::{
    db::SharedConnection,
    domain::{
        character::{Character, CharacterListFilter, CharacterState, CharacterStatus},
        revision::CanonStatus,
    },
    error::{AppError, AppResult},
};

#[derive(Clone)]
pub struct CharacterRepository {
    connection: SharedConnection,
}

impl CharacterRepository {
    pub fn new(connection: SharedConnection) -> Self {
        Self { connection }
    }

    pub fn project_is_active(&self, project_id: &str) -> AppResult<()> {
        let connection = self.connection.lock()?;
        let status = connection
            .query_row(
                "SELECT status FROM projects WHERE id = ?1",
                [project_id],
                |row| row.get::<_, String>(0),
            )
            .map_err(|error| match error {
                rusqlite::Error::QueryReturnedNoRows => AppError::NotFound,
                other => other.into(),
            })?;
        if status == "archived" {
            return Err(AppError::ArchivedProject);
        }
        Ok(())
    }

    pub fn create(&self, character: &Character) -> AppResult<Character> {
        let connection = self.connection.lock()?;
        connection
            .execute(
                "INSERT INTO characters
                 (id, project_id, name, summary, role, status, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                rusqlite::params![
                    character.id,
                    character.project_id,
                    character.name,
                    character.summary,
                    character.role,
                    character.status.as_str(),
                    character.created_at,
                    character.updated_at,
                ],
            )
            .map_err(map_character_write_error)?;
        Ok(character.clone())
    }

    pub fn get(&self, id: &str) -> AppResult<Character> {
        let connection = self.connection.lock()?;
        connection
            .query_row(
                "SELECT id, project_id, name, summary, role, status, revision, canon_status, created_at, updated_at
                 FROM characters WHERE id = ?1",
                [id],
                map_character,
            )
            .map_err(map_not_found)
    }

    pub fn list(&self, project_id: &str, filter: CharacterListFilter) -> AppResult<Vec<Character>> {
        let connection = self.connection.lock()?;
        let mut statement = if filter.include_archived {
            connection.prepare(
                "SELECT id, project_id, name, summary, role, status, revision, canon_status, created_at, updated_at
                 FROM characters WHERE project_id = ?1 ORDER BY updated_at DESC",
            )?
        } else {
            connection.prepare(
                "SELECT id, project_id, name, summary, role, status, revision, canon_status, created_at, updated_at
                 FROM characters WHERE project_id = ?1 AND status = 'active'
                 ORDER BY updated_at DESC",
            )?
        };
        let rows = statement.query_map([project_id], map_character)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn update(
        &self,
        id: &str,
        name: &str,
        summary: &str,
        role: &str,
        updated_at: &str,
    ) -> AppResult<Character> {
        let connection = self.connection.lock()?;
        let existing = connection
            .query_row(
                "SELECT id, project_id, name, summary, role, status, revision, canon_status, created_at, updated_at
                 FROM characters WHERE id = ?1",
                [id],
                map_character,
            )
            .map_err(map_not_found)?;
        if existing.status == CharacterStatus::Archived {
            return Err(AppError::ArchivedCharacter);
        }
        connection
            .execute(
                "UPDATE characters SET name = ?1, summary = ?2, role = ?3, updated_at = ?4
                 WHERE id = ?5 AND status = 'active'",
                rusqlite::params![name, summary, role, updated_at, id],
            )
            .map_err(map_character_write_error)?;
        drop(connection);
        self.get(id)
    }

    pub fn archive(&self, id: &str, updated_at: &str) -> AppResult<Character> {
        let connection = self.connection.lock()?;
        let changed = connection.execute(
            "UPDATE characters SET status = 'archived', updated_at = ?1
             WHERE id = ?2 AND status = 'active'",
            rusqlite::params![updated_at, id],
        )?;
        if changed == 0 {
            let existing = connection
                .query_row(
                    "SELECT id, project_id, name, summary, role, status, revision, canon_status, created_at, updated_at
                     FROM characters WHERE id = ?1",
                    [id],
                    map_character,
                )
                .map_err(map_not_found)?;
            return Ok(existing);
        }
        drop(connection);
        self.get(id)
    }

    pub fn get_state(&self, character_id: &str) -> AppResult<Option<CharacterState>> {
        let connection = self.connection.lock()?;
        connection
            .query_row(
                "SELECT character_id, current_location, physical_condition, injuries,
                        emotional_state, goals, beliefs, knowledge, secrets_known,
                        current_conflicts, possessions, promises, last_appearance,
                        current_arc_role, revision, canon_status, updated_at
                 FROM character_states WHERE character_id = ?1",
                [character_id],
                map_state,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn upsert_state(&self, state: &CharacterState) -> AppResult<CharacterState> {
        let connection = self.connection.lock()?;
        connection.execute(
            "INSERT INTO character_states
             (character_id, current_location, physical_condition, injuries, emotional_state,
              goals, beliefs, knowledge, secrets_known, current_conflicts, possessions,
              promises, last_appearance, current_arc_role, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
             ON CONFLICT(character_id) DO UPDATE SET
              current_location = excluded.current_location,
              physical_condition = excluded.physical_condition,
              injuries = excluded.injuries,
              emotional_state = excluded.emotional_state,
              goals = excluded.goals,
              beliefs = excluded.beliefs,
              knowledge = excluded.knowledge,
              secrets_known = excluded.secrets_known,
              current_conflicts = excluded.current_conflicts,
              possessions = excluded.possessions,
              promises = excluded.promises,
              last_appearance = excluded.last_appearance,
              current_arc_role = excluded.current_arc_role,
              updated_at = excluded.updated_at",
            rusqlite::params![
                state.character_id,
                state.current_location,
                state.physical_condition,
                state.injuries,
                state.emotional_state,
                state.goals,
                state.beliefs,
                state.knowledge,
                state.secrets_known,
                state.current_conflicts,
                state.possessions,
                state.promises,
                state.last_appearance,
                state.current_arc_role,
                state.updated_at,
            ],
        )?;
        Ok(state.clone())
    }
}

fn map_character(row: &rusqlite::Row<'_>) -> rusqlite::Result<Character> {
    let status: String = row.get(5)?;
    let status = CharacterStatus::try_from(status.as_str()).map_err(|_| {
        rusqlite::Error::FromSqlConversionFailure(
            5,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid character status",
            )),
        )
    })?;
    Ok(Character {
        id: row.get(0)?,
        project_id: row.get(1)?,
        name: row.get(2)?,
        summary: row.get(3)?,
        role: row.get(4)?,
        status,
        revision: row.get::<_, i64>(6)? as u64,
        canon_status: parse_canon_status(row.get::<_, String>(7)?)?,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
    })
}

fn map_state(row: &rusqlite::Row<'_>) -> rusqlite::Result<CharacterState> {
    Ok(CharacterState {
        character_id: row.get(0)?,
        current_location: row.get(1)?,
        physical_condition: row.get(2)?,
        injuries: row.get(3)?,
        emotional_state: row.get(4)?,
        goals: row.get(5)?,
        beliefs: row.get(6)?,
        knowledge: row.get(7)?,
        secrets_known: row.get(8)?,
        current_conflicts: row.get(9)?,
        possessions: row.get(10)?,
        promises: row.get(11)?,
        last_appearance: row.get(12)?,
        current_arc_role: row.get(13)?,
        revision: row.get::<_, i64>(14)? as u64,
        canon_status: parse_canon_status(row.get::<_, String>(15)?)?,
        updated_at: row.get(16)?,
    })
}

fn parse_canon_status(value: String) -> rusqlite::Result<CanonStatus> {
    CanonStatus::try_from(value.as_str()).map_err(|_| {
        rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid canon status",
            )),
        )
    })
}

fn map_not_found(error: rusqlite::Error) -> AppError {
    match error {
        rusqlite::Error::QueryReturnedNoRows => AppError::NotFound,
        other => other.into(),
    }
}

fn map_character_write_error(error: rusqlite::Error) -> AppError {
    if matches!(
        error,
        rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error {
                code: rusqlite::ErrorCode::ConstraintViolation,
                ..
            },
            _
        )
    ) {
        AppError::DuplicateName
    } else {
        error.into()
    }
}

use rusqlite::OptionalExtension;
