use serde::Serialize;
use std::collections::HashMap;

#[derive(Clone, Serialize)]
pub struct LogEvent {
    pub source: String,
    pub level: String,
    pub text: String,
    pub timestamp: String,
    pub project_id: String,
}

#[derive(Clone, Serialize)]
pub struct ServiceStatus {
    pub running: bool,
    pub url: Option<String>,
}

#[derive(Clone, Serialize)]
pub struct StatusEvent {
    pub services: HashMap<String, ServiceStatus>,
}

#[derive(Clone)]
pub enum ManagerEvent {
    Log(LogEvent),
    Status(StatusEvent),
    ServiceUrl { service_id: String, url: String },
}
