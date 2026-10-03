use crate::{
    db::SharedConnection,
    domain::project::now_utc,
    error::{AppError, AppResult},
};

use super::{ModelProfile, ProviderDescriptor, ProviderSettings};

#[derive(Clone)]
pub struct ProviderSettingsRepository {
    connection: SharedConnection,
}

impl ProviderSettingsRepository {
    pub fn new(connection: SharedConnection) -> Self {
        Self { connection }
    }

    pub fn get(&self, provider_id: &str) -> AppResult<ProviderSettings> {
        let connection = self.connection.lock()?;
        self.get_with_connection(&connection, provider_id)
    }

    pub fn list(&self) -> AppResult<Vec<ProviderSettings>> {
        let connection = self.connection.lock()?;
        let mut statement = connection.prepare(
            "SELECT provider_id FROM provider_settings ORDER BY updated_at DESC, provider_id ASC",
        )?;
        let ids = statement
            .query_map([], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        ids.iter()
            .map(|provider_id| self.get_with_connection(&connection, provider_id))
            .collect()
    }

    pub fn save(&self, settings: &ProviderSettings) -> AppResult<()> {
        let connection = self.connection.lock()?;
        let transaction = connection.unchecked_transaction()?;
        let now = now_utc();
        transaction.execute(
            "INSERT INTO provider_settings
                (provider_id, display_name, base_url, credential_id, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?5)
             ON CONFLICT(provider_id) DO UPDATE SET
                display_name = excluded.display_name,
                base_url = excluded.base_url,
                credential_id = excluded.credential_id,
                updated_at = excluded.updated_at",
            rusqlite::params![
                settings.descriptor.id,
                settings.descriptor.display_name,
                settings.base_url,
                settings.credential_id,
                now,
            ],
        )?;
        transaction.execute(
            "DELETE FROM provider_models WHERE provider_id = ?1",
            [&settings.descriptor.id],
        )?;
        for model in &settings.models {
            transaction.execute(
                "INSERT INTO provider_models
                    (provider_id, model_id, display_name, context_window_tokens,
                     default_output_tokens, strengths, weaknesses, strategy, tier, capabilities)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                rusqlite::params![
                    model.provider_id,
                    model.model_id,
                    model.display_name,
                    model.context_window_tokens,
                    model.default_output_tokens,
                    serde_json::to_string(&model.strengths).map_err(|_| AppError::Storage)?,
                    serde_json::to_string(&model.weaknesses).map_err(|_| AppError::Storage)?,
                    serde_json::to_string(&model.strategy).map_err(|_| AppError::Storage)?,
                    tier_to_sql(model.tier)?,
                    serde_json::to_string(&model.capabilities).map_err(|_| AppError::Storage)?,
                ],
            )?;
        }
        transaction.commit()?;
        Ok(())
    }

    pub fn delete(&self, provider_id: &str) -> AppResult<()> {
        let connection = self.connection.lock()?;
        let changed = connection.execute(
            "DELETE FROM provider_settings WHERE provider_id = ?1",
            [provider_id],
        )?;
        if changed == 0 {
            return Err(AppError::NotFound);
        }
        Ok(())
    }

    fn get_with_connection(
        &self,
        connection: &rusqlite::Connection,
        provider_id: &str,
    ) -> AppResult<ProviderSettings> {
        let descriptor = connection
            .query_row(
                "SELECT provider_id, display_name, base_url, credential_id
                 FROM provider_settings WHERE provider_id = ?1",
                [provider_id],
                |row| {
                    Ok((
                        ProviderDescriptor {
                            id: row.get(0)?,
                            display_name: row.get(1)?,
                        },
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                    ))
                },
            )
            .map_err(|error| match error {
                rusqlite::Error::QueryReturnedNoRows => AppError::NotFound,
                other => other.into(),
            })?;
        let mut statement = connection.prepare(
            "SELECT model_id, display_name, context_window_tokens,
                    default_output_tokens, strengths, weaknesses, strategy, tier, capabilities
             FROM provider_models WHERE provider_id = ?1 ORDER BY model_id ASC",
        )?;
        let models = statement
            .query_map([provider_id], |row| {
                let tier_value: String = row.get(7)?;
                let tier = serde_json::from_value(serde_json::Value::String(tier_value))
                    .map_err(|error| json_conversion_error(7, error))?;
                let capabilities_json: String = row.get(8)?;
                let capabilities = serde_json::from_str(&capabilities_json)
                    .map_err(|error| json_conversion_error(8, error))?;
                let strengths_json: String = row.get(4)?;
                let weaknesses_json: String = row.get(5)?;
                let strategy_json: String = row.get(6)?;
                Ok(ModelProfile {
                    provider_id: provider_id.to_string(),
                    model_id: row.get(0)?,
                    display_name: row.get(1)?,
                    context_window_tokens: row.get(2)?,
                    default_output_tokens: row.get(3)?,
                    strengths: serde_json::from_str(&strengths_json)
                        .map_err(|error| json_conversion_error(4, error))?,
                    weaknesses: serde_json::from_str(&weaknesses_json)
                        .map_err(|error| json_conversion_error(5, error))?,
                    strategy: serde_json::from_str(&strategy_json)
                        .map_err(|error| json_conversion_error(6, error))?,
                    tier,
                    capabilities,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(ProviderSettings {
            descriptor: descriptor.0,
            base_url: descriptor.1,
            credential_id: descriptor.2,
            models,
        })
    }
}

fn json_conversion_error(column: usize, error: serde_json::Error) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(column, rusqlite::types::Type::Text, Box::new(error))
}

fn tier_to_sql(tier: super::ModelTier) -> AppResult<String> {
    serde_json::to_value(tier)
        .map_err(|_| AppError::Storage)?
        .as_str()
        .map(ToOwned::to_owned)
        .ok_or(AppError::Storage)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        db,
        error::AppError,
        provider::{
            ModelProfile, ModelTier, ProviderCapabilities, ProviderDescriptor, ProviderSettings,
        },
    };

    fn settings() -> ProviderSettings {
        ProviderSettings {
            descriptor: ProviderDescriptor {
                id: "openrouter".into(),
                display_name: "OpenRouter".into(),
            },
            base_url: "https://openrouter.ai/api/v1".into(),
            credential_id: "openrouter-primary".into(),
            models: vec![ModelProfile {
                provider_id: "openrouter".into(),
                model_id: "deepseek/deepseek-v4-flash-0731".into(),
                display_name: "DeepSeek V4 Flash 0731".into(),
                context_window_tokens: 1_310_720,
                default_output_tokens: 393_216,
                strengths: vec!["fast".into()],
                weaknesses: vec!["preview".into()],
                strategy: vec!["balanced".into()],
                tier: ModelTier::Medium,
                capabilities: ProviderCapabilities {
                    streaming: true,
                    ..ProviderCapabilities::default()
                },
            }],
        }
    }

    #[test]
    fn saves_reads_and_deletes_provider_metadata_without_credentials() {
        let connection = db::in_memory().expect("database should initialize");
        let repository = ProviderSettingsRepository::new(connection);
        let value = settings();

        repository.save(&value).expect("settings should save");
        assert_eq!(repository.get("openrouter").unwrap(), value);
        assert_eq!(repository.list().unwrap(), vec![value]);

        repository
            .delete("openrouter")
            .expect("settings should delete");
        assert_eq!(repository.get("openrouter"), Err(AppError::NotFound));
        assert!(repository.list().unwrap().is_empty());
    }

    #[test]
    fn saving_same_provider_replaces_models_atomically() {
        let connection = db::in_memory().expect("database should initialize");
        let repository = ProviderSettingsRepository::new(connection);
        let mut value = settings();
        repository.save(&value).unwrap();
        value.models[0].model_id = "deepseek/deepseek-v4-flash-0731:free".into();
        repository.save(&value).unwrap();

        let stored = repository.get("openrouter").unwrap();
        assert_eq!(stored.models.len(), 1);
        assert_eq!(
            stored.models[0].model_id,
            "deepseek/deepseek-v4-flash-0731:free"
        );
    }
}
