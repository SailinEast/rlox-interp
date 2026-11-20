use crate::error::ErrorReporter;
use crate::token::{Token, TokenType};

pub struct Scanner {
    pub source: String,
    pub tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        Self {
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(mut self, reporter: &mut ErrorReporter) -> Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token(reporter);
        }

        self.tokens
            .push(Token::new(TokenType::Eof, String::new(), self.line));

        self.tokens
    }

    fn scan_token(&mut self, reporter: &mut ErrorReporter) {
        self.current += 1;
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }
}
