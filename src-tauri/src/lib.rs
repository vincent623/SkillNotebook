pub mod cli;
pub mod commands;
pub mod config;
pub mod domain;
pub mod services;
pub mod state;
pub mod storage;
pub mod utils;

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
            commands::project_root::project_root_create,
            commands::project_root::project_root_open,
            commands::project_root::project_root_list_recent,
            commands::package::package_list,
            commands::package::package_get,
            commands::package::package_create_from_nl,
            commands::package::package_generate_preview_from_nl,
            commands::package::package_generate_preview_from_sources,
            commands::package::package_generate_preview_from_url,
            commands::package::package_commit_preview,
            commands::package::package_discard_preview,
            commands::package::package_file_tree,
            commands::package::package_file_read,
            commands::package::package_file_write,
            commands::package::package_update,
            commands::package::package_export_zip,
            commands::search::package_search,
            commands::eval::package_run_eval,
            commands::version::package_list_versions,
            commands::version::package_save_version,
            commands::version::package_diff_version,
            commands::version::package_restore_version,
            commands::test::package_run_test,
            commands::settings::settings_get,
            commands::settings::settings_update,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Skill Notebook");
}
