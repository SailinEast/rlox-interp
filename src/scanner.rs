use crate::error::ErrorReporter;
use crate::token::{Token, TokenType};

pub struct Scanner<'a> {
    pub source: &'a str,
    pub tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
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

    fn lookup_keyword(identifier: &str) -> TokenType {
        match identifier {
            "and" => TokenType::And,
            "class" => TokenType::Class,
            "else" => TokenType::Else,
            "false" => TokenType::False,
            "for" => TokenType::For,
            "fun" => TokenType::Fun,
            "if" => TokenType::If,
            "nil" => TokenType::Nil,
            "or" => TokenType::Or,
            "print" => TokenType::Print,
            "return" => TokenType::Return,
            "super" => TokenType::Super,
            "this" => TokenType::This,
            "true" => TokenType::True,
            "var" => TokenType::Var,
            "while" => TokenType::While,
            _ => TokenType::Identifier(identifier.to_string()),
        }
    }

    fn scan_token(&mut self, reporter: &mut ErrorReporter) {
        let c = self.advance();
        match c {
            // Single-char tokens
            b'(' => self.add_token(TokenType::LeftParen),
            b')' => self.add_token(TokenType::RightParen),
            b'{' => self.add_token(TokenType::LeftBrace),
            b'}' => self.add_token(TokenType::RightBrace),
            b',' => self.add_token(TokenType::Comma),
            b'.' => self.add_token(TokenType::Dot),
            b'-' => self.add_token(TokenType::Minus),
            b'+' => self.add_token(TokenType::Plus),
            b';' => self.add_token(TokenType::Semicolon),
            b'*' => self.add_token(TokenType::Star),
            b'!' => self.add_conditional_token(b'=', TokenType::BangEqual, TokenType::Bang),
            b'=' => self.add_conditional_token(b'=', TokenType::EqualEqual, TokenType::Equal),
            b'<' => self.add_conditional_token(b'=', TokenType::LessEqual, TokenType::Less),
            b'>' => self.add_conditional_token(b'=', TokenType::GreaterEqual, TokenType::Greater),
            b'/' => {
                if self.r#match(b'/') {
                    // Ignore comments
                    while self.peek() != b'\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else if self.r#match(b'*') {
                    self.comment(reporter);
                } else {
                    self.add_token(TokenType::Slash);
                }
            }

            // Whitespace
            b' ' | b'\r' | b'\t' => {}
            b'\n' => self.line += 1,

            // String literals
            b'"' => self.string(reporter),

            b'0'..=b'9' => self.number(),
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => self.identifier(),

            _ => reporter.error(self.line, "Unexpected character."),
        }
    }

    fn comment(&mut self, reporter: &mut ErrorReporter) {
        while !self.is_at_end() {
            if self.peek() == b'*' && self.peek_next() == b'/' {
                self.advance();
                self.advance();
                return;
            }
            if self.advance() == b'\n' {
                self.line += 1;
            }
        }
        reporter.error(self.line, "Unterminated comment.");
    }

    fn identifier(&mut self) {
        while Self::is_alpha_numeric(self.peek()) {
            self.advance();
        }

        let text = &self.source[self.start..self.current];
        self.add_token(Self::lookup_keyword(text));
    }

    fn number(&mut self) {
        while Self::is_digit(self.peek()) {
            self.advance();
        }

        // Look for a fractional part
        if self.peek() == b'.' && Self::is_digit(self.peek_next()) {
            // Consume the dot
            self.advance();
            while Self::is_digit(self.peek()) {
                self.advance();
            }
        }

        self.add_token(TokenType::Number(
            self.source[self.start..self.current].parse().unwrap(),
        ));
    }

    fn string(&mut self, reporter: &mut ErrorReporter) {
        while self.peek() != b'"' && !self.is_at_end() {
            if self.peek() == b'\n' {
                self.line += 1
            }
            self.advance();
        }

        if self.is_at_end() {
            reporter.error(self.line, "Unterminated string.");
            return;
        }

        // The closing "
        self.advance();

        // Trim surrounding quotes
        let value = self.source[self.start + 1..self.current - 1].to_string();
        self.add_token(TokenType::String(value));
    }

    fn r#match(&mut self, expected: u8) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.source.as_bytes()[self.current] != expected {
            return false;
        }
        self.current += 1;
        true
    }

    fn add_token(&mut self, token_type: TokenType) {
        let text = &self.source[self.start..self.current];
        self.tokens.push(Token {
            token_type,
            lexeme: text.to_string(),
            line: self.line,
        });
    }

    fn add_conditional_token(&mut self, expected: u8, matched: TokenType, unmatched: TokenType) {
        let token_type = if self.r#match(expected) {
            matched
        } else {
            unmatched
        };
        self.add_token(token_type);
    }

    fn advance(&mut self) -> u8 {
        let byte = self.source.as_bytes()[self.current];
        self.current += 1;
        byte
    }

    fn peek(&self) -> u8 {
        if self.is_at_end() {
            return b'\0';
        }
        self.source.as_bytes()[self.current]
    }

    fn peek_next(&self) -> u8 {
        if self.current + 1 >= self.source.len() {
            return b'\0';
        }
        self.source.as_bytes()[self.current + 1]
    }

    fn is_alpha(c: u8) -> bool {
        c.is_ascii_alphabetic() || c == b'_'
    }

    fn is_alpha_numeric(c: u8) -> bool {
        Self::is_alpha(c) || Self::is_digit(c)
    }

    fn is_digit(c: u8) -> bool {
        c.is_ascii_digit()
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scan(source: &str) -> (Vec<Token>, bool) {
        let mut reporter = ErrorReporter::new();
        let tokens = Scanner::new(source).scan_tokens(&mut reporter);
        (tokens, reporter.had_error)
    }

    #[test]
    fn empty_source_produces_only_eof() {
        let (tokens, had_error) = scan("");
        assert!(!had_error);
        assert_eq!(tokens, vec![Token::new(TokenType::Eof, "".into(), 1)]);
    }

    #[test]
    fn single_char_tokens() {
        let (tokens, had_error) = scan("(){},.-+;* /");
        assert!(!had_error);
        assert_eq!(
            tokens,
            vec![
                Token::new(TokenType::LeftParen, "(".into(), 1),
                Token::new(TokenType::RightParen, ")".into(), 1),
                Token::new(TokenType::LeftBrace, "{".into(), 1),
                Token::new(TokenType::RightBrace, "}".into(), 1),
                Token::new(TokenType::Comma, ",".into(), 1),
                Token::new(TokenType::Dot, ".".into(), 1),
                Token::new(TokenType::Minus, "-".into(), 1),
                Token::new(TokenType::Plus, "+".into(), 1),
                Token::new(TokenType::Semicolon, ";".into(), 1),
                Token::new(TokenType::Star, "*".into(), 1),
                Token::new(TokenType::Slash, "/".into(), 1),
                Token::new(TokenType::Eof, "".into(), 1),
            ]
        );
    }

    #[test]
    fn one_or_two_char_tokens() {
        let (tokens, had_error) = scan("! != = == > >= < <=");
        assert!(!had_error);
        assert_eq!(
            tokens,
            vec![
                Token::new(TokenType::Bang, "!".into(), 1),
                Token::new(TokenType::BangEqual, "!=".into(), 1),
                Token::new(TokenType::Equal, "=".into(), 1),
                Token::new(TokenType::EqualEqual, "==".into(), 1),
                Token::new(TokenType::Greater, ">".into(), 1),
                Token::new(TokenType::GreaterEqual, ">=".into(), 1),
                Token::new(TokenType::Less, "<".into(), 1),
                Token::new(TokenType::LessEqual, "<=".into(), 1),
                Token::new(TokenType::Eof, "".into(), 1),
            ]
        );
    }

    #[test]
    fn comments_are_ignored() {
        let source = "\
        // comment on line 1
        + // trailing comment
        - // comment at EOF without newline";
        let (tokens, had_error) = scan(source);
        assert!(!had_error);
        assert_eq!(
            tokens,
            vec![
                Token::new(TokenType::Plus, "+".into(), 2),
                Token::new(TokenType::Minus, "-".into(), 3),
                Token::new(TokenType::Eof, "".into(), 3),
            ]
        );
    }

    #[test]
    fn multiline_comments_are_ignored() {
        let source = "/*\n\
        var language = \"lox\";\n\
        */";
        let (tokens, had_error) = scan(source);
        assert!(!had_error);
        assert_eq!(tokens, vec![Token::new(TokenType::Eof, "".into(), 3)],);
    }

    #[test]
    fn block_comment_edge_cases() {
        let (tokens, had_error) = scan("/**/ + /***/");
        assert!(!had_error);
        assert_eq!(
            tokens,
            vec![
                Token::new(TokenType::Plus, "+".into(), 1),
                Token::new(TokenType::Eof, "".into(), 1),
            ]
        );

        let (tokens, had_error) = scan("/*");
        assert!(had_error);
        assert_eq!(tokens, vec![Token::new(TokenType::Eof, "".into(), 1)]);

        let (tokens, had_error) = scan("/*/");
        assert!(had_error);
        assert_eq!(tokens, vec![Token::new(TokenType::Eof, "".into(), 1)]);
    }

    #[test]
    fn multiline_comments_track_line_numbers() {
        let source = "/*\nvar language = \"lox\";\n*/\n+";
        let (tokens, had_error) = scan(source);
        assert!(!had_error);
        assert_eq!(
            tokens,
            vec![
                Token::new(TokenType::Plus, "+".into(), 4),
                Token::new(TokenType::Eof, "".into(), 4),
            ]
        );
    }

    #[test]
    fn unterminated_comment_reports_error() {
        let (tokens, had_error) = scan("/*unterminated");
        assert!(had_error);
        assert_eq!(tokens, vec![Token::new(TokenType::Eof, "".into(), 1)]);
    }

    #[test]
    fn whitespace_and_line_tracking() {
        let (tokens, had_error) = scan(" \t\r\n(\n\n)");
        assert!(!had_error);
        assert_eq!(
            tokens,
            vec![
                Token::new(TokenType::LeftParen, "(".into(), 2),
                Token::new(TokenType::RightParen, ")".into(), 4),
                Token::new(TokenType::Eof, "".into(), 4),
            ]
        );
    }

    #[test]
    fn string_literals_and_multiline_strings() {
        let (tokens, had_error) = scan("\"hello\" \"\"\n\"multi\nline\"\n+");
        assert!(!had_error);
        assert_eq!(tokens.len(), 5);

        assert_eq!(
            tokens[0],
            Token::new(TokenType::String("hello".into()), "\"hello\"".into(), 1)
        );
        assert_eq!(
            tokens[1],
            Token::new(TokenType::String("".into()), "\"\"".into(), 1)
        );
        assert_eq!(
            tokens[2].token_type,
            TokenType::String("multi\nline".into())
        );
        assert_eq!(tokens[2].lexeme, "\"multi\nline\"");
        assert_eq!(tokens[3], Token::new(TokenType::Plus, "+".into(), 4));
        assert_eq!(tokens[4], Token::new(TokenType::Eof, "".into(), 4));
    }

    #[test]
    fn unterminated_string_reports_error() {
        let (tokens, had_error) = scan("\"unterminated");
        assert!(had_error);
        assert_eq!(tokens, vec![Token::new(TokenType::Eof, "".into(), 1)]);
    }

    #[test]
    fn number_literals() {
        let (tokens, had_error) = scan("0 42 12.34 .5 5.");
        assert!(!had_error);
        assert_eq!(
            tokens,
            vec![
                Token::new(TokenType::Number(0.0), "0".into(), 1),
                Token::new(TokenType::Number(42.0), "42".into(), 1),
                Token::new(TokenType::Number(12.34), "12.34".into(), 1),
                Token::new(TokenType::Dot, ".".into(), 1),
                Token::new(TokenType::Number(5.0), "5".into(), 1),
                Token::new(TokenType::Number(5.0), "5".into(), 1),
                Token::new(TokenType::Dot, ".".into(), 1),
                Token::new(TokenType::Eof, "".into(), 1),
            ]
        );
    }

    #[test]
    fn keywords_and_identifiers() {
        let keywords =
            "and class else false for fun if nil or print return super this true var while";
        let (tokens, had_error) = scan(keywords);
        assert!(!had_error);

        let types: Vec<TokenType> = tokens.into_iter().map(|t| t.token_type).collect();
        assert_eq!(
            types,
            vec![
                TokenType::And,
                TokenType::Class,
                TokenType::Else,
                TokenType::False,
                TokenType::For,
                TokenType::Fun,
                TokenType::If,
                TokenType::Nil,
                TokenType::Or,
                TokenType::Print,
                TokenType::Return,
                TokenType::Super,
                TokenType::This,
                TokenType::True,
                TokenType::Var,
                TokenType::While,
                TokenType::Eof,
            ]
        );

        let (tokens, had_error) = scan("foo _bar baz_123 orchid iff classy");
        assert!(!had_error);
        assert_eq!(
            tokens,
            vec![
                Token::new(TokenType::Identifier("foo".into()), "foo".into(), 1),
                Token::new(TokenType::Identifier("_bar".into()), "_bar".into(), 1),
                Token::new(TokenType::Identifier("baz_123".into()), "baz_123".into(), 1),
                Token::new(TokenType::Identifier("orchid".into()), "orchid".into(), 1),
                Token::new(TokenType::Identifier("iff".into()), "iff".into(), 1),
                Token::new(TokenType::Identifier("classy".into()), "classy".into(), 1),
                Token::new(TokenType::Eof, "".into(), 1),
            ]
        );
    }

    #[test]
    fn unexpected_character_reports_error_and_continues() {
        let (tokens, had_error) = scan("@ + #");
        assert!(had_error);
        assert_eq!(
            tokens,
            vec![
                Token::new(TokenType::Plus, "+".into(), 1),
                Token::new(TokenType::Eof, "".into(), 1),
            ]
        );
    }

    #[test]
    fn utf8_in_strings_and_comments() {
        let (tokens, had_error) = scan("// café 🦀\n\"héllo 🦀\"");
        assert!(!had_error);
        assert_eq!(
            tokens,
            vec![
                Token::new(
                    TokenType::String("héllo 🦀".into()),
                    "\"héllo 🦀\"".into(),
                    2
                ),
                Token::new(TokenType::Eof, "".into(), 2),
            ]
        );
    }

    #[test]
    fn small_lox_program() {
        let source = "\
        var x = 10;
        if (x >= 5) {
            print \"big\";
        }";
        let (tokens, had_error) = scan(source);
        assert!(!had_error);
        assert_eq!(
            tokens,
            vec![
                Token::new(TokenType::Var, "var".into(), 1),
                Token::new(TokenType::Identifier("x".into()), "x".into(), 1),
                Token::new(TokenType::Equal, "=".into(), 1),
                Token::new(TokenType::Number(10.0), "10".into(), 1),
                Token::new(TokenType::Semicolon, ";".into(), 1),
                Token::new(TokenType::If, "if".into(), 2),
                Token::new(TokenType::LeftParen, "(".into(), 2),
                Token::new(TokenType::Identifier("x".into()), "x".into(), 2),
                Token::new(TokenType::GreaterEqual, ">=".into(), 2),
                Token::new(TokenType::Number(5.0), "5".into(), 2),
                Token::new(TokenType::RightParen, ")".into(), 2),
                Token::new(TokenType::LeftBrace, "{".into(), 2),
                Token::new(TokenType::Print, "print".into(), 3),
                Token::new(TokenType::String("big".into()), "\"big\"".into(), 3),
                Token::new(TokenType::Semicolon, ";".into(), 3),
                Token::new(TokenType::RightBrace, "}".into(), 4),
                Token::new(TokenType::Eof, "".into(), 4),
            ]
        );
    }
}
