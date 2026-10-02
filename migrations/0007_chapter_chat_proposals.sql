ALTER TABLE conversations ADD COLUMN chapter_id TEXT REFERENCES chapters(id) ON DELETE CASCADE;

CREATE INDEX IF NOT EXISTS idx_conversations_chapter_updated
    ON conversations (chapter_id, updated_at DESC);

CREATE TABLE IF NOT EXISTS manuscript_proposals (
    id TEXT PRIMARY KEY NOT NULL,
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    chapter_id TEXT NOT NULL REFERENCES chapters(id) ON DELETE CASCADE,
    base_revision INTEGER NOT NULL CHECK (base_revision > 0),
    proposed_content TEXT NOT NULL,
    content_format TEXT NOT NULL CHECK (content_format IN ('plain_text', 'html')),
    rationale TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'accepted', 'rejected')),
    actor_type TEXT NOT NULL CHECK (actor_type IN ('user', 'ai', 'system')),
    actor_id TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_manuscript_proposals_chapter_status
    ON manuscript_proposals (chapter_id, status, updated_at DESC);
