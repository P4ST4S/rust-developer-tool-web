#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod process;
mod state;

use commands::*;
use state::AppState;
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            start_frontend,
            stop_frontend,
            start_backend,
            stop_backend,
            get_status,
            open_browser,
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                let app = window.app_handle();
                if let Some(state) = app.try_state::<AppState>() {
                    cleanup_processes_sync(&state);
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn cleanup_processes_sync(state: &AppState) {
    use std::process::Command;

    // Kill frontend
    if let Ok(frontend_guard) = state.frontend.try_lock() {
        if let Some(child) = frontend_guard.child.as_ref() {
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

    // Kill backend
    if let Ok(backend_guard) = state.backend.try_lock() {
        if let Some(child) = backend_guard.child.as_ref() {
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
