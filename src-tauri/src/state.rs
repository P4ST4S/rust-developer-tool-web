use crate::config::Config;
use crate::events::ManagerEvent;
use crate::process_manager::ProcessManager;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

pub struct AppState {
    pub config: Arc<Mutex<Option<Config>>>,
    pub process_manager: Arc<ProcessManager>,
}

impl Default for AppState {
    fn default() -> Self {
        let (event_tx, _event_rx) = mpsc::channel::<ManagerEvent>(1);
        Self::new(event_tx)
    }
}

impl AppState {
    pub fn new(event_tx: mpsc::Sender<ManagerEvent>) -> Self {
        Self {
            config: Arc::new(Mutex::new(None)),
            process_manager: Arc::new(ProcessManager::new(event_tx)),
        }
    }
}
