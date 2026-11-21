use std::fmt;
use std::fmt::Formatter;

#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    // Single-char Tokens
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // 1 or 2 char Tokens
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals
    Identifier(String),
    String(String),
    Number(f64),

    // Keywords
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Eof,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: usize,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, line: usize) -> Self {
        Self {
            token_type,
            lexeme,
            line,
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} {}", self.token_type, self.lexeme)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_new_sets_fields() {
        let token = Token::new(TokenType::Plus, "+".to_string(), 3);
        assert_eq!(token.token_type, TokenType::Plus);
        assert_eq!(token.lexeme, "+");
        assert_eq!(token.line, 3);
    }

    #[test]
    fn token_display_formats_correctly() {
        let plus = Token::new(TokenType::Plus, "+".to_string(), 1);
        assert_eq!(format!("{plus}"), "Plus +");

        let num = Token::new(TokenType::Number(42.5), "42.5".to_string(), 1);
        assert_eq!(format!("{num}"), "Number(42.5) 42.5");

        let text = Token::new(TokenType::String("hi".to_string()), "\"hi\"".to_string(), 1);
        assert_eq!(format!("{text}"), "String(\"hi\") \"hi\"");
    }
}
