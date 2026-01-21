use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error, Serialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum AppError {
    #[error("No config loaded")]
    NoConfigLoaded,
    #[error("Service not found: {service_id}")]
    ServiceNotFound { service_id: String },
    #[error("Project not found: {project_id}")]
    ProjectNotFound { project_id: String },
    #[error("Service already running: {service_id}")]
    ServiceAlreadyRunning { service_id: String },
    #[error("Empty command")]
    EmptyCommand,
    #[error("Failed to start {service_name}: {message}")]
    ProcessStartFailed { service_name: String, message: String },
    #[error("Failed to save config: {message}")]
    SaveConfig { message: String },
    #[error("Failed to open browser: {message}")]
    OpenBrowser { message: String },
}
