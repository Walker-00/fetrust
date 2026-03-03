# fetrust Configuration Examples

This directory contains example configuration files for fetrust.

## Available Colors

- `black` - Black text
- `red` - Red text
- `green` - Green text
- `yellow` - Yellow text
- `blue` - Blue text
- `purple` - Purple text
- `cyan` - Cyan text
- `white` - White text
- `rand` or `random` - Random RGB color (different each run)

## Available Fields

| Field | Description |
|-------|-------------|
| `username` | Current user name |
| `hostname` | System hostname |
| `os` | Operating system name |
| `os_release` | OS version/release |
| `kernel_name` | Kernel name |
| `kernel` | Kernel version |
| `shell` | Current shell |
| `family` | OS family |
| `uptime` | System uptime |
| `resolution` | Display resolution |
| `cpu_type` | CPU model and speed |
| `memory` | Memory usage |
| `desktop` | Desktop environment |
| `theme` | GTK theme name |
| `icon` | Icon theme name |
| `font` | Font name |
| `cursor` | Cursor theme name |

## Config Format

```json
{
	"key": [["field1", " @ ", "field2"], "color"]
}
```

- **key**: Internal identifier (e.g., `user_a_host_name`, `os`, `kernel`)
- **fields**: Array of field names and/or literal strings to concatenate
- **color**: Color for this line

**Note:** Literal strings (like `" @ "`, `" - "`, `": "`) are printed as-is.

## Example Configs

### Minimal (minimal.config.json)
Only shows essential system information.

### Colorful (colorful.config.json)
Each line has a different color for visual appeal.

### Monochrome (monochrome.config.json)
All white text - useful for minimal terminals or screenshots.

### Complete (complete.config.json)
Shows all available fields with documentation.

## Installation

Copy your preferred config to your fetrust config directory:

```bash
# Copy example config
cp examples/colorful.config.json ~/.config/fetrust/config.json
```

## Custom Configurations

You can create custom configurations by combining fields and literal strings:

```json
{
	"custom": [["CPU: ", "cpu_type", " | ", "memory"], "cyan"]
}
```

This would display: `CPU: Intel(R) Core(TM) i7 | 4096MiB / 16384MiB`
