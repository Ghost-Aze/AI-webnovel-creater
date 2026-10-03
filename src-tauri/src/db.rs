use std::{
    path::Path,
    sync::{Arc, Mutex},
};

use rusqlite::Connection;

use crate::error::{AppError, AppResult};

struct Migration {
    name: &'static str,
    sql: &'static str,
}

const MIGRATIONS: &[Migration] = &[
    Migration {
        name: "0001_create_projects.sql",
        sql: include_str!("../../migrations/0001_create_projects.sql"),
    },
    Migration {
        name: "0002_create_characters.sql",
        sql: include_str!("../../migrations/0002_create_characters.sql"),
    },
    Migration {
        name: "0003_create_memory_revisions.sql",
        sql: include_str!("../../migrations/0003_create_memory_revisions.sql"),
    },
    Migration {
        name: "0004_create_conversations.sql",
        sql: include_str!("../../migrations/0004_create_conversations.sql"),
    },
    Migration {
        name: "0005_create_manuscripts.sql",
        sql: include_str!("../../migrations/0005_create_manuscripts.sql"),
    },
    Migration {
        name: "0006_manuscript_content_format.sql",
        sql: include_str!("../../migrations/0006_manuscript_content_format.sql"),
    },
    Migration {
        name: "0007_chapter_chat_proposals.sql",
        sql: include_str!("../../migrations/0007_chapter_chat_proposals.sql"),
    },
    Migration {
        name: "0008_create_user_profile_preferences.sql",
        sql: include_str!("../../migrations/0008_create_user_profile_preferences.sql"),
    },
    Migration {
        name: "0009_project_memory.sql",
        sql: include_str!("../../migrations/0009_project_memory.sql"),
    },
    Migration {
        name: "0010_provider_settings.sql",
        sql: include_str!("../../migrations/0010_provider_settings.sql"),
    },
];

pub type SharedConnection = Arc<Mutex<Connection>>;

pub fn open(path: impl AsRef<Path>) -> AppResult<SharedConnection> {
    let connection = Connection::open(path).map_err(|_| AppError::Storage)?;
    run_migrations(&connection)?;
    Ok(Arc::new(Mutex::new(connection)))
}

pub fn in_memory() -> AppResult<SharedConnection> {
    let connection = Connection::open_in_memory().map_err(|_| AppError::Storage)?;
    run_migrations(&connection)?;
    Ok(Arc::new(Mutex::new(connection)))
}

pub fn run_migrations(connection: &Connection) -> AppResult<()> {
    connection.execute_batch("PRAGMA foreign_keys = ON;")?;
    connection.execute_batch(
        "CREATE TABLE IF NOT EXISTS _migrations (
            name TEXT PRIMARY KEY NOT NULL,
            applied_at TEXT NOT NULL
        );",
    )?;

    for migration in MIGRATIONS {
        let already_applied: Option<String> = connection
            .query_row(
                "SELECT name FROM _migrations WHERE name = ?1",
                [migration.name],
                |row| row.get(0),
            )
            .optional()?;

        if already_applied.is_none() {
            let transaction = connection.unchecked_transaction()?;
            transaction.execute_batch(migration.sql)?;
            transaction.execute(
                "INSERT INTO _migrations (name, applied_at) VALUES (?1, ?2)",
                rusqlite::params![migration.name, crate::domain::project::now_utc()],
            )?;
            transaction.commit()?;
        }
    }

    Ok(())
}

use rusqlite::OptionalExtension;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_profile_and_preferences_migration_is_idempotent_with_deterministic_defaults() {
        let connection = in_memory().expect("in-memory database should initialize");
        let guard = connection.lock().expect("connection lock should succeed");

        run_migrations(&guard).expect("running migrations twice should succeed");

        let profile: (String, String, i64) = guard
            .query_row(
                "SELECT id, display_name, revision FROM user_profile",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("default profile should exist");
        assert_eq!(profile, ("local_user".to_string(), "Writer".to_string(), 1));
        assert_eq!(
            guard
                .query_row::<String, _, _>(
                    "SELECT preferred_language FROM user_profile WHERE id = 'local_user'",
                    [],
                    |row| row.get(0),
                )
                .unwrap(),
            "en"
        );

        let preferences: (String, String, i64, i64, i64, String, String, i64) = guard
            .query_row(
                "SELECT preferred_narrator, preferred_pov, chapter_length, scene_length,
                        dialogue_density, prose_level, pacing, avoid_repetition
                 FROM user_preferences WHERE id = 'local_user'",
                [],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                        row.get(6)?,
                        row.get(7)?,
                    ))
                },
            )
            .expect("default preferences should exist");
        assert_eq!(
            preferences,
            (
                "third_person".to_string(),
                "limited".to_string(),
                2000,
                600,
                40,
                "standard".to_string(),
                "balanced".to_string(),
                1,
            )
        );

        assert_eq!(
            guard
                .query_row::<i64, _, _>(
                    "SELECT COUNT(*) FROM user_profile WHERE id = 'local_user'",
                    [],
                    |row| row.get(0),
                )
                .unwrap(),
            1
        );
        assert_eq!(
            guard
                .query_row::<i64, _, _>(
                    "SELECT COUNT(*) FROM user_preferences WHERE id = 'local_user'",
                    [],
                    |row| row.get(0),
                )
                .unwrap(),
            1
        );
        assert_eq!(
            guard
                .query_row::<i64, _, _>(
                    "SELECT COUNT(*) FROM _migrations WHERE name = '0008_create_user_profile_preferences.sql'",
                    [],
                    |row| row.get(0),
                )
                .unwrap(),
            1
        );
    }

    #[test]
    fn project_memory_migration_is_idempotent_and_preserves_revision_indexes() {
        let connection = in_memory().expect("in-memory database should initialize");
        let guard = connection.lock().expect("connection lock should succeed");

        guard
            .execute(
                "INSERT INTO projects (id, name, description, status, created_at, updated_at)
                 VALUES ('project', 'Project', '', 'active', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
                [],
            )
            .expect("legacy project should exist");
        guard
            .execute(
                "INSERT INTO memory_revisions
                 (id, project_id, entity_type, entity_id, revision, operation, actor_type,
                  actor_id, base_revision, previous_value, new_value, source_type, source_id, created_at)
                 VALUES ('legacy-revision', 'project', 'character', 'character', 1, 'create',
                         'user', NULL, 0, NULL, '{}', 'manual', NULL, '2026-01-01T00:00:00Z')",
                [],
            )
            .expect("legacy revision should remain insertable");
        guard
            .execute(
                "INSERT INTO memory_proposals
                 (id, project_id, entity_type, entity_id, operation, payload, base_revision,
                  status, actor_type, actor_id, created_at, updated_at)
                 VALUES ('legacy-proposal', 'project', 'character', 'character', 'update',
                         '{}', 1, 'draft', 'ai', NULL, '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
                [],
            )
            .expect("legacy proposal should remain insertable");

        run_migrations(&guard).expect("running migrations twice should succeed");

        assert_eq!(
            guard
                .query_row::<i64, _, _>("SELECT COUNT(*) FROM story_facts", [], |row| row.get(0),)
                .unwrap(),
            0
        );
        assert_eq!(
            guard
                .query_row::<String, _, _>(
                    "SELECT entity_type FROM memory_revisions WHERE id = 'legacy-revision'",
                    [],
                    |row| row.get(0),
                )
                .unwrap(),
            "character"
        );
        assert_eq!(
            guard
                .query_row::<i64, _, _>(
                    "SELECT COUNT(*) FROM _migrations WHERE name = '0009_project_memory.sql'",
                    [],
                    |row| row.get(0),
                )
                .unwrap(),
            1
        );
    }
}
