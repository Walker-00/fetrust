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
    let mut user_host_value: Option<String> = None;

    let info_configs = [
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
        // Handle user_a_host_name separately for header
        if let Some(value) = json.get("user_a_host_name") {
            if let Some(arr) = value.as_array() {
                if !arr.is_empty() {
                    if let Some(inner_arr) = arr[0].as_array() {
                        let mut content = String::new();
                        for item in inner_arr {
                            let field_name = item.print();
                            let field_value = match field_name.as_str() {
                                "username" => &infos.username,
                                "hostname" => &infos.hostname,
                                _ => field_name.as_str(),
                            };
                            content.push_str(field_value);
                        }

                        let color = if arr.len() > 1 {
                            arr[1].as_str().unwrap_or("white")
                        } else {
                            "white"
                        };

                        let color_val = if color == "null" { "white" } else { color };
                        user_host_value = Some(apply_color_wrapper(color_val, &content));
                    }
                }
            }
        }

        // Process other info lines
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
                                    _ => field_name.as_str(),
                                };
                                content.push_str(field_value);
                            }

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
    let banner_lines: Vec<String> = banner
        .lines()
        .filter(|line| {
            let stripped = strip_ansi_codes(line);
            !stripped.is_empty() && !stripped.trim().is_empty()
        })
        .map(|s| s.to_string())
        .collect();

    // =========================================================================
    // PHASE 1 — Structural Layout: Two-Column Grid
    // =========================================================================

    const GAP: usize = 5;
    const SEP: &str = "  ==>  "; // Enhanced separator with breathing room

    // Banner box: fixed width = widest banner line (visible chars)
    let banner_box_width = banner_lines
        .iter()
        .map(|line| visible_len(line))
        .max()
        .unwrap_or(0);

    // Info box: calculate dimensions
    let label_width = info_lines
        .iter()
        .map(|(label, _content)| label.len())
        .max()
        .unwrap_or(6);

    let content_width = info_lines
        .iter()
        .map(|(_label, content)| visible_len(content))
        .max()
        .unwrap_or(20);

    let sep_len = visible_len(SEP);
    let info_box_width = label_width + sep_len + content_width;

    // Info column starts after banner box + gap
    let info_start = banner_box_width + GAP;

    // =========================================================================
    // PHASE 4 — ANSI-Safe Visible Length & Padding
    // =========================================================================

    // Build formatted info lines with ANSI-safe padding
    // Each info line: label (padded) + sep + content (padded)
    let formatted_info_lines: Vec<String> = info_lines
        .iter()
        .map(|(label, content)| {
            // Pad label to label_width (label has no ANSI codes)
            let label_padded = format!("{:<width$}", label, width = label_width);
            // Pad content to content_width using visible length (content HAS ANSI codes)
            let content_padded = pad_right_visible(content, content_width);
            format!("{}{}{}", label_padded, SEP, content_padded)
        })
        .collect();

    // =========================================================================
    // PHASE 3 — Vertical Balance: Center Banner in Info Block
    // =========================================================================

    let total_rows = info_lines.len().max(banner_lines.len());

    // Calculate vertical padding to center banner within info block
    let banner_vertical_pad = if total_rows > banner_lines.len() {
        (total_rows - banner_lines.len()) / 2
    } else {
        0
    };

    // =========================================================================
    // PHASE 6 — Output Order
    // =========================================================================

    // 1) Blank line
    println!();

    // 2) Header centered inside info box
    // 3) Rail under header
    if let Some(ref user_host) = user_host_value {
        let header_visible_len = visible_len(user_host);

        // Center header within info box (or left-align if too wide)
        let header_left_pad = if info_box_width > header_visible_len {
            (info_box_width - header_visible_len) / 2
        } else {
            0
        };

        // Print header: info_start spaces + header_left_pad spaces + header text
        let total_spaces = info_start + header_left_pad;
        println!("{:width$}{}", "", user_host, width = total_spaces);

        // Print rail under header (visual anchor)
        // Rail width = min(info_box_width, max(header_visible_len, 12))
        let rail_width = info_box_width.min(header_visible_len.max(12));
        let rail_left_pad = if info_box_width > rail_width {
            (info_box_width - rail_width) / 2
        } else {
            0
        };
        let rail_total_spaces = info_start + rail_left_pad;

        // Use box-drawing character, fallback to dash
        let rail: String = std::iter::repeat('─').take(rail_width).collect();
        println!("{:width$}{}", "", rail, width = rail_total_spaces);
    }

    // 4) Blank line
    println!();

    // 5) Main grid rows (banner + info combined)
    for i in 0..total_rows {
        // Calculate which banner line to print (with vertical padding)
        let banner_idx = i.saturating_sub(banner_vertical_pad);
        let should_print_banner = i >= banner_vertical_pad && banner_idx < banner_lines.len();

        // Banner box: always print banner_box_width visible characters
        if should_print_banner {
            let banner_line = &banner_lines[banner_idx];
            print!("{}", pad_right_visible(banner_line, banner_box_width));
        } else {
            // Vertical padding or no more banner lines: print spaces
            print!("{:width$}", "", width = banner_box_width);
        }

        // Gap between boxes
        print!("{:width$}", "", width = GAP);

        // Info box: print formatted line if exists
        if i < formatted_info_lines.len() {
            print!("{}", formatted_info_lines[i]);
        }

        println!();
    }

    // 6) Final newline
    println!();
}

/// Strip ANSI escape sequences from a string.
///
/// Handles:
/// - CSI sequences: ESC '[' params... final_byte (0x40..=0x7E)
/// - OSC sequences: ESC ']' ... ESC '\' or BEL
/// - Two-char escapes: ESC + any char
fn strip_ansi_codes(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '\x1b' => {
                // ESC - start of escape sequence
                if let Some(&next) = chars.peek() {
                    match next {
                        '[' => {
                            // CSI sequence: ESC '[' params... final_byte
                            chars.next(); // consume '['
                            // Skip until final byte (0x40..=0x7E)
                            for c in chars.by_ref() {
                                let byte = c as u32;
                                if (0x40..=0x7E).contains(&byte) {
                                    break;
                                }
                            }
                        }
                        ']' => {
                            // OSC sequence: ESC ']' ... ESC '\' or BEL
                            chars.next(); // consume ']'
                            while let Some(c) = chars.next() {
                                if c == '\x1b' {
                                    if let Some(&next) = chars.peek() {
                                        if next == '\\' {
                                            chars.next();
                                            break;
                                        }
                                    }
                                } else if c == '\x07' {
                                    break;
                                }
                            }
                        }
                        _ => {
                            // Two-character escape (e.g., ESC '7', ESC '8', ESC 'H', etc.)
                            chars.next();
                        }
                    }
                }
            }
            _ => result.push(c),
        }
    }

    result
}

/// Return the visible length of a string (character count, excluding ANSI escape sequences).
#[inline]
fn visible_len(s: &str) -> usize {
    strip_ansi_codes(s).chars().count()
}

/// Pad a string on the right with spaces to reach a target visible width.
///
/// ANSI escape sequences are preserved; spaces are added AFTER the string
/// based on visible character count (not byte count).
///
/// This ensures colored strings align correctly regardless of ANSI code length.
fn pad_right_visible(s: &str, target_width: usize) -> String {
    let current_len = visible_len(s);
    let padding_needed = target_width.saturating_sub(current_len);
    let mut result = String::with_capacity(s.len() + padding_needed);
    result.push_str(s);
    result.push_str(&" ".repeat(padding_needed));
    result
}
