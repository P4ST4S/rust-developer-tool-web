use crate::config::Config;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::process::Child;
use tokio::sync::Mutex;

pub struct ProcessState {
    pub child: Option<Child>,
    pub running: bool,
}

impl Default for ProcessState {
    fn default() -> Self {
        Self {
            child: None,
            running: false,
        }
    }
}

pub struct AppState {
    pub config: Arc<Mutex<Option<Config>>>,
    pub processes: Arc<Mutex<HashMap<String, ProcessState>>>,
    pub detected_urls: Arc<Mutex<HashMap<String, String>>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            config: Arc::new(Mutex::new(None)),
            processes: Arc::new(Mutex::new(HashMap::new())),
            detected_urls: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}
