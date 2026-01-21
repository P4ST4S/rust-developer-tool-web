use crate::config::{save_config, Config};
use crate::error::AppError;
use crate::events::StatusEvent;
use crate::process_manager::ServiceSpec;
use crate::state::AppState;
use tauri::State;

// Config commands
#[tauri::command]
pub async fn get_config(state: State<'_, AppState>) -> Result<Option<Config>, AppError> {
    let config = state.config.lock().await;
    Ok(config.clone())
}

#[tauri::command]
pub async fn save_app_config(config: Config, state: State<'_, AppState>) -> Result<(), AppError> {
    save_config(&config)?;
    let mut state_config = state.config.lock().await;
    *state_config = Some(config);
    Ok(())
}

#[tauri::command]
pub async fn set_active_project(
    project_id: String,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let mut config = state.config.lock().await;
    let current = config.clone().ok_or(AppError::NoConfigLoaded)?;
    if current.get_project(&project_id).is_none() {
        return Err(AppError::ProjectNotFound { project_id });
    }
    let mut updated = current;
    updated.active_project = Some(project_id);
    save_config(&updated)?;
    *config = Some(updated);
    Ok(())
}

// Service commands
#[tauri::command]
pub async fn start_service(
    project_id: String,
    service_id: String,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let spec = {
        let config_lock = state.config.lock().await;
        let config = config_lock.as_ref().ok_or(AppError::NoConfigLoaded)?;
        let service = config.get_service(&project_id, &service_id).ok_or(
            AppError::ServiceNotFound {
                service_id: service_id.clone(),
            },
        )?;
        ServiceSpec {
            project_id,
            service_id,
            name: service.name.clone(),
            path: service.path.clone(),
            command: service.command.clone(),
            detect_url: service.detect_url,
        }
    };

    state.process_manager.start_service(spec).await
}

#[tauri::command]
pub async fn stop_service(
    project_id: String,
    service_id: String,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let service_name = {
        let config_lock = state.config.lock().await;
        config_lock
            .as_ref()
            .and_then(|config| config.get_service(&project_id, &service_id))
            .map(|service| service.name.clone())
            .unwrap_or_else(|| service_id.clone())
    };

    state
        .process_manager
        .stop_service(project_id, service_id, service_name)
        .await
}

#[tauri::command]
pub async fn get_status(state: State<'_, AppState>) -> Result<StatusEvent, AppError> {
    Ok(state.process_manager.status().await)
}

#[tauri::command]
pub fn open_browser(url: String) -> Result<(), AppError> {
    open::that(&url).map_err(|e| AppError::OpenBrowser {
        message: e.to_string(),
    })
}
