use crate::{
    db::SharedConnection,
    domain::{
        manuscript::ManuscriptContentFormat,
        manuscript_proposal::ManuscriptProposal,
        project::{new_id, now_utc},
        revision::{ActorType, ProposalStatus},
    },
    error::{AppError, AppResult},
};

#[derive(Clone)]
pub struct ManuscriptProposalRepository {
    connection: SharedConnection,
}

impl ManuscriptProposalRepository {
    pub fn new(connection: SharedConnection) -> Self {
        Self { connection }
    }

    pub fn create(&self, proposal: &ManuscriptProposal) -> AppResult<ManuscriptProposal> {
        let connection = self.connection.lock()?;
        connection.execute(
            "INSERT INTO manuscript_proposals
             (id, project_id, chapter_id, base_revision, proposed_content, content_format,
              rationale, status, actor_type, actor_id, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            rusqlite::params![
                proposal.id,
                proposal.project_id,
                proposal.chapter_id,
                proposal.base_revision as i64,
                proposal.proposed_content,
                proposal.content_format.as_str(),
                proposal.rationale,
                proposal.status.as_str(),
                proposal.actor_type.as_str(),
                proposal.actor_id,
                proposal.created_at,
                proposal.updated_at,
            ],
        )?;
        Ok(proposal.clone())
    }

    pub fn get(&self, id: &str) -> AppResult<ManuscriptProposal> {
        let connection = self.connection.lock()?;
        connection
            .query_row(
                "SELECT id, project_id, chapter_id, base_revision, proposed_content,
                        content_format, rationale, status, actor_type, actor_id,
                        created_at, updated_at
                 FROM manuscript_proposals WHERE id = ?1",
                [id],
                map_proposal,
            )
            .map_err(map_not_found)
    }

    pub fn list(
        &self,
        chapter_id: &str,
        status: Option<ProposalStatus>,
    ) -> AppResult<Vec<ManuscriptProposal>> {
        let connection = self.connection.lock()?;
        let mut statement = if status.is_some() {
            connection.prepare(
                "SELECT id, project_id, chapter_id, base_revision, proposed_content,
                        content_format, rationale, status, actor_type, actor_id,
                        created_at, updated_at
                 FROM manuscript_proposals WHERE chapter_id = ?1 AND status = ?2
                 ORDER BY updated_at DESC",
            )?
        } else {
            connection.prepare(
                "SELECT id, project_id, chapter_id, base_revision, proposed_content,
                        content_format, rationale, status, actor_type, actor_id,
                        created_at, updated_at
                 FROM manuscript_proposals WHERE chapter_id = ?1
                 ORDER BY updated_at DESC",
            )?
        };
        let rows = if let Some(status) = status {
            statement.query_map(rusqlite::params![chapter_id, status.as_str()], map_proposal)?
        } else {
            statement.query_map([chapter_id], map_proposal)?
        };
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn promote(
        &self,
        id: &str,
        expected_revision: u64,
    ) -> AppResult<crate::domain::manuscript::Manuscript> {
        let connection = self.connection.lock()?;
        let transaction = connection.unchecked_transaction()?;
        let proposal = transaction
            .query_row(
                "SELECT id, project_id, chapter_id, base_revision, proposed_content,
                        content_format, rationale, status, actor_type, actor_id,
                        created_at, updated_at
                 FROM manuscript_proposals WHERE id = ?1",
                [id],
                map_proposal,
            )
            .map_err(map_not_found)?;
        if proposal.status != ProposalStatus::Draft {
            return Err(AppError::InvalidProposal);
        }
        let chapter_status: String = transaction
            .query_row(
                "SELECT status FROM chapters WHERE id = ?1 AND project_id = ?2",
                rusqlite::params![proposal.chapter_id, proposal.project_id],
                |row| row.get(0),
            )
            .map_err(map_not_found)?;
        if chapter_status == "archived" {
            return Err(AppError::ArchivedChapter);
        }
        let manuscript = transaction
            .query_row(
                "SELECT id, chapter_id, content, content_format, revision, created_at, updated_at
                 FROM manuscripts WHERE chapter_id = ?1",
                [&proposal.chapter_id],
                map_manuscript,
            )
            .map_err(map_not_found)?;
        if manuscript.revision != expected_revision || proposal.base_revision != expected_revision {
            return Err(AppError::Conflict);
        }
        let timestamp = now_utc();
        let updated = crate::domain::manuscript::Manuscript {
            id: manuscript.id.clone(),
            chapter_id: manuscript.chapter_id.clone(),
            content: proposal.proposed_content.clone(),
            content_format: proposal.content_format,
            revision: expected_revision + 1,
            created_at: manuscript.created_at,
            updated_at: timestamp.clone(),
        };
        transaction.execute(
            "UPDATE manuscripts SET content = ?1, content_format = ?2, revision = ?3, updated_at = ?4
             WHERE id = ?5 AND revision = ?6",
            rusqlite::params![
                updated.content,
                updated.content_format.as_str(),
                updated.revision as i64,
                updated.updated_at,
                updated.id,
                expected_revision as i64,
            ],
        )?;
        transaction.execute(
            "INSERT INTO manuscript_revisions
             (id, manuscript_id, revision, content, content_format, label, actor_type, actor_id, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                new_id(),
                updated.id,
                updated.revision as i64,
                updated.content,
                updated.content_format.as_str(),
                if proposal.rationale.is_empty() {
                    "AI proposal"
                } else {
                    proposal.rationale.as_str()
                },
                proposal.actor_type.as_str(),
                proposal.actor_id,
                updated.updated_at,
            ],
        )?;
        transaction.execute(
            "UPDATE manuscript_proposals SET status = 'accepted', updated_at = ?1
             WHERE id = ?2 AND status = 'draft'",
            rusqlite::params![now_utc(), id],
        )?;
        transaction.commit()?;
        Ok(updated)
    }

    pub fn reject(&self, id: &str) -> AppResult<ManuscriptProposal> {
        let connection = self.connection.lock()?;
        let existing = connection
            .query_row(
                "SELECT id, project_id, chapter_id, base_revision, proposed_content,
                        content_format, rationale, status, actor_type, actor_id,
                        created_at, updated_at
                 FROM manuscript_proposals WHERE id = ?1",
                [id],
                map_proposal,
            )
            .map_err(map_not_found)?;
        if existing.status != ProposalStatus::Draft {
            return Err(AppError::InvalidProposal);
        }
        connection.execute(
            "UPDATE manuscript_proposals SET status = 'rejected', updated_at = ?1
             WHERE id = ?2 AND status = 'draft'",
            rusqlite::params![now_utc(), id],
        )?;
        drop(connection);
        self.get(id)
    }
}

fn map_proposal(row: &rusqlite::Row<'_>) -> rusqlite::Result<ManuscriptProposal> {
    let content_format: String = row.get(5)?;
    let content_format = ManuscriptContentFormat::try_from(content_format.as_str())
        .map_err(|_| enum_error("content format"))?;
    let status: String = row.get(7)?;
    let status = match status.as_str() {
        "draft" => ProposalStatus::Draft,
        "accepted" => ProposalStatus::Accepted,
        "rejected" => ProposalStatus::Rejected,
        _ => return Err(enum_error("proposal status")),
    };
    let actor_type: String = row.get(8)?;
    let actor_type = match actor_type.as_str() {
        "user" => ActorType::User,
        "ai" => ActorType::Ai,
        "system" => ActorType::System,
        _ => return Err(enum_error("actor type")),
    };
    Ok(ManuscriptProposal {
        id: row.get(0)?,
        project_id: row.get(1)?,
        chapter_id: row.get(2)?,
        base_revision: row.get::<_, i64>(3)? as u64,
        proposed_content: row.get(4)?,
        content_format,
        rationale: row.get(6)?,
        status,
        actor_type,
        actor_id: row.get(9)?,
        created_at: row.get(10)?,
        updated_at: row.get(11)?,
    })
}

fn map_manuscript(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<crate::domain::manuscript::Manuscript> {
    let content_format: String = row.get(3)?;
    let content_format = ManuscriptContentFormat::try_from(content_format.as_str())
        .map_err(|_| enum_error("manuscript format"))?;
    Ok(crate::domain::manuscript::Manuscript {
        id: row.get(0)?,
        chapter_id: row.get(1)?,
        content: row.get(2)?,
        content_format,
        revision: row.get::<_, i64>(4)? as u64,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
    })
}

fn map_not_found(error: rusqlite::Error) -> AppError {
    match error {
        rusqlite::Error::QueryReturnedNoRows => AppError::NotFound,
        other => other.into(),
    }
}

fn enum_error(value: &str) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(
        0,
        rusqlite::types::Type::Text,
        Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("invalid manuscript proposal {value}"),
        )),
    )
}
