use serde::Serialize;

use crate::{
    db::SharedConnection,
    domain::{
        character::{Character, CharacterListFilter, CharacterState, CharacterStatus},
        project::{new_id, now_utc},
        revision::{ActorType, CanonStatus, MemoryEntityType, MemoryRevision, RevisionOperation},
    },
    error::{AppError, AppResult},
    revisions::repository::RevisionRepository,
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
        let transaction = connection.unchecked_transaction()?;
        let mut current = character.clone();
        current.revision = 1;
        transaction
            .execute(
                "INSERT INTO characters
                 (id, project_id, name, summary, role, status, revision, canon_status, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                rusqlite::params![
                    current.id,
                    current.project_id,
                    current.name,
                    current.summary,
                    current.role,
                    current.status.as_str(),
                    current.revision as i64,
                    current.canon_status.as_str(),
                    current.created_at,
                    current.updated_at,
                ],
            )
            .map_err(map_character_write_error)?;
        let revision = build_revision(
            &current.project_id,
            MemoryEntityType::Character,
            &current.id,
            1,
            RevisionOperation::Create,
            0,
            None,
            &current,
            "manual",
            None,
        )?;
        RevisionRepository::insert_tx(&transaction, &revision)?;
        transaction.commit()?;
        Ok(current)
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
        expected_revision: u64,
    ) -> AppResult<Character> {
        let connection = self.connection.lock()?;
        let transaction = connection.unchecked_transaction()?;
        let existing = transaction
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
        if existing.canon_status == CanonStatus::LockedCanon {
            return Err(AppError::LockedCanon);
        }
        if existing.revision != expected_revision {
            return Err(AppError::Conflict);
        }
        let mut current = existing.clone();
        current.name = name.to_string();
        current.summary = summary.to_string();
        current.role = role.to_string();
        current.updated_at = updated_at.to_string();
        current.revision = expected_revision + 1;
        transaction
            .execute(
                "UPDATE characters SET name = ?1, summary = ?2, role = ?3, revision = ?4, updated_at = ?5
                 WHERE id = ?6 AND revision = ?7 AND status = 'active'",
                rusqlite::params![
                    current.name,
                    current.summary,
                    current.role,
                    current.revision as i64,
                    current.updated_at,
                    id,
                    expected_revision as i64,
                ],
            )
            .map_err(map_character_write_error)?;
        let revision = build_revision(
            &current.project_id,
            MemoryEntityType::Character,
            &current.id,
            current.revision,
            RevisionOperation::Update,
            expected_revision,
            Some(&existing),
            &current,
            "manual",
            None,
        )?;
        RevisionRepository::insert_tx(&transaction, &revision)?;
        transaction.commit()?;
        Ok(current)
    }

    pub fn archive(
        &self,
        id: &str,
        updated_at: &str,
        expected_revision: u64,
    ) -> AppResult<Character> {
        let connection = self.connection.lock()?;
        let transaction = connection.unchecked_transaction()?;
        let existing = transaction
            .query_row(
                "SELECT id, project_id, name, summary, role, status, revision, canon_status, created_at, updated_at
                 FROM characters WHERE id = ?1",
                [id],
                map_character,
            )
            .map_err(map_not_found)?;
        if existing.status == CharacterStatus::Archived {
            return Ok(existing);
        }
        if existing.canon_status == CanonStatus::LockedCanon {
            return Err(AppError::LockedCanon);
        }
        if existing.revision != expected_revision {
            return Err(AppError::Conflict);
        }
        let mut current = existing.clone();
        current.status = CharacterStatus::Archived;
        current.updated_at = updated_at.to_string();
        current.revision = expected_revision + 1;
        transaction.execute(
            "UPDATE characters SET status = 'archived', revision = ?1, updated_at = ?2
             WHERE id = ?3 AND revision = ?4 AND status = 'active'",
            rusqlite::params![
                current.revision as i64,
                current.updated_at,
                id,
                expected_revision as i64
            ],
        )?;
        let revision = build_revision(
            &current.project_id,
            MemoryEntityType::Character,
            &current.id,
            current.revision,
            RevisionOperation::Archive,
            expected_revision,
            Some(&existing),
            &current,
            "manual",
            None,
        )?;
        RevisionRepository::insert_tx(&transaction, &revision)?;
        transaction.commit()?;
        Ok(current)
    }

    pub fn promote_character_update(
        &self,
        id: &str,
        name: &str,
        summary: &str,
        role: &str,
        expected_revision: u64,
        proposal_id: &str,
    ) -> AppResult<MemoryRevision> {
        let connection = self.connection.lock()?;
        let transaction = connection.unchecked_transaction()?;
        let existing = transaction
            .query_row(
                "SELECT id, project_id, name, summary, role, status, revision, canon_status, created_at, updated_at
                 FROM characters WHERE id = ?1",
                [id],
                map_character,
            )
            .map_err(map_not_found)?;
        ensure_mutable(&existing.canon_status, existing.revision, expected_revision)?;
        let mut current = existing.clone();
        current.name = name.to_string();
        current.summary = summary.to_string();
        current.role = role.to_string();
        current.revision += 1;
        current.updated_at = now_utc();
        transaction
            .execute(
                "UPDATE characters SET name = ?1, summary = ?2, role = ?3, revision = ?4, updated_at = ?5
                 WHERE id = ?6 AND revision = ?7 AND status = 'active'",
                rusqlite::params![
                    current.name,
                    current.summary,
                    current.role,
                    current.revision as i64,
                    current.updated_at,
                    id,
                    expected_revision as i64,
                ],
            )
            .map_err(map_character_write_error)?;
        let changed = transaction.execute(
            "UPDATE memory_proposals SET status = 'accepted', updated_at = ?1
             WHERE id = ?2 AND project_id = ?3 AND status = 'draft'",
            rusqlite::params![now_utc(), proposal_id, current.project_id],
        )?;
        if changed != 1 {
            return Err(AppError::InvalidProposal);
        }
        let revision = build_revision(
            &current.project_id,
            MemoryEntityType::Character,
            &current.id,
            current.revision,
            RevisionOperation::Promote,
            expected_revision,
            Some(&existing),
            &current,
            "proposal",
            Some(proposal_id),
        )?;
        RevisionRepository::insert_tx(&transaction, &revision)?;
        transaction.commit()?;
        Ok(revision)
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
        let transaction = connection.unchecked_transaction()?;
        let existing = transaction
            .query_row(
                "SELECT character_id, current_location, physical_condition, injuries,
                        emotional_state, goals, beliefs, knowledge, secrets_known,
                        current_conflicts, possessions, promises, last_appearance,
                        current_arc_role, revision, canon_status, updated_at
                 FROM character_states WHERE character_id = ?1",
                [&state.character_id],
                map_state,
            )
            .optional()?;
        if let Some(existing) = &existing {
            if existing.canon_status == CanonStatus::LockedCanon {
                return Err(AppError::LockedCanon);
            }
            if existing.revision != state.revision {
                return Err(AppError::Conflict);
            }
        } else if state.revision != 0 {
            return Err(AppError::Conflict);
        }
        let mut current = state.clone();
        current.revision = state.revision + 1;
        current.canon_status = existing
            .as_ref()
            .map(|value| value.canon_status)
            .unwrap_or(CanonStatus::Canon);
        transaction.execute(
            "INSERT INTO character_states
             (character_id, current_location, physical_condition, injuries, emotional_state,
              goals, beliefs, knowledge, secrets_known, current_conflicts, possessions,
              promises, last_appearance, current_arc_role, revision, canon_status, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)
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
              revision = excluded.revision,
              canon_status = excluded.canon_status,
              updated_at = excluded.updated_at",
            rusqlite::params![
                current.character_id,
                current.current_location,
                current.physical_condition,
                current.injuries,
                current.emotional_state,
                current.goals,
                current.beliefs,
                current.knowledge,
                current.secrets_known,
                current.current_conflicts,
                current.possessions,
                current.promises,
                current.last_appearance,
                current.current_arc_role,
                current.revision as i64,
                current.canon_status.as_str(),
                current.updated_at,
            ],
        )?;
        let project_id: String = transaction.query_row(
            "SELECT project_id FROM characters WHERE id = ?1",
            [&current.character_id],
            |row| row.get(0),
        )?;
        let revision = build_revision(
            &project_id,
            MemoryEntityType::CharacterState,
            &current.character_id,
            current.revision,
            if existing.is_some() {
                RevisionOperation::Update
            } else {
                RevisionOperation::Create
            },
            state.revision,
            existing.as_ref(),
            &current,
            "manual",
            None,
        )?;
        RevisionRepository::insert_tx(&transaction, &revision)?;
        transaction.commit()?;
        Ok(current)
    }

    pub fn restore_character(
        &self,
        snapshot: &Character,
        expected_revision: u64,
        target_revision: u64,
    ) -> AppResult<MemoryRevision> {
        let connection = self.connection.lock()?;
        let transaction = connection.unchecked_transaction()?;
        let existing = transaction
            .query_row(
                "SELECT id, project_id, name, summary, role, status, revision, canon_status, created_at, updated_at
                 FROM characters WHERE id = ?1",
                [&snapshot.id],
                map_character,
            )
            .map_err(map_not_found)?;
        ensure_mutable(&existing.canon_status, existing.revision, expected_revision)?;
        let mut current = snapshot.clone();
        current.revision = expected_revision + 1;
        current.updated_at = now_utc();
        transaction.execute(
            "UPDATE characters SET name = ?1, summary = ?2, role = ?3, status = ?4,
                    canon_status = ?5, revision = ?6, updated_at = ?7
             WHERE id = ?8 AND revision = ?9",
            rusqlite::params![
                current.name,
                current.summary,
                current.role,
                current.status.as_str(),
                current.canon_status.as_str(),
                current.revision as i64,
                current.updated_at,
                current.id,
                expected_revision as i64,
            ],
        )?;
        let revision = build_revision(
            &current.project_id,
            MemoryEntityType::Character,
            &current.id,
            current.revision,
            RevisionOperation::Restore,
            expected_revision,
            Some(&existing),
            &current,
            "restore",
            Some(&target_revision.to_string()),
        )?;
        RevisionRepository::insert_tx(&transaction, &revision)?;
        transaction.commit()?;
        Ok(revision)
    }

    pub fn restore_state(
        &self,
        snapshot: &CharacterState,
        expected_revision: u64,
        target_revision: u64,
    ) -> AppResult<MemoryRevision> {
        let connection = self.connection.lock()?;
        let transaction = connection.unchecked_transaction()?;
        let existing = transaction
            .query_row(
                "SELECT character_id, current_location, physical_condition, injuries,
                        emotional_state, goals, beliefs, knowledge, secrets_known,
                        current_conflicts, possessions, promises, last_appearance,
                        current_arc_role, revision, canon_status, updated_at
                 FROM character_states WHERE character_id = ?1",
                [&snapshot.character_id],
                map_state,
            )
            .map_err(map_not_found)?;
        ensure_mutable(&existing.canon_status, existing.revision, expected_revision)?;
        let mut current = snapshot.clone();
        current.revision = expected_revision + 1;
        current.updated_at = now_utc();
        transaction.execute(
            "UPDATE character_states SET current_location = ?1, physical_condition = ?2,
                    injuries = ?3, emotional_state = ?4, goals = ?5, beliefs = ?6,
                    knowledge = ?7, secrets_known = ?8, current_conflicts = ?9,
                    possessions = ?10, promises = ?11, last_appearance = ?12,
                    current_arc_role = ?13, revision = ?14, canon_status = ?15, updated_at = ?16
             WHERE character_id = ?17 AND revision = ?18",
            rusqlite::params![
                current.current_location,
                current.physical_condition,
                current.injuries,
                current.emotional_state,
                current.goals,
                current.beliefs,
                current.knowledge,
                current.secrets_known,
                current.current_conflicts,
                current.possessions,
                current.promises,
                current.last_appearance,
                current.current_arc_role,
                current.revision as i64,
                current.canon_status.as_str(),
                current.updated_at,
                current.character_id,
                expected_revision as i64,
            ],
        )?;
        let project_id: String = transaction.query_row(
            "SELECT project_id FROM characters WHERE id = ?1",
            [&current.character_id],
            |row| row.get(0),
        )?;
        let revision = build_revision(
            &project_id,
            MemoryEntityType::CharacterState,
            &current.character_id,
            current.revision,
            RevisionOperation::Restore,
            expected_revision,
            Some(&existing),
            &current,
            "restore",
            Some(&target_revision.to_string()),
        )?;
        RevisionRepository::insert_tx(&transaction, &revision)?;
        transaction.commit()?;
        Ok(revision)
    }

    pub fn set_canon_status(
        &self,
        entity_type: MemoryEntityType,
        entity_id: &str,
        status: CanonStatus,
        expected_revision: u64,
    ) -> AppResult<MemoryRevision> {
        let connection = self.connection.lock()?;
        let transaction = connection.unchecked_transaction()?;
        match entity_type {
            MemoryEntityType::Character => {
                let existing = transaction
                    .query_row(
                        "SELECT id, project_id, name, summary, role, status, revision, canon_status, created_at, updated_at
                         FROM characters WHERE id = ?1",
                        [entity_id],
                        map_character,
                    )
                    .map_err(map_not_found)?;
                if existing.revision != expected_revision {
                    return Err(AppError::Conflict);
                }
                let mut current = existing.clone();
                current.canon_status = status;
                current.revision += 1;
                current.updated_at = now_utc();
                transaction.execute(
                    "UPDATE characters SET canon_status = ?1, revision = ?2, updated_at = ?3
                     WHERE id = ?4 AND revision = ?5",
                    rusqlite::params![
                        current.canon_status.as_str(),
                        current.revision as i64,
                        current.updated_at,
                        entity_id,
                        expected_revision as i64,
                    ],
                )?;
                let revision = build_revision(
                    &current.project_id,
                    entity_type,
                    entity_id,
                    current.revision,
                    RevisionOperation::CanonStatus,
                    expected_revision,
                    Some(&existing),
                    &current,
                    "manual",
                    None,
                )?;
                RevisionRepository::insert_tx(&transaction, &revision)?;
                transaction.commit()?;
                Ok(revision)
            }
            MemoryEntityType::CharacterState => {
                let existing = transaction
                    .query_row(
                        "SELECT character_id, current_location, physical_condition, injuries,
                                emotional_state, goals, beliefs, knowledge, secrets_known,
                                current_conflicts, possessions, promises, last_appearance,
                                current_arc_role, revision, canon_status, updated_at
                         FROM character_states WHERE character_id = ?1",
                        [entity_id],
                        map_state,
                    )
                    .map_err(map_not_found)?;
                if existing.revision != expected_revision {
                    return Err(AppError::Conflict);
                }
                let mut current = existing.clone();
                current.canon_status = status;
                current.revision += 1;
                current.updated_at = now_utc();
                transaction.execute(
                    "UPDATE character_states SET canon_status = ?1, revision = ?2, updated_at = ?3
                     WHERE character_id = ?4 AND revision = ?5",
                    rusqlite::params![
                        current.canon_status.as_str(),
                        current.revision as i64,
                        current.updated_at,
                        entity_id,
                        expected_revision as i64,
                    ],
                )?;
                let project_id: String = transaction.query_row(
                    "SELECT project_id FROM characters WHERE id = ?1",
                    [entity_id],
                    |row| row.get(0),
                )?;
                let revision = build_revision(
                    &project_id,
                    entity_type,
                    entity_id,
                    current.revision,
                    RevisionOperation::CanonStatus,
                    expected_revision,
                    Some(&existing),
                    &current,
                    "manual",
                    None,
                )?;
                RevisionRepository::insert_tx(&transaction, &revision)?;
                transaction.commit()?;
                Ok(revision)
            }
        }
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

fn ensure_mutable(
    canon_status: &CanonStatus,
    current_revision: u64,
    expected_revision: u64,
) -> AppResult<()> {
    if *canon_status == CanonStatus::LockedCanon {
        return Err(AppError::LockedCanon);
    }
    if current_revision != expected_revision {
        return Err(AppError::Conflict);
    }
    Ok(())
}

fn build_revision<T: Serialize>(
    project_id: &str,
    entity_type: MemoryEntityType,
    entity_id: &str,
    revision: u64,
    operation: RevisionOperation,
    base_revision: u64,
    previous_value: Option<&T>,
    new_value: &T,
    source_type: &str,
    source_id: Option<&str>,
) -> AppResult<MemoryRevision> {
    Ok(MemoryRevision {
        id: new_id(),
        project_id: project_id.to_string(),
        entity_type,
        entity_id: entity_id.to_string(),
        revision,
        operation,
        actor_type: ActorType::User,
        actor_id: None,
        base_revision,
        previous_value: previous_value
            .map(serde_json::to_value)
            .transpose()
            .map_err(|_| AppError::Internal)?,
        new_value: serde_json::to_value(new_value).map_err(|_| AppError::Internal)?,
        source_type: Some(source_type.to_string()),
        source_id: source_id.map(str::to_string),
        created_at: now_utc(),
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
