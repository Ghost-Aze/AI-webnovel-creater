pub mod characters;
pub mod commands;
pub mod db;
pub mod domain;
pub mod error;
pub mod projects;

#[derive(Clone)]
pub struct AppState {
    pub project_service: projects::service::ProjectService,
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
                projects::repository::ProjectRepository::new(database),
            );
            app.manage(AppState {
                project_service: service,
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::project_create,
            commands::project_list,
            commands::project_get,
            commands::project_update,
            commands::project_archive
        ])
        .run(tauri::generate_context!())
        .expect("error while running Webnovel AI Studio");
}
