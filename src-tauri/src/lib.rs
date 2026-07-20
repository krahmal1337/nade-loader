mod error;
mod steam;
mod downloader;
mod theme;
mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            commands::load_launcher_theme,
            commands::load_launcher_settings,
            commands::save_launcher_profile,
            commands::load_git_metadata,
            commands::prepare_version,
            commands::launch_game_process,
            commands::launch_csgo_standalone,
            commands::launch_csgo_legacy_branch,
            commands::wait_and_inject,
            commands::minimize_main_window,
            commands::close_main_window,
            commands::kill_background_processes,
            commands::detect_installed_games,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
