use rusqlite::{params, OptionalExtension, Transaction};
use serde_json::Value;

use crate::{
    db::SharedConnection,
    domain::revision::{ActorType, MemoryEntityType, MemoryRevision, RevisionOperation},
    error::{AppError, AppResult},
};

#[derive(Clone)]
pub struct RevisionRepository {
    connection: SharedConnection,
}

impl RevisionRepository {
    pub fn new(connection: SharedConnection) -> Self {
        Self { connection }
    }

    pub fn list(
        &self,
        entity_type: MemoryEntityType,
        entity_id: &str,
    ) -> AppResult<Vec<MemoryRevision>> {
        let connection = self.connection.lock()?;
        let mut statement = connection.prepare(
            "SELECT id, project_id, entity_type, entity_id, revision, operation,
                    actor_type, actor_id, base_revision, previous_value, new_value,
                    source_type, source_id, created_at
             FROM memory_revisions
             WHERE entity_type = ?1 AND entity_id = ?2
             ORDER BY revision ASC",
        )?;
        let rows = statement.query_map(params![entity_type.as_str(), entity_id], map_revision)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn get(
        &self,
        entity_type: MemoryEntityType,
        entity_id: &str,
        revision: u64,
    ) -> AppResult<MemoryRevision> {
        let connection = self.connection.lock()?;
        connection
            .query_row(
                "SELECT id, project_id, entity_type, entity_id, revision, operation,
                        actor_type, actor_id, base_revision, previous_value, new_value,
                        source_type, source_id, created_at
                 FROM memory_revisions
                 WHERE entity_type = ?1 AND entity_id = ?2 AND revision = ?3",
                params![entity_type.as_str(), entity_id, revision as i64],
                map_revision,
            )
            .optional()?
            .ok_or(AppError::NotFound)
    }

    pub(crate) fn insert_tx(
        transaction: &Transaction<'_>,
        revision: &MemoryRevision,
    ) -> AppResult<()> {
        transaction.execute(
            "INSERT INTO memory_revisions
             (id, project_id, entity_type, entity_id, revision, operation, actor_type,
              actor_id, base_revision, previous_value, new_value, source_type, source_id, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                revision.id,
                revision.project_id,
                revision.entity_type.as_str(),
                revision.entity_id,
                revision.revision as i64,
                revision.operation.as_str(),
                revision.actor_type.as_str(),
                revision.actor_id,
                revision.base_revision as i64,
                revision
                    .previous_value
                    .as_ref()
                    .map(serde_json::to_string)
                    .transpose()
                    .map_err(|_| AppError::Internal)?,
                serde_json::to_string(&revision.new_value).map_err(|_| AppError::Internal)?,
                revision.source_type,
                revision.source_id,
                revision.created_at,
            ],
        )?;
        Ok(())
    }
}

fn map_revision(row: &rusqlite::Row<'_>) -> rusqlite::Result<MemoryRevision> {
    let entity_type = parse_entity_type(row.get::<_, String>(2)?)?;
    let operation = parse_operation(row.get::<_, String>(5)?)?;
    let actor_type = parse_actor_type(row.get::<_, String>(6)?)?;
    let previous_value = row
        .get::<_, Option<String>>(9)?
        .map(|value| serde_json::from_str::<Value>(&value))
        .transpose()
        .map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                9,
                rusqlite::types::Type::Text,
                Box::new(error),
            )
        })?;
    let new_value_text: String = row.get(10)?;
    let new_value = serde_json::from_str::<Value>(&new_value_text).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(10, rusqlite::types::Type::Text, Box::new(error))
    })?;
    Ok(MemoryRevision {
        id: row.get(0)?,
        project_id: row.get(1)?,
        entity_type,
        entity_id: row.get(3)?,
        revision: row.get::<_, i64>(4)? as u64,
        operation,
        actor_type,
        actor_id: row.get(7)?,
        base_revision: row.get::<_, i64>(8)? as u64,
        previous_value,
        new_value,
        source_type: row.get(11)?,
        source_id: row.get(12)?,
        created_at: row.get(13)?,
    })
}

fn enum_error(value: &str) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(
        0,
        rusqlite::types::Type::Text,
        Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("invalid revision value: {value}"),
        )),
    )
}

fn parse_entity_type(value: String) -> rusqlite::Result<MemoryEntityType> {
    match value.as_str() {
        "character" => Ok(MemoryEntityType::Character),
        "character_state" => Ok(MemoryEntityType::CharacterState),
        _ => Err(enum_error(&value)),
    }
}

fn parse_operation(value: String) -> rusqlite::Result<RevisionOperation> {
    match value.as_str() {
        "create" => Ok(RevisionOperation::Create),
        "update" => Ok(RevisionOperation::Update),
        "archive" => Ok(RevisionOperation::Archive),
        "restore" => Ok(RevisionOperation::Restore),
        "canon_status" => Ok(RevisionOperation::CanonStatus),
        "promote" => Ok(RevisionOperation::Promote),
        _ => Err(enum_error(&value)),
    }
}

fn parse_actor_type(value: String) -> rusqlite::Result<ActorType> {
    match value.as_str() {
        "user" => Ok(ActorType::User),
        "ai" => Ok(ActorType::Ai),
        "system" => Ok(ActorType::System),
        _ => Err(enum_error(&value)),
    }
}
