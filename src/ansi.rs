//! Custom ANSI color module - no external dependencies

pub struct Color {
    code: &'static str,
}

impl Color {
    pub const fn black() -> Self {
        Color { code: "\x1b[30m" }
    }

    pub const fn red() -> Self {
        Color { code: "\x1b[31m" }
    }

    pub const fn green() -> Self {
        Color { code: "\x1b[32m" }
    }

    pub const fn yellow() -> Self {
        Color { code: "\x1b[33m" }
    }

    pub const fn blue() -> Self {
        Color { code: "\x1b[34m" }
    }

    pub const fn purple() -> Self {
        Color { code: "\x1b[35m" }
    }

    pub const fn cyan() -> Self {
        Color { code: "\x1b[36m" }
    }

    pub const fn white() -> Self {
        Color { code: "\x1b[37m" }
    }

    pub const fn reset() -> &'static str {
        "\x1b[0m"
    }

    pub fn paint(&self, text: &str) -> String {
        if self.code.is_empty() {
            // Special case for RGB - will be handled by apply_color_rgb
            text.to_string()
        } else {
            format!("{}{}{}", self.code, text, Self::reset())
        }
    }
}

pub fn apply_color(color: &str, text: &str) -> String {
    match color {
        "black" => Color::black().paint(text),
        "red" => Color::red().paint(text),
        "green" => Color::green().paint(text),
        "yellow" => Color::yellow().paint(text),
        "blue" => Color::blue().paint(text),
        "purple" => Color::purple().paint(text),
        "cyan" => Color::cyan().paint(text),
        "white" => Color::white().paint(text),
        "rand" | "random" => {
            let (r, g, b) = random_color_codes();
            apply_color_rgb(r, g, b, text)
        }
        _ => {
            eprintln!(
                "{}Warning: Color \"{}\" isn't defined, so it's default color.{}",
                Color::yellow().code,
                color,
                Color::reset()
            );
            text.to_string()
        }
    }
}

pub fn apply_color_rgb(r: u8, g: u8, b: u8, text: &str) -> String {
    // Use 24-bit true color if terminal supports it
    format!("\x1b[38;2;{};{};{}m{}{}", r, g, b, text, Color::reset())
}

/// Simple LCG-based random number generator (no external dependencies)
pub struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    pub fn new(seed: u64) -> Self {
        SimpleRng { state: seed }
    }

    pub fn next_u64(&mut self) -> u64 {
        // LCG parameters from Numerical Recipes
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.state
    }

    pub fn next_u8(&mut self) -> u8 {
        (self.next_u64() >> 56) as u8
    }
}

/// Get a seed from the system (using current time)
fn get_system_seed() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}

pub fn thread_rand() -> SimpleRng {
    SimpleRng::new(get_system_seed())
}

pub fn random_color_codes() -> (u8, u8, u8) {
    let mut rand = thread_rand();
    (rand.next_u8(), rand.next_u8(), rand.next_u8())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_paint() {
        let red = Color::red();
        let painted = red.paint("test");
        assert!(painted.contains("test"));
        assert!(painted.contains("\x1b[30m") || painted.contains("\x1b[31m"));
    }

    #[test]
    fn test_rng() {
        let mut rng = SimpleRng::new(42);
        let val1 = rng.next_u64();
        let val2 = rng.next_u64();
        assert_ne!(val1, val2);
    }
}
