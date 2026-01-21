#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod config;
mod process;
mod state;

use commands::*;
use config::load_config;
use state::AppState;
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // Load config at startup
            let state = app.state::<AppState>();
            if let Some(config) = load_config() {
                let mut state_config = state.config.blocking_lock();
                *state_config = Some(config);
            }
            Ok(())
        })
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            get_config,
            save_app_config,
            set_active_project,
            start_service,
            stop_service,
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

    if let Ok(processes) = state.processes.try_lock() {
        for (_, process) in processes.iter() {
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
