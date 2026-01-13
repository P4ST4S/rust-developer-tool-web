use eframe::egui;
use regex::Regex;

#[derive(Clone)]
pub(crate) struct ColoredText {
    pub text: String,
    pub color: egui::Color32,
}

#[derive(Clone)]
pub(crate) struct LogLine {
    pub segments: Vec<ColoredText>,
}

pub(crate) fn get_prefix_color(prefix: &str, dark_mode: bool) -> egui::Color32 {
    match prefix {
        "[FRONTEND]" => egui::Color32::from_rgb(100, 180, 255),
        "[BACKEND]" => egui::Color32::from_rgb(100, 255, 150),
        "[ERROR]" => egui::Color32::from_rgb(255, 100, 100),
        "[SYSTEM]" => egui::Color32::from_rgb(255, 200, 100),
        _ => if dark_mode {
            egui::Color32::from_rgb(200, 200, 200)
        } else {
            egui::Color32::from_rgb(60, 60, 60)
        },
    }
}

pub(crate) fn ansi_code_to_color(code: &str, dark_mode: bool) -> egui::Color32 {
    let codes: Vec<u8> = code.split(';')
        .filter_map(|s| s.parse().ok())
        .collect();
    
    if codes.is_empty() {
        return if dark_mode {
            egui::Color32::from_rgb(200, 200, 200)
        } else {
            egui::Color32::from_rgb(60, 60, 60)
        };
    }
    
    match codes[0] {
        0 => if dark_mode { egui::Color32::from_rgb(200, 200, 200) } else { egui::Color32::from_rgb(60, 60, 60) },
        30 => egui::Color32::from_rgb(50, 50, 50),
        31 => egui::Color32::from_rgb(255, 100, 100),
        32 => egui::Color32::from_rgb(100, 255, 100),
        33 => egui::Color32::from_rgb(255, 255, 100),
        34 => egui::Color32::from_rgb(100, 150, 255),
        35 => egui::Color32::from_rgb(255, 100, 255),
        36 => egui::Color32::from_rgb(100, 255, 255),
        37 => egui::Color32::from_rgb(220, 220, 220),
        38 => {
            if codes.len() >= 3 && codes[1] == 5 {
                ansi_256_to_color(codes[2])
            } else if codes.len() >= 5 && codes[1] == 2 {
                egui::Color32::from_rgb(codes[2], codes[3], codes[4])
            } else {
                if dark_mode { egui::Color32::from_rgb(200, 200, 200) } else { egui::Color32::from_rgb(60, 60, 60) }
            }
        }
        39 => if dark_mode { egui::Color32::from_rgb(200, 200, 200) } else { egui::Color32::from_rgb(60, 60, 60) },
        90 => egui::Color32::from_rgb(128, 128, 128),
        91 => egui::Color32::from_rgb(255, 150, 150),
        92 => egui::Color32::from_rgb(150, 255, 150),
        93 => egui::Color32::from_rgb(255, 255, 150),
        94 => egui::Color32::from_rgb(150, 180, 255),
        95 => egui::Color32::from_rgb(255, 150, 255),
        96 => egui::Color32::from_rgb(150, 255, 255),
        97 => egui::Color32::from_rgb(255, 255, 255),
        _ => if dark_mode { egui::Color32::from_rgb(200, 200, 200) } else { egui::Color32::from_rgb(60, 60, 60) },
    }
}

fn ansi_256_to_color(code: u8) -> egui::Color32 {
    match code {
        0..=15 => {
            let colors = [
                (0, 0, 0), (128, 0, 0), (0, 128, 0), (128, 128, 0),
                (0, 0, 128), (128, 0, 128), (0, 128, 128), (192, 192, 192),
                (128, 128, 128), (255, 0, 0), (0, 255, 0), (255, 255, 0),
                (0, 0, 255), (255, 0, 255), (0, 255, 255), (255, 255, 255),
            ];
            let (r, g, b) = colors[code as usize];
            egui::Color32::from_rgb(r, g, b)
        }
        16..=231 => {
            let idx = code - 16;
            let r = ((idx / 36) * 51) as u8;
            let g = (((idx % 36) / 6) * 51) as u8;
            let b = ((idx % 6) * 51) as u8;
            egui::Color32::from_rgb(r, g, b)
        }
        232..=255 => {
            let gray = (8 + (code - 232) * 10) as u8;
            egui::Color32::from_rgb(gray, gray, gray)
        }
    }
}

pub(crate) fn parse_ansi_line(ansi_regex: &Regex, text: &str, prefix: &str, dark_mode: bool) -> Vec<ColoredText> {
    let mut result = Vec::new();
    let mut current_color = get_prefix_color(prefix, dark_mode);
    let mut last_end = 0;
    
    for cap in ansi_regex.captures_iter(text) {
        let match_start = cap.get(0).unwrap().start();
        let match_end = cap.get(0).unwrap().end();
        
        if match_start > last_end {
            let segment = &text[last_end..match_start];
            if !segment.is_empty() {
                result.push(ColoredText {
                    text: segment.to_string(),
                    color: current_color,
                });
            }
        }
        
        // Only update color if it's a color code (ends with 'm')
        if let Some(codes) = cap.get(1) {
            if cap.get(0).unwrap().as_str().ends_with('m') {
                current_color = ansi_code_to_color(codes.as_str(), dark_mode);
            }
        }
        
        last_end = match_end;
    }
    
    if last_end < text.len() {
        let segment = &text[last_end..];
        if !segment.is_empty() {
            result.push(ColoredText {
                text: segment.to_string(),
                color: current_color,
            });
        }
    }
    
    if result.is_empty() {
        result.push(ColoredText {
            text: text.to_string(),
            color: current_color,
        });
    }
    
    result
}
