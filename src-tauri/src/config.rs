use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    pub id: String,
    pub name: String,
    pub path: String,
    pub command: String,
    #[serde(default)]
    pub detect_url: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub services: Vec<Service>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub version: u32,
    pub active_project: Option<String>,
    pub projects: Vec<Project>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: 1,
            active_project: None,
            projects: Vec::new(),
        }
    }
}

pub fn get_config_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".dev-stack-launcher"))
}

pub fn get_config_path() -> Option<PathBuf> {
    get_config_dir().map(|dir| dir.join("config.json"))
}

pub fn load_config() -> Option<Config> {
    let path = get_config_path()?;
    if !path.exists() {
        return None;
    }
    let content = fs::read_to_string(&path).ok()?;
    serde_json::from_str(&content).ok()
}

pub fn save_config(config: &Config) -> Result<(), AppError> {
    let dir = get_config_dir().ok_or_else(|| AppError::SaveConfig {
        message: "Could not determine config directory".to_string(),
    })?;
    let path = get_config_path().ok_or_else(|| AppError::SaveConfig {
        message: "Could not determine config path".to_string(),
    })?;

    // Create directory if it doesn't exist
    if !dir.exists() {
        fs::create_dir_all(&dir)
            .map_err(|e| AppError::SaveConfig { message: e.to_string() })?;
    }

    let content = serde_json::to_string_pretty(config)
        .map_err(|e| AppError::SaveConfig { message: e.to_string() })?;

    let tmp_path = path.with_extension("json.tmp");
    fs::write(&tmp_path, content)
        .map_err(|e| AppError::SaveConfig { message: e.to_string() })?;

    fs::rename(&tmp_path, &path)
        .map_err(|e| AppError::SaveConfig { message: e.to_string() })?;

    Ok(())
}

impl Config {
    pub fn get_project(&self, project_id: &str) -> Option<&Project> {
        self.projects.iter().find(|p| p.id == project_id)
    }

    pub fn get_service(&self, project_id: &str, service_id: &str) -> Option<&Service> {
        self.get_project(project_id)
            .and_then(|p| p.services.iter().find(|s| s.id == service_id))
    }
}
