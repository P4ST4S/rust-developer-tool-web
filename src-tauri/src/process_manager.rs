use crate::error::AppError;
use crate::events::{LogEvent, ManagerEvent, ServiceStatus, StatusEvent};
use crate::process::{create_process_group_command, kill_process_group};
use regex::Regex;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Child;
use tokio::sync::{mpsc, Mutex};

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

pub struct ProcessManager {
    processes: Arc<Mutex<HashMap<String, ProcessState>>>,
    detected_urls: Arc<Mutex<HashMap<String, String>>>,
    event_tx: mpsc::Sender<ManagerEvent>,
    vite_url_regex: Regex,
}

pub struct ServiceSpec {
    pub project_id: String,
    pub service_id: String,
    pub name: String,
    pub path: String,
    pub command: String,
    pub detect_url: bool,
}

impl ProcessManager {
    pub fn new(event_tx: mpsc::Sender<ManagerEvent>) -> Self {
        let vite_url_regex =
            Regex::new(r"(?:Local|local):\s+(https?://[^\s]+)").unwrap_or_else(|_| {
                Regex::new("$^").expect("fallback regex should be valid")
            });
        Self {
            processes: Arc::new(Mutex::new(HashMap::new())),
            detected_urls: Arc::new(Mutex::new(HashMap::new())),
            event_tx,
            vite_url_regex,
        }
    }

    pub async fn start_service(
        &self,
        spec: ServiceSpec,
    ) -> Result<(), AppError> {
        let ServiceSpec {
            project_id,
            service_id,
            name,
            path,
            command,
            detect_url,
        } = spec;
        let composite_id = format!("{}:{}", project_id, service_id);

        {
            let processes = self.processes.lock().await;
            if let Some(process) = processes.get(&composite_id) {
                if process.running {
                    return Err(AppError::ServiceAlreadyRunning {
                        service_id,
                    });
                }
            }
        }

        let service_name = name;
        let service_path = path;
        let service_command = command;

        self.emit_log(LogEvent {
            source: "system".to_string(),
            level: "normal".to_string(),
            text: format!(
                "{}Starting {}...",
                format_log_prefix("system", false),
                service_name
            ),
            timestamp: get_timestamp(),
            project_id: project_id.clone(),
        });

        let parts: Vec<&str> = service_command.split_whitespace().collect();
        if parts.is_empty() {
            return Err(AppError::EmptyCommand);
        }
        let program = parts[0];
        let args: Vec<&str> = parts[1..].to_vec();

        let mut cmd = create_process_group_command(program, &args, &service_path);
        let service_name_for_error = service_name.clone();
        let mut child = cmd.spawn().map_err(|e| {
            self.emit_log(LogEvent {
                source: "system".to_string(),
                level: "error".to_string(),
                text: format!(
                    "{}Failed to start {}: {}",
                    format_log_prefix("system", true),
                    service_name_for_error,
                    e
                ),
                timestamp: get_timestamp(),
                project_id: project_id.clone(),
            });
            AppError::ProcessStartFailed {
                service_name: service_name_for_error,
                message: e.to_string(),
            }
        })?;

        let child_id = child.id();
        let composite_id_clone = composite_id.clone();
        let project_id_clone = project_id.clone();
        let service_name_clone = service_name.clone();
        let event_tx = self.event_tx.clone();
        let vite_url_regex = self.vite_url_regex.clone();
        let detected_urls = self.detected_urls.clone();

        if let Some(stdout) = child.stdout.take() {
            let event_tx = event_tx.clone();
            let detected_urls = detected_urls.clone();
            let composite_id_stdout = composite_id.clone();
            let project_id_stdout = project_id.clone();
            let service_name_stdout = service_name.clone();
            let detect_url_stdout = detect_url;
            let vite_url_regex = vite_url_regex.clone();
            tokio::spawn(async move {
                let reader = BufReader::new(stdout);
                let mut lines = reader.lines();

                while let Ok(Some(line)) = lines.next_line().await {
                    if detect_url_stdout {
                        if let Some(url) = extract_vite_url(&vite_url_regex, &line) {
                            {
                                let mut urls = detected_urls.lock().await;
                                urls.insert(composite_id_stdout.clone(), url.clone());
                            }
                            let _ = event_tx
                                .send(ManagerEvent::ServiceUrl {
                                    service_id: composite_id_stdout.clone(),
                                    url: url.clone(),
                                })
                                .await;
                            let _ = event_tx
                                .send(ManagerEvent::Log(LogEvent {
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
                                }))
                                .await;
                        }
                    }

                    let _ = event_tx.try_send(ManagerEvent::Log(LogEvent {
                        source: service_name_stdout.to_lowercase(),
                        level: "normal".to_string(),
                        text: format!(
                            "{}{}",
                            format_log_prefix(&service_name_stdout, false),
                            line
                        ),
                        timestamp: get_timestamp(),
                        project_id: project_id_stdout.clone(),
                    }));
                }
            });
        }

        if let Some(stderr) = child.stderr.take() {
            let event_tx = event_tx.clone();
            let detected_urls = detected_urls.clone();
            let composite_id_stderr = composite_id.clone();
            let project_id_stderr = project_id.clone();
            let service_name_stderr = service_name.clone();
            let detect_url_stderr = detect_url;
            let vite_url_regex = vite_url_regex.clone();
            tokio::spawn(async move {
                let reader = BufReader::new(stderr);
                let mut lines = reader.lines();

                while let Ok(Some(line)) = lines.next_line().await {
                    if detect_url_stderr {
                        if let Some(url) = extract_vite_url(&vite_url_regex, &line) {
                            {
                                let mut urls = detected_urls.lock().await;
                                urls.insert(composite_id_stderr.clone(), url.clone());
                            }
                            let _ = event_tx
                                .send(ManagerEvent::ServiceUrl {
                                    service_id: composite_id_stderr.clone(),
                                    url,
                                })
                                .await;
                        }
                    }

                    let _ = event_tx.try_send(ManagerEvent::Log(LogEvent {
                        source: service_name_stderr.to_lowercase(),
                        level: "error".to_string(),
                        text: format!(
                            "{}{}",
                            format_log_prefix(&service_name_stderr, true),
                            line
                        ),
                        timestamp: get_timestamp(),
                        project_id: project_id_stderr.clone(),
                    }));
                }
            });
        }

        {
            let mut processes = self.processes.lock().await;
            processes.insert(
                composite_id.clone(),
                ProcessState {
                    child: Some(child),
                    running: true,
                },
            );
        }

        self.emit_status().await;

        let processes_state = self.processes.clone();
        let detected_urls = self.detected_urls.clone();
        let event_tx = self.event_tx.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

                let mut processes = processes_state.lock().await;
                if let Some(process) = processes.get_mut(&composite_id_clone) {
                    if let Some(child) = &mut process.child {
                        match child.try_wait() {
                            Ok(Some(status)) => {
                                let _ = event_tx.try_send(ManagerEvent::Log(LogEvent {
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
                            }));
                                process.child = None;
                                process.running = false;

                                let mut urls = detected_urls.lock().await;
                                urls.remove(&composite_id_clone);
                                let status = build_status(&processes, &urls);
                                drop(urls);
                                drop(processes);
                                let _ = event_tx.send(ManagerEvent::Status(status)).await;
                                break;
                            }
                            Ok(None) => {}
                            Err(e) => {
                                let _ = event_tx.try_send(ManagerEvent::Log(LogEvent {
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
                            }));
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

    pub async fn stop_service(
        &self,
        project_id: String,
        service_id: String,
        service_name: String,
    ) -> Result<(), AppError> {
        let composite_id = format!("{}:{}", project_id, service_id);

        let mut processes = self.processes.lock().await;
        if let Some(process) = processes.get_mut(&composite_id) {
            if let Some(mut child) = process.child.take() {
                process.running = false;
                drop(processes);

                self.emit_log(LogEvent {
                    source: "system".to_string(),
                    level: "normal".to_string(),
                    text: format!(
                        "{}Stopping {}...",
                        format_log_prefix("system", false),
                        service_name
                    ),
                    timestamp: get_timestamp(),
                    project_id: project_id.clone(),
                });

                match kill_process_group(&mut child).await {
                    Ok(_) => match child.wait().await {
                        Ok(status) => {
                            self.emit_log(LogEvent {
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
                            });
                        }
                        Err(e) => {
                            self.emit_log(LogEvent {
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
                            });
                        }
                    },
                    Err(e) => {
                        self.emit_log(LogEvent {
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
                        });
                    }
                }

                let mut urls = self.detected_urls.lock().await;
                urls.remove(&composite_id);
            }
        }

        self.emit_status().await;
        Ok(())
    }

    pub async fn status(&self) -> StatusEvent {
        let processes = self.processes.lock().await;
        let urls = self.detected_urls.lock().await;
        build_status(&processes, &urls)
    }

    pub fn cleanup_processes_sync(&self) {
        use std::process::Command;

        if let Ok(processes) = self.processes.try_lock() {
            for process in processes.values() {
                if let Some(child) = process.child.as_ref() {
                    if let Some(pid) = child.id() {
                        #[cfg(unix)]
                        {
                            let _ = Command::new("kill")
                                .args(["-TERM", &format!("-{}", pid)])
                                .output();
                            std::thread::sleep(std::time::Duration::from_millis(100));
                            let _ = Command::new("kill")
                                .args(["-KILL", &format!("-{}", pid)])
                                .output();
                        }

                        #[cfg(windows)]
                        {
                            let _ = Command::new("taskkill")
                                .args(["/F", "/T", "/PID", &pid.to_string()])
                                .output();
                        }
                    }
                }
            }
        }
    }

    async fn emit_status(&self) {
        let status = self.status().await;
        let _ = self.event_tx.send(ManagerEvent::Status(status)).await;
    }

    fn emit_log(&self, log: LogEvent) {
        let _ = self.event_tx.try_send(ManagerEvent::Log(log));
    }
}

fn extract_vite_url(regex: &Regex, line: &str) -> Option<String> {
    regex
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

fn build_status(
    processes: &HashMap<String, ProcessState>,
    urls: &HashMap<String, String>,
) -> StatusEvent {
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
    StatusEvent { services }
}
