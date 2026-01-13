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
        "[FRONTEND]" => if dark_mode { 
            egui::Color32::from_rgb(100, 180, 255) 
        } else { 
            egui::Color32::from_rgb(0, 80, 180)  // Dark blue for light mode
        },
        "[BACKEND]" => if dark_mode { 
            egui::Color32::from_rgb(100, 255, 150) 
        } else { 
            egui::Color32::from_rgb(0, 120, 40)  // Dark green for light mode
        },
        "[ERROR]" => if dark_mode { 
            egui::Color32::from_rgb(255, 100, 100) 
        } else { 
            egui::Color32::from_rgb(180, 0, 0)  // Dark red for light mode
        },
        "[SYSTEM]" => if dark_mode { 
            egui::Color32::from_rgb(255, 200, 100) 
        } else { 
            egui::Color32::from_rgb(150, 90, 0)  // Dark orange for light mode
        },
        _ => if dark_mode {
            egui::Color32::from_rgb(200, 200, 200)
        } else {
            egui::Color32::from_rgb(0, 0, 0)  // Black for light mode
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
            egui::Color32::from_rgb(0, 0, 0) // Black text for light mode
        };
    }
    
    match codes[0] {
        0 => if dark_mode { egui::Color32::from_rgb(200, 200, 200) } else { egui::Color32::from_rgb(0, 0, 0) },
        30 => if dark_mode { egui::Color32::from_rgb(50, 50, 50) } else { egui::Color32::from_rgb(0, 0, 0) },
        31 => if dark_mode { egui::Color32::from_rgb(255, 100, 100) } else { egui::Color32::from_rgb(180, 0, 0) },
        32 => if dark_mode { egui::Color32::from_rgb(100, 255, 100) } else { egui::Color32::from_rgb(0, 120, 0) },
        33 => if dark_mode { egui::Color32::from_rgb(255, 255, 100) } else { egui::Color32::from_rgb(150, 100, 0) },
        34 => if dark_mode { egui::Color32::from_rgb(100, 150, 255) } else { egui::Color32::from_rgb(0, 0, 180) },
        35 => if dark_mode { egui::Color32::from_rgb(255, 100, 255) } else { egui::Color32::from_rgb(150, 0, 150) },
        36 => if dark_mode { egui::Color32::from_rgb(100, 255, 255) } else { egui::Color32::from_rgb(0, 120, 120) },
        37 => if dark_mode { egui::Color32::from_rgb(220, 220, 220) } else { egui::Color32::from_rgb(70, 70, 70) },
        38 => {
            if codes.len() >= 3 && codes[1] == 5 {
                ansi_256_to_color(codes[2])
            } else if codes.len() >= 5 && codes[1] == 2 {
                egui::Color32::from_rgb(codes[2], codes[3], codes[4])
            } else {
                if dark_mode { egui::Color32::from_rgb(200, 200, 200) } else { egui::Color32::from_rgb(0, 0, 0) }
            }
        }
        39 => if dark_mode { egui::Color32::from_rgb(200, 200, 200) } else { egui::Color32::from_rgb(0, 0, 0) },
        90 => if dark_mode { egui::Color32::from_rgb(128, 128, 128) } else { egui::Color32::from_rgb(60, 60, 60) },
        91 => if dark_mode { egui::Color32::from_rgb(255, 150, 150) } else { egui::Color32::from_rgb(200, 50, 50) },
        92 => if dark_mode { egui::Color32::from_rgb(150, 255, 150) } else { egui::Color32::from_rgb(50, 150, 50) },
        93 => if dark_mode { egui::Color32::from_rgb(255, 255, 150) } else { egui::Color32::from_rgb(180, 120, 0) },
        94 => if dark_mode { egui::Color32::from_rgb(150, 180, 255) } else { egui::Color32::from_rgb(50, 50, 200) },
        95 => if dark_mode { egui::Color32::from_rgb(255, 150, 255) } else { egui::Color32::from_rgb(180, 50, 180) },
        96 => if dark_mode { egui::Color32::from_rgb(150, 255, 255) } else { egui::Color32::from_rgb(50, 150, 150) },
        97 => if dark_mode { egui::Color32::from_rgb(255, 255, 255) } else { egui::Color32::from_rgb(100, 100, 100) },
        _ => if dark_mode { egui::Color32::from_rgb(200, 200, 200) } else { egui::Color32::from_rgb(0, 0, 0) },
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
