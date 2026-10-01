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
