pub mod characters;
pub mod commands;
pub mod context;
pub mod conversations;
pub mod db;
pub mod domain;
pub mod error;
pub mod manuscript_proposals;
pub mod manuscripts;
pub mod memory_tools;
pub mod orchestration;
pub mod project_memory;
pub mod projects;
pub mod provider;
pub mod revisions;
pub mod users;

#[derive(Clone)]
pub struct AppState {
    pub character_service: characters::service::CharacterService,
    pub conversation_service: conversations::service::ConversationService,
    pub context_compiler: context::ContextCompiler,
    pub context_source: context::ServiceContextSource,
    pub memory_tool_service: memory_tools::MemoryToolService,
    pub manuscript_service: manuscripts::service::ManuscriptService,
    pub manuscript_proposal_service: manuscript_proposals::service::ManuscriptProposalService,
    pub project_memory_service: project_memory::service::ProjectMemoryService,
    pub project_service: projects::service::ProjectService,
    pub provider_registry: provider::ProviderRegistry,
    pub provider_runtime: provider::ProviderRuntime,
    pub provider_settings_repository: provider::ProviderSettingsRepository,
    pub revision_service: revisions::service::RevisionService,
    pub proposal_service: revisions::service::ProposalService,
    pub user_service: users::service::UserService,
}

#[cfg(feature = "tauri-app")]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    use tauri::Manager;

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
            let conversation_service = conversations::service::ConversationService::new(
                conversations::repository::ConversationRepository::new(database.clone()),
                projects::repository::ProjectRepository::new(database.clone()),
            );
            let revision_service = revisions::service::RevisionService::new(
                revisions::repository::RevisionRepository::new(database.clone()),
                characters::repository::CharacterRepository::new(database.clone()),
                project_memory::repository::ProjectMemoryRepository::new(database.clone()),
            );
            let proposal_service = revisions::service::ProposalService::new(
                revisions::repository::RevisionRepository::new(database.clone()),
                characters::repository::CharacterRepository::new(database.clone()),
                project_memory::repository::ProjectMemoryRepository::new(database.clone()),
            );
            let memory_tool_service =
                memory_tools::MemoryToolService::new(proposal_service.clone());
            let manuscript_service = manuscripts::service::ManuscriptService::new(
                manuscripts::repository::ManuscriptRepository::new(database.clone()),
                projects::repository::ProjectRepository::new(database.clone()),
            );
            let manuscript_proposal_service =
                manuscript_proposals::service::ManuscriptProposalService::new(
                    manuscript_proposals::repository::ManuscriptProposalRepository::new(
                        database.clone(),
                    ),
                    projects::repository::ProjectRepository::new(database.clone()),
                );
            let project_memory_service = project_memory::service::ProjectMemoryService::new(
                project_memory::repository::ProjectMemoryRepository::new(database.clone()),
                projects::repository::ProjectRepository::new(database.clone()),
            );
            let context_source = context::ServiceContextSource::new(
                service.clone(),
                character_service.clone(),
                project_memory_service.clone(),
            );
            let provider_runtime =
                provider::ProviderRuntime::new(provider::application_credential_store());
            let provider_registry = provider_runtime.registry();
            let provider_settings_repository =
                provider::ProviderSettingsRepository::new(database.clone());
            for settings in provider_settings_repository
                .list()
                .map_err(|error| error.to_string())?
            {
                if let Err(error) = provider_runtime.restore(settings.clone()) {
                    tracing::warn!(
                        provider_id = %settings.descriptor.id,
                        error = ?error,
                        "stored provider was not activated"
                    );
                }
            }
            let user_service = users::service::UserService::new(
                users::repository::UserRepository::new(database.clone()),
            );
            app.manage(AppState {
                character_service,
                conversation_service,
                context_compiler: context::ContextCompiler::default(),
                context_source,
                memory_tool_service,
                manuscript_service,
                manuscript_proposal_service,
                project_memory_service,
                project_service: service,
                provider_registry,
                provider_runtime,
                provider_settings_repository,
                revision_service,
                proposal_service,
                user_service,
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::project_create,
            commands::project_list,
            commands::project_get,
            commands::project_update,
            commands::project_archive,
            commands::project_delete,
            commands::user_profile_get,
            commands::user_profile_update,
            commands::user_preferences_get,
            commands::user_preferences_update,
            commands::conversation_create,
            commands::conversation_list,
            commands::conversation_get,
            commands::conversation_message_list,
            commands::conversation_message_append,
            commands::developer_chat_send,
            commands::chapter_chat_send,
            commands::memory_tool_propose,
            commands::chapter_create,
            commands::chapter_list,
            commands::chapter_get,
            commands::chapter_update,
            commands::chapter_archive,
            commands::manuscript_get,
            commands::manuscript_save,
            commands::manuscript_revision_list,
            commands::manuscript_restore,
            commands::manuscript_proposal_create,
            commands::manuscript_proposal_list,
            commands::manuscript_proposal_get,
            commands::manuscript_proposal_promote,
            commands::manuscript_proposal_reject,
            commands::story_fact_create,
            commands::story_fact_list,
            commands::story_fact_get,
            commands::story_fact_update,
            commands::story_fact_archive,
            commands::canon_rule_create,
            commands::canon_rule_list,
            commands::canon_rule_get,
            commands::canon_rule_update,
            commands::canon_rule_archive,
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
            commands::orchestrator_run,
            commands::context_compile,
            commands::provider_generate,
            commands::provider_configure,
            commands::provider_get,
            commands::provider_update,
            commands::provider_test,
            commands::provider_remove,
            commands::provider_credential_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running Webnovel AI Studio");
}
