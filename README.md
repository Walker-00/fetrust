# fetrust

A minimal and fast system info fetcher written in Rust with zero external dependencies.

![screenshot](https://github.com/user-attachments/assets/2dbc72b9-8983-4432-87a9-309af7cdd15f)

## Features

- **Fast by default**: No external process spawning in default mode
- **Probe mode**: Optional `--probe` flag for enhanced accuracy
- **Zero dependencies**: Pure Rust with custom JSON, INI, and FIGfont parsers
- **Configurable**: JSON config at `~/.config/fetrust/config.json`
- **Themed output**: 8 colors + random per-line coloring
- **ASCII art**: Dynamic FIGlet banner based on OS

## Installation

### Build from source

```bash
# Build release
cargo build --release

# Or with just
just build
```

### Quick commands (with just)

```bash
just install      # Install to ~/.local/bin
just uninstall    # Remove from ~/.local/bin
just list         # List all available just recipes
```

## Usage

```bash
# Fast mode (default - no external commands)
fetrust

# Probe mode (uses hyprctl, xrandr, lspci, gsettings for accurate info)
fetrust --probe
# or
fetrust --full
```

## Configuration

Config file location: `~/.config/fetrust/config.json`

### Example config

```json
{
    "user_a_host_name": [["username", "@", "hostname"], "rand"],
    "host_model":       [["host_model"], "cyan"],
    "os":               [["os", " ", "os_release"], "rand"],
    "kernel":           [["kernel_name", " ", "kernel"], "blue"],
    "shell":            [["shell"], "green"],
    "cpu_type":         [["cpu_type"], "yellow"],
    "memory":           [["memory"], "rand"],
    "gpu":              [["gpu"], "magenta"],
    "theme":            [["theme"], "cyan"],
    "icon":             [["icon"], "rand"],
    "cursor":           [["cursor"], "rand"]
}
```

### Available fields

| Field | Description |
|-------|-------------|
| `username` | Current user |
| `hostname` | System hostname |
| `host_model` | Vendor + product model (from DMI) |
| `os` | Operating system name |
| `os_release` | OS version |
| `kernel_name` | Kernel name |
| `kernel` | Kernel version |
| `shell` | User shell |
| `family` | OS family (Unix/Windows) |
| `uptime` | System uptime |
| `resolution` | Screen resolution |
| `cpu_type` | CPU model and clock |
| `memory` | RAM usage |
| `gpu` | Graphics card |
| `terminal` | Terminal emulator |
| `desktop` | Desktop environment |
| `theme` | GTK theme |
| `icon` | Icon theme |
| `font` | System font |
| `cursor` | Cursor theme |

### Available colors

`black`, `red`, `green`, `yellow`, `blue`, `purple`, `cyan`, `white`, `rand`/`random`

## Fast vs Probe Mode

| Field | Fast (default) | Probe (`--probe`) |
|-------|---------------|-------------------|
| Resolution | `Unknown` | `hyprctl` → `xrandr` → `xdpyinfo` |
| GPU | `Unknown` | `lspci` |
| Themes | GTK settings.ini only | + `gsettings` fallback |
| OS Release | `/etc/os-release` | `lsb_release` → fallback |

**Fast mode spawns ZERO external processes** (except `uname -r` for kernel).

## Example Output

```
                                                             walker@Rissk
                                                             ────────────

                             Host        ==>  LENOVO 82RJ IdeaPad 3 14IAU7                                
                             OS          ==>  Arch Linux unknown                                          
                             Kernel      ==>  linux 6.18.13-zen1-1-zen                                    
                             Shell       ==>  /bin/bash                                                   
                             Family      ==>  unix                                                        
                             Uptime      ==>  3 days, 10 hours, 42 minutes, 46 seconds                    
   ___               __      Resolution  ==>  1920x1080                                                   
  / _ |  ____ ____  / /      CPU         ==>  12th Gen Intel(R) Core(TM) i3-1215U @ 993.847 MHz           
 / __ | / __// __/ / _ \     Memory      ==>  6307MiB / 7658MiB                                           
/_/ |_|/_/   \__/ /_//_/     GPU         ==>  Intel Corporation Alder Lake-UP3 GT1 [UHD Graphics] (rev 0c)
                             Terminal    ==>  xterm                                                       
                             DE          ==>  Hyprland                                                    
                             Theme       ==>  adw-gtk3-dark                                               
                             Icons       ==>  Papirus-Dark                                                
                             Font        ==>  Google Sans Flex Medium 11 @opsz=11,wght=500                
                             Cursor      ==>  Bibata-Modern-Classic                                       
```

## License

This project is licensed under the **GNU General Public License v3.0** (GPL-3.0).  
See the [LICENSE](LICENSE) file for details.

## TODO List

- [x] Config support
- [x] Automatic creating config folder/files
- [x] Making it works without any library and bloat blobs (zero external dependencies achieved!)
- [x] Get more information from device (host, gpu, terminal, resolution, themes)
- [ ] Installation function for once (make more theming)
- [x] Writing ASCII art depends on distro
- [x] Get ram usage
- [x] Get theme (wm, gtk, font name, cursor etc.)
- [ ] ASCII according to the tropics of the moon

## Acknowledgments

Thanks to:
- [Mertoalex](https://github.com/mertoalex)
- [Walker-00](https://github.com/Walker-00)
- [Speretta](https://github.com/Speretta)
