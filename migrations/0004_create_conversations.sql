CREATE TABLE IF NOT EXISTS conversations (
    id TEXT PRIMARY KEY NOT NULL,
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    kind TEXT NOT NULL CHECK (kind IN ('developer_chat', 'arc_chat', 'chapter_chat')),
    title TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_conversations_project_updated
    ON conversations (project_id, updated_at DESC);

CREATE TABLE IF NOT EXISTS conversation_messages (
    id TEXT PRIMARY KEY NOT NULL,
    conversation_id TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    sequence INTEGER NOT NULL CHECK (sequence > 0),
    role TEXT NOT NULL CHECK (role IN ('system', 'user', 'assistant', 'tool')),
    content TEXT NOT NULL CHECK (length(trim(content)) > 0),
    model_provider_id TEXT,
    model_id TEXT,
    created_at TEXT NOT NULL,
    UNIQUE (conversation_id, sequence)
);

CREATE INDEX IF NOT EXISTS idx_conversation_messages_conversation_sequence
    ON conversation_messages (conversation_id, sequence ASC);
