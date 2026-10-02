use crate::{
    db::SharedConnection,
    domain::{
        manuscript::{Chapter, ChapterListFilter, ChapterStatus, Manuscript, ManuscriptRevision},
        revision::ActorType,
    },
    error::{AppError, AppResult},
};

#[derive(Clone)]
pub struct ManuscriptRepository {
    connection: SharedConnection,
}

pub struct ChapterUpdateData<'a> {
    pub number: u32,
    pub title: &'a str,
    pub synopsis: &'a str,
    pub status: ChapterStatus,
    pub updated_at: &'a str,
    pub expected_revision: u64,
}

pub struct ManuscriptSaveData<'a> {
    pub content: &'a str,
    pub label: &'a str,
    pub actor_type: ActorType,
    pub actor_id: Option<&'a str>,
    pub expected_revision: u64,
    pub updated_at: &'a str,
}

impl ManuscriptRepository {
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
            .map_err(map_not_found)?;
        if status == "archived" {
            return Err(AppError::ArchivedProject);
        }
        Ok(())
    }

    pub fn create_chapter(&self, chapter: &Chapter) -> AppResult<Chapter> {
        let connection = self.connection.lock()?;
        let transaction = connection.unchecked_transaction()?;
        transaction
            .execute(
                "INSERT INTO chapters
                 (id, project_id, number, title, synopsis, status, revision, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                rusqlite::params![
                    chapter.id,
                    chapter.project_id,
                    i64::from(chapter.number),
                    chapter.title,
                    chapter.synopsis,
                    chapter.status.as_str(),
                    chapter.revision as i64,
                    chapter.created_at,
                    chapter.updated_at,
                ],
            )
            .map_err(map_chapter_write_error)?;
        let manuscript_id = crate::domain::project::new_id();
        transaction.execute(
            "INSERT INTO manuscripts (id, chapter_id, content, revision, created_at, updated_at)
             VALUES (?1, ?2, '', 1, ?3, ?3)",
            rusqlite::params![manuscript_id, chapter.id, chapter.created_at],
        )?;
        transaction.execute(
            "INSERT INTO manuscript_revisions
             (id, manuscript_id, revision, content, label, actor_type, actor_id, created_at)
             VALUES (?1, ?2, 1, '', 'Initial draft', 'user', NULL, ?3)",
            rusqlite::params![
                crate::domain::project::new_id(),
                manuscript_id,
                chapter.created_at
            ],
        )?;
        transaction.commit()?;
        Ok(chapter.clone())
    }

    pub fn get_chapter(&self, id: &str) -> AppResult<Chapter> {
        let connection = self.connection.lock()?;
        connection
            .query_row(
                "SELECT id, project_id, number, title, synopsis, status, revision, created_at, updated_at
                 FROM chapters WHERE id = ?1",
                [id],
                map_chapter,
            )
            .map_err(map_not_found)
    }

    pub fn list_chapters(
        &self,
        project_id: &str,
        filter: ChapterListFilter,
    ) -> AppResult<Vec<Chapter>> {
        let connection = self.connection.lock()?;
        let mut statement = if filter.include_archived {
            connection.prepare(
                "SELECT id, project_id, number, title, synopsis, status, revision, created_at, updated_at
                 FROM chapters WHERE project_id = ?1 ORDER BY number ASC",
            )?
        } else {
            connection.prepare(
                "SELECT id, project_id, number, title, synopsis, status, revision, created_at, updated_at
                 FROM chapters WHERE project_id = ?1 AND status != 'archived' ORDER BY number ASC",
            )?
        };
        let rows = statement.query_map([project_id], map_chapter)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn update_chapter(&self, id: &str, data: ChapterUpdateData<'_>) -> AppResult<Chapter> {
        let connection = self.connection.lock()?;
        let existing = connection
            .query_row(
                "SELECT id, project_id, number, title, synopsis, status, revision, created_at, updated_at
                 FROM chapters WHERE id = ?1",
                [id],
                map_chapter,
            )
            .map_err(map_not_found)?;
        if existing.status == ChapterStatus::Archived {
            return Err(AppError::ArchivedChapter);
        }
        if existing.revision != data.expected_revision {
            return Err(AppError::Conflict);
        }
        let next_revision = data.expected_revision + 1;
        connection
            .execute(
                "UPDATE chapters
                 SET number = ?1, title = ?2, synopsis = ?3, status = ?4, revision = ?5, updated_at = ?6
                 WHERE id = ?7 AND revision = ?8 AND status != 'archived'",
                rusqlite::params![
                    i64::from(data.number),
                    data.title,
                    data.synopsis,
                    data.status.as_str(),
                    next_revision as i64,
                    data.updated_at,
                    id,
                    data.expected_revision as i64,
                ],
            )
            .map_err(map_chapter_write_error)?;
        drop(connection);
        self.get_chapter(id)
    }

    pub fn archive_chapter(
        &self,
        id: &str,
        updated_at: &str,
        expected_revision: u64,
    ) -> AppResult<Chapter> {
        let connection = self.connection.lock()?;
        let existing = connection
            .query_row(
                "SELECT id, project_id, number, title, synopsis, status, revision, created_at, updated_at
                 FROM chapters WHERE id = ?1",
                [id],
                map_chapter,
            )
            .map_err(map_not_found)?;
        if existing.status == ChapterStatus::Archived {
            return Ok(existing);
        }
        if existing.revision != expected_revision {
            return Err(AppError::Conflict);
        }
        connection.execute(
            "UPDATE chapters SET status = 'archived', revision = ?1, updated_at = ?2
             WHERE id = ?3 AND revision = ?4 AND status != 'archived'",
            rusqlite::params![
                (expected_revision + 1) as i64,
                updated_at,
                id,
                expected_revision as i64,
            ],
        )?;
        drop(connection);
        self.get_chapter(id)
    }

    pub fn get_manuscript(&self, chapter_id: &str) -> AppResult<Manuscript> {
        let connection = self.connection.lock()?;
        connection
            .query_row(
                "SELECT id, chapter_id, content, revision, created_at, updated_at
                 FROM manuscripts WHERE chapter_id = ?1",
                [chapter_id],
                map_manuscript,
            )
            .map_err(map_not_found)
    }

    pub fn save_manuscript(
        &self,
        chapter_id: &str,
        data: ManuscriptSaveData<'_>,
    ) -> AppResult<Manuscript> {
        let connection = self.connection.lock()?;
        let transaction = connection.unchecked_transaction()?;
        let chapter_status: String = transaction
            .query_row(
                "SELECT status FROM chapters WHERE id = ?1",
                [chapter_id],
                |row| row.get(0),
            )
            .map_err(map_not_found)?;
        if chapter_status == "archived" {
            return Err(AppError::ArchivedChapter);
        }
        let existing = transaction
            .query_row(
                "SELECT id, chapter_id, content, revision, created_at, updated_at
                 FROM manuscripts WHERE chapter_id = ?1",
                [chapter_id],
                map_manuscript,
            )
            .map_err(map_not_found)?;
        if existing.revision != data.expected_revision {
            return Err(AppError::Conflict);
        }
        let current = Manuscript {
            id: existing.id.clone(),
            chapter_id: existing.chapter_id.clone(),
            content: data.content.to_string(),
            revision: data.expected_revision + 1,
            created_at: existing.created_at.clone(),
            updated_at: data.updated_at.to_string(),
        };
        transaction.execute(
            "UPDATE manuscripts SET content = ?1, revision = ?2, updated_at = ?3
             WHERE id = ?4 AND revision = ?5",
            rusqlite::params![
                current.content,
                current.revision as i64,
                current.updated_at,
                current.id,
                data.expected_revision as i64,
            ],
        )?;
        transaction.execute(
            "INSERT INTO manuscript_revisions
             (id, manuscript_id, revision, content, label, actor_type, actor_id, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                crate::domain::project::new_id(),
                current.id,
                current.revision as i64,
                current.content,
                data.label,
                data.actor_type.as_str(),
                data.actor_id,
                data.updated_at,
            ],
        )?;
        transaction.commit()?;
        Ok(current)
    }

    pub fn list_revisions(&self, chapter_id: &str) -> AppResult<Vec<ManuscriptRevision>> {
        let manuscript = self.get_manuscript(chapter_id)?;
        let connection = self.connection.lock()?;
        let mut statement = connection.prepare(
            "SELECT id, manuscript_id, revision, content, label, actor_type, actor_id, created_at
             FROM manuscript_revisions WHERE manuscript_id = ?1 ORDER BY revision DESC",
        )?;
        let rows = statement.query_map([manuscript.id], map_manuscript_revision)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn get_revision(&self, chapter_id: &str, revision: u64) -> AppResult<ManuscriptRevision> {
        let manuscript = self.get_manuscript(chapter_id)?;
        let connection = self.connection.lock()?;
        connection
            .query_row(
                "SELECT id, manuscript_id, revision, content, label, actor_type, actor_id, created_at
                 FROM manuscript_revisions WHERE manuscript_id = ?1 AND revision = ?2",
                rusqlite::params![manuscript.id, revision as i64],
                map_manuscript_revision,
            )
            .map_err(map_not_found)
    }
}

fn map_chapter(row: &rusqlite::Row<'_>) -> rusqlite::Result<Chapter> {
    let status: String = row.get(5)?;
    let status = ChapterStatus::try_from(status.as_str()).map_err(|_| {
        rusqlite::Error::FromSqlConversionFailure(
            5,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid chapter status",
            )),
        )
    })?;
    Ok(Chapter {
        id: row.get(0)?,
        project_id: row.get(1)?,
        number: row.get::<_, i64>(2)? as u32,
        title: row.get(3)?,
        synopsis: row.get(4)?,
        status,
        revision: row.get::<_, i64>(6)? as u64,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

fn map_manuscript(row: &rusqlite::Row<'_>) -> rusqlite::Result<Manuscript> {
    Ok(Manuscript {
        id: row.get(0)?,
        chapter_id: row.get(1)?,
        content: row.get(2)?,
        revision: row.get::<_, i64>(3)? as u64,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
}

fn map_manuscript_revision(row: &rusqlite::Row<'_>) -> rusqlite::Result<ManuscriptRevision> {
    let actor_type: String = row.get(5)?;
    let actor_type = match actor_type.as_str() {
        "user" => ActorType::User,
        "ai" => ActorType::Ai,
        "system" => ActorType::System,
        _ => {
            return Err(rusqlite::Error::FromSqlConversionFailure(
                5,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "invalid actor type",
                )),
            ));
        }
    };
    Ok(ManuscriptRevision {
        id: row.get(0)?,
        manuscript_id: row.get(1)?,
        revision: row.get::<_, i64>(2)? as u64,
        content: row.get(3)?,
        label: row.get(4)?,
        actor_type,
        actor_id: row.get(6)?,
        created_at: row.get(7)?,
    })
}

fn map_not_found(error: rusqlite::Error) -> AppError {
    match error {
        rusqlite::Error::QueryReturnedNoRows => AppError::NotFound,
        other => other.into(),
    }
}

fn map_chapter_write_error(error: rusqlite::Error) -> AppError {
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
        AppError::DuplicateChapterNumber
    } else {
        error.into()
    }
}
