mod ansi;
mod extra_fn;
mod figfont;
mod ini_parser;
mod json_parser;
mod resource;

use extra_fn::apply_color_wrapper;
use json_parser::JsonValue;
use std::path::Path;
use std::{env, fs};

fn main() {
    let infos = crate::resource::sys::init();
    let config_file = format!(
        "{}/.config/fetrust/{}",
        env::var("HOME").unwrap_or_else(|_| ".".to_string()),
        "config.json"
    );
    if !Path::new(&config_file).exists() {
        eprintln!("Creating default config ({})", config_file);
        if fs::create_dir_all(format!("{}/.config/fetrust", env::var("HOME").unwrap_or_else(|_| ".".to_string()))).is_err() {
            eprintln!(
                "Error: Something happened wrong (creating {}/.config)",
                env::var("HOME").unwrap_or_else(|_| ".".to_string())
            )
        }
        if fs::write(&config_file, include_bytes!("default.config.json")).is_err() {
            eprintln!("Error: Something happened wrong (writing {})", config_file)
        }
    }
    let config_json = &fs::read(&config_file).unwrap();
    let json = json_parser::Json::parse(config_json).unwrap_or(JsonValue::Null);

    // Build info lines
    let mut info_lines: Vec<(String, String)> = Vec::new();

    let info_configs = [
        ("user_a_host_name", "user @ host", vec!["username", " @ ", "hostname"]),
        ("os", "OS", vec!["os", " ", "release"]),
        ("kernel", "Kernel", vec!["kernel_name", " ", "kernel"]),
        ("shell", "Shell", vec!["shell"]),
        ("family", "Family", vec!["family"]),
        ("uptime", "Uptime", vec!["uptime"]),
        ("resolution", "Resolution", vec!["resolution"]),
        ("cpu_type", "CPU", vec!["cpu_type"]),
        ("memory", "Memory", vec!["memory"]),
        ("desktop", "DE", vec!["desktop"]),
        ("theme", "Theme", vec!["theme"]),
        ("icon", "Icons", vec!["icon"]),
        ("font", "Font", vec!["font"]),
        ("cursor", "Cursor", vec!["cursor"]),
    ];

    if let JsonValue::Object(_pairs) = &json {
        for (key, label, _fields) in info_configs.iter() {
            if let Some(value) = json.get(key) {
                if let Some(arr) = value.as_array() {
                    if !arr.is_empty() {
                        if let Some(inner_arr) = arr[0].as_array() {
                            let mut content = String::new();
                            for item in inner_arr {
                                let field_name = item.print();
                                let field_value = match field_name.as_str() {
                                    "os" => &infos.os,
                                    "os_release" => &infos.os_release,
                                    "username" => &infos.username,
                                    "hostname" => &infos.hostname,
                                    "kernel_name" => &infos.kernel_name,
                                    "kernel" => &infos.kernel,
                                    "shell" => &infos.shell,
                                    "family" => &infos.family,
                                    "uptime" => &infos.uptime,
                                    "resolution" => &infos.resolution,
                                    "cpu_type" => &infos.cpu_type,
                                    "memory" => &infos.memory,
                                    "theme" => &infos.theme_name,
                                    "icon" => &infos.icon_theme,
                                    "font" => &infos.font_name,
                                    "cursor" => &infos.cursor_theme,
                                    "desktop" => &infos.desktop,
                                    _ => field_name.as_str(), // Treat unknown fields as literal strings
                                };
                                content.push_str(field_value);
                            }
                            
                            // Get color
                            let color = if arr.len() > 1 {
                                arr[1].as_str().unwrap_or("white")
                            } else {
                                "white"
                            };
                            
                            let color_val = if color == "null" { "white" } else { color };
                            let colored_content = apply_color_wrapper(color_val, &content);

                            info_lines.push((label.to_string(), colored_content));
                        }
                    }
                }
            }
        }
    }

    // Get banner - use OS name without "Linux"
    let os_name = infos.os.replace("Linux", "").trim().to_string();
    let banner = extra_fn::get_banner(os_name);
    
    // Filter out empty banner lines (lines that are empty or contain only ANSI codes)
    let banner_lines: Vec<String> = banner.lines()
        .filter(|line| {
            let stripped = strip_ansi_codes(line);
            !stripped.is_empty() && !stripped.trim().is_empty()
        })
        .map(|s| s.to_string())
        .collect();
    
    // Calculate visible banner width (excluding ANSI codes)
    let banner_width = banner_lines.iter()
        .map(|line| strip_ansi_codes(line).len())
        .max()
        .unwrap_or(0);

    // Find max label width for alignment (unused for now)
    let _max_label_width: usize = info_lines.iter()
        .map(|(label, _content)| label.len())
        .max()
        .unwrap_or(6);

    // Gap between banner and info
    let gap = 5;
    
    // Print banner with info lines side by side
    for (i, banner_line) in banner_lines.iter().enumerate() {
        print!("{}", banner_line);
        
        // Add info line on corresponding row
        if i < info_lines.len() {
            let (label, content) = &info_lines[i];
            // Calculate visible length of banner line
            let visible_len = strip_ansi_codes(banner_line).len();
            
            // Calculate padding to align info column
            let padding_needed = if visible_len < banner_width {
                banner_width - visible_len + gap
            } else {
                gap
            };
            
            // Print padded label and content
            print!("{:width$}{} ==> {}", "", label, content, width = padding_needed);
        }
        
        println!();
    }
    
    // Print remaining info lines aligned with the ones above
    for (_i, (label, content)) in info_lines.iter().enumerate().skip(banner_lines.len()) {
        // Align with the info column above
        print!("{:width$}{} ==> {}", "", label, content, width = banner_width + gap);
        println!();
    }
}

fn strip_ansi_codes(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            if let Some(&'[') = chars.peek() {
                chars.next(); // consume '['
                // Skip until end of ANSI sequence
                for c in chars.by_ref() {
                    if c == 'm' || c == 'A' || c == 'B' || c == 'C' || c == 'D' {
                        break;
                    }
                }
            } else {
                result.push(c);
            }
        } else {
            result.push(c);
        }
    }
    
    result
}
