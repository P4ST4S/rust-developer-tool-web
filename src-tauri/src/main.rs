#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod config;
mod events;
mod process;
mod process_manager;
mod state;

use commands::*;
use config::load_config;
use events::{LogEvent, ManagerEvent};
use state::AppState;
use tauri::{Emitter, Manager};
use tokio::sync::mpsc;
use tokio::time::{self, Duration, MissedTickBehavior};

fn main() {
    let (event_tx, event_rx) = mpsc::channel::<ManagerEvent>(2048);
    let app_state = AppState::new(event_tx);

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(move |app| {
            let app_handle = app.handle();
            tauri::async_runtime::spawn(forward_manager_events(app_handle.clone(), event_rx));

            // Load config at startup
            let state = app.state::<AppState>();
            if let Some(config) = load_config() {
                let mut state_config = state.config.blocking_lock();
                *state_config = Some(config);
            }
            Ok(())
        })
        .manage(app_state)
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
    state.process_manager.cleanup_processes_sync();
}

async fn forward_manager_events(
    app: tauri::AppHandle,
    mut event_rx: mpsc::Receiver<ManagerEvent>,
) {
    const LOG_FLUSH_INTERVAL: Duration = Duration::from_millis(100);
    const LOG_BATCH_SIZE: usize = 200;

    let mut pending_logs: Vec<LogEvent> = Vec::new();
    let mut interval = time::interval(LOG_FLUSH_INTERVAL);
    interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

    loop {
        tokio::select! {
            _ = interval.tick() => {
                flush_logs(&app, &mut pending_logs);
            }
            event = event_rx.recv() => {
                match event {
                    Some(ManagerEvent::Log(log)) => {
                        pending_logs.push(log);
                        if pending_logs.len() >= LOG_BATCH_SIZE {
                            flush_logs(&app, &mut pending_logs);
                        }
                    }
                    Some(ManagerEvent::Status(status)) => {
                        let _ = app.emit("status-change", status);
                    }
                    Some(ManagerEvent::ServiceUrl { service_id, url }) => {
                        let _ = app.emit(
                            "service-url",
                            serde_json::json!({
                                "serviceId": service_id,
                                "url": url,
                            }),
                        );
                    }
                    None => break,
                }
            }
        }
    }

    flush_logs(&app, &mut pending_logs);
}

fn flush_logs(app: &tauri::AppHandle, pending_logs: &mut Vec<LogEvent>) {
    match pending_logs.len() {
        0 => {}
        1 => {
            let log = pending_logs.pop().expect("len checked");
            let _ = app.emit("log", log);
        }
        _ => {
            let batch = std::mem::take(pending_logs);
            let _ = app.emit("log-batch", batch);
        }
    }
}
