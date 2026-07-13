use tauri::{AppHandle, Emitter, Manager};
use crate::error::LauncherError;
use crate::steam;
use crate::downloader;
use crate::theme;

#[tauri::command]
pub async fn load_launcher_theme() -> Result<theme::LauncherTheme, LauncherError> {
    theme::load_launcher_theme().await
}

#[tauri::command]
pub fn minimize_main_window(app: AppHandle) -> Result<(), LauncherError> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| LauncherError::System("main window was not found".to_string()))?;
    window
        .minimize()
        .map_err(|error| LauncherError::System(format!("failed to minimize main window: {error}")))
}

#[tauri::command]
pub fn close_main_window(app: AppHandle) -> Result<(), LauncherError> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| LauncherError::System("main window was not found".to_string()))?;
    window
        .close()
        .map_err(|error| LauncherError::System(format!("failed to close main window: {error}")))
}

#[tauri::command]
pub async fn load_launcher_settings() -> Result<theme::LauncherSettings, LauncherError> {
    theme::read_launcher_settings().await
}

#[tauri::command]
pub async fn save_launcher_profile(
    username: String,
    avatar_bytes: Option<Vec<u8>>,
) -> Result<theme::LauncherSettings, LauncherError> {
    theme::save_launcher_profile(username, avatar_bytes).await
}

#[tauri::command]
pub async fn load_git_metadata(product: String) -> Result<downloader::LauncherGitMetadata, LauncherError> {
    downloader::load_git_metadata(&product).await
}

#[tauri::command]
pub async fn prepare_version(app: AppHandle, tag: String, dll_name: String) -> Result<String, LauncherError> {
    let _ = app.emit("log", "downloading DLL...");
    let result = downloader::prepare_version(tag, dll_name).await;
    let _ = app.emit("log", "DLL ready");
    result
}

#[tauri::command]
pub fn launch_game_process(app: AppHandle, appid: i32) -> Result<(), LauncherError> {
    let _ = app.emit("log", &format!("launching game {}", appid));
    steam::restart_csgo(appid)
}

#[tauri::command]
pub async fn wait_and_inject(app: AppHandle, dll_path: String, dll_name: String) -> Result<(), LauncherError> {
    let _ = app.emit("log", "waiting for CSGO window...");
    let result = downloader::wait_and_inject(dll_path, dll_name).await;
    let _ = app.emit("log", "injection complete");
    result
}

#[tauri::command]
pub fn kill_background_processes() -> Result<(), LauncherError> {
    downloader::kill_background_processes()
}

#[tauri::command]
pub fn detect_installed_games() -> Result<steam::InstalledGames, LauncherError> {
    Ok(steam::InstalledGames {
        cs2_legacy_branch: true,
        csgo_standalone: true,
    })
}


