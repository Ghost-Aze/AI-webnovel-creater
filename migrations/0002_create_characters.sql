CREATE TABLE IF NOT EXISTS characters (
    id TEXT PRIMARY KEY NOT NULL,
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    summary TEXT NOT NULL DEFAULT '',
    role TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL CHECK (status IN ('active', 'archived')),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE (project_id, name COLLATE NOCASE)
);

CREATE INDEX IF NOT EXISTS idx_characters_project_status_updated_at
    ON characters (project_id, status, updated_at DESC);

CREATE TABLE IF NOT EXISTS character_states (
    character_id TEXT PRIMARY KEY NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
    current_location TEXT NOT NULL DEFAULT '',
    physical_condition TEXT NOT NULL DEFAULT '',
    injuries TEXT NOT NULL DEFAULT '',
    emotional_state TEXT NOT NULL DEFAULT '',
    goals TEXT NOT NULL DEFAULT '',
    beliefs TEXT NOT NULL DEFAULT '',
    knowledge TEXT NOT NULL DEFAULT '',
    secrets_known TEXT NOT NULL DEFAULT '',
    current_conflicts TEXT NOT NULL DEFAULT '',
    possessions TEXT NOT NULL DEFAULT '',
    promises TEXT NOT NULL DEFAULT '',
    last_appearance TEXT NOT NULL DEFAULT '',
    current_arc_role TEXT NOT NULL DEFAULT '',
    updated_at TEXT NOT NULL
);
