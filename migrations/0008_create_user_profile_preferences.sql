CREATE TABLE IF NOT EXISTS user_profile (
    id TEXT PRIMARY KEY NOT NULL,
    display_name TEXT NOT NULL,
    preferred_language TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    revision INTEGER NOT NULL CHECK (revision > 0)
);

CREATE TABLE IF NOT EXISTS user_preferences (
    id TEXT PRIMARY KEY NOT NULL,
    preferred_narrator TEXT NOT NULL,
    preferred_pov TEXT NOT NULL,
    chapter_length INTEGER NOT NULL CHECK (chapter_length > 0),
    scene_length INTEGER NOT NULL CHECK (scene_length > 0),
    dialogue_density INTEGER NOT NULL CHECK (dialogue_density BETWEEN 0 AND 100),
    prose_level TEXT NOT NULL,
    pacing TEXT NOT NULL,
    avoid_repetition INTEGER NOT NULL CHECK (avoid_repetition IN (0, 1)),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    revision INTEGER NOT NULL CHECK (revision > 0)
);

INSERT INTO user_profile (
    id, display_name, preferred_language, created_at, updated_at, revision
)
SELECT 'local_user', 'Writer', 'en', '2026-01-01T00:00:00.000000000Z',
       '2026-01-01T00:00:00.000000000Z', 1
WHERE NOT EXISTS (SELECT 1 FROM user_profile WHERE id = 'local_user');

INSERT INTO user_preferences (
    id, preferred_narrator, preferred_pov, chapter_length, scene_length,
    dialogue_density, prose_level, pacing, avoid_repetition, created_at,
    updated_at, revision
)
SELECT 'local_user', 'third_person', 'limited', 2000, 600, 40, 'standard',
       'balanced', 1, '2026-01-01T00:00:00.000000000Z',
       '2026-01-01T00:00:00.000000000Z', 1
WHERE NOT EXISTS (SELECT 1 FROM user_preferences WHERE id = 'local_user');
