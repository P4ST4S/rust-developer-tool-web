use crate::process::{create_process_group_command, kill_process_group};
use crate::state::AppState;
use regex::Regex;
use serde::Serialize;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, State};
use tokio::io::{AsyncBufReadExt, BufReader};

fn get_project_root(_app: &AppHandle) -> PathBuf {
    // CARGO_MANIFEST_DIR is set during compilation to src-tauri directory
    // We go up 2 levels: src-tauri -> rust-gui -> datakeen-refacto
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir)
        .parent() // rust-gui
        .and_then(|p| p.parent()) // datakeen-refacto
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("/Users/antoinerospars/projects/datakeen-refacto"))
}

#[derive(Clone, Serialize)]
pub struct LogEvent {
    pub source: String,
    pub level: String,
    pub text: String,
    pub timestamp: String,
}

#[derive(Clone, Serialize)]
pub struct StatusEvent {
    pub frontend_running: bool,
    pub backend_running: bool,
    pub frontend_url: Option<String>,
}

fn extract_vite_url(line: &str) -> Option<String> {
    let url_regex = Regex::new(r"(?:Local|local):\s+(https?://[^\s]+)").ok()?;
    url_regex
        .captures(line)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().to_string())
}

fn format_log_prefix(source: &str, is_error: bool) -> String {
    let color_code = match (source, is_error) {
        ("frontend", false) => "\x1b[38;5;75m",
        ("frontend", true) => "\x1b[38;5;196m",
        ("backend", false) => "\x1b[38;5;114m",
        ("backend", true) => "\x1b[38;5;196m",
        ("system", _) => "\x1b[38;5;214m",
        _ => "\x1b[0m",
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
    let frontend = state.frontend.lock().await;
    let backend = state.backend.lock().await;
    let url = state.frontend_url.lock().await;

    let _ = app.emit(
        "status-change",
        StatusEvent {
            frontend_running: frontend.running,
            backend_running: backend.running,
            frontend_url: url.clone(),
        },
    );
}

#[tauri::command]
pub async fn start_frontend(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    {
        let frontend = state.frontend.lock().await;
        if frontend.running {
            return Err("Frontend already running".to_string());
        }
    }

    let _ = app.emit(
        "log",
        LogEvent {
            source: "system".to_string(),
            level: "normal".to_string(),
            text: format!("{}Starting frontend...", format_log_prefix("system", false)),
            timestamp: get_timestamp(),
        },
    );

    let project_root = get_project_root(&app);
    let frontend_dir = project_root.join("frontend");
    let frontend_dir_str = frontend_dir.to_string_lossy().to_string();

    let mut cmd = create_process_group_command("pnpm", &["dev"], &frontend_dir_str);
    let mut child = cmd.spawn().map_err(|e| {
        let _ = app.emit(
            "log",
            LogEvent {
                source: "system".to_string(),
                level: "error".to_string(),
                text: format!(
                    "{}Failed to start frontend: {}",
                    format_log_prefix("system", true),
                    e
                ),
                timestamp: get_timestamp(),
            },
        );
        e.to_string()
    })?;

    let child_id = child.id();

    if let Some(stdout) = child.stdout.take() {
        let app_clone = app.clone();
        let url_state = state.frontend_url.clone();
        tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();

            while let Ok(Some(line)) = lines.next_line().await {
                if let Some(url) = extract_vite_url(&line) {
                    let mut url_guard = url_state.lock().await;
                    *url_guard = Some(url.clone());
                    let _ = app_clone.emit("frontend-url", url.clone());
                    let _ = app_clone.emit(
                        "log",
                        LogEvent {
                            source: "system".to_string(),
                            level: "normal".to_string(),
                            text: format!(
                                "{}Frontend URL detected: {}",
                                format_log_prefix("system", false),
                                url
                            ),
                            timestamp: get_timestamp(),
                        },
                    );
                }

                let _ = app_clone.emit(
                    "log",
                    LogEvent {
                        source: "frontend".to_string(),
                        level: "normal".to_string(),
                        text: format!("{}{}", format_log_prefix("frontend", false), line),
                        timestamp: get_timestamp(),
                    },
                );
            }
        });
    }

    if let Some(stderr) = child.stderr.take() {
        let app_clone = app.clone();
        let url_state = state.frontend_url.clone();
        tokio::spawn(async move {
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();

            while let Ok(Some(line)) = lines.next_line().await {
                if let Some(url) = extract_vite_url(&line) {
                    let mut url_guard = url_state.lock().await;
                    *url_guard = Some(url.clone());
                    let _ = app_clone.emit("frontend-url", url.clone());
                }

                let _ = app_clone.emit(
                    "log",
                    LogEvent {
                        source: "frontend".to_string(),
                        level: "error".to_string(),
                        text: format!("{}{}", format_log_prefix("frontend", true), line),
                        timestamp: get_timestamp(),
                    },
                );
            }
        });
    }

    {
        let mut frontend = state.frontend.lock().await;
        frontend.child = Some(child);
        frontend.running = true;
    }

    emit_status(&app, &state).await;

    let app_monitor = app.clone();
    let frontend_state = state.frontend.clone();
    let url_state = state.frontend_url.clone();
    let backend_state = state.backend.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

            let mut guard = frontend_state.lock().await;
            if let Some(child) = &mut guard.child {
                match child.try_wait() {
                    Ok(Some(status)) => {
                        let _ = app_monitor.emit(
                            "log",
                            LogEvent {
                                source: "system".to_string(),
                                level: "normal".to_string(),
                                text: format!(
                                    "{}Frontend stopped (PID: {:?}, status: {:?})",
                                    format_log_prefix("system", false),
                                    child_id,
                                    status
                                ),
                                timestamp: get_timestamp(),
                            },
                        );
                        guard.child = None;
                        guard.running = false;

                        let mut url = url_state.lock().await;
                        *url = None;

                        let backend = backend_state.lock().await;
                        let _ = app_monitor.emit(
                            "status-change",
                            StatusEvent {
                                frontend_running: false,
                                backend_running: backend.running,
                                frontend_url: None,
                            },
                        );
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
                                    "{}Error checking frontend status: {}",
                                    format_log_prefix("system", true),
                                    e
                                ),
                                timestamp: get_timestamp(),
                            },
                        );
                        guard.child = None;
                        guard.running = false;
                        break;
                    }
                }
            } else {
                break;
            }
        }
    });

    Ok(())
}

#[tauri::command]
pub async fn stop_frontend(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let mut guard = state.frontend.lock().await;
    if let Some(mut child) = guard.child.take() {
        guard.running = false;
        drop(guard);

        let _ = app.emit(
            "log",
            LogEvent {
                source: "system".to_string(),
                level: "normal".to_string(),
                text: format!("{}Stopping frontend...", format_log_prefix("system", false)),
                timestamp: get_timestamp(),
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
                                "{}Frontend killed successfully (status: {:?})",
                                format_log_prefix("system", false),
                                status
                            ),
                            timestamp: get_timestamp(),
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
                                "{}Error waiting for frontend: {}",
                                format_log_prefix("system", true),
                                e
                            ),
                            timestamp: get_timestamp(),
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
                            "{}Failed to kill frontend: {}",
                            format_log_prefix("system", true),
                            e
                        ),
                        timestamp: get_timestamp(),
                    },
                );
            }
        }

        let mut url = state.frontend_url.lock().await;
        *url = None;
    }

    emit_status(&app, &state).await;
    Ok(())
}

#[tauri::command]
pub async fn start_backend(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    {
        let backend = state.backend.lock().await;
        if backend.running {
            return Err("Backend already running".to_string());
        }
    }

    let _ = app.emit(
        "log",
        LogEvent {
            source: "system".to_string(),
            level: "normal".to_string(),
            text: format!("{}Starting backend...", format_log_prefix("system", false)),
            timestamp: get_timestamp(),
        },
    );

    let backend_dir = get_project_root(&app).join("backend");
    let backend_dir_str = backend_dir.to_string_lossy().to_string();
    let mut cmd = create_process_group_command("pnpm", &["start:dev"], &backend_dir_str);
    let mut child = cmd.spawn().map_err(|e| {
        let _ = app.emit(
            "log",
            LogEvent {
                source: "system".to_string(),
                level: "error".to_string(),
                text: format!(
                    "{}Failed to start backend: {}",
                    format_log_prefix("system", true),
                    e
                ),
                timestamp: get_timestamp(),
            },
        );
        e.to_string()
    })?;

    let child_id = child.id();

    if let Some(stdout) = child.stdout.take() {
        let app_clone = app.clone();
        tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();

            while let Ok(Some(line)) = lines.next_line().await {
                let _ = app_clone.emit(
                    "log",
                    LogEvent {
                        source: "backend".to_string(),
                        level: "normal".to_string(),
                        text: format!("{}{}", format_log_prefix("backend", false), line),
                        timestamp: get_timestamp(),
                    },
                );
            }
        });
    }

    if let Some(stderr) = child.stderr.take() {
        let app_clone = app.clone();
        tokio::spawn(async move {
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();

            while let Ok(Some(line)) = lines.next_line().await {
                let _ = app_clone.emit(
                    "log",
                    LogEvent {
                        source: "backend".to_string(),
                        level: "error".to_string(),
                        text: format!("{}{}", format_log_prefix("backend", true), line),
                        timestamp: get_timestamp(),
                    },
                );
            }
        });
    }

    {
        let mut backend = state.backend.lock().await;
        backend.child = Some(child);
        backend.running = true;
    }

    emit_status(&app, &state).await;

    let app_monitor = app.clone();
    let backend_state = state.backend.clone();
    let frontend_state = state.frontend.clone();
    let url_state = state.frontend_url.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

            let mut guard = backend_state.lock().await;
            if let Some(child) = &mut guard.child {
                match child.try_wait() {
                    Ok(Some(status)) => {
                        let _ = app_monitor.emit(
                            "log",
                            LogEvent {
                                source: "system".to_string(),
                                level: "normal".to_string(),
                                text: format!(
                                    "{}Backend stopped (PID: {:?}, status: {:?})",
                                    format_log_prefix("system", false),
                                    child_id,
                                    status
                                ),
                                timestamp: get_timestamp(),
                            },
                        );
                        guard.child = None;
                        guard.running = false;

                        let frontend = frontend_state.lock().await;
                        let url = url_state.lock().await;
                        let _ = app_monitor.emit(
                            "status-change",
                            StatusEvent {
                                frontend_running: frontend.running,
                                backend_running: false,
                                frontend_url: url.clone(),
                            },
                        );
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
                                    "{}Error checking backend status: {}",
                                    format_log_prefix("system", true),
                                    e
                                ),
                                timestamp: get_timestamp(),
                            },
                        );
                        guard.child = None;
                        guard.running = false;
                        break;
                    }
                }
            } else {
                break;
            }
        }
    });

    Ok(())
}

#[tauri::command]
pub async fn stop_backend(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let mut guard = state.backend.lock().await;
    if let Some(mut child) = guard.child.take() {
        guard.running = false;
        drop(guard);

        let _ = app.emit(
            "log",
            LogEvent {
                source: "system".to_string(),
                level: "normal".to_string(),
                text: format!("{}Stopping backend...", format_log_prefix("system", false)),
                timestamp: get_timestamp(),
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
                                "{}Backend killed successfully (status: {:?})",
                                format_log_prefix("system", false),
                                status
                            ),
                            timestamp: get_timestamp(),
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
                                "{}Error waiting for backend: {}",
                                format_log_prefix("system", true),
                                e
                            ),
                            timestamp: get_timestamp(),
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
                            "{}Failed to kill backend: {}",
                            format_log_prefix("system", true),
                            e
                        ),
                        timestamp: get_timestamp(),
                    },
                );
            }
        }
    }

    emit_status(&app, &state).await;
    Ok(())
}

#[tauri::command]
pub async fn get_status(state: State<'_, AppState>) -> Result<StatusEvent, String> {
    let frontend = state.frontend.lock().await;
    let backend = state.backend.lock().await;
    let url = state.frontend_url.lock().await;

    Ok(StatusEvent {
        frontend_running: frontend.running,
        backend_running: backend.running,
        frontend_url: url.clone(),
    })
}

#[tauri::command]
pub fn open_browser(url: String) -> Result<(), String> {
    open::that(&url).map_err(|e| e.to_string())
}
