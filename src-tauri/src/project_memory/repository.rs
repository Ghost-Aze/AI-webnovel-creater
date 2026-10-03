use rusqlite::{params, OptionalExtension, Transaction};
use serde::Serialize;

use crate::{
    db::SharedConnection,
    domain::{
        project::{new_id, now_utc},
        project_memory::{CanonRule, ProjectMemoryListFilter, ProjectMemoryStatus, StoryFact},
        revision::{ActorType, CanonStatus, MemoryEntityType, MemoryRevision, RevisionOperation},
    },
    error::{AppError, AppResult},
    revisions::repository::RevisionRepository,
};

#[derive(Clone)]
pub struct ProjectMemoryRepository {
    connection: SharedConnection,
}

impl ProjectMemoryRepository {
    pub fn new(connection: SharedConnection) -> Self {
        Self { connection }
    }

    pub fn create_story_fact(&self, fact: &StoryFact) -> AppResult<StoryFact> {
        let connection = self.connection.lock()?;
        let transaction = connection.unchecked_transaction()?;
        transaction.execute(
            "INSERT INTO story_facts
             (id, project_id, title, content, status, canon_status, revision, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                fact.id,
                fact.project_id,
                fact.title,
                fact.content,
                fact.status.as_str(),
                fact.canon_status.as_str(),
                fact.revision as i64,
                fact.created_at,
                fact.updated_at,
            ],
        )?;
        let revision = build_revision(
            &fact.project_id,
            MemoryEntityType::StoryFact,
            &fact.id,
            fact.revision,
            RevisionOperation::Create,
            0,
            None,
            fact,
            "manual",
            None,
        )?;
        RevisionRepository::insert_tx(&transaction, &revision)?;
        transaction.commit()?;
        Ok(fact.clone())
    }

    pub fn list_story_facts(
        &self,
        project_id: &str,
        filter: ProjectMemoryListFilter,
    ) -> AppResult<Vec<StoryFact>> {
        let connection = self.connection.lock()?;
        let mut statement = if filter.include_archived {
            connection.prepare(
                "SELECT id, project_id, title, content, status, canon_status, revision,
                        created_at, updated_at
                 FROM story_facts WHERE project_id = ?1 ORDER BY updated_at DESC",
            )?
        } else {
            connection.prepare(
                "SELECT id, project_id, title, content, status, canon_status, revision,
                        created_at, updated_at
                 FROM story_facts WHERE project_id = ?1 AND status = 'active'
                 ORDER BY updated_at DESC",
            )?
        };
        let rows = statement.query_map([project_id], map_story_fact)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn get_story_fact(&self, id: &str) -> AppResult<StoryFact> {
        let connection = self.connection.lock()?;
        connection
            .query_row(
                "SELECT id, project_id, title, content, status, canon_status, revision,
                        created_at, updated_at
                 FROM story_facts WHERE id = ?1",
                [id],
                map_story_fact,
            )
            .optional()?
            .ok_or(AppError::NotFound)
    }

    pub fn update_story_fact(
        &self,
        id: &str,
        title: &str,
        content: &str,
        updated_at: &str,
        expected_revision: u64,
    ) -> AppResult<StoryFact> {
        let connection = self.connection.lock()?;
        let transaction = connection.unchecked_transaction()?;
        let existing = transaction
            .query_row(
                "SELECT id, project_id, title, content, status, canon_status, revision,
                        created_at, updated_at
                 FROM story_facts WHERE id = ?1",
                [id],
                map_story_fact,
            )
            .optional()?
            .ok_or(AppError::NotFound)?;
        if existing.status == ProjectMemoryStatus::Archived {
            return Err(AppError::ArchivedMemory);
        }
        if existing.canon_status == CanonStatus::LockedCanon {
            return Err(AppError::LockedCanon);
        }
        if existing.revision != expected_revision {
            return Err(AppError::Conflict);
        }
        let mut current = existing.clone();
        current.title = title.to_string();
        current.content = content.to_string();
        current.revision = expected_revision + 1;
        current.updated_at = updated_at.to_string();
        let changed = transaction.execute(
            "UPDATE story_facts SET title = ?1, content = ?2, revision = ?3, updated_at = ?4
             WHERE id = ?5 AND revision = ?6 AND status = 'active'",
            params![
                current.title,
                current.content,
                current.revision as i64,
                current.updated_at,
                id,
                expected_revision as i64,
            ],
        )?;
        if changed != 1 {
            return Err(AppError::Conflict);
        }
        let revision = build_revision(
            &current.project_id,
            MemoryEntityType::StoryFact,
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

    pub fn archive_story_fact(
        &self,
        id: &str,
        updated_at: &str,
        expected_revision: u64,
    ) -> AppResult<StoryFact> {
        let connection = self.connection.lock()?;
        let transaction = connection.unchecked_transaction()?;
        let existing = transaction
            .query_row(
                "SELECT id, project_id, title, content, status, canon_status, revision,
                        created_at, updated_at
                 FROM story_facts WHERE id = ?1",
                [id],
                map_story_fact,
            )
            .optional()?
            .ok_or(AppError::NotFound)?;
        if existing.status == ProjectMemoryStatus::Archived {
            return Ok(existing);
        }
        if existing.canon_status == CanonStatus::LockedCanon {
            return Err(AppError::LockedCanon);
        }
        if existing.revision != expected_revision {
            return Err(AppError::Conflict);
        }
        let mut current = existing.clone();
        current.status = ProjectMemoryStatus::Archived;
        current.revision = expected_revision + 1;
        current.updated_at = updated_at.to_string();
        let changed = transaction.execute(
            "UPDATE story_facts SET status = 'archived', revision = ?1, updated_at = ?2
             WHERE id = ?3 AND revision = ?4 AND status = 'active'",
            params![
                current.revision as i64,
                current.updated_at,
                id,
                expected_revision as i64,
            ],
        )?;
        if changed != 1 {
            return Err(AppError::Conflict);
        }
        let revision = build_revision(
            &current.project_id,
            MemoryEntityType::StoryFact,
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

    pub fn create_canon_rule(&self, rule: &CanonRule) -> AppResult<CanonRule> {
        let connection = self.connection.lock()?;
        let transaction = connection.unchecked_transaction()?;
        transaction.execute(
            "INSERT INTO canon_rules
             (id, project_id, title, rule, scope, status, canon_status, revision, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                rule.id,
                rule.project_id,
                rule.title,
                rule.rule,
                rule.scope,
                rule.status.as_str(),
                rule.canon_status.as_str(),
                rule.revision as i64,
                rule.created_at,
                rule.updated_at,
            ],
        )?;
        let revision = build_revision(
            &rule.project_id,
            MemoryEntityType::CanonRule,
            &rule.id,
            rule.revision,
            RevisionOperation::Create,
            0,
            None,
            rule,
            "manual",
            None,
        )?;
        RevisionRepository::insert_tx(&transaction, &revision)?;
        transaction.commit()?;
        Ok(rule.clone())
    }

    pub fn list_canon_rules(
        &self,
        project_id: &str,
        filter: ProjectMemoryListFilter,
    ) -> AppResult<Vec<CanonRule>> {
        let connection = self.connection.lock()?;
        let mut statement = if filter.include_archived {
            connection.prepare(
                "SELECT id, project_id, title, rule, scope, status, canon_status, revision,
                        created_at, updated_at
                 FROM canon_rules WHERE project_id = ?1 ORDER BY updated_at DESC",
            )?
        } else {
            connection.prepare(
                "SELECT id, project_id, title, rule, scope, status, canon_status, revision,
                        created_at, updated_at
                 FROM canon_rules WHERE project_id = ?1 AND status = 'active'
                 ORDER BY updated_at DESC",
            )?
        };
        let rows = statement.query_map([project_id], map_canon_rule)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn get_canon_rule(&self, id: &str) -> AppResult<CanonRule> {
        let connection = self.connection.lock()?;
        connection
            .query_row(
                "SELECT id, project_id, title, rule, scope, status, canon_status, revision,
                        created_at, updated_at
                 FROM canon_rules WHERE id = ?1",
                [id],
                map_canon_rule,
            )
            .optional()?
            .ok_or(AppError::NotFound)
    }

    pub fn update_canon_rule(
        &self,
        id: &str,
        title: &str,
        rule: &str,
        scope: &str,
        updated_at: &str,
        expected_revision: u64,
    ) -> AppResult<CanonRule> {
        let connection = self.connection.lock()?;
        let transaction = connection.unchecked_transaction()?;
        let existing = transaction
            .query_row(
                "SELECT id, project_id, title, rule, scope, status, canon_status, revision,
                        created_at, updated_at
                 FROM canon_rules WHERE id = ?1",
                [id],
                map_canon_rule,
            )
            .optional()?
            .ok_or(AppError::NotFound)?;
        if existing.status == ProjectMemoryStatus::Archived {
            return Err(AppError::ArchivedMemory);
        }
        if existing.canon_status == CanonStatus::LockedCanon {
            return Err(AppError::LockedCanon);
        }
        if existing.revision != expected_revision {
            return Err(AppError::Conflict);
        }
        let mut current = existing.clone();
        current.title = title.to_string();
        current.rule = rule.to_string();
        current.scope = scope.to_string();
        current.revision = expected_revision + 1;
        current.updated_at = updated_at.to_string();
        let changed = transaction.execute(
            "UPDATE canon_rules SET title = ?1, rule = ?2, scope = ?3, revision = ?4, updated_at = ?5
             WHERE id = ?6 AND revision = ?7 AND status = 'active'",
            params![
                current.title,
                current.rule,
                current.scope,
                current.revision as i64,
                current.updated_at,
                id,
                expected_revision as i64,
            ],
        )?;
        if changed != 1 {
            return Err(AppError::Conflict);
        }
        let revision = build_revision(
            &current.project_id,
            MemoryEntityType::CanonRule,
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

    pub fn archive_canon_rule(
        &self,
        id: &str,
        updated_at: &str,
        expected_revision: u64,
    ) -> AppResult<CanonRule> {
        let connection = self.connection.lock()?;
        let transaction = connection.unchecked_transaction()?;
        let existing = transaction
            .query_row(
                "SELECT id, project_id, title, rule, scope, status, canon_status, revision,
                        created_at, updated_at
                 FROM canon_rules WHERE id = ?1",
                [id],
                map_canon_rule,
            )
            .optional()?
            .ok_or(AppError::NotFound)?;
        if existing.status == ProjectMemoryStatus::Archived {
            return Ok(existing);
        }
        if existing.canon_status == CanonStatus::LockedCanon {
            return Err(AppError::LockedCanon);
        }
        if existing.revision != expected_revision {
            return Err(AppError::Conflict);
        }
        let mut current = existing.clone();
        current.status = ProjectMemoryStatus::Archived;
        current.revision = expected_revision + 1;
        current.updated_at = updated_at.to_string();
        let changed = transaction.execute(
            "UPDATE canon_rules SET status = 'archived', revision = ?1, updated_at = ?2
             WHERE id = ?3 AND revision = ?4 AND status = 'active'",
            params![
                current.revision as i64,
                current.updated_at,
                id,
                expected_revision as i64,
            ],
        )?;
        if changed != 1 {
            return Err(AppError::Conflict);
        }
        let revision = build_revision(
            &current.project_id,
            MemoryEntityType::CanonRule,
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

    pub fn restore_story_fact(
        &self,
        snapshot: &StoryFact,
        expected_revision: u64,
        target_revision: u64,
    ) -> AppResult<MemoryRevision> {
        let connection = self.connection.lock()?;
        let transaction = connection.unchecked_transaction()?;
        let existing = transaction
            .query_row(
                "SELECT id, project_id, title, content, status, canon_status, revision,
                        created_at, updated_at
                 FROM story_facts WHERE id = ?1",
                [&snapshot.id],
                map_story_fact,
            )
            .optional()?
            .ok_or(AppError::NotFound)?;
        ensure_mutable_memory(&transaction, &existing.project_id, existing.status, existing.canon_status, existing.revision, expected_revision)?;
        let mut current = snapshot.clone();
        current.revision = expected_revision + 1;
        current.updated_at = now_utc();
        let changed = transaction.execute(
            "UPDATE story_facts SET title = ?1, content = ?2, status = ?3, canon_status = ?4,
                    revision = ?5, updated_at = ?6
             WHERE id = ?7 AND revision = ?8 AND status = 'active'",
            params![
                current.title,
                current.content,
                current.status.as_str(),
                current.canon_status.as_str(),
                current.revision as i64,
                current.updated_at,
                current.id,
                expected_revision as i64,
            ],
        )?;
        if changed != 1 {
            return Err(AppError::Conflict);
        }
        let revision = build_revision(
            &current.project_id,
            MemoryEntityType::StoryFact,
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

    pub fn restore_canon_rule(
        &self,
        snapshot: &CanonRule,
        expected_revision: u64,
        target_revision: u64,
    ) -> AppResult<MemoryRevision> {
        let connection = self.connection.lock()?;
        let transaction = connection.unchecked_transaction()?;
        let existing = transaction
            .query_row(
                "SELECT id, project_id, title, rule, scope, status, canon_status, revision,
                        created_at, updated_at
                 FROM canon_rules WHERE id = ?1",
                [&snapshot.id],
                map_canon_rule,
            )
            .optional()?
            .ok_or(AppError::NotFound)?;
        ensure_mutable_memory(&transaction, &existing.project_id, existing.status, existing.canon_status, existing.revision, expected_revision)?;
        let mut current = snapshot.clone();
        current.revision = expected_revision + 1;
        current.updated_at = now_utc();
        let changed = transaction.execute(
            "UPDATE canon_rules SET title = ?1, rule = ?2, scope = ?3, status = ?4,
                    canon_status = ?5, revision = ?6, updated_at = ?7
             WHERE id = ?8 AND revision = ?9 AND status = 'active'",
            params![
                current.title,
                current.rule,
                current.scope,
                current.status.as_str(),
                current.canon_status.as_str(),
                current.revision as i64,
                current.updated_at,
                current.id,
                expected_revision as i64,
            ],
        )?;
        if changed != 1 {
            return Err(AppError::Conflict);
        }
        let revision = build_revision(
            &current.project_id,
            MemoryEntityType::CanonRule,
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

    pub fn set_story_fact_canon_status(
        &self,
        id: &str,
        status: CanonStatus,
        expected_revision: u64,
    ) -> AppResult<MemoryRevision> {
        let connection = self.connection.lock()?;
        let transaction = connection.unchecked_transaction()?;
        let existing = transaction
            .query_row(
                "SELECT id, project_id, title, content, status, canon_status, revision,
                        created_at, updated_at
                 FROM story_facts WHERE id = ?1",
                [id],
                map_story_fact,
            )
            .optional()?
            .ok_or(AppError::NotFound)?;
        ensure_active_for_canon_status(&transaction, &existing.project_id, existing.status)?;
        if existing.revision != expected_revision {
            return Err(AppError::Conflict);
        }
        let mut current = existing.clone();
        current.canon_status = status;
        current.revision += 1;
        current.updated_at = now_utc();
        let changed = transaction.execute(
            "UPDATE story_facts SET canon_status = ?1, revision = ?2, updated_at = ?3
             WHERE id = ?4 AND revision = ?5 AND status = 'active'",
            params![
                current.canon_status.as_str(),
                current.revision as i64,
                current.updated_at,
                id,
                expected_revision as i64,
            ],
        )?;
        if changed != 1 {
            return Err(AppError::Conflict);
        }
        let revision = build_revision(
            &current.project_id,
            MemoryEntityType::StoryFact,
            id,
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

    pub fn set_canon_rule_canon_status(
        &self,
        id: &str,
        status: CanonStatus,
        expected_revision: u64,
    ) -> AppResult<MemoryRevision> {
        let connection = self.connection.lock()?;
        let transaction = connection.unchecked_transaction()?;
        let existing = transaction
            .query_row(
                "SELECT id, project_id, title, rule, scope, status, canon_status, revision,
                        created_at, updated_at
                 FROM canon_rules WHERE id = ?1",
                [id],
                map_canon_rule,
            )
            .optional()?
            .ok_or(AppError::NotFound)?;
        ensure_active_for_canon_status(&transaction, &existing.project_id, existing.status)?;
        if existing.revision != expected_revision {
            return Err(AppError::Conflict);
        }
        let mut current = existing.clone();
        current.canon_status = status;
        current.revision += 1;
        current.updated_at = now_utc();
        let changed = transaction.execute(
            "UPDATE canon_rules SET canon_status = ?1, revision = ?2, updated_at = ?3
             WHERE id = ?4 AND revision = ?5 AND status = 'active'",
            params![
                current.canon_status.as_str(),
                current.revision as i64,
                current.updated_at,
                id,
                expected_revision as i64,
            ],
        )?;
        if changed != 1 {
            return Err(AppError::Conflict);
        }
        let revision = build_revision(
            &current.project_id,
            MemoryEntityType::CanonRule,
            id,
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

fn ensure_mutable_memory(
    transaction: &Transaction<'_>,
    project_id: &str,
    status: ProjectMemoryStatus,
    canon_status: CanonStatus,
    current_revision: u64,
    expected_revision: u64,
) -> AppResult<()> {
    ensure_active_for_canon_status(transaction, project_id, status)?;
    if canon_status == CanonStatus::LockedCanon {
        return Err(AppError::LockedCanon);
    }
    if current_revision != expected_revision {
        return Err(AppError::Conflict);
    }
    Ok(())
}

fn ensure_active_for_canon_status(
    transaction: &Transaction<'_>,
    project_id: &str,
    status: ProjectMemoryStatus,
) -> AppResult<()> {
    let project_status: String = transaction
        .query_row(
            "SELECT status FROM projects WHERE id = ?1",
            [project_id],
            |row| row.get(0),
        )
        .optional()?
        .ok_or(AppError::NotFound)?;
    if project_status == "archived" {
        return Err(AppError::ArchivedProject);
    }
    if status == ProjectMemoryStatus::Archived {
        return Err(AppError::ArchivedMemory);
    }
    Ok(())
}

fn map_story_fact(row: &rusqlite::Row<'_>) -> rusqlite::Result<StoryFact> {
    Ok(StoryFact {
        id: row.get(0)?,
        project_id: row.get(1)?,
        title: row.get(2)?,
        content: row.get(3)?,
        status: parse_memory_status(row.get::<_, String>(4)?)?,
        canon_status: parse_canon_status(row.get::<_, String>(5)?)?,
        revision: row.get::<_, i64>(6)? as u64,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

fn map_canon_rule(row: &rusqlite::Row<'_>) -> rusqlite::Result<CanonRule> {
    Ok(CanonRule {
        id: row.get(0)?,
        project_id: row.get(1)?,
        title: row.get(2)?,
        rule: row.get(3)?,
        scope: row.get(4)?,
        status: parse_memory_status(row.get::<_, String>(5)?)?,
        canon_status: parse_canon_status(row.get::<_, String>(6)?)?,
        revision: row.get::<_, i64>(7)? as u64,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
    })
}

fn parse_memory_status(value: String) -> rusqlite::Result<ProjectMemoryStatus> {
    ProjectMemoryStatus::try_from(value.as_str()).map_err(|_| enum_error("memory status"))
}

fn parse_canon_status(value: String) -> rusqlite::Result<CanonStatus> {
    CanonStatus::try_from(value.as_str()).map_err(|_| enum_error("canon status"))
}

fn enum_error(value: &str) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(
        0,
        rusqlite::types::Type::Text,
        Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("invalid project memory {value}"),
        )),
    )
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
