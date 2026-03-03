pub struct SystemInfos {
    pub os: String,
    pub os_release: String,
    pub username: String,
    pub hostname: String,
    pub kernel_name: String,
    pub kernel: String,
    pub shell: String,
    pub family: String,
    pub uptime: String,
    pub resolution: String,
    pub cpu_type: String,
    pub memory: String,
    pub theme_name: String,
    pub icon_theme: String,
    pub font_name: String,
    pub cursor_theme: String,
    pub desktop: String,
    pub host_model: String,
    pub terminal: String,
    pub gpu: String,
}

pub mod sys {
    use crate::{extra_fn::format_memory, resource::SystemInfos};
    use std::{
        env,
        fs::File,
        io::{BufRead, BufReader},
        path::Path,
        process::Command,
    };

    /// Initialize system info collection.
    /// 
    /// # Arguments
    /// * `probe` - If true, use external commands for enhanced accuracy.
    ///            If false (default), use only file/env reads for maximum speed.
    pub fn init(probe: bool) -> SystemInfos {
        let themes = get_themes(probe);

        SystemInfos {
            os: get_os(),
            os_release: get_release(probe),
            username: get_username(),
            hostname: get_hostname(),
            kernel_name: get_kernel_name(),
            kernel: get_kernel(),
            shell: get_shell(),
            family: get_family(),
            uptime: get_uptime(),
            resolution: get_res(probe),
            cpu_type: get_cput(),
            memory: get_memory(),
            theme_name: themes.name,
            icon_theme: themes.icon,
            font_name: themes.font,
            cursor_theme: themes.cursor,
            desktop: get_desktop(),
            host_model: get_host_model(),
            terminal: get_terminal(),
            gpu: get_gpu(probe),
        }
    }

    pub struct Themes {
        pub name: String,
        pub icon: String,
        pub font: String,
        pub cursor: String,
    }

    pub fn get_os() -> String {
        #[cfg(target_os = "windows")]
        return "Windows NT".to_string();
        #[cfg(target_os = "macos")]
        return "XNU/darwin".to_string();
        #[cfg(target_os = "android")]
        return "GNU/Linux".to_string();
        #[cfg(any(target_os = "linux", target_os = "freebsd"))]
        return get_unix_distro("/etc/os-release");
        #[cfg(target_os = "dragonfly")]
        return "DragonflyBSD".to_string();
        #[cfg(target_os = "openbsd")]
        return "OpenBSD".to_string();
        #[cfg(target_os = "netbsd")]
        return "NetBSD".to_string();
    }

    /// Get OS release version.
    /// 
    /// FAST mode (probe=false): Parse /etc/os-release directly.
    /// PROBE mode (probe=true): Try lsb_release command first, fallback to file.
    #[cfg(any(target_os = "linux", target_os = "android"))]
    pub fn get_release(probe: bool) -> String {
        if probe {
            // PROBE: Try lsb_release command first
            if let Ok(release_d) = Command::new("lsb_release").arg("-sr").output() {
                let version = String::from_utf8_lossy(&release_d.stdout)
                    .trim()
                    .to_string();
                if !version.is_empty() {
                    return version;
                }
            }
        }
        
        // FAST: Parse /etc/os-release directly (no external commands)
        if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
            for line in content.lines() {
                if line.starts_with("VERSION_ID=") {
                    return line.trim_start_matches("VERSION_ID=")
                        .trim_matches('"')
                        .trim_matches('\'')
                        .to_string();
                }
            }
        }
        
        "unknown".to_string()
    }

    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    pub fn get_unix_distro(file: &str) -> String {
        use std::fs;
        let os_release = fs::read_to_string(file).unwrap();
        let os_release: Vec<&str> = os_release.split('\n').collect();
        #[cfg(target_os = "linux")]
        let mut linux_distro = "GNU/Linux".to_string();
        #[cfg(target_os = "freebsd")]
        let mut linux_distro = "BSD".to_string();
        for readed_line in &os_release {
            if readed_line.starts_with("PRETTY_NAME=\"") {
                linux_distro = readed_line.replace("PRETTY_NAME=", "").replace('\"', "");
                break;
            }
        }
        linux_distro
    }

    pub fn get_kernel() -> String {
        #[cfg(target_os = "windows")]
        return "NT".to_string();
        #[cfg(target_os = "macos")]
        return "XNU".to_string();
        #[cfg(target_os = "ios")]
        return "XNU".to_string();
        #[cfg(any(target_os = "linux", target_os = "android", target_os = "freebsd"))]
        return get_ukernel_info();
        #[cfg(any(target_os = "dragonfly", target_os = "openbsd", target_os = "netbsd"))]
        return "BSD".to_string();
    }

    pub fn get_ukernel_info() -> String {
        let krl_vr = std::process::Command::new("uname").arg("-r").output();
        let krl_vr = match krl_vr {
            Ok(x) => {
                let rev_kernel_ver: String =
                    String::from_utf8(x.stdout).unwrap().chars().rev().collect();
                let rev_kernel_ver = rev_kernel_ver
                    .split('\n')
                    .next_back()
                    .unwrap()
                    .chars()
                    .rev()
                    .collect();

                rev_kernel_ver
            }
            Err(_) => "Unknown".to_string(),
        };
        krl_vr
    }

    #[cfg(target_os = "windows")]
    pub fn get_memory() -> String {
        let output = Command::new("cmd").args(["/C", "systeminfo"]).output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);

                    let mut total: f64 = 0.0;
                    let mut free: f64 = 0.0;

                    for line in stdout.lines() {
                        if line.contains("Total Physical Memory") {
                            let parts: Vec<&str> = line.split(':').collect();
                            if let Some(value) = parts.get(1) {
                                let value = value.trim().replace(",", "").replace(" MB", "");
                                total = value.parse::<f64>().unwrap() * 1024.0 * 1024.0;
                            }
                        } else if line.contains("Available Physical Memory") {
                            let parts: Vec<&str> = line.split(':').collect();
                            if let Some(value) = parts.get(1) {
                                let value = value.trim().replace(",", "").replace(" MB", "");
                                free = value.parse::<f64>().unwrap() * 1024.0 * 1024.0;
                            }
                        }

                        if total > 0.0 && free > 0.0 {
                            break;
                        }
                    }

                    let used = total - free;
                    format_memory(total, used)
                } else {
                    format!(
                        "Failed to retrieve memory info: {}",
                        String::from_utf8_lossy(&output.stderr)
                    )
                }
            }
            Err(e) => format!("Failed to run command: {}", e),
        }
    }

    #[cfg(target_os = "linux")]
    pub fn get_memory() -> String {
        let file = File::open("/proc/meminfo").unwrap();
        let reader = BufReader::new(file);

        let mut total = 0.0;
        let mut free = 0.0;

        for line in reader.lines() {
            let line = line.unwrap();
            if line.starts_with("MemTotal") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                total = parts[1].parse::<f64>().unwrap_or(0.0);
            } else if line.starts_with("MemAvailable") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                free = parts[1].parse::<f64>().unwrap_or(0.0);
            }
            if total > 0.0 && free > 0.0 {
                break;
            }
        }

        let used = total - free;
        format_memory(total, used)
    }

    #[cfg(target_os = "macos")]
    pub fn get_memory() -> String {
        use std::process::Command;

        let total_output = Command::new("sysctl").arg("hw.memsize").output().unwrap();
        let total = String::from_utf8_lossy(&total_output.stdout)
            .split_whitespace()
            .last()
            .unwrap_or("0")
            .parse::<f64>()
            .unwrap_or(0.0)
            / 1024.0; // Convert Bytes to KiB

        let free_output = Command::new("vm_stat").output().unwrap();
        let free_pages: f64 = String::from_utf8_lossy(&free_output.stdout)
            .lines()
            .filter_map(|line| {
                if line.contains("Pages free:") {
                    line.split_whitespace()
                        .nth(2)
                        .and_then(|val| val.trim_end_matches('.').parse::<f64>().ok())
                } else {
                    None
                }
            })
            .next()
            .unwrap_or(0.0);

        let free = free_pages * 4096.0 / 1024.0; // Convert Pages to Bytes, then to KiB
        let used = total - free;
        format_memory(total, used)
    }

    #[cfg(any(target_os = "freebsd", target_os = "openbsd", target_os = "netbsd"))]
    pub fn get_memory() -> String {
        let total_output = Command::new("sysctl").arg("hw.physmem").output().unwrap();
        let total = String::from_utf8_lossy(&total_output.stdout)
            .split_whitespace()
            .last()
            .unwrap_or("0")
            .parse::<f64>()
            .unwrap_or(0.0)
            / 1024.0; // Convert Bytes to KiB

        let free_pages_output = Command::new("sysctl")
            .arg("vm.stats.vm.v_free_count")
            .output()
            .unwrap();
        let free_pages = String::from_utf8_lossy(&free_pages_output.stdout)
            .split_whitespace()
            .last()
            .unwrap_or("0")
            .parse::<f64>()
            .unwrap_or(0.0);

        let page_size_output = Command::new("sysctl").arg("hw.pagesize").output().unwrap();
        let page_size = String::from_utf8_lossy(&page_size_output.stdout)
            .split_whitespace()
            .last()
            .unwrap_or("0")
            .parse::<f64>()
            .unwrap_or(0.0);

        let free = free_pages * page_size / 1024.0; // Convert Pages to KiB
        let used = total - free;
        format_memory(total, used)
    }

    pub fn get_username() -> String {
        std::env::var(if cfg!(any(target_os = "linux", target_os = "freebsd")) {
            "USER"
        } else {
            "USERNAME"
        })
        .unwrap()
    }

    #[cfg(target_os = "windows")]
    pub fn get_hostname() -> String {
        Command::new("cmd")
            .args(["/C", "hostname"])
            .output()
            .expect("hostname")
    }

    #[cfg(not(target_os = "windows"))]
    pub fn get_hostname() -> String {
        let mut hostname_str = "unknown hostname".to_string();
        match std::fs::read_to_string("/etc/hostname") {
            Ok(file) => {
                hostname_str = file.trim().to_string();
            }
            _ => {
                if let Ok(host) = std::env::var("HOST") {
                    hostname_str = host;
                }
                if hostname_str == "unknown hostname" {
                    hostname_str = std::str::from_utf8(
                        &std::process::Command::new("sh")
                            .arg("-c")
                            .arg("hostname")
                            .output()
                            .expect("[E] error on hostname command.")
                            .stdout,
                    )
                    .expect("[E] hostname contains non-utf8 characters.")
                    .to_string()
                    .replace('\n', "");
                }
            }
        }
        hostname_str
    }

    pub fn get_shell() -> String {
        use std::env::var;
        let shell_var = if cfg!(target_os = "linux") || cfg!(target_os = "freebsd") {
            "SHELL"
        } else {
            "COMSPEC"
        };

        match var(shell_var) {
            Ok(val) => {
                #[cfg(target_os = "freebsd")]
                let val = val
                    .split('/')
                    .collect::<Vec<&str>>()
                    .pop()
                    .unwrap()
                    .to_string();
                val
            }
            _ => "Unknown".to_string(),
        }
    }

    #[cfg(target_os = "windows")]
    pub fn get_uptime() -> String {
        Command::new("cmd")
            .args(["/C", "systeminfo | find 'Boot Time' "])
            .output()
            .expect("1")
    }

    #[cfg(not(target_os = "windows"))]
    fn get_uptime() -> String {
        use std::fs;

        #[allow(unused_imports)]
        use crate::extra_fn::{format_uptime, get_elapsed_seconds_since, parse_sysctl_boottime};

        #[cfg(target_os = "linux")]
        {
            if let Ok(contents) = fs::read_to_string("/proc/uptime") {
                let parts: Vec<&str> = contents.split_whitespace().collect();
                if let Some(uptime_str) = parts.first() {
                    if let Ok(uptime) = uptime_str.parse::<f64>() {
                        return format_uptime(uptime);
                    }
                }
            }
            "EUPTM".to_string()
        }

        #[cfg(any(
            target_os = "macos",
            target_os = "freebsd",
            target_os = "openbsd",
            target_os = "netbsd"
        ))]
        {
            if let Ok(output) = Command::new("sysctl")
                .arg("-n")
                .arg("kern.boottime")
                .output()
            {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    if let Some(boot_time) = parse_sysctl_boottime(&output_str) {
                        let uptime = get_elapsed_seconds_since(boot_time);
                        return format_uptime(uptime);
                    }
                }
            }
            "EUPTM".to_string()
        }
    }

    /// Get screen resolution.
    /// 
    /// FAST mode (probe=false): Returns "Unknown" (no external commands).
    /// PROBE mode (probe=true): 
    ///   - Hyprland: hyprctl monitors -j (focused monitor)
    ///   - X11: xrandr (connected output with *)
    ///   - Fallback: xdpyinfo
    #[cfg(target_os = "linux")]
    pub fn get_res(probe: bool) -> String {
        if !probe {
            // FAST mode: no external commands
            return "Unknown".to_string();
        }

        // PROBE mode: use external commands
        let desktop = env::var("XDG_CURRENT_DESKTOP").unwrap_or_default();
        let hyprland_sig = env::var("HYPRLAND_INSTANCE_SIGNATURE").unwrap_or_default();
        let is_hyprland = desktop.contains("Hyprland") || !hyprland_sig.is_empty();

        if is_hyprland {
            // Try hyprctl monitors -j (JSON output)
            if let Ok(output) = Command::new("hyprctl").args(["monitors", "-j"]).output() {
                if let Ok(json_str) = String::from_utf8(output.stdout) {
                    let monitors: Vec<&str> = json_str.split('{').collect();
                    let mut best_resolution = None;

                    for monitor in monitors {
                        if monitor.is_empty() {
                            continue;
                        }

                        let is_focused = monitor.contains("\"focused\":true")
                            || monitor.contains("\"focused\": true");

                        let width = extract_json_number(monitor, "width");
                        let height = extract_json_number(monitor, "height");

                        if let (Some(w), Some(h)) = (width, height) {
                            if is_focused {
                                return format!("{}x{}", w, h);
                            }
                            if best_resolution.is_none() {
                                best_resolution = Some(format!("{}x{}", w, h));
                            }
                        }
                    }

                    if let Some(res) = best_resolution {
                        return res;
                    }
                }
            }
        }

        // Try xrandr
        if let Ok(output) = Command::new("xrandr").output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.contains(" connected") {
                    if let Some(res_part) = line.split_whitespace().find(|s| s.contains('x')) {
                        let res = res_part.split('+').next().unwrap_or(res_part);
                        if res.contains('x') {
                            return res.to_string();
                        }
                    }
                }
            }
            for line in stdout.lines() {
                if line.contains("*") {
                    if let Some(res) = line.split_whitespace().next() {
                        if res.contains('x') {
                            return res.to_string();
                        }
                    }
                }
            }
        }

        // Try xdpyinfo
        if let Ok(output) = Command::new("xdpyinfo").output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.contains("dimensions:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    for part in parts {
                        if part.contains('x') && part.contains(char::is_numeric) {
                            return part.to_string();
                        }
                    }
                }
            }
        }

        "N/A".to_string()
    }

    /// Helper to extract a number from simple JSON-like text
    fn extract_json_number(text: &str, key: &str) -> Option<u32> {
        let search_key = format!("\"{}\":", key);
        if let Some(start) = text.find(&search_key) {
            let after_key = &text[start + search_key.len()..];
            let after_key = after_key.trim_start();
            let num_str: String = after_key.chars().take_while(|c| c.is_numeric()).collect();
            if num_str.is_empty() {
                None
            } else {
                num_str.parse().ok()
            }
        } else {
            None
        }
    }

    pub fn get_kernel_name() -> String {
        let kernel_name: String = String::from(std::env::consts::OS);
        kernel_name
    }

    pub fn get_family() -> String {
        let family: String = String::from(std::env::consts::FAMILY);
        family
    }

    #[cfg(target_os = "linux")]
    pub fn get_cput() -> String {
        let file = File::open("/proc/cpuinfo");
        if let Ok(file) = file {
            let reader = BufReader::new(file);

            let mut model_name = String::new();
            let mut clock_rate_mhz: f64 = 0.0;

            for line in reader.lines().map_while(Result::ok) {
                if line.starts_with("model name") {
                    model_name = line.split(':').nth(1).unwrap().trim().to_string();
                } else if line.starts_with("cpu MHz") {
                    clock_rate_mhz = line
                        .split(':')
                        .nth(1)
                        .unwrap()
                        .trim()
                        .parse::<f64>()
                        .unwrap_or(0.0);
                }

                if !model_name.is_empty() && clock_rate_mhz > 0.0 {
                    break;
                }
            }

            let clock_rate = if clock_rate_mhz >= 1000.0 {
                format!("{:.3} GHz", clock_rate_mhz / 1000.0)
            } else {
                format!("{:.3} MHz", clock_rate_mhz)
            };

            return format!("{} @ {}", model_name, clock_rate);
        }

        "ECPUI".to_string()
    }

    #[cfg(target_os = "windows")]
    fn get_cput() -> String {
        use std::process::Command;

        if let Ok(output) = Command::new("wmic")
            .args(["cpu", "get", "Name,MaxClockSpeed", "/format:list"])
            .output()
        {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let mut model_name = String::new();
            let mut clock_rate_mhz: f64 = 0.0;

            for line in output_str.lines() {
                if line.starts_with("Name=") {
                    model_name = line.split('=').nth(1).unwrap().to_string();
                } else if line.starts_with("MaxClockSpeed=") {
                    clock_rate_mhz = line
                        .split('=')
                        .nth(1)
                        .unwrap()
                        .parse::<f64>()
                        .unwrap_or(0.0);
                }
            }

            let clock_rate = if clock_rate_mhz >= 1000.0 {
                format!("{:.3} GHz", clock_rate_mhz / 1000.0)
            } else {
                format!("{:.3} MHz", clock_rate_mhz)
            };

            return format!("{} @ {}", model_name, clock_rate);
        }

        "ECPUI".to_string()
    }

    #[cfg(target_os = "macos")]
    fn get_cput() -> String {
        use std::process::Command;

        if let Ok(output) = Command::new("sysctl")
            .args(["-n", "machdep.cpu.brand_string"])
            .output()
        {
            let model_name = String::from_utf8_lossy(&output.stdout).trim().to_string();
            return format!("{} @ Unknown Clock Rate", model_name);
        }

        "ECPUI".to_string()
    }

    /// Get host model from DMI info (Linux).
    /// Always uses file reads (no external commands) - fast by default.
    #[cfg(target_os = "linux")]
    pub fn get_host_model() -> String {
        let mut vendor = String::new();
        let mut product_name = String::new();
        let mut product_version = String::new();

        if let Ok(content) = std::fs::read_to_string("/sys/devices/virtual/dmi/id/sys_vendor") {
            vendor = content.trim().to_string();
        }
        if let Ok(content) = std::fs::read_to_string("/sys/devices/virtual/dmi/id/product_name") {
            product_name = content.trim().to_string();
        }
        if let Ok(content) = std::fs::read_to_string("/sys/devices/virtual/dmi/id/product_version") {
            product_version = content.trim().to_string();
        }

        let mut parts = Vec::new();
        if !vendor.is_empty() && vendor != "Default String" && vendor != "To be filled by O.E.M." {
            parts.push(vendor);
        }
        if !product_name.is_empty() && product_name != "Default String" && product_name != "To be filled by O.E.M." {
            parts.push(product_name);
        }
        if !product_version.is_empty() && product_version != "Default String" && product_version != "To be filled by O.E.M." {
            parts.push(product_version);
        }

        if parts.is_empty() {
            "N/A".to_string()
        } else {
            parts.join(" ")
        }
    }

    #[cfg(not(target_os = "linux"))]
    pub fn get_host_model() -> String {
        "N/A".to_string()
    }

    /// Get terminal name from environment or parent process.
    /// Always uses env/file reads (no external commands) - fast by default.
    #[cfg(target_os = "linux")]
    pub fn get_terminal() -> String {
        if let Ok(term_program) = env::var("TERM_PROGRAM") {
            return term_program;
        }

        if let Ok(term) = env::var("TERM") {
            let term_clean = term.split('-').next().unwrap_or(&term).to_string();
            if !term_clean.is_empty() && term_clean != "unknown" {
                return term_clean;
            }
        }

        if let Ok(ppid) = env::var("PPID") {
            let comm_path = format!("/proc/{}/comm", ppid);
            if let Ok(content) = std::fs::read_to_string(&comm_path) {
                let terminal = content.trim().to_string();
                if !terminal.is_empty() {
                    return terminal;
                }
            }
        }

        if let Ok(ppid) = env::var("PPID") {
            let stat_path = format!("/proc/{}/stat", ppid);
            if let Ok(content) = std::fs::read_to_string(&stat_path) {
                if let Some(start) = content.find(')') {
                    let after_comm = &content[start + 1..];
                    let parts: Vec<&str> = after_comm.split_whitespace().collect();
                    if parts.len() > 1 {
                        let grandparent_pid = parts[1];
                        let gp_comm_path = format!("/proc/{}/comm", grandparent_pid);
                        if let Ok(gp_content) = std::fs::read_to_string(&gp_comm_path) {
                            let terminal = gp_content.trim().to_string();
                            if !terminal.is_empty() {
                                return terminal;
                            }
                        }
                    }
                }
            }
        }

        "N/A".to_string()
    }

    #[cfg(not(target_os = "linux"))]
    pub fn get_terminal() -> String {
        if let Ok(term) = env::var("TERM") {
            return term.split('-').next().unwrap_or(&term).to_string();
        }
        "N/A".to_string()
    }

    /// Get GPU info.
    /// 
    /// FAST mode (probe=false): Returns "Unknown" (no lspci).
    /// PROBE mode (probe=true): Runs lspci and parses VGA/3D/Display controller.
    #[cfg(target_os = "linux")]
    pub fn get_gpu(probe: bool) -> String {
        if !probe {
            // FAST mode: no external commands
            return "Unknown".to_string();
        }

        // PROBE mode: use lspci
        if let Ok(output) = Command::new("lspci").output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.contains("VGA compatible controller")
                    || line.contains("3D controller")
                    || line.contains("Display controller")
                {
                    if let Some(colon_pos) = line.find(':') {
                        let after_colon = &line[colon_pos + 1..];
                        if let Some(type_colon) = after_colon.find(':') {
                            let gpu_name = after_colon[type_colon + 1..].trim();
                            if !gpu_name.is_empty() {
                                return gpu_name.to_string();
                            }
                        }
                    }
                }
            }
        }

        "N/A".to_string()
    }

    #[cfg(not(target_os = "linux"))]
    pub fn get_gpu() -> String {
        "N/A".to_string()
    }

    #[cfg(not(target_os = "linux"))]
    fn get_themes(_probe: bool) -> Themes {
        let na = String::from("N/A");
        Themes {
            name: na.clone(),
            icon: na.clone(),
            font: na.clone(),
            cursor: na.clone(),
        }
    }

    /// Get theme info.
    /// 
    /// FAST mode (probe=false): Read only GTK 3.0/4.0 settings.ini files.
    /// PROBE mode (probe=true): Also try gsettings commands and cursor fallback.
    #[cfg(target_os = "linux")]
    fn get_themes(probe: bool) -> Themes {
        use crate::ini_parser::ini_parser;

        // Method 1: Try GTK 3.0 settings.ini (always, both modes)
        let config_path_gtk3 = format!("{}/.config/gtk-3.0/settings.ini", env::var("HOME").unwrap_or_default());
        if Path::new(&config_path_gtk3).exists() {
            if let Ok(ini) = ini_parser(&config_path_gtk3) {
                if let Some(section) = ini.get("Settings") {
                    let theme_name = section.get("gtk-theme-name").cloned().unwrap_or_default();
                    let icon_theme = section.get("gtk-icon-theme-name").cloned().unwrap_or_default();
                    let font_name = section.get("gtk-font-name").cloned().unwrap_or_default();
                    let cursor_theme = section.get("gtk-cursor-theme-name").cloned().unwrap_or_default();

                    if !theme_name.is_empty() || !icon_theme.is_empty() {
                        return Themes {
                            name: if theme_name.is_empty() { "N/A".to_string() } else { theme_name },
                            icon: if icon_theme.is_empty() { "N/A".to_string() } else { icon_theme },
                            font: if font_name.is_empty() { "N/A".to_string() } else { font_name },
                            cursor: if cursor_theme.is_empty() { "N/A".to_string() } else { cursor_theme },
                        };
                    }
                }
            }
        }

        // Method 2: Try GTK 4.0 settings.ini (always, both modes)
        let config_path_gtk4 = format!("{}/.config/gtk-4.0/settings.ini", env::var("HOME").unwrap_or_default());
        if Path::new(&config_path_gtk4).exists() {
            if let Ok(ini) = ini_parser(&config_path_gtk4) {
                if let Some(section) = ini.get("Settings") {
                    let theme_name = section.get("gtk-theme-name").cloned().unwrap_or_default();
                    let icon_theme = section.get("gtk-icon-theme-name").cloned().unwrap_or_default();
                    let font_name = section.get("gtk-font-name").cloned().unwrap_or_default();
                    let cursor_theme = section.get("gtk-cursor-theme-name").cloned().unwrap_or_default();

                    if !theme_name.is_empty() || !icon_theme.is_empty() {
                        return Themes {
                            name: if theme_name.is_empty() { "N/A".to_string() } else { theme_name },
                            icon: if icon_theme.is_empty() { "N/A".to_string() } else { icon_theme },
                            font: if font_name.is_empty() { "N/A".to_string() } else { font_name },
                            cursor: if cursor_theme.is_empty() { "N/A".to_string() } else { cursor_theme },
                        };
                    }
                }
            }
        }

        if !probe {
            // FAST mode: no more fallbacks
            return Themes {
                name: "N/A".to_string(),
                icon: "N/A".to_string(),
                font: "N/A".to_string(),
                cursor: "N/A".to_string(),
            };
        }

        // PROBE mode: Try gsettings (GNOME)
        let mut theme_name = String::new();
        let mut icon_theme = String::new();
        let mut font_name = String::new();
        let mut cursor_theme = String::new();

        if let Ok(output) = Command::new("gsettings")
            .args(["get", "org.gnome.desktop.interface", "gtk-theme"])
            .output()
        {
            theme_name = String::from_utf8_lossy(&output.stdout)
                .trim()
                .trim_matches('\'')
                .to_string();
        }

        if let Ok(output) = Command::new("gsettings")
            .args(["get", "org.gnome.desktop.interface", "icon-theme"])
            .output()
        {
            icon_theme = String::from_utf8_lossy(&output.stdout)
                .trim()
                .trim_matches('\'')
                .to_string();
        }

        if let Ok(output) = Command::new("gsettings")
            .args(["get", "org.gnome.desktop.interface", "font-name"])
            .output()
        {
            font_name = String::from_utf8_lossy(&output.stdout)
                .trim()
                .trim_matches('\'')
                .to_string();
        }

        if let Ok(output) = Command::new("gsettings")
            .args(["get", "org.gnome.desktop.interface", "cursor-theme"])
            .output()
        {
            cursor_theme = String::from_utf8_lossy(&output.stdout)
                .trim()
                .trim_matches('\'')
                .to_string();
        }

        // Cursor theme fallback from ~/.icons/default/index.theme
        if cursor_theme.is_empty() {
            let cursor_config = format!("{}/.icons/default/index.theme", env::var("HOME").unwrap_or_default());
            if Path::new(&cursor_config).exists() {
                if let Ok(ini) = ini_parser(&cursor_config) {
                    if let Some(section) = ini.get("Icon Theme") {
                        if let Some(inherits) = section.get("Inherits") {
                            cursor_theme = inherits.clone();
                        }
                    }
                }
            }
        }

        Themes {
            name: if theme_name.is_empty() { "N/A".to_string() } else { theme_name },
            icon: if icon_theme.is_empty() { "N/A".to_string() } else { icon_theme },
            font: if font_name.is_empty() { "N/A".to_string() } else { font_name },
            cursor: if cursor_theme.is_empty() { "N/A".to_string() } else { cursor_theme },
        }
    }

    #[cfg(not(target_os = "linux"))]
    fn get_desktop() -> String {
        String::from("N/A")
    }

    #[cfg(target_os = "linux")]
    fn get_desktop() -> String {
        env::var("XDG_CURRENT_DESKTOP").unwrap_or_else(|_| "N/A".to_string())
    }
}
