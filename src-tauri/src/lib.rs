pub mod commands;
pub mod config;
pub mod domain;
pub mod repositories;
pub mod services;
pub mod state;
pub mod storage;
pub mod utils;
pub mod watchers;

use state::app_state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::app::app_bootstrap,
            commands::workspace::workspace_create,
            commands::workspace::workspace_open,
            commands::workspace::workspace_list_recent,
            commands::package::package_list,
            commands::package::package_get,
            commands::package::package_create_from_nl,
            commands::package::package_update,
            commands::search::package_search,
            commands::eval::package_run_eval,
            commands::version::package_list_versions,
            commands::version::package_save_version,
            commands::version::package_restore_version,
            commands::test::package_run_test,
            commands::settings::settings_get,
            commands::settings::settings_update,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Skill Notebook");
}
