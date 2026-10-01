ALTER TABLE characters ADD COLUMN revision INTEGER NOT NULL DEFAULT 0;
ALTER TABLE characters ADD COLUMN canon_status TEXT NOT NULL DEFAULT 'canon'
    CHECK (canon_status IN ('canon', 'locked_canon'));

ALTER TABLE character_states ADD COLUMN revision INTEGER NOT NULL DEFAULT 0;
ALTER TABLE character_states ADD COLUMN canon_status TEXT NOT NULL DEFAULT 'canon'
    CHECK (canon_status IN ('canon', 'locked_canon'));

CREATE TABLE IF NOT EXISTS memory_revisions (
    id TEXT PRIMARY KEY NOT NULL,
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    entity_type TEXT NOT NULL CHECK (entity_type IN ('character', 'character_state')),
    entity_id TEXT NOT NULL,
    revision INTEGER NOT NULL CHECK (revision > 0),
    operation TEXT NOT NULL CHECK (operation IN ('create', 'update', 'archive', 'restore', 'canon_status', 'promote')),
    actor_type TEXT NOT NULL CHECK (actor_type IN ('user', 'ai', 'system')),
    actor_id TEXT,
    base_revision INTEGER NOT NULL CHECK (base_revision >= 0),
    previous_value TEXT,
    new_value TEXT NOT NULL,
    source_type TEXT,
    source_id TEXT,
    created_at TEXT NOT NULL,
    UNIQUE (entity_type, entity_id, revision)
);

CREATE INDEX IF NOT EXISTS idx_memory_revisions_project_entity
    ON memory_revisions (project_id, entity_type, entity_id, revision DESC);

CREATE TABLE IF NOT EXISTS memory_proposals (
    id TEXT PRIMARY KEY NOT NULL,
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    entity_type TEXT NOT NULL CHECK (entity_type IN ('character', 'character_state')),
    entity_id TEXT,
    operation TEXT NOT NULL CHECK (operation IN ('create', 'update', 'archive', 'restore', 'canon_status', 'promote')),
    payload TEXT NOT NULL,
    base_revision INTEGER NOT NULL CHECK (base_revision >= 0),
    status TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'accepted', 'rejected')),
    actor_type TEXT NOT NULL CHECK (actor_type IN ('user', 'ai', 'system')),
    actor_id TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_memory_proposals_project_status
    ON memory_proposals (project_id, status, updated_at DESC);
