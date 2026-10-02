CREATE TABLE IF NOT EXISTS chapters (
    id TEXT PRIMARY KEY NOT NULL,
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    number INTEGER NOT NULL CHECK (number > 0),
    title TEXT NOT NULL CHECK (length(trim(title)) > 0),
    synopsis TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'final', 'archived')),
    revision INTEGER NOT NULL DEFAULT 1 CHECK (revision > 0),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE (project_id, number)
);

CREATE INDEX IF NOT EXISTS idx_chapters_project_status_number
    ON chapters (project_id, status, number ASC);

CREATE TABLE IF NOT EXISTS manuscripts (
    id TEXT PRIMARY KEY NOT NULL,
    chapter_id TEXT NOT NULL UNIQUE REFERENCES chapters(id) ON DELETE CASCADE,
    content TEXT NOT NULL DEFAULT '',
    revision INTEGER NOT NULL DEFAULT 1 CHECK (revision > 0),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS manuscript_revisions (
    id TEXT PRIMARY KEY NOT NULL,
    manuscript_id TEXT NOT NULL REFERENCES manuscripts(id) ON DELETE CASCADE,
    revision INTEGER NOT NULL CHECK (revision > 0),
    content TEXT NOT NULL,
    label TEXT NOT NULL DEFAULT '',
    actor_type TEXT NOT NULL CHECK (actor_type IN ('user', 'ai', 'system')),
    actor_id TEXT,
    created_at TEXT NOT NULL,
    UNIQUE (manuscript_id, revision)
);

CREATE INDEX IF NOT EXISTS idx_manuscript_revisions_manuscript_revision
    ON manuscript_revisions (manuscript_id, revision DESC);
