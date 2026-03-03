//! Custom FIGfont parser - no external dependencies
//! Parses FIGlet font files (.flf)

use std::collections::HashMap;
use std::fs;

pub struct FIGfont {
    chars: HashMap<char, Vec<String>>,
    height: usize,
}

impl FIGfont {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path).map_err(|e| format!("Failed to read font file: {}", e))?;
        Self::parse(&content)
    }

    pub fn parse(content: &str) -> Result<Self, String> {
        let lines: Vec<&str> = content.lines().collect();
        if lines.is_empty() {
            return Err("Empty font file".to_string());
        }

        // Parse header line (flf2a format)
        let header = lines[0];
        let parts: Vec<&str> = header.split_whitespace().collect();
        if parts.is_empty() || !parts[0].starts_with("flf2") {
            return Err("Invalid FIGfont header".to_string());
        }

        // Parse header values
        let mut values: Vec<i32> = Vec::new();
        for part in parts {
            if let Ok(val) = part.parse::<i32>() {
                values.push(val);
            }
        }

        if values.len() < 6 {
            return Err("Invalid FIGfont header values".to_string());
        }

        let height = values[0] as usize;
        let _hardblank = char::from_u32(values[1] as u32).unwrap_or('$');
        let _full_width = values[2] as usize;
        let _smush2 = values[3] as usize;
        let comment_lines = values[4] as usize;
        let _print_dir = values[5] as usize;

        if height == 0 {
            return Err("Invalid font height".to_string());
        }

        // Find the start of character data (skip header and comments)
        let mut line_idx = 1 + comment_lines;

        // Skip any empty lines after comments
        while line_idx < lines.len() && lines[line_idx].trim().is_empty() {
            line_idx += 1;
        }

        let mut chars = HashMap::new();

        // Parse character data - characters start from ASCII 32 (space)
        let mut char_code: usize = 32;

        while line_idx + height <= lines.len() {
            let mut char_lines: Vec<String> = Vec::with_capacity(height);
            let mut max_line_len = 0;

            // Check for label line (e.g., "160  NO-BREAK SPACE") - only for extended chars
            if char_code > 127
                && line_idx < lines.len()
            {
                let check_line = lines[line_idx].trim();
                if !check_line.is_empty() {
                    let mut chars_iter = check_line.chars();
                    let mut has_digit = false;
                    let mut has_space_after_digit = false;
                    for c in chars_iter.by_ref() {
                        if c.is_ascii_digit() {
                            has_digit = true;
                        } else if c.is_whitespace() && has_digit {
                            has_space_after_digit = true;
                            break;
                        } else {
                            break;
                        }
                    }
                    if has_digit && has_space_after_digit {
                        // This is a label line, skip it
                        line_idx += 1;
                    }
                }
            }

            // Check if we have enough lines left
            if line_idx + height > lines.len() {
                break;
            }

            // Read `height` lines for this character
            for i in 0..height {
                let line = lines[line_idx + i];

                if line.is_empty() {
                    char_lines.push(String::new());
                } else {
                    // Process the character line - strip end marker and replace hardblank
                    let processed_line = process_char_line(line);
                    max_line_len = max_line_len.max(processed_line.len());
                    char_lines.push(processed_line);
                }
            }

            if let Some(c) = char::from_u32(char_code as u32) {
                // Pad all lines to the same length
                for line in &mut char_lines {
                    while line.len() < max_line_len {
                        line.push(' ');
                    }
                }
                chars.insert(c, char_lines);
            }

            char_code += 1;
            line_idx += height;

            // Skip empty lines between characters
            while line_idx < lines.len() && lines[line_idx].trim().is_empty() {
                line_idx += 1;
            }
        }

        Ok(FIGfont { chars, height })
    }

    pub fn convert(&self, text: &str) -> Option<String> {
        if self.height == 0 {
            return None;
        }

        let mut output_lines: Vec<String> = vec![String::new(); self.height];

        for ch in text.chars() {
            if let Some(char_lines) = self.chars.get(&ch) {
                for (i, line) in char_lines.iter().enumerate() {
                    output_lines[i].push_str(line);
                }
            } else if let Some(space_lines) = self.chars.get(&' ') {
                // Use space for unknown characters
                for (i, line) in space_lines.iter().enumerate() {
                    output_lines[i].push_str(line);
                }
            }
        }

        Some(output_lines.join("\n"))
    }
}

/// Process a character line by removing the end marker
fn process_char_line(line: &str) -> String {
    if line.is_empty() {
        return String::new();
    }

    // Strip trailing @ or @@ (end marker)
    let result = if let Some(stripped) = line.strip_suffix("@@") {
        stripped
    } else if let Some(stripped) = line.strip_suffix('@') {
        stripped
    } else {
        line
    };

    // Replace hardblank ($) with space
    result.replace('$', " ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let font_content = "flf2a$ 3 3 10 10 0 0
   $@
  $ @
 $  @@
";
        let font = FIGfont::parse(font_content).unwrap();
        assert_eq!(font.height, 3);
        assert!(font.chars.contains_key(&' '));
    }
}
