use crate::lexer::token::Token;

pub struct Scanner {
    source: Vec<char>,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
    col: usize,
}

impl Scanner {
    pub fn new(source: &str) -> Self {
        Scanner {
            source: source.chars().collect(),
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
            col: 1,
        }
    }

    pub fn scan_tokens(&mut self) -> Result<Vec<Token>, ScanError> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token()?;
        }

        self.auto_insert_semicolon();
        self.tokens.push(Token::Eof);
        Ok(self.tokens.clone())
    }

    fn scan_token(&mut self) -> Result<(), ScanError> {
        let c = self.advance();
        match c {
            '(' => self.tokens.push(Token::LeftParen),
            ')' => self.tokens.push(Token::RightParen),
            '{' => self.tokens.push(Token::LeftBrace),
            '}' => self.tokens.push(Token::RightBrace),
            '[' => self.tokens.push(Token::LeftBracket),
            ']' => self.tokens.push(Token::RightBracket),
            ',' => self.tokens.push(Token::Comma),
            ':' => self.tokens.push(Token::Colon),
            ';' => self.tokens.push(Token::Semicolon),
            '+' => self.tokens.push(Token::Plus),
            '-' => self.tokens.push(Token::Minus),
            '*' => self.tokens.push(Token::Star),
            '%' => self.tokens.push(Token::Percent),
            '.' => {
                if self.peek() == '.' {
                    self.advance();
                    self.tokens.push(Token::DotDot);
                } else {
                    self.tokens.push(Token::Dot);
                }
            }
            '/' => {
                if self.peek() == '/' {
                    // Comment — skip to end of line
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.tokens.push(Token::Slash);
                }
            }
            '!' => {
                if self.peek() == '=' {
                    self.advance();
                    self.tokens.push(Token::BangEqual);
                } else {
                    self.tokens.push(Token::Bang);
                }
            }
            '=' => {
                if self.peek() == '=' {
                    self.advance();
                    self.tokens.push(Token::EqualEqual);
                } else {
                    self.tokens.push(Token::Assign);
                }
            }
            '>' => {
                if self.peek() == '=' {
                    self.advance();
                    self.tokens.push(Token::GreaterEqual);
                } else {
                    self.tokens.push(Token::GreaterThan);
                }
            }
            '<' => {
                if self.peek() == '=' {
                    self.advance();
                    self.tokens.push(Token::LessEqual);
                } else {
                    self.tokens.push(Token::LessThan);
                }
            }
            '&' => {
                if self.peek() == '&' {
                    self.advance();
                    self.tokens.push(Token::AmpAmp);
                } else {
                    return Err(ScanError {
                        line: self.line,
                        col: self.col,
                        message: "unexpected '&', did you mean '&&'?".to_string(),
                    });
                }
            }
            '|' => {
                if self.peek() == '|' {
                    self.advance();
                    self.tokens.push(Token::PipePipe);
                } else {
                    return Err(ScanError {
                        line: self.line,
                        col: self.col,
                        message: "unexpected '|', did you mean '||'?".to_string(),
                    });
                }
            }
            '"' => self.scan_string()?,
            '\n' => {
                self.auto_insert_semicolon();
                self.line += 1;
                self.col = 1;
            }
            ' ' | '\r' | '\t' => {}
            _ => {
                if c.is_ascii_digit() {
                    self.scan_number()?;
                } else if c.is_alphabetic() || c == '_' {
                    self.scan_identifier();
                } else {
                    return Err(ScanError {
                        line: self.line,
                        col: self.col,
                        message: format!("unexpected character '{}'", c),
                    });
                }
            }
        }
        Ok(())
    }

    fn scan_string(&mut self) -> Result<(), ScanError> {
        let mut parts: Vec<(String, String)> = Vec::new();
        let mut current_text = String::new();
        let mut has_interpolation = false;

        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
                self.col = 1;
            }
            if self.peek() == '\\' {
                self.advance();
                match self.peek() {
                    'n' => { self.advance(); current_text.push('\n'); }
                    't' => { self.advance(); current_text.push('\t'); }
                    '\\' => { self.advance(); current_text.push('\\'); }
                    '"' => { self.advance(); current_text.push('"'); }
                    '{' => { self.advance(); current_text.push('{'); }
                    '}' => { self.advance(); current_text.push('}'); }
                    _ => {
                        return Err(ScanError {
                            line: self.line,
                            col: self.col,
                            message: format!("unknown escape sequence '\\{}'", self.peek()),
                        });
                    }
                }
            } else if self.peek() == '{' {
                has_interpolation = true;
                self.advance(); // consume '{'
                let mut expr_src = String::new();
                let mut depth = 1;
                while depth > 0 && !self.is_at_end() {
                    if self.peek() == '{' { depth += 1; }
                    if self.peek() == '}' { depth -= 1; }
                    if depth > 0 {
                        expr_src.push(self.advance());
                    }
                }
                if self.is_at_end() {
                    return Err(ScanError {
                        line: self.line,
                        col: self.col,
                        message: "unterminated interpolation in string".to_string(),
                    });
                }
                self.advance(); // consume closing '}'
                parts.push((current_text.clone(), expr_src));
                current_text.clear();
            } else {
                current_text.push(self.advance());
            }
        }

        if self.is_at_end() {
            return Err(ScanError {
                line: self.line,
                col: self.col,
                message: "unterminated string".to_string(),
            });
        }

        self.advance(); // closing "

        if has_interpolation {
            // Push trailing text as a part with empty expr
            if !current_text.is_empty() {
                parts.push((current_text, String::new()));
            }
            self.tokens.push(Token::InterpolatedText(parts));
        } else {
            self.tokens.push(Token::TextLit(current_text));
        }
        Ok(())
    }

    fn scan_number(&mut self) -> Result<(), ScanError> {
        let mut is_float = false;

        while self.peek().is_ascii_digit() {
            self.advance();
        }

        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            is_float = true;
            self.advance(); // consume '.'
            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        let text: String = self.source[self.start..self.current].iter().collect();
        let value: f64 = text.parse().map_err(|_| ScanError {
            line: self.line,
            col: self.col,
            message: format!("invalid number '{}'", text),
        })?;

        self.tokens.push(Token::NumberLit(value, is_float));
        Ok(())
    }

    fn scan_identifier(&mut self) {
        while self.peek().is_alphanumeric() || self.peek() == '_' {
            self.advance();
        }

        let text: String = self.source[self.start..self.current].iter().collect();

        if let Some(keyword) = Token::from_keyword(&text) {
            self.tokens.push(keyword);
        } else {
            self.tokens.push(Token::Identifier(text));
        }
    }

    /// Auto-insert semicolon at newline (Go-style) if the last token
    /// could end a statement.
    fn auto_insert_semicolon(&mut self) {
        if let Some(last) = self.tokens.last() {
            match last {
                Token::NumberLit(_, _)
                | Token::TextLit(_)
                | Token::BoolLit(_)
                | Token::InterpolatedText(_)
                | Token::Identifier(_)
                | Token::RightParen
                | Token::RightBrace
                | Token::RightBracket
                | Token::Return
                | Token::Fail => {
                    self.tokens.push(Token::Semicolon);
                }
                _ => {}
            }
        }
    }

    fn advance(&mut self) -> char {
        let c = self.source[self.current];
        self.current += 1;
        self.col += 1;
        c
    }

    fn peek(&self) -> char {
        if self.is_at_end() { '\0' } else { self.source[self.current] }
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() { '\0' } else { self.source[self.current + 1] }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }
}

#[derive(Debug)]
pub struct ScanError {
    pub line: usize,
    pub col: usize,
    pub message: String,
}

impl std::fmt::Display for ScanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error at line {}:{}: {}", self.line, self.col, self.message)
    }
}
