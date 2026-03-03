use crate::ansi::apply_color;
use crate::figfont::FIGfont;
use std::env;
use std::fs;
use std::path::Path;

pub fn get_banner(bannered: String) -> String {
    let config_file = format!("{}/.config/fetrust/{}", env!("HOME"), "font.flf");
    if !Path::new(&config_file).exists() {
        eprintln!("Creating default font ({})", config_file);
        if fs::create_dir_all(format!("{}/.config/fetrust", env!("HOME"))).is_err() {
            eprintln!(
                "Error: Something happened wrong (creating {}/.config/)",
                env!("HOME")
            )
        }
        if fs::write(&config_file, include_str!("default-figlet.font.flf")).is_err() {
            eprintln!("Error: Something happened wrong (writing {})", config_file)
        }
    }

    match FIGfont::from_file(config_file.as_str()) {
        Ok(slant) => {
            let bannered = slant.convert(&bannered.replace("Linux", ""));
            bannered.unwrap_or_default()
        }
        Err(e) => {
            eprintln!("Warning: Failed to load font: {}", e);
            bannered
        }
    }
}

#[allow(dead_code)]
pub fn handle_spacing(cache_text: &mut String, printing: &str, max_length: usize, padding: usize) {
    if cache_text.len() < max_length {
        let spaces = " ".repeat(max_length - cache_text.len() - padding);
        cache_text.push_str(&spaces);
    }
    cache_text.push_str(printing);
}

pub fn apply_color_wrapper(color: &str, text: &str) -> String {
    apply_color(color, text)
}

pub fn format_memory(total: f64, used: f64) -> String {
    if total / 1024.0 > 1024.0 {
        format!(
            "{}MiB / {}MiB",
            (used / 1024.0).round() as u64,
            (total / 1024.0).round() as u64
        )
    } else if total > 1024.0 {
        format!("{}KiB / {}KiB", used.round() as u64, total.round() as u64)
    } else {
        format!(
            "{}Bytes / {}Bytes",
            (used * 1024.0).round() as u64,
            (total * 1024.0).round() as u64
        )
    }
}

pub fn format_uptime(uptime: f64) -> String {
    let days = (uptime / 86400.0).floor() as u64;
    let hours = ((uptime % 86400.0) / 3600.0).floor() as u64;
    let minutes = ((uptime % 3600.0) / 60.0).floor() as u64;
    let seconds = (uptime % 60.0).floor() as u64;

    if days > 0 {
        format!(
            "{} days, {} hours, {} minutes, {} seconds",
            days, hours, minutes, seconds
        )
    } else if hours > 0 {
        format!("{} hours, {} minutes, {} seconds", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{} minutes, {} seconds", minutes, seconds)
    } else {
        format!("{} seconds", seconds)
    }
}

#[allow(dead_code)]
pub fn parse_sysctl_boottime(output: &str) -> Option<i64> {
    let parts: Vec<&str> = output.split(',').collect();
    if !parts.is_empty() {
        if let Some(sec_part) = parts[0].split('=').nth(1) {
            return sec_part.trim().parse::<i64>().ok();
        }
    }
    None
}

#[allow(dead_code)]
pub fn get_elapsed_seconds_since(boot_time: i64) -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    if let Ok(duration_since_boot) = SystemTime::now().duration_since(UNIX_EPOCH) {
        return duration_since_boot.as_secs() as f64 - boot_time as f64;
    }
    0.0
}
