use crate::{
    db::SharedConnection,
    domain::project::{Project, ProjectListFilter, ProjectStatus},
    error::{AppError, AppResult},
};

#[derive(Clone)]
pub struct ProjectRepository {
    connection: SharedConnection,
}

impl ProjectRepository {
    pub fn new(connection: SharedConnection) -> Self {
        Self { connection }
    }

    pub fn create(&self, project: &Project) -> AppResult<Project> {
        let connection = self.connection.lock()?;
        connection.execute(
            "INSERT INTO projects (id, name, description, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                project.id,
                project.name,
                project.description,
                project.status.as_str(),
                project.created_at,
                project.updated_at,
            ],
        )?;
        Ok(project.clone())
    }

    pub fn get(&self, id: &str) -> AppResult<Project> {
        let connection = self.connection.lock()?;
        connection
            .query_row(
                "SELECT id, name, description, status, created_at, updated_at
                 FROM projects WHERE id = ?1",
                [id],
                map_project,
            )
            .map_err(|error| match error {
                rusqlite::Error::QueryReturnedNoRows => AppError::NotFound,
                other => other.into(),
            })
    }

    pub fn chapter_belongs_to_project(&self, chapter_id: &str, project_id: &str) -> AppResult<()> {
        let connection = self.connection.lock()?;
        let owner: String = connection
            .query_row(
                "SELECT project_id FROM chapters WHERE id = ?1",
                [chapter_id],
                |row| row.get(0),
            )
            .map_err(|error| match error {
                rusqlite::Error::QueryReturnedNoRows => AppError::NotFound,
                other => other.into(),
            })?;
        if owner != project_id {
            return Err(AppError::NotFound);
        }
        Ok(())
    }

    pub fn chapter_is_active(&self, chapter_id: &str) -> AppResult<()> {
        let connection = self.connection.lock()?;
        let status: String = connection
            .query_row(
                "SELECT status FROM chapters WHERE id = ?1",
                [chapter_id],
                |row| row.get(0),
            )
            .map_err(|error| match error {
                rusqlite::Error::QueryReturnedNoRows => AppError::NotFound,
                other => other.into(),
            })?;
        if status == "archived" {
            return Err(AppError::ArchivedChapter);
        }
        Ok(())
    }

    pub fn list(&self, filter: ProjectListFilter) -> AppResult<Vec<Project>> {
        let connection = self.connection.lock()?;
        let mut statement = if filter.include_archived {
            connection.prepare(
                "SELECT id, name, description, status, created_at, updated_at
                 FROM projects ORDER BY updated_at DESC",
            )?
        } else {
            connection.prepare(
                "SELECT id, name, description, status, created_at, updated_at
                 FROM projects WHERE status = 'active' ORDER BY updated_at DESC",
            )?
        };
        let rows = statement.query_map([], map_project)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn update(
        &self,
        id: &str,
        name: &str,
        description: &str,
        updated_at: &str,
    ) -> AppResult<Project> {
        let connection = self.connection.lock()?;
        let existing = connection
            .query_row(
                "SELECT id, name, description, status, created_at, updated_at
                 FROM projects WHERE id = ?1",
                [id],
                map_project,
            )
            .map_err(|error| match error {
                rusqlite::Error::QueryReturnedNoRows => AppError::NotFound,
                other => other.into(),
            })?;
        if existing.status == ProjectStatus::Archived {
            return Err(AppError::ArchivedProject);
        }

        connection.execute(
            "UPDATE projects SET name = ?1, description = ?2, updated_at = ?3
             WHERE id = ?4 AND status = 'active'",
            rusqlite::params![name, description, updated_at, id],
        )?;
        drop(connection);
        self.get(id)
    }

    pub fn archive(&self, id: &str, updated_at: &str) -> AppResult<Project> {
        let connection = self.connection.lock()?;
        let changed = connection.execute(
            "UPDATE projects SET status = 'archived', updated_at = ?1
             WHERE id = ?2 AND status = 'active'",
            rusqlite::params![updated_at, id],
        )?;
        if changed == 0 {
            let project = connection
                .query_row(
                    "SELECT id, name, description, status, created_at, updated_at
                     FROM projects WHERE id = ?1",
                    [id],
                    map_project,
                )
                .map_err(|error| match error {
                    rusqlite::Error::QueryReturnedNoRows => AppError::NotFound,
                    other => other.into(),
                })?;
            return Ok(project);
        }
        drop(connection);
        self.get(id)
    }

    pub fn delete(&self, id: &str) -> AppResult<()> {
        let connection = self.connection.lock()?;
        let changed = connection.execute("DELETE FROM projects WHERE id = ?1", [id])?;
        if changed == 0 {
            return Err(AppError::NotFound);
        }
        Ok(())
    }
}

fn map_project(row: &rusqlite::Row<'_>) -> rusqlite::Result<Project> {
    let status: String = row.get(3)?;
    let status = ProjectStatus::try_from(status.as_str()).map_err(|_| {
        rusqlite::Error::FromSqlConversionFailure(
            3,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid project status",
            )),
        )
    })?;
    Ok(Project {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        status,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
}
