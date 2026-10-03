CREATE TABLE IF NOT EXISTS story_facts (
    id TEXT PRIMARY KEY NOT NULL,
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('active', 'archived')),
    canon_status TEXT NOT NULL CHECK (canon_status IN ('canon', 'locked_canon')),
    revision INTEGER NOT NULL CHECK (revision > 0),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_story_facts_project_status
    ON story_facts (project_id, status, updated_at DESC);

CREATE TABLE IF NOT EXISTS canon_rules (
    id TEXT PRIMARY KEY NOT NULL,
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    rule TEXT NOT NULL,
    scope TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('active', 'archived')),
    canon_status TEXT NOT NULL CHECK (canon_status IN ('canon', 'locked_canon')),
    revision INTEGER NOT NULL CHECK (revision > 0),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_canon_rules_project_status
    ON canon_rules (project_id, status, updated_at DESC);

CREATE TABLE memory_revisions_new (
    id TEXT PRIMARY KEY NOT NULL,
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    entity_type TEXT NOT NULL CHECK (entity_type IN ('character', 'character_state', 'story_fact', 'canon_rule')),
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

INSERT INTO memory_revisions_new (
    id, project_id, entity_type, entity_id, revision, operation, actor_type,
    actor_id, base_revision, previous_value, new_value, source_type, source_id, created_at
)
SELECT id, project_id, entity_type, entity_id, revision, operation, actor_type,
       actor_id, base_revision, previous_value, new_value, source_type, source_id, created_at
FROM memory_revisions;

DROP TABLE memory_revisions;
ALTER TABLE memory_revisions_new RENAME TO memory_revisions;

CREATE INDEX idx_memory_revisions_project_entity
    ON memory_revisions (project_id, entity_type, entity_id, revision DESC);

CREATE TABLE memory_proposals_new (
    id TEXT PRIMARY KEY NOT NULL,
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    entity_type TEXT NOT NULL CHECK (entity_type IN ('character', 'character_state', 'story_fact', 'canon_rule')),
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

INSERT INTO memory_proposals_new (
    id, project_id, entity_type, entity_id, operation, payload, base_revision,
    status, actor_type, actor_id, created_at, updated_at
)
SELECT id, project_id, entity_type, entity_id, operation, payload, base_revision,
       status, actor_type, actor_id, created_at, updated_at
FROM memory_proposals;

DROP TABLE memory_proposals;
ALTER TABLE memory_proposals_new RENAME TO memory_proposals;

CREATE INDEX idx_memory_proposals_project_status
    ON memory_proposals (project_id, status, updated_at DESC);
