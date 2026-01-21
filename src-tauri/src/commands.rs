use crate::config::{save_config, Config};
use crate::process::{create_process_group_command, kill_process_group};
use crate::state::{AppState, ProcessState};
use regex::Regex;
use serde::Serialize;
use std::collections::HashMap;

use tauri::{AppHandle, Emitter, State};
use tokio::io::{AsyncBufReadExt, BufReader};

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

fn extract_vite_url(line: &str) -> Option<String> {
    let url_regex = Regex::new(r"(?:Local|local):\s+(https?://[^\s]+)").ok()?;
    url_regex
        .captures(line)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().to_string())
}

fn format_log_prefix(source: &str, is_error: bool) -> String {
    let color_code = if is_error {
        "\x1b[38;5;196m"
    } else {
        match source.to_lowercase().as_str() {
            "system" => "\x1b[38;5;214m",
            _ => "\x1b[38;5;75m",
        }
    };
    let reset = "\x1b[0m";
    let label = if is_error {
        format!("[{} ERROR]", source.to_uppercase())
    } else {
        format!("[{}]", source.to_uppercase())
    };
    format!("{}{}{} ", color_code, label, reset)
}

fn get_timestamp() -> String {
    chrono::Local::now().format("%H:%M:%S%.3f").to_string()
}

async fn emit_status(app: &AppHandle, state: &AppState) {
    let processes = state.processes.lock().await;
    let urls = state.detected_urls.lock().await;

    let mut services = HashMap::new();
    for (service_id, process) in processes.iter() {
        services.insert(
            service_id.clone(),
            ServiceStatus {
                running: process.running,
                url: urls.get(service_id).cloned(),
            },
        );
    }

    let _ = app.emit("status-change", StatusEvent { services });
}

// Config commands
#[tauri::command]
pub async fn get_config(state: State<'_, AppState>) -> Result<Option<Config>, String> {
    let config = state.config.lock().await;
    Ok(config.clone())
}

#[tauri::command]
pub async fn save_app_config(
    config: Config,
    state: State<'_, AppState>,
) -> Result<(), String> {
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
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let composite_id = format!("{}:{}", project_id, service_id);

    // Check if already running
    {
        let processes = state.processes.lock().await;
        if let Some(process) = processes.get(&composite_id) {
            if process.running {
                return Err(format!("Service {} already running", service_id));
            }
        }
    }

    // Get service config
    let (service_name, service_path, service_command, detect_url) = {
        let config = state.config.lock().await;
        let cfg = config.as_ref().ok_or("No config loaded")?;
        let service = cfg
            .get_service(&project_id, &service_id)
            .ok_or(format!("Service {} not found", service_id))?;
        (
            service.name.clone(),
            service.path.clone(),
            service.command.clone(),
            service.detect_url,
        )
    };

    let _ = app.emit(
        "log",
        LogEvent {
            source: "system".to_string(),
            level: "normal".to_string(),
            text: format!("{}Starting {}...", format_log_prefix("system", false), service_name),
            timestamp: get_timestamp(),
            project_id: project_id.clone(),
        },
    );

    // Parse command
    let parts: Vec<&str> = service_command.split_whitespace().collect();
    if parts.is_empty() {
        return Err("Empty command".to_string());
    }
    let program = parts[0];
    let args: Vec<&str> = parts[1..].to_vec();

    let mut cmd = create_process_group_command(program, &args, &service_path);
    let mut child = cmd.spawn().map_err(|e| {
        let _ = app.emit(
            "log",
            LogEvent {
                source: "system".to_string(),
                level: "error".to_string(),
                text: format!(
                    "{}Failed to start {}: {}",
                    format_log_prefix("system", true),
                    service_name,
                    e
                ),
                timestamp: get_timestamp(),
                project_id: project_id.clone(),
            },
        );
        e.to_string()
    })?;

    let child_id = child.id();
    let composite_id_clone = composite_id.clone();
    let project_id_clone = project_id.clone();
    let service_name_clone = service_name.clone();

    // Handle stdout
    if let Some(stdout) = child.stdout.take() {
        let app_clone = app.clone();
        let urls_state = state.detected_urls.clone();
        let composite_id_stdout = composite_id.clone();
        let project_id_stdout = project_id.clone();
        let service_name_stdout = service_name.clone();
        let detect_url_stdout = detect_url;
        tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();

            while let Ok(Some(line)) = lines.next_line().await {
                if detect_url_stdout {
                    if let Some(url) = extract_vite_url(&line) {
                        let mut urls = urls_state.lock().await;
                        urls.insert(composite_id_stdout.clone(), url.clone());
                        let _ = app_clone.emit("service-url", serde_json::json!({
                            "serviceId": composite_id_stdout.clone(),
                            "url": url.clone()
                        }));
                        let _ = app_clone.emit(
                            "log",
                            LogEvent {
                                source: "system".to_string(),
                                level: "normal".to_string(),
                                text: format!(
                                    "{}{} URL detected: {}",
                                    format_log_prefix("system", false),
                                    service_name_stdout,
                                    url
                                ),
                                timestamp: get_timestamp(),
                                project_id: project_id_stdout.clone(),
                            },
                        );
                    }
                }

                let _ = app_clone.emit(
                    "log",
                    LogEvent {
                        source: service_name_stdout.to_lowercase(),
                        level: "normal".to_string(),
                        text: format!("{}{}", format_log_prefix(&service_name_stdout, false), line),
                        timestamp: get_timestamp(),
                        project_id: project_id_stdout.clone(),
                    },
                );
            }
        });
    }

    // Handle stderr
    if let Some(stderr) = child.stderr.take() {
        let app_clone = app.clone();
        let urls_state = state.detected_urls.clone();
        let composite_id_stderr = composite_id.clone();
        let project_id_stderr = project_id.clone();
        let service_name_stderr = service_name.clone();
        let detect_url_stderr = detect_url;
        tokio::spawn(async move {
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();

            while let Ok(Some(line)) = lines.next_line().await {
                if detect_url_stderr {
                    if let Some(url) = extract_vite_url(&line) {
                        let mut urls = urls_state.lock().await;
                        urls.insert(composite_id_stderr.clone(), url.clone());
                        let _ = app_clone.emit("service-url", serde_json::json!({
                            "serviceId": composite_id_stderr.clone(),
                            "url": url
                        }));
                    }
                }

                let _ = app_clone.emit(
                    "log",
                    LogEvent {
                        source: service_name_stderr.to_lowercase(),
                        level: "error".to_string(),
                        text: format!("{}{}", format_log_prefix(&service_name_stderr, true), line),
                        timestamp: get_timestamp(),
                        project_id: project_id_stderr.clone(),
                    },
                );
            }
        });
    }

    // Store process
    {
        let mut processes = state.processes.lock().await;
        processes.insert(
            composite_id.clone(),
            ProcessState {
                child: Some(child),
                running: true,
            },
        );
    }

    emit_status(&app, &state).await;

    // Monitor process
    let app_monitor = app.clone();
    let processes_state = state.processes.clone();
    let urls_state = state.detected_urls.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

            let mut processes = processes_state.lock().await;
            if let Some(process) = processes.get_mut(&composite_id_clone) {
                if let Some(child) = &mut process.child {
                    match child.try_wait() {
                        Ok(Some(status)) => {
                            let _ = app_monitor.emit(
                                "log",
                                LogEvent {
                                    source: "system".to_string(),
                                    level: "normal".to_string(),
                                    text: format!(
                                        "{}{} stopped (PID: {:?}, status: {:?})",
                                        format_log_prefix("system", false),
                                        service_name_clone,
                                        child_id,
                                        status
                                    ),
                                    timestamp: get_timestamp(),
                                    project_id: project_id_clone.clone(),
                                },
                            );
                            process.child = None;
                            process.running = false;

                            let mut urls = urls_state.lock().await;
                            urls.remove(&composite_id_clone);

                            // Emit status update
                            let mut services = HashMap::new();
                            for (id, p) in processes.iter() {
                                services.insert(
                                    id.clone(),
                                    ServiceStatus {
                                        running: p.running,
                                        url: urls.get(id).cloned(),
                                    },
                                );
                            }
                            let _ = app_monitor.emit("status-change", StatusEvent { services });
                            break;
                        }
                        Ok(None) => {}
                        Err(e) => {
                            let _ = app_monitor.emit(
                                "log",
                                LogEvent {
                                    source: "system".to_string(),
                                    level: "error".to_string(),
                                    text: format!(
                                        "{}Error checking {} status: {}",
                                        format_log_prefix("system", true),
                                        service_name_clone,
                                        e
                                    ),
                                    timestamp: get_timestamp(),
                                    project_id: project_id_clone.clone(),
                                },
                            );
                            process.child = None;
                            process.running = false;
                            break;
                        }
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    });

    Ok(())
}

#[tauri::command]
pub async fn stop_service(
    project_id: String,
    service_id: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let composite_id = format!("{}:{}", project_id, service_id);

    // Get service name for logging
    let service_name = {
        let config = state.config.lock().await;
        config
            .as_ref()
            .and_then(|cfg| cfg.get_service(&project_id, &service_id))
            .map(|s| s.name.clone())
            .unwrap_or_else(|| service_id.clone())
    };

    let mut processes = state.processes.lock().await;
    if let Some(process) = processes.get_mut(&composite_id) {
        if let Some(mut child) = process.child.take() {
            process.running = false;
            drop(processes);

            let _ = app.emit(
                "log",
                LogEvent {
                    source: "system".to_string(),
                    level: "normal".to_string(),
                    text: format!("{}Stopping {}...", format_log_prefix("system", false), service_name),
                    timestamp: get_timestamp(),
                    project_id: project_id.clone(),
                },
            );

            match kill_process_group(&mut child).await {
                Ok(_) => match child.wait().await {
                    Ok(status) => {
                        let _ = app.emit(
                            "log",
                            LogEvent {
                                source: "system".to_string(),
                                level: "normal".to_string(),
                                text: format!(
                                    "{}{} killed successfully (status: {:?})",
                                    format_log_prefix("system", false),
                                    service_name,
                                    status
                                ),
                                timestamp: get_timestamp(),
                                project_id: project_id.clone(),
                            },
                        );
                    }
                    Err(e) => {
                        let _ = app.emit(
                            "log",
                            LogEvent {
                                source: "system".to_string(),
                                level: "error".to_string(),
                                text: format!(
                                    "{}Error waiting for {}: {}",
                                    format_log_prefix("system", true),
                                    service_name,
                                    e
                                ),
                                timestamp: get_timestamp(),
                                project_id: project_id.clone(),
                            },
                        );
                    }
                },
                Err(e) => {
                    let _ = app.emit(
                        "log",
                        LogEvent {
                            source: "system".to_string(),
                            level: "error".to_string(),
                            text: format!(
                                "{}Failed to kill {}: {}",
                                format_log_prefix("system", true),
                                service_name,
                                e
                            ),
                            timestamp: get_timestamp(),
                            project_id: project_id.clone(),
                        },
                    );
                }
            }

            let mut urls = state.detected_urls.lock().await;
            urls.remove(&composite_id);
        }
    }

    emit_status(&app, &state).await;
    Ok(())
}

#[tauri::command]
pub async fn get_status(state: State<'_, AppState>) -> Result<StatusEvent, String> {
    let processes = state.processes.lock().await;
    let urls = state.detected_urls.lock().await;

    let mut services = HashMap::new();
    for (service_id, process) in processes.iter() {
        services.insert(
            service_id.clone(),
            ServiceStatus {
                running: process.running,
                url: urls.get(service_id).cloned(),
            },
        );
    }

    Ok(StatusEvent { services })
}

#[tauri::command]
pub fn open_browser(url: String) -> Result<(), String> {
    open::that(&url).map_err(|e| e.to_string())
}
