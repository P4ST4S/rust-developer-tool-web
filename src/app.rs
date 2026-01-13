use eframe::egui;
use regex::Regex;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::Mutex;

use crate::logs::{self, ColoredText, LogLine};
use crate::process::{self, ProcessHandle};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogSource {
    All,
    Frontend,
    Backend,
    System,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    All,
    Normal,
    Error,
}

pub struct DevLauncher {
    logs: Arc<Mutex<Vec<LogLine>>>,
    frontend_handle: Arc<Mutex<ProcessHandle>>,
    backend_handle: Arc<Mutex<ProcessHandle>>,
    frontend_url: Arc<Mutex<Option<String>>>,
    dark_mode: bool,
    filter_source: LogSource,
    filter_level: LogLevel,
    search_query: String,
    current_search_match: usize,
}

impl DevLauncher {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Load custom font with better Unicode support
        let mut fonts = egui::FontDefinitions::default();
        
        // Load Noto Sans Mono for better Unicode support
        fonts.font_data.insert(
            "noto_sans_mono".to_owned(),
            Arc::new(egui::FontData::from_static(include_bytes!("../NotoSansMono-Regular.ttf"))),
        );
        
        // Set as the primary monospace font
        fonts
            .families
            .get_mut(&egui::FontFamily::Monospace)
            .unwrap()
            .insert(0, "noto_sans_mono".to_owned());
        
        cc.egui_ctx.set_fonts(fonts);
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        
        Self {
            logs: Arc::new(Mutex::new(Vec::new())),
            frontend_handle: Arc::new(Mutex::new(ProcessHandle::default())),
            backend_handle: Arc::new(Mutex::new(ProcessHandle::default())),
            frontend_url: Arc::new(Mutex::new(None)),
            dark_mode: true,
            filter_source: LogSource::All,
            filter_level: LogLevel::All,
            search_query: String::new(),
            current_search_match: 0,
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
                let ansi_regex = Regex::new(r"\x1b\[([0-9;]*[A-Za-z])").unwrap();
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
                let ansi_regex = Regex::new(r"\x1b\[([0-9;]*[A-Za-z])").unwrap();
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
                let ansi_regex = Regex::new(r"\x1b\[([0-9;]*[A-Za-z])").unwrap();
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
                let ansi_regex = Regex::new(r"\x1b\[([0-9;]*[A-Za-z])").unwrap();
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
        use std::process::Command;
        
        // Kill frontend synchronously
        if let Ok(mut frontend_guard) = self.frontend_handle.try_lock() {
            if let Some(child) = frontend_guard.child.as_ref() {
                let pid = child.id().expect("Failed to get PID");
                
                #[cfg(unix)]
                {
                    // Kill the entire process group
                    let _ = Command::new("kill")
                        .args(&["-TERM", &format!("-{}", pid)])
                        .output();
                    
                    // Give it a moment to terminate gracefully
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    
                    // Force kill if still alive
                    let _ = Command::new("kill")
                        .args(&["-KILL", &format!("-{}", pid)])
                        .output();
                }
                
                #[cfg(windows)]
                {
                    let _ = Command::new("taskkill")
                        .args(&["/F", "/T", "/PID", &pid.to_string()])
                        .output();
                }
            }
            frontend_guard.child = None;
        }
        
        // Kill backend synchronously
        if let Ok(mut backend_guard) = self.backend_handle.try_lock() {
            if let Some(child) = backend_guard.child.as_ref() {
                let pid = child.id().expect("Failed to get PID");
                
                #[cfg(unix)]
                {
                    // Kill the entire process group
                    let _ = Command::new("kill")
                        .args(&["-TERM", &format!("-{}", pid)])
                        .output();
                    
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    
                    let _ = Command::new("kill")
                        .args(&["-KILL", &format!("-{}", pid)])
                        .output();
                }
                
                #[cfg(windows)]
                {
                    let _ = Command::new("taskkill")
                        .args(&["/F", "/T", "/PID", &pid.to_string()])
                        .output();
                }
            }
            backend_guard.child = None;
        }
    }

    fn should_display_log(&self, log_line: &LogLine) -> bool {
        // Determine log source and level from first segment
        let first_text = log_line.segments.first().map(|s| s.text.as_str()).unwrap_or("");
        
        let (source, level) = if first_text.contains("[FRONTEND ERROR]") {
            (LogSource::Frontend, LogLevel::Error)
        } else if first_text.contains("[FRONTEND]") {
            (LogSource::Frontend, LogLevel::Normal)
        } else if first_text.contains("[BACKEND ERROR]") {
            (LogSource::Backend, LogLevel::Error)
        } else if first_text.contains("[BACKEND]") {
            (LogSource::Backend, LogLevel::Normal)
        } else if first_text.contains("[ERROR]") {
            (LogSource::System, LogLevel::Error)
        } else if first_text.contains("[SYSTEM]") {
            (LogSource::System, LogLevel::Normal)
        } else {
            (LogSource::System, LogLevel::Normal)
        };
        
        // Check source filter
        let source_match = self.filter_source == LogSource::All || self.filter_source == source;
        
        // Check level filter
        let level_match = self.filter_level == LogLevel::All || self.filter_level == level;
        
        // Check search query
        let search_match = if self.search_query.is_empty() {
            true
        } else {
            let query_lower = self.search_query.to_lowercase();
            log_line.segments.iter().any(|seg| seg.text.to_lowercase().contains(&query_lower))
        };
        
        source_match && level_match && search_match
    }


}

// Helper function to adjust color brightness for light mode
fn adjust_color_for_theme(color: egui::Color32, dark_mode: bool) -> egui::Color32 {
    if dark_mode {
        // In dark mode, keep original bright colors
        color
    } else {
        // In light mode, darken bright colors for better contrast on white
        let [r, g, b, a] = color.to_array();
        
        // If it's a bright color (high RGB values), darken it
        if r > 150 || g > 150 || b > 150 {
            egui::Color32::from_rgba_premultiplied(
                (r as f32 * 0.4) as u8,
                (g as f32 * 0.4) as u8,
                (b as f32 * 0.4) as u8,
                a
            )
        } else {
            // Already dark enough
            color
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
            
            // Filter section
            ui.horizontal(|ui| {
                ui.label("🔍 Filtres:");
                
                ui.separator();
                ui.label("Source:");
                if ui.selectable_label(self.filter_source == LogSource::All, "Tous").clicked() {
                    self.filter_source = LogSource::All;
                }
                if ui.selectable_label(self.filter_source == LogSource::Frontend, "Frontend").clicked() {
                    self.filter_source = LogSource::Frontend;
                }
                if ui.selectable_label(self.filter_source == LogSource::Backend, "Backend").clicked() {
                    self.filter_source = LogSource::Backend;
                }
                if ui.selectable_label(self.filter_source == LogSource::System, "System").clicked() {
                    self.filter_source = LogSource::System;
                }
                
                ui.separator();
                ui.label("Niveau:");
                if ui.selectable_label(self.filter_level == LogLevel::All, "Tous").clicked() {
                    self.filter_level = LogLevel::All;
                }
                if ui.selectable_label(self.filter_level == LogLevel::Normal, "Normal").clicked() {
                    self.filter_level = LogLevel::Normal;
                }
                if ui.selectable_label(self.filter_level == LogLevel::Error, "Error").clicked() {
                    self.filter_level = LogLevel::Error;
                }
            });
            
            ui.separator();
            
            // Search section with match counter and navigation
            ui.horizontal(|ui| {
                ui.label("🔎 Recherche:");
                let search_changed = ui.text_edit_singleline(&mut self.search_query).changed();
                
                // Reset match index if search changed
                if search_changed {
                    self.current_search_match = 0;
                }
                
                // Count matches
                let match_count = if !self.search_query.is_empty() {
                    if let Ok(logs) = self.logs.try_lock() {
                        let query_lower = self.search_query.to_lowercase();
                        logs.iter()
                            .filter(|log_line| self.should_display_log(log_line))
                            .map(|log_line| {
                                log_line.segments.iter()
                                    .map(|seg| {
                                        let text_lower = seg.text.to_lowercase();
                                        let mut count = 0;
                                        let mut start = 0;
                                        while let Some(pos) = text_lower[start..].find(&query_lower) {
                                            count += 1;
                                            start += pos + query_lower.len();
                                        }
                                        count
                                    })
                                    .sum::<usize>()
                            })
                            .sum()
                    } else {
                        0
                    }
                } else {
                    0
                };
                
                // Navigation buttons with ASCII arrows
                if match_count > 0 {
                    ui.label(format!("{}/{}", self.current_search_match + 1, match_count));
                    
                    if ui.button("  ^  ").on_hover_text("Match précédent").clicked() {
                        self.current_search_match = if self.current_search_match == 0 {
                            match_count - 1
                        } else {
                            self.current_search_match - 1
                        };
                    }
                    
                    if ui.button("  v  ").on_hover_text("Match suivant").clicked() {
                        self.current_search_match = (self.current_search_match + 1) % match_count;
                    }
                }
                
                if ui.button("❌").on_hover_text("Effacer la recherche").clicked() {
                    self.search_query.clear();
                    self.current_search_match = 0;
                }
                
                // Copy visible logs button
                if ui.button("📋").on_hover_text("Copier les logs visibles").clicked() {
                    if let Ok(logs) = self.logs.try_lock() {
                        let mut text = String::new();
                        for log_line in logs.iter() {
                            if !self.should_display_log(log_line) {
                                continue;
                            }
                            for segment in &log_line.segments {
                                text.push_str(&segment.text);
                            }
                            text.push('\n');
                        }
                        ctx.copy_text(text);
                    }
                }
            });
            
            ui.separator();
            
            // Display logs with color, word wrap, search highlighting (scroll enabled, no text selection)
            let mut scroll_to_match = false;
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);
                    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
                    
                    let query_lower = self.search_query.to_lowercase();
                    let has_search = !self.search_query.is_empty();
                    let mut global_match_index = 0;
                    
                    if let Ok(logs) = self.logs.try_lock() {
                        for log_line in logs.iter() {
                            if !self.should_display_log(log_line) {
                                continue;
                            }
                            
                            ui.horizontal_wrapped(|ui| {
                                ui.spacing_mut().item_spacing.x = 0.0;
                                
                                for segment in &log_line.segments {
                                    // Adjust color based on current theme
                                    let adjusted_color = adjust_color_for_theme(segment.color, self.dark_mode);
                                    
                                    if has_search {
                                        // Split text by search query and highlight matches
                                        let text_lower = segment.text.to_lowercase();
                                        let mut last_end = 0;
                                        let mut match_positions = Vec::new();
                                        
                                        // Find all match positions
                                        let mut start = 0;
                                        while let Some(pos) = text_lower[start..].find(&query_lower) {
                                            let absolute_pos = start + pos;
                                            match_positions.push(absolute_pos);
                                            start = absolute_pos + query_lower.len();
                                        }
                                        
                                        // Render text with highlights
                                        for match_pos in match_positions {
                                            // Text before match
                                            if match_pos > last_end {
                                                let before = &segment.text[last_end..match_pos];
                                                ui.label(
                                                    egui::RichText::new(before).color(adjusted_color)
                                                );
                                            }
                                            
                                            // Highlighted match
                                            let match_end = (match_pos + self.search_query.len()).min(segment.text.len());
                                            let matched_text = &segment.text[match_pos..match_end];
                                            
                                            let is_current_match = global_match_index == self.current_search_match;
                                            let bg_color = if is_current_match {
                                                egui::Color32::from_rgb(255, 165, 0) // Orange for current
                                            } else {
                                                egui::Color32::from_rgb(255, 255, 0) // Yellow for others
                                            };
                                            
                                            let label_response = ui.label(
                                                egui::RichText::new(matched_text)
                                                    .color(egui::Color32::BLACK)
                                                    .background_color(bg_color)
                                            );
                                            
                                            // Scroll to current match
                                            if is_current_match && !scroll_to_match {
                                                label_response.scroll_to_me(Some(egui::Align::Center));
                                                scroll_to_match = true;
                                            }
                                            
                                            global_match_index += 1;
                                            last_end = match_end;
                                        }
                                        
                                        // Text after last match
                                        if last_end < segment.text.len() {
                                            let after = &segment.text[last_end..];
                                            ui.label(
                                                egui::RichText::new(after).color(adjusted_color)
                                            );
                                        }
                                    } else {
                                        // No search - just display with adjusted color
                                        ui.label(
                                            egui::RichText::new(&segment.text).color(adjusted_color)
                                        );
                                    }
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
