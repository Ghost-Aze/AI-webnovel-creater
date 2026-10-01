pub mod characters;
pub mod commands;
pub mod context;
pub mod db;
pub mod domain;
pub mod error;
pub mod projects;
pub mod provider;
pub mod revisions;

#[derive(Clone)]
pub struct AppState {
    pub character_service: characters::service::CharacterService,
    pub context_compiler: context::ContextCompiler,
    pub context_source: context::ServiceContextSource,
    pub project_service: projects::service::ProjectService,
    pub provider_registry: provider::ProviderRegistry,
    pub provider_runtime: provider::ProviderRuntime,
    pub revision_service: revisions::service::RevisionService,
    pub proposal_service: revisions::service::ProposalService,
}

#[cfg(feature = "tauri-app")]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let _ = tracing_subscriber::fmt()
                .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
                .with_target(false)
                .with_ansi(false)
                .try_init();
            tracing::info!("Webnovel AI Studio initialized");
            let data_dir = app
                .path()
                .app_data_dir()
                .map_err(|error| error.to_string())?;
            std::fs::create_dir_all(&data_dir).map_err(|error| error.to_string())?;
            let database =
                db::open(data_dir.join("webnovel.sqlite")).map_err(|error| error.to_string())?;
            let service = projects::service::ProjectService::new(
                projects::repository::ProjectRepository::new(database.clone()),
            );
            let character_service = characters::service::CharacterService::new(
                characters::repository::CharacterRepository::new(database.clone()),
            );
            let revision_service = revisions::service::RevisionService::new(
                revisions::repository::RevisionRepository::new(database.clone()),
                characters::repository::CharacterRepository::new(database.clone()),
            );
            let proposal_service = revisions::service::ProposalService::new(
                revisions::repository::RevisionRepository::new(database.clone()),
                characters::repository::CharacterRepository::new(database.clone()),
            );
            let context_source =
                context::ServiceContextSource::new(service.clone(), character_service.clone());
            let provider_runtime =
                provider::ProviderRuntime::new(provider::application_credential_store());
            let provider_registry = provider_runtime.registry();
            app.manage(AppState {
                character_service,
                context_compiler: context::ContextCompiler::default(),
                context_source,
                project_service: service,
                provider_registry,
                provider_runtime,
                revision_service,
                proposal_service,
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::project_create,
            commands::project_list,
            commands::project_get,
            commands::project_update,
            commands::project_archive,
            commands::character_create,
            commands::character_list,
            commands::character_get,
            commands::character_update,
            commands::character_archive,
            commands::character_state_get,
            commands::character_state_update,
            commands::memory_history_list,
            commands::memory_restore,
            commands::memory_set_canon_status,
            commands::memory_proposal_create,
            commands::memory_proposal_list,
            commands::memory_proposal_promote,
            commands::memory_proposal_reject,
            commands::provider_list,
            commands::model_list,
            commands::model_route,
            commands::context_compile,
            commands::provider_generate,
            commands::provider_configure,
            commands::provider_remove,
            commands::provider_credential_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running Webnovel AI Studio");
}
