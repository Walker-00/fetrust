mod extra_fn;
mod resource;

use another_json_minimal::*;
use extra_fn::{apply_color, get_banner, handle_spacing};
use std::fs;
use std::path::Path;

fn main() {
    let mut cache: Vec<String> = Vec::new();
    let mut max_length = 0;
    let infos = crate::resource::sys::init();
    let config_file = format!("{}/.config/fetrust/{}", env!("HOME"), "config.json");
    if !Path::new(&config_file).exists() {
        println!("Creating default config ({})", config_file);
        if fs::create_dir_all(format!("{}/.config/fetrust", env!("HOME"))).is_err() {
            println!(
                "Error: Something happened wrong (creating {}/.config)",
                env!("HOME")
            )
        }
        if fs::write(&config_file, include_bytes!("default.config.json")).is_err() {
            println!("Error: Something happened wrong (writing {})", config_file)
        }
    }
    let config_json = &fs::read(config_file).unwrap();
    let json = Json::parse(config_json);
    for info in [
        "banner",
        "user_a_host_name",
        "os",
        "kernel",
        "shell",
        "family",
        "uptime",
        "cpu_type",
    ] {
        if let Some(Json::OBJECT { name, value }) = json.as_ref().unwrap().get(info) {
            let mut printing = "".to_string();
            let mut printingc = "".to_string();
            if let Json::ARRAY(value) = value.unbox() {
                if let Json::ARRAY(value) = value[0].unbox() {
                    let mut printingl = vec!["".to_string(); value.len()];
                    let mut i = 0;
                    for string in value {
                        printingl[i] = string.print();
                        i += 1;
                    }
                    i = 0;

                    let bprintingl = printingl.clone();

                    for getter in bprintingl {
                        match getter.as_str() {
                            "os" => printingl[i] = infos.os.clone(),
                            "os_release" => printingl[i] = infos.os_release.clone(),
                            "username" => printingl[i] = infos.username.clone(),
                            "hostname" => printingl[i] = infos.hostname.clone(),
                            "kernel_name" => printingl[i] = infos.kernel_name.clone(),
                            "kernel" => printingl[i] = infos.kernel.clone(),
                            "shell" => printingl[i] = infos.shell.clone(),
                            "family" => printingl[i] = infos.family.clone(),
                            "uptime" => printingl[i] = infos.uptime.clone(),
                            "cpu_type" => printingl[i] = infos.cpu_type.clone(),
                            _ => {}
                        }
                        i += 1;
                    }
                    printingc = printingl.join("");
                }

                match value[1].unbox() {
                    Json::STRING(value) => {
                        if name.as_str() == "banner" {
                            printingc = get_banner(printingc.to_string());
                        }
                        printing = apply_color(value.as_str(), &printingc);
                    }
                    Json::NULL => {
                        printing = printingc;
                    }
                    _ => {}
                }

                match name.as_str() {
                    "banner" => {
                        for j in 0..8 {
                            let mut temp_string = String::new();
                            if (2..7).contains(&j) {
                                if let Some(banner_line) = printing.lines().nth(j - 2) {
                                    temp_string.push_str(banner_line);
                                    temp_string.push_str("       ");
                                    max_length = temp_string.len();
                                }
                            }
                            cache.push(temp_string);
                        }
                    }
                    "uptime" | "user_a_host_name" => {
                        let padding = if name == "uptime" { 0 } else { 5 };
                        if let Some(cache_text) = cache.get_mut(match name.as_str() {
                            "uptime" => 6,
                            _ => 1,
                        }) {
                            handle_spacing(cache_text, &printing, max_length, padding);
                        }
                    }
                    _ => {
                        let cache_index = match name.as_str() {
                            "os" => 2,
                            "kernel" => 3,
                            "shell" => 4,
                            "family" => 5,
                            "cpu_type" => 7,
                            _ => usize::MAX,
                        };
                        if cache_index != usize::MAX {
                            if let Some(cache_text) = cache.get_mut(cache_index) {
                                cache_text.push_str(&printing);
                            }
                        }
                    }
                }
            }
        }
    }
    let printable_text = cache.join("\r\n");
    println!("{}", printable_text);
}
