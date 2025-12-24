use eframe::egui;
use regex::Regex;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::Mutex;

use crate::logs::{self, ColoredText, LogLine};
use crate::process::{self, ProcessHandle};

pub struct DevLauncher {
    logs: Arc<Mutex<Vec<LogLine>>>,
    frontend_handle: Arc<Mutex<ProcessHandle>>,
    backend_handle: Arc<Mutex<ProcessHandle>>,
    frontend_url: Arc<Mutex<Option<String>>>,
    dark_mode: bool,
}

impl DevLauncher {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        
        Self {
            logs: Arc::new(Mutex::new(Vec::new())),
            frontend_handle: Arc::new(Mutex::new(ProcessHandle::default())),
            backend_handle: Arc::new(Mutex::new(ProcessHandle::default())),
            frontend_url: Arc::new(Mutex::new(None)),
            dark_mode: true,
        }
    }

    fn is_frontend_running(&self) -> bool {
        self.frontend_handle.try_lock().map(|g| g.child.is_some()).unwrap_or(false)
    }

    fn is_backend_running(&self) -> bool {
        self.backend_handle.try_lock().map(|g| g.child.is_some()).unwrap_or(false)
    }

    fn get_frontend_url(&self) -> Option<String> {
        self.frontend_url.try_lock().ok().and_then(|guard| guard.clone())
    }

    fn extract_vite_url(line: &str) -> Option<String> {
        // Match Vite URLs like "Local:   http://localhost:5173/"
        let url_regex = Regex::new(r"(?:Local|local):\s+(https?://[^\s]+)").ok()?;
        url_regex.captures(line)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().to_string())
    }

    fn launch_frontend(&self) {
        let logs = Arc::clone(&self.logs);
        let handle = Arc::clone(&self.frontend_handle);
        let frontend_url = Arc::clone(&self.frontend_url);
        let dark_mode = self.dark_mode;
        
        // Reset URL when starting
        if let Ok(mut url) = frontend_url.try_lock() {
            *url = None;
        }
        
        tokio::spawn(async move {
            logs.lock().await.push(LogLine {
                segments: vec![ColoredText {
                    text: "[SYSTEM] Starting frontend...".to_string(),
                    color: egui::Color32::from_rgb(255, 200, 100),
                }],
            });
            
            let mut cmd = process::create_process_group_command("pnpm", &["dev"], "../frontend");
            let mut child = match cmd.spawn() {
                Ok(child) => child,
                Err(e) => {
                    logs.lock().await.push(LogLine {
                        segments: vec![ColoredText {
                            text: format!("[ERROR] Failed to start frontend: {}", e),
                            color: egui::Color32::from_rgb(255, 100, 100),
                        }],
                    });
                    return;
                }
            };

            let child_id = child.id();
            
            if let Some(stdout) = child.stdout.take() {
                let logs_clone = Arc::clone(&logs);
                let url_clone = Arc::clone(&frontend_url);
                let ansi_regex = Regex::new(r"\x1b\[([0-9;]+)m").unwrap();
                tokio::spawn(async move {
                    let reader = BufReader::new(stdout);
                    let mut lines = reader.lines();
                    
                    while let Ok(Some(line)) = lines.next_line().await {
                        // Try to extract Vite URL
                        if let Some(url) = Self::extract_vite_url(&line) {
                            let mut url_guard = url_clone.lock().await;
                            *url_guard = Some(url.clone());
                            logs_clone.lock().await.push(LogLine {
                                segments: vec![ColoredText {
                                    text: format!("[SYSTEM] Frontend URL detected: {}", url),
                                    color: egui::Color32::from_rgb(100, 255, 100),
                                }],
                            });
                        }
                        
                        let full_line = format!("[FRONTEND] {}", line);
                        let segments = logs::parse_ansi_line(&ansi_regex, &full_line, "[FRONTEND]", dark_mode);
                        logs_clone.lock().await.push(LogLine { segments });
                    }
                });
            }

            if let Some(stderr) = child.stderr.take() {
                let logs_clone = Arc::clone(&logs);
                let url_clone = Arc::clone(&frontend_url);
                let ansi_regex = Regex::new(r"\x1b\[([0-9;]+)m").unwrap();
                tokio::spawn(async move {
                    let reader = BufReader::new(stderr);
                    let mut lines = reader.lines();
                    
                    while let Ok(Some(line)) = lines.next_line().await {
                        // Try to extract Vite URL from stderr too
                        if let Some(url) = Self::extract_vite_url(&line) {
                            let mut url_guard = url_clone.lock().await;
                            *url_guard = Some(url.clone());
                            logs_clone.lock().await.push(LogLine {
                                segments: vec![ColoredText {
                                    text: format!("[SYSTEM] Frontend URL detected: {}", url),
                                    color: egui::Color32::from_rgb(100, 255, 100),
                                }],
                            });
                        }
                        
                        let full_line = format!("[FRONTEND ERROR] {}", line);
                        let segments = logs::parse_ansi_line(&ansi_regex, &full_line, "[ERROR]", dark_mode);
                        logs_clone.lock().await.push(LogLine { segments });
                    }
                });
            }

            handle.lock().await.child = Some(child);
            
            let handle_clone = Arc::clone(&handle);
            let logs_clone = Arc::clone(&logs);
            let url_clone = Arc::clone(&frontend_url);
            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    
                    let mut guard = handle_clone.lock().await;
                    if let Some(child) = &mut guard.child {
                        match child.try_wait() {
                            Ok(Some(status)) => {
                                logs_clone.lock().await.push(LogLine {
                                    segments: vec![ColoredText {
                                        text: format!("[SYSTEM] Frontend stopped (PID: {:?}, status: {:?})", child_id, status),
                                        color: egui::Color32::from_rgb(255, 200, 100),
                                    }],
                                });
                                guard.child = None;
                                // Clear URL when frontend stops
                                let mut url = url_clone.lock().await;
                                *url = None;
                                break;
                            }
                            Ok(None) => {}
                            Err(e) => {
                                logs_clone.lock().await.push(LogLine {
                                    segments: vec![ColoredText {
                                        text: format!("[ERROR] Error checking frontend status: {}", e),
                                        color: egui::Color32::from_rgb(255, 100, 100),
                                    }],
                                });
                                guard.child = None;
                                let mut url = url_clone.lock().await;
                                *url = None;
                                break;
                            }
                        }
                    } else {
                        break;
                    }
                    drop(guard);
                }
            });
        });
    }

    fn launch_backend(&self) {
        let logs = Arc::clone(&self.logs);
        let handle = Arc::clone(&self.backend_handle);
        let dark_mode = self.dark_mode;
        
        tokio::spawn(async move {
            logs.lock().await.push(LogLine {
                segments: vec![ColoredText {
                    text: "[SYSTEM] Starting backend...".to_string(),
                    color: egui::Color32::from_rgb(255, 200, 100),
                }],
            });
            
            let mut cmd = process::create_process_group_command("pnpm", &["start:dev"], "../backend");
            let mut child = match cmd.spawn() {
                Ok(child) => child,
                Err(e) => {
                    logs.lock().await.push(LogLine {
                        segments: vec![ColoredText {
                            text: format!("[ERROR] Failed to start backend: {}", e),
                            color: egui::Color32::from_rgb(255, 100, 100),
                        }],
                    });
                    return;
                }
            };

            let child_id = child.id();
            
            if let Some(stdout) = child.stdout.take() {
                let logs_clone = Arc::clone(&logs);
                let ansi_regex = Regex::new(r"\x1b\[([0-9;]+)m").unwrap();
                tokio::spawn(async move {
                    let reader = BufReader::new(stdout);
                    let mut lines = reader.lines();
                    
                    while let Ok(Some(line)) = lines.next_line().await {
                        let full_line = format!("[BACKEND] {}", line);
                        let segments = logs::parse_ansi_line(&ansi_regex, &full_line, "[BACKEND]", dark_mode);
                        logs_clone.lock().await.push(LogLine { segments });
                    }
                });
            }

            if let Some(stderr) = child.stderr.take() {
                let logs_clone = Arc::clone(&logs);
                let ansi_regex = Regex::new(r"\x1b\[([0-9;]+)m").unwrap();
                tokio::spawn(async move {
                    let reader = BufReader::new(stderr);
                    let mut lines = reader.lines();
                    
                    while let Ok(Some(line)) = lines.next_line().await {
                        let full_line = format!("[BACKEND ERROR] {}", line);
                        let segments = logs::parse_ansi_line(&ansi_regex, &full_line, "[ERROR]", dark_mode);
                        logs_clone.lock().await.push(LogLine { segments });
                    }
                });
            }

            handle.lock().await.child = Some(child);
            
            let handle_clone = Arc::clone(&handle);
            let logs_clone = Arc::clone(&logs);
            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    
                    let mut guard = handle_clone.lock().await;
                    if let Some(child) = &mut guard.child {
                        match child.try_wait() {
                            Ok(Some(status)) => {
                                logs_clone.lock().await.push(LogLine {
                                    segments: vec![ColoredText {
                                        text: format!("[SYSTEM] Backend stopped (PID: {:?}, status: {:?})", child_id, status),
                                        color: egui::Color32::from_rgb(255, 200, 100),
                                    }],
                                });
                                guard.child = None;
                                break;
                            }
                            Ok(None) => {}
                            Err(e) => {
                                logs_clone.lock().await.push(LogLine {
                                    segments: vec![ColoredText {
                                        text: format!("[ERROR] Error checking backend status: {}", e),
                                        color: egui::Color32::from_rgb(255, 100, 100),
                                    }],
                                });
                                guard.child = None;
                                break;
                            }
                        }
                    } else {
                        break;
                    }
                    drop(guard);
                }
            });
        });
    }

    fn stop_frontend(&self) {
        let handle = Arc::clone(&self.frontend_handle);
        let logs = Arc::clone(&self.logs);
        let frontend_url = Arc::clone(&self.frontend_url);
        
        tokio::spawn(async move {
            let mut handle_guard = handle.lock().await;
            if let Some(mut child) = handle_guard.child.take() {
                drop(handle_guard);
                
                logs.lock().await.push(LogLine {
                    segments: vec![ColoredText {
                        text: "[SYSTEM] Stopping frontend...".to_string(),
                        color: egui::Color32::from_rgb(255, 200, 100),
                    }],
                });
                
                match process::kill_process_group(&mut child).await {
                    Ok(_) => {
                        match child.wait().await {
                            Ok(status) => {
                                logs.lock().await.push(LogLine {
                                    segments: vec![ColoredText {
                                        text: format!("[SYSTEM] Frontend killed successfully (status: {:?})", status),
                                        color: egui::Color32::from_rgb(255, 200, 100),
                                    }],
                                });
                            }
                            Err(e) => {
                                logs.lock().await.push(LogLine {
                                    segments: vec![ColoredText {
                                        text: format!("[ERROR] Error waiting for frontend: {}", e),
                                        color: egui::Color32::from_rgb(255, 100, 100),
                                    }],
                                });
                            }
                        }
                    }
                    Err(e) => {
                        logs.lock().await.push(LogLine {
                            segments: vec![ColoredText {
                                text: format!("[ERROR] Failed to kill frontend: {}", e),
                                color: egui::Color32::from_rgb(255, 100, 100),
                            }],
                        });
                    }
                }
                
                // Clear URL when stopped
                let mut url = frontend_url.lock().await;
                *url = None;
            }
        });
    }

    fn stop_backend(&self) {
        let handle = Arc::clone(&self.backend_handle);
        let logs = Arc::clone(&self.logs);
        
        tokio::spawn(async move {
            let mut handle_guard = handle.lock().await;
            if let Some(mut child) = handle_guard.child.take() {
                drop(handle_guard);
                
                logs.lock().await.push(LogLine {
                    segments: vec![ColoredText {
                        text: "[SYSTEM] Stopping backend...".to_string(),
                        color: egui::Color32::from_rgb(255, 200, 100),
                    }],
                });
                
                match process::kill_process_group(&mut child).await {
                    Ok(_) => {
                        match child.wait().await {
                            Ok(status) => {
                                logs.lock().await.push(LogLine {
                                    segments: vec![ColoredText {
                                        text: format!("[SYSTEM] Backend killed successfully (status: {:?})", status),
                                        color: egui::Color32::from_rgb(255, 200, 100),
                                    }],
                                });
                            }
                            Err(e) => {
                                logs.lock().await.push(LogLine {
                                    segments: vec![ColoredText {
                                        text: format!("[ERROR] Error waiting for backend: {}", e),
                                        color: egui::Color32::from_rgb(255, 100, 100),
                                    }],
                                });
                            }
                        }
                    }
                    Err(e) => {
                        logs.lock().await.push(LogLine {
                            segments: vec![ColoredText {
                                text: format!("[ERROR] Failed to kill backend: {}", e),
                                color: egui::Color32::from_rgb(255, 100, 100),
                            }],
                        });
                    }
                }
            }
        });
    }

    fn cleanup_processes(&self) {
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            let mut frontend_guard = self.frontend_handle.blocking_lock();
            if let Some(mut child) = frontend_guard.child.take() {
                handle.block_on(async {
                    let _ = process::kill_process_group(&mut child).await;
                    let _ = child.wait().await;
                });
            }
            drop(frontend_guard);
            
            let mut backend_guard = self.backend_handle.blocking_lock();
            if let Some(mut child) = backend_guard.child.take() {
                handle.block_on(async {
                    let _ = process::kill_process_group(&mut child).await;
                    let _ = child.wait().await;
                });
            }
            drop(backend_guard);
        }
    }


}

impl eframe::App for DevLauncher {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();
        
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🚀 Dev Stack Launcher");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(if self.dark_mode { "☀ Light" } else { "🌙 Dark" }).clicked() {
                        self.dark_mode = !self.dark_mode;
                        ctx.set_visuals(if self.dark_mode {
                            egui::Visuals::dark()
                        } else {
                            egui::Visuals::light()
                        });
                    }
                });
            });
            
            ui.add_space(10.0);
            
            let frontend_running = self.is_frontend_running();
            let backend_running = self.is_backend_running();
            let frontend_url = self.get_frontend_url();
            
            ui.horizontal(|ui| {
                ui.label("Frontend:");
                if ui.add_enabled(!frontend_running, egui::Button::new("▶ Start")).clicked() {
                    self.launch_frontend();
                }
                if ui.add_enabled(frontend_running, egui::Button::new("⏹ Stop")).clicked() {
                    self.stop_frontend();
                }
                
                if frontend_running {
                    ui.colored_label(egui::Color32::from_rgb(100, 255, 100), "● Running");
                } else {
                    ui.colored_label(egui::Color32::from_rgb(150, 150, 150), "○ Stopped");
                }
            });
            
            ui.horizontal(|ui| {
                ui.label("Backend:");
                if ui.add_enabled(!backend_running, egui::Button::new("▶ Start")).clicked() {
                    self.launch_backend();
                }
                if ui.add_enabled(backend_running, egui::Button::new("⏹ Stop")).clicked() {
                    self.stop_backend();
                }
                
                if backend_running {
                    ui.colored_label(egui::Color32::from_rgb(100, 255, 100), "● Running");
                } else {
                    ui.colored_label(egui::Color32::from_rgb(150, 150, 150), "○ Stopped");
                }
            });
            
            ui.add_space(10.0);
            
            ui.horizontal(|ui| {
                let can_open = frontend_running && backend_running && frontend_url.is_some();
                
                if ui.add_enabled(can_open, egui::Button::new("🌐 Open Frontend in Browser")).clicked() {
                    if let Some(url) = frontend_url {
                        if let Err(e) = open::that(&url) {
                            eprintln!("Failed to open browser: {}", e);
                        }
                    }
                }
                
                if ui.button("🗑 Clear Logs").clicked() {
                    if let Ok(mut logs) = self.logs.try_lock() {
                        logs.clear();
                    }
                }
            });
            
            ui.separator();
            
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);
                    
                    if let Ok(logs) = self.logs.try_lock() {
                        for log_line in logs.iter() {
                            ui.horizontal_wrapped(|ui| {
                                for segment in &log_line.segments {
                                    ui.colored_label(segment.color, &segment.text);
                                }
                            });
                        }
                    }
                });
        });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        println!("Application closing - cleaning up processes...");
        self.cleanup_processes();
    }
}
