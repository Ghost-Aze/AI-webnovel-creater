CREATE TABLE IF NOT EXISTS conversation_runtime_settings (
    conversation_id TEXT PRIMARY KEY NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    assistant_id TEXT NOT NULL,
    provider_id TEXT,
    model_id TEXT,
    quality TEXT NOT NULL DEFAULT 'balanced'
        CHECK (quality IN ('fast', 'balanced', 'deep')),
    temperature REAL CHECK (temperature IS NULL OR (temperature >= 0.0 AND temperature <= 2.0)),
    updated_at TEXT NOT NULL
);

