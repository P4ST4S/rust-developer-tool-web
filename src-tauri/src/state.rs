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
    pub frontend: Arc<Mutex<ProcessState>>,
    pub backend: Arc<Mutex<ProcessState>>,
    pub frontend_url: Arc<Mutex<Option<String>>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            frontend: Arc::new(Mutex::new(ProcessState::default())),
            backend: Arc::new(Mutex::new(ProcessState::default())),
            frontend_url: Arc::new(Mutex::new(None)),
        }
    }
}
