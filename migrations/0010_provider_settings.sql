CREATE TABLE IF NOT EXISTS provider_settings (
    provider_id TEXT PRIMARY KEY NOT NULL,
    display_name TEXT NOT NULL,
    base_url TEXT NOT NULL,
    credential_id TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS provider_models (
    provider_id TEXT NOT NULL REFERENCES provider_settings(provider_id) ON DELETE CASCADE,
    model_id TEXT NOT NULL,
    display_name TEXT NOT NULL,
    context_window_tokens INTEGER NOT NULL CHECK (context_window_tokens > 0),
    default_output_tokens INTEGER NOT NULL CHECK (default_output_tokens > 0),
    strengths TEXT NOT NULL,
    weaknesses TEXT NOT NULL,
    strategy TEXT NOT NULL,
    tier TEXT NOT NULL CHECK (tier IN ('local', 'small', 'medium', 'large')),
    capabilities TEXT NOT NULL,
    PRIMARY KEY (provider_id, model_id)
);

CREATE INDEX IF NOT EXISTS idx_provider_models_provider
    ON provider_models (provider_id, model_id);
