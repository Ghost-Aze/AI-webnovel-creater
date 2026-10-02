use rusqlite::OptionalExtension;

use crate::{
    db::SharedConnection,
    domain::{
        conversation::{
            AppendMessageInput, Conversation, ConversationKind, ConversationListFilter,
            ConversationMessage, MessageRole,
        },
        project::{new_id, now_utc},
    },
    error::{AppError, AppResult},
    provider::ModelRef,
};

#[derive(Clone)]
pub struct ConversationRepository {
    connection: SharedConnection,
}

impl ConversationRepository {
    pub fn new(connection: SharedConnection) -> Self {
        Self { connection }
    }

    pub fn create(&self, conversation: &Conversation) -> AppResult<Conversation> {
        let connection = self.connection.lock()?;
        connection.execute(
            "INSERT INTO conversations (id, project_id, kind, title, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                conversation.id,
                conversation.project_id,
                conversation.kind.as_str(),
                conversation.title,
                conversation.created_at,
                conversation.updated_at,
            ],
        )?;
        Ok(conversation.clone())
    }

    pub fn get(&self, id: &str) -> AppResult<Conversation> {
        let connection = self.connection.lock()?;
        connection
            .query_row(
                "SELECT id, project_id, kind, title, created_at, updated_at
                 FROM conversations WHERE id = ?1",
                [id],
                map_conversation,
            )
            .optional()?
            .ok_or(AppError::NotFound)
    }

    pub fn list(
        &self,
        project_id: &str,
        filter: ConversationListFilter,
    ) -> AppResult<Vec<Conversation>> {
        let connection = self.connection.lock()?;
        let mut statement = if filter.kind.is_some() {
            connection.prepare(
                "SELECT id, project_id, kind, title, created_at, updated_at
                 FROM conversations WHERE project_id = ?1 AND kind = ?2
                 ORDER BY updated_at DESC",
            )?
        } else {
            connection.prepare(
                "SELECT id, project_id, kind, title, created_at, updated_at
                 FROM conversations WHERE project_id = ?1
                 ORDER BY updated_at DESC",
            )?
        };
        let rows = if let Some(kind) = filter.kind {
            statement.query_map(
                rusqlite::params![project_id, kind.as_str()],
                map_conversation,
            )?
        } else {
            statement.query_map([project_id], map_conversation)?
        };
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn append_message(
        &self,
        conversation_id: &str,
        input: AppendMessageInput,
    ) -> AppResult<ConversationMessage> {
        let connection = self.connection.lock()?;
        let conversation_exists: Option<String> = connection
            .query_row(
                "SELECT id FROM conversations WHERE id = ?1",
                [conversation_id],
                |row| row.get(0),
            )
            .optional()?;
        if conversation_exists.is_none() {
            return Err(AppError::NotFound);
        }

        let sequence: i64 = connection.query_row(
            "SELECT COALESCE(MAX(sequence), 0) + 1 FROM conversation_messages
             WHERE conversation_id = ?1",
            [conversation_id],
            |row| row.get(0),
        )?;
        let created_at = now_utc();
        let message = ConversationMessage {
            id: new_id(),
            conversation_id: conversation_id.into(),
            sequence: sequence as u64,
            role: input.role,
            content: input.content,
            model: input.model,
            created_at,
        };
        connection.execute(
            "INSERT INTO conversation_messages
             (id, conversation_id, sequence, role, content, model_provider_id, model_id, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                message.id,
                message.conversation_id,
                message.sequence as i64,
                message.role.as_str(),
                message.content,
                message
                    .model
                    .as_ref()
                    .map(|model| model.provider_id.as_str()),
                message.model.as_ref().map(|model| model.model_id.as_str()),
                message.created_at,
            ],
        )?;
        connection.execute(
            "UPDATE conversations SET updated_at = ?1 WHERE id = ?2",
            rusqlite::params![now_utc(), conversation_id],
        )?;
        Ok(message)
    }

    pub fn list_messages(
        &self,
        conversation_id: &str,
        limit: Option<u32>,
    ) -> AppResult<Vec<ConversationMessage>> {
        let connection = self.connection.lock()?;
        let conversation_exists: Option<String> = connection
            .query_row(
                "SELECT id FROM conversations WHERE id = ?1",
                [conversation_id],
                |row| row.get(0),
            )
            .optional()?;
        if conversation_exists.is_none() {
            return Err(AppError::NotFound);
        }

        let mut messages = if let Some(limit) = limit {
            let mut statement = connection.prepare(
                "SELECT id, conversation_id, sequence, role, content,
                        model_provider_id, model_id, created_at
                 FROM conversation_messages WHERE conversation_id = ?1
                 ORDER BY sequence DESC LIMIT ?2",
            )?;
            let rows = statement.query_map(
                rusqlite::params![conversation_id, limit as i64],
                map_message,
            )?;
            rows.collect::<Result<Vec<_>, _>>()?
        } else {
            let mut statement = connection.prepare(
                "SELECT id, conversation_id, sequence, role, content,
                        model_provider_id, model_id, created_at
                 FROM conversation_messages WHERE conversation_id = ?1
                 ORDER BY sequence ASC",
            )?;
            let rows = statement.query_map([conversation_id], map_message)?;
            rows.collect::<Result<Vec<_>, _>>()?
        };
        if limit.is_some() {
            messages.reverse();
        }
        Ok(messages)
    }
}

fn map_conversation(row: &rusqlite::Row<'_>) -> rusqlite::Result<Conversation> {
    let kind: String = row.get(2)?;
    let kind = ConversationKind::try_from(kind.as_str()).map_err(|_| enum_error(&kind))?;
    Ok(Conversation {
        id: row.get(0)?,
        project_id: row.get(1)?,
        kind,
        title: row.get(3)?,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
}

fn map_message(row: &rusqlite::Row<'_>) -> rusqlite::Result<ConversationMessage> {
    let role: String = row.get(3)?;
    let role = MessageRole::try_from(role.as_str()).map_err(|_| enum_error(&role))?;
    let provider_id: Option<String> = row.get(5)?;
    let model_id: Option<String> = row.get(6)?;
    let model = match (provider_id, model_id) {
        (Some(provider_id), Some(model_id)) => Some(ModelRef {
            provider_id,
            model_id,
        }),
        (None, None) => None,
        _ => return Err(enum_error("partial model reference")),
    };
    Ok(ConversationMessage {
        id: row.get(0)?,
        conversation_id: row.get(1)?,
        sequence: row.get::<_, i64>(2)? as u64,
        role,
        content: row.get(4)?,
        model,
        created_at: row.get(7)?,
    })
}

fn enum_error(value: &str) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(
        0,
        rusqlite::types::Type::Text,
        Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("invalid conversation value: {value}"),
        )),
    )
}
