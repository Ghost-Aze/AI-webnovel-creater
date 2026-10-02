ALTER TABLE manuscripts ADD COLUMN content_format TEXT NOT NULL DEFAULT 'plain_text'
    CHECK (content_format IN ('plain_text', 'html'));

ALTER TABLE manuscript_revisions ADD COLUMN content_format TEXT NOT NULL DEFAULT 'plain_text'
    CHECK (content_format IN ('plain_text', 'html'));
