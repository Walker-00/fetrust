//! Minimal JSON parser - no external dependencies

#[derive(Debug, Clone)]
pub enum JsonValue {
    Null,
    String(String),
    Array(Vec<JsonValue>),
    Object(Vec<(String, JsonValue)>),
}

impl JsonValue {
    pub fn get(&self, key: &str) -> Option<&JsonValue> {
        if let JsonValue::Object(pairs) = self {
            for (k, v) in pairs {
                if k == key {
                    return Some(v);
                }
            }
        }
        None
    }

    pub fn as_array(&self) -> Option<&Vec<JsonValue>> {
        if let JsonValue::Array(arr) = self {
            Some(arr)
        } else {
            None
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        if let JsonValue::String(s) = self {
            Some(s)
        } else {
            None
        }
    }

    pub fn print(&self) -> String {
        match self {
            JsonValue::String(s) => s.clone(),
            _ => String::new(),
        }
    }
}

pub struct JsonParser {
    chars: Vec<char>,
    pos: usize,
}

impl JsonParser {
    fn new(input: &[u8]) -> Self {
        let chars: Vec<char> = input.iter().map(|&b| b as char).collect();
        JsonParser { chars, pos: 0 }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.peek();
        self.pos += 1;
        ch
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn parse(&mut self) -> Result<JsonValue, &'static str> {
        self.skip_whitespace();
        match self.peek() {
            Some('{') => self.parse_object(),
            Some('[') => self.parse_array(),
            Some('"') => self.parse_string().map(JsonValue::String),
            Some('t') | Some('f') => self.parse_bool().map(|b| JsonValue::String(b.to_string())),
            Some('n') => self.parse_null().map(|_| JsonValue::String("null".to_string())),
            Some(c) if c == '-' || c.is_ascii_digit() => self.parse_number().map(JsonValue::String),
            Some(c) => Err(Box::leak(format!("Unexpected character: {}", c).into_boxed_str())),
            None => Err(Box::leak("Unexpected end of input".to_string().into_boxed_str())),
        }
    }

    fn parse_object(&mut self) -> Result<JsonValue, &'static str> {
        self.advance(); // consume '{'
        self.skip_whitespace();
        let mut pairs = Vec::new();

        if self.peek() == Some('}') {
            self.advance();
            return Ok(JsonValue::Object(pairs));
        }

        loop {
            self.skip_whitespace();
            let key = self.parse_string()?;

            self.skip_whitespace();
            if self.advance() != Some(':') {
                return Err("Expected ':' after object key");
            }

            let value = self.parse()?;
            pairs.push((key, value));

            self.skip_whitespace();
            match self.peek() {
                Some(',') => {
                    self.advance();
                }
                Some('}') => {
                    self.advance();
                    break;
                }
                _ => return Err("Expected ',' or '}' in object"),
            }
        }

        Ok(JsonValue::Object(pairs))
    }

    fn parse_array(&mut self) -> Result<JsonValue, &'static str> {
        self.advance(); // consume '['
        self.skip_whitespace();
        let mut items = Vec::new();

        if self.peek() == Some(']') {
            self.advance();
            return Ok(JsonValue::Array(items));
        }

        loop {
            let value = self.parse()?;
            items.push(value);

            self.skip_whitespace();
            match self.peek() {
                Some(',') => {
                    self.advance();
                }
                Some(']') => {
                    self.advance();
                    break;
                }
                _ => return Err("Expected ',' or ']' in array"),
            }
        }

        Ok(JsonValue::Array(items))
    }

    fn parse_string(&mut self) -> Result<String, &'static str> {
        self.advance(); // consume opening '"'
        let mut result = String::new();

        while let Some(ch) = self.advance() {
            match ch {
                '"' => return Ok(result),
                '\\' => {
                    match self.advance() {
                        Some('"') => result.push('"'),
                        Some('\\') => result.push('\\'),
                        Some('/') => result.push('/'),
                        Some('n') => result.push('\n'),
                        Some('r') => result.push('\r'),
                        Some('t') => result.push('\t'),
                        Some('b') => result.push('\x08'),
                        Some('f') => result.push('\x0C'),
                        Some('u') => {
                            let mut hex = String::new();
                            for _ in 0..4 {
                                if let Some(h) = self.advance() {
                                    hex.push(h);
                                }
                            }
                            if let Ok(code) = u32::from_str_radix(&hex, 16) {
                                if let Some(c) = char::from_u32(code) {
                                    result.push(c);
                                }
                            }
                        }
                        Some(c) => {
                            result.push('\\');
                            result.push(c);
                        }
                        None => return Err("Unexpected end of string in escape sequence"),
                    }
                }
                c => result.push(c),
            }
        }

        Err("Unterminated string")
    }

    fn parse_number(&mut self) -> Result<String, &'static str> {
        let mut num_str = String::new();

        if self.peek() == Some('-') {
            num_str.push(self.advance().unwrap());
        }

        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                num_str.push(self.advance().unwrap());
            } else {
                break;
            }
        }

        if self.peek() == Some('.') {
            num_str.push(self.advance().unwrap());
            while let Some(ch) = self.peek() {
                if ch.is_ascii_digit() {
                    num_str.push(self.advance().unwrap());
                } else {
                    break;
                }
            }
        }

        if let Some(ch) = self.peek() {
            if ch == 'e' || ch == 'E' {
                num_str.push(self.advance().unwrap());
                if let Some(ch) = self.peek() {
                    if ch == '+' || ch == '-' {
                        num_str.push(self.advance().unwrap());
                    }
                }
                while let Some(ch) = self.peek() {
                    if ch.is_ascii_digit() {
                        num_str.push(self.advance().unwrap());
                    } else {
                        break;
                    }
                }
            }
        }

        num_str
            .parse::<f64>()
            .map(|_| num_str)
            .map_err(|_| "Invalid number")
    }

    fn parse_bool(&mut self) -> Result<bool, &'static str> {
        if self.peek() == Some('t') {
            for expected in ['t', 'r', 'u', 'e'] {
                if self.advance() != Some(expected) {
                    return Err("Invalid boolean");
                }
            }
            Ok(true)
        } else {
            for expected in ['f', 'a', 'l', 's', 'e'] {
                if self.advance() != Some(expected) {
                    return Err("Invalid boolean");
                }
            }
            Ok(false)
        }
    }

    fn parse_null(&mut self) -> Result<(), &'static str> {
        for expected in ['n', 'u', 'l', 'l'] {
            if self.advance() != Some(expected) {
                return Err("Invalid null");
            }
        }
        Ok(())
    }
}

pub struct Json;

impl Json {
    pub fn parse(input: &[u8]) -> Result<JsonValue, &'static str> {
        let mut parser = JsonParser::new(input);
        parser.parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_object() {
        let json = br#"{"key": "value", "num": 42}"#;
        let result = Json::parse(json).unwrap();
        if let JsonValue::Object(pairs) = result {
            assert_eq!(pairs.len(), 2);
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_parse_array() {
        let json = br#"[1, 2, 3]"#;
        let result = Json::parse(json).unwrap();
        if let JsonValue::Array(arr) = result {
            assert_eq!(arr.len(), 3);
        } else {
            panic!("Expected array");
        }
    }
}
