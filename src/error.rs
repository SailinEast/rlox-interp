use crate::token::{Token, TokenType};

pub struct ErrorReporter {
    had_error: bool,
}

impl ErrorReporter {
    pub fn new() -> Self {
        Self { had_error: false }
    }

    pub fn error(&mut self, line: usize, message: &str) {
        self.report(line, "", message);
    }

    pub fn had_error(&self) -> bool {
        self.had_error
    }

    pub fn token_error(&mut self, token: &Token, message: &str) {
        if token.token_type == TokenType::Eof {
            self.report(token.line, " at end", message);
        } else {
            self.report(token.line, &format!(" at '{}'", token.lexeme), message);
        }
    }

    pub fn report(&mut self, line: usize, location: &str, message: &str) {
        eprintln!("[line {line}] Error{location}: {message}");
        self.had_error = true;
    }

    pub fn reset(&mut self) {
        self.had_error = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_reporter_starts_without_error() {
        let reporter = ErrorReporter::new();
        assert!(!reporter.had_error);
    }

    #[test]
    fn error_sets_had_error() {
        let mut reporter = ErrorReporter::new();
        reporter.error(1, "Unexpected character.");
        assert!(reporter.had_error);
    }

    #[test]
    fn report_sets_had_error() {
        let mut reporter = ErrorReporter::new();
        reporter.report(4, " at 'foo'", "Expect ';' after expression.");
        assert!(reporter.had_error);
    }

    #[test]
    fn reset_clears_had_error() {
        let mut reporter = ErrorReporter::new();
        reporter.error(1, "Something went wrong.");
        assert!(reporter.had_error);

        reporter.reset();
        assert!(!reporter.had_error);
    }

    #[test]
    fn token_error_sets_had_error_at_eof() {
        let mut reporter = ErrorReporter::new();
        let eof_token = Token {
            token_type: TokenType::Eof,
            lexeme: String::new(),
            line: 10,
        };

        reporter.token_error(&eof_token, "Expect '}' after block.");
        assert!(reporter.had_error());
    }

    #[test]
    fn token_error_sets_had_error_at_lexeme() {
        let mut reporter = ErrorReporter::new();
        let token = Token {
            token_type: TokenType::Nil,
            lexeme: "foo".to_string(),
            line: 4,
        };

        reporter.token_error(&token, "Expect ';' after expression.");
        assert!(reporter.had_error());
    }
}
