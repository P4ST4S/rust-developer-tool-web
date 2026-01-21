use crate::config::{save_config, Config};
use crate::events::StatusEvent;
use crate::state::AppState;
use tauri::State;

// Config commands
#[tauri::command]
pub async fn get_config(state: State<'_, AppState>) -> Result<Option<Config>, String> {
    let config = state.config.lock().await;
    Ok(config.clone())
}

#[tauri::command]
pub async fn save_app_config(config: Config, state: State<'_, AppState>) -> Result<(), String> {
    save_config(&config)?;
    let mut state_config = state.config.lock().await;
    *state_config = Some(config);
    Ok(())
}

#[tauri::command]
pub async fn set_active_project(
    project_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut config = state.config.lock().await;
    if let Some(ref mut cfg) = *config {
        cfg.active_project = Some(project_id);
        save_config(cfg)?;
    }
    Ok(())
}

// Service commands
#[tauri::command]
pub async fn start_service(
    project_id: String,
    service_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let config = {
        let config_lock = state.config.lock().await;
        config_lock.clone().ok_or_else(|| "No config loaded".to_string())?
    };

    state
        .process_manager
        .start_service(&config, project_id, service_id)
        .await
}

#[tauri::command]
pub async fn stop_service(
    project_id: String,
    service_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let config = { state.config.lock().await.clone() };

    state
        .process_manager
        .stop_service(config.as_ref(), project_id, service_id)
        .await
}

#[tauri::command]
pub async fn get_status(state: State<'_, AppState>) -> Result<StatusEvent, String> {
    Ok(state.process_manager.status().await)
}

#[tauri::command]
pub fn open_browser(url: String) -> Result<(), String> {
    open::that(&url).map_err(|e| e.to_string())
}
