use crate::ast::{Expr, Literal};
use crate::error::ErrorReporter;
use crate::token::{Token, TokenType};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

struct ParseError {
    token: Token,
    message: &'static str,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self, reporter: &mut ErrorReporter) -> Option<Expr> {
        let expr = self.parse_expr(0);
        match expr {
            Ok(expr) => {
                if !self.is_at_end() {
                    reporter.token_error(self.peek(), "Unexpected token.");
                    None
                } else {
                    Some(expr)
                }
            }
            Err(err) => {
                reporter.token_error(&err.token, err.message);
                None
            }
        }
    }

    // Pratt parser for expressions
    fn parse_expr(&mut self, min_bp: u8) -> Result<Expr, ParseError> {
        let token = self.advance().clone();
        let mut lhs = match &token.token_type {
            // Literals
            TokenType::Number(n) => Expr::Literal(Literal::Number(*n)),
            TokenType::String(s) => Expr::Literal(Literal::String(s.to_string())),
            TokenType::Nil => Expr::Literal(Literal::Nil),
            TokenType::True => Expr::Literal(Literal::Bool(true)),
            TokenType::False => Expr::Literal(Literal::Bool(false)),

            // Grouping
            TokenType::LeftParen => {
                let grouped = Expr::Grouping(Box::from(self.parse_expr(0)?));
                self.consume(&TokenType::RightParen, "Expect ')' after expression.")?;
                grouped
            }

            // Unary
            TokenType::Minus | TokenType::Bang => {
                let bp = Self::prefix_bp(&token.token_type).unwrap();
                let unary = Box::from(self.parse_expr(bp)?);
                Expr::Unary(token, unary)
            }

            _ => {
                return Err(ParseError {
                    token,
                    message: "Expect expression.",
                });
            }
        };

        loop {
            let next = self.peek();
            if let Some(bp) = Self::infix_bp(&next.token_type) {
                if bp.0 < min_bp {
                    break;
                }
                let op = self.advance().clone();

                lhs = match op.token_type {
                    TokenType::Question => {
                        let truthy = self.parse_expr(0)?;
                        self.consume(&TokenType::Colon, "Expect ':' after then branch.")?;
                        let falsy = self.parse_expr(bp.1)?;
                        Expr::Ternary(Box::from(lhs), op, Box::from(truthy), Box::from(falsy))
                    }
                    _ => {
                        let rhs = self.parse_expr(bp.1)?;
                        Expr::Binary(Box::from(lhs), op, Box::from(rhs))
                    }
                }
            } else {
                break;
            }
        }
        Ok(lhs)
    }

    // expr [op] expr, (main_bp, associativity)
    fn infix_bp(op: &TokenType) -> Option<(u8, u8)> {
        match op {
            TokenType::Comma => Some((3, 4)),
            TokenType::Question => Some((6, 5)),
            TokenType::EqualEqual | TokenType::BangEqual => Some((7, 8)),
            TokenType::Less
            | TokenType::LessEqual
            | TokenType::Greater
            | TokenType::GreaterEqual => Some((9, 10)),
            TokenType::Plus | TokenType::Minus => Some((11, 12)),
            TokenType::Star | TokenType::Slash => Some((13, 14)),
            _ => None,
        }
    }

    // [op] expr
    fn prefix_bp(op: &TokenType) -> Option<u8> {
        match op {
            TokenType::Bang | TokenType::Minus => Some(15),
            _ => None,
        }
    }

    // Expect a consumed token type, error otherwise
    fn consume(
        &mut self,
        token_type: &TokenType,
        message: &'static str,
    ) -> Result<Token, ParseError> {
        if self.check(token_type) {
            return Ok(self.advance().clone());
        }
        Err(ParseError {
            token: self.peek().clone(),
            message,
        })
    }

    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        };
        self.peek().token_type == *token_type
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1
        } else {
            return self.peek();
        };
        self.previous()
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.current).unwrap()
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::Eof
    }

    fn previous(&self) -> &Token {
        self.tokens.get(self.current - 1).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::Scanner;

    fn parse_str(source: &str) -> String {
        let mut reporter = ErrorReporter::new();
        let tokens = Scanner::new(source).scan_tokens(&mut reporter);
        assert!(
            !reporter.had_error(),
            "unexpected scanner error on `{source}`"
        );
        let mut parser = Parser::new(tokens);
        let expr = parser.parse(&mut reporter).unwrap();
        assert!(
            !reporter.had_error(),
            "unexpected parser error on `{source}`"
        );
        format!("{expr}")
    }

    fn parse_err(source: &str) {
        let mut reporter = ErrorReporter::new();
        let tokens = Scanner::new(source).scan_tokens(&mut reporter);
        assert!(
            !reporter.had_error(),
            "unexpected scanner error on `{source}`"
        );
        let mut parser = Parser::new(tokens);
        let expr = parser.parse(&mut reporter);
        assert!(
            expr.is_none(),
            "expected None for invalid input `{source}`, got `{expr:?}`"
        );
        assert!(
            reporter.had_error(),
            "expected parser error to be reported on `{source}`"
        );
    }

    #[test]
    fn comma_associativity_and_precedence() {
        assert_eq!(parse_str("1, 2, 3"), "(, (, 1 2) 3)");
        assert_eq!(parse_str("1, 2, 3, 4"), "(, (, (, 1 2) 3) 4)");
        assert_eq!(
            parse_str("1 == 2, 3 < 4 != 5 > 6"),
            "(, (== 1 2) (!= (< 3 4) (> 5 6)))"
        );
    }

    #[test]
    fn ternary_basic_and_arithmetic() {
        assert_eq!(parse_str("1 ? 2 : 3"), "(? 1 2 3)");
        assert_eq!(
            parse_str("1 + 2 == 3 ? 4 * 5 : 6 - 7"),
            "(? (== (+ 1 2) 3) (* 4 5) (- 6 7))"
        );
    }

    #[test]
    fn ternary_associativity_and_nesting() {
        assert_eq!(parse_str("1 ? 2 : 3 ? 4 : 5"), "(? 1 2 (? 3 4 5))");
        assert_eq!(
            parse_str("1 ? 2 : 3 ? 4 : 5 ? 6 : 7"),
            "(? 1 2 (? 3 4 (? 5 6 7)))"
        );
        assert_eq!(parse_str("1 ? 2 ? 3 : 4 : 5"), "(? 1 (? 2 3 4) 5)");
        assert_eq!(
            parse_str("1 ? 2 ? 3 : 4 : 5 ? 6 : 7"),
            "(? 1 (? 2 3 4) (? 5 6 7))"
        );
    }

    #[test]
    fn ternary_and_comma_interactions() {
        assert_eq!(parse_str("1 ? 2, 3 : 4"), "(? 1 (, 2 3) 4)");
        assert_eq!(parse_str("1 ? 2, 3, 4 : 5"), "(? 1 (, (, 2 3) 4) 5)");
        assert_eq!(parse_str("1 ? 2 : 3, 4"), "(, (? 1 2 3) 4)");
        assert_eq!(parse_str("1, 2 ? 3 : 4"), "(, 1 (? 2 3 4))");
        assert_eq!(
            parse_str("1, 2 ? 3, 4 : 5, 6"),
            "(, (, 1 (? 2 (, 3 4) 5)) 6)"
        );
    }

    #[test]
    fn literals() {
        assert_eq!(parse_str("42"), "42");
        assert_eq!(parse_str("\"hi\""), "hi");
        assert_eq!(parse_str("true"), "true");
        assert_eq!(parse_str("false"), "false");
        assert_eq!(parse_str("nil"), "nil");
    }

    #[test]
    fn operator_precedence() {
        assert_eq!(parse_str("1 + 2 * 3"), "(+ 1 (* 2 3))");
        assert_eq!(parse_str("1 + 2 * 3 - 4 / 2"), "(- (+ 1 (* 2 3)) (/ 4 2))");
        assert_eq!(
            parse_str("1 + 2 < 3 * 4 == true"),
            "(== (< (+ 1 2) (* 3 4)) true)"
        );
    }

    #[test]
    fn left_associativity() {
        assert_eq!(parse_str("1 - 2 - 3"), "(- (- 1 2) 3)");
        assert_eq!(parse_str("12 / 3 / 2"), "(/ (/ 12 3) 2)");
        assert_eq!(parse_str("1 == 2 != false"), "(!= (== 1 2) false)");
    }

    #[test]
    fn grouping_overrides_precedence() {
        assert_eq!(parse_str("(1 + 2) * 3"), "(* (group (+ 1 2)) 3)");
        assert_eq!(parse_str("-(1 + 2)"), "(- (group (+ 1 2)))");
    }

    #[test]
    fn chained_unary() {
        assert_eq!(parse_str("--1"), "(- (- 1))");
        assert_eq!(parse_str("!!true"), "(! (! true))");
        assert_eq!(parse_str("!-1"), "(! (- 1))");
        assert_eq!(parse_str("-!false"), "(- (! false))");
    }

    #[test]
    fn unary_minus_binds_tighter_than_factor() {
        assert_eq!(parse_str("-1 * 2"), "(* (- 1) 2)");
        assert_eq!(parse_str("-123 * (45.67)"), "(* (- 123) (group 45.67))");
    }

    #[test]
    fn complex_mixed_precedence() {
        assert_eq!(
            parse_str("!(1 + 2 * 3 == 7) != (-4 / 2 < 0)"),
            "(!= (! (group (== (+ 1 (* 2 3)) 7))) (group (< (/ (- 4) 2) 0)))"
        );
    }

    #[test]
    fn syntax_errors() {
        parse_err("");
        parse_err("1 +");
        parse_err("-");
        parse_err("!");
        parse_err("+ 1");
        parse_err("* 2");
        parse_err(")");
        parse_err("()");
        parse_err("(1 + 2");
        parse_err("(1 + 2 3)");
        parse_err("((((1 + ))))");
        parse_err("1 ! 2");
        parse_err("1 + 2 )");
        parse_err("1 2");
    }

    #[test]
    fn comma_and_ternary_syntax_errors() {
        parse_err("1,");
        parse_err(", 1");
        parse_err("1,, 2");
        parse_err("1 ? 2");
        parse_err("1 ? 2 :");
        parse_err("1 ? : 3");
        parse_err("? 2 : 3");
        parse_err("1 : 2");
        parse_err("1 ? 2 : 3 : 4");
        parse_err("1 ? 2 ? 3 : 4");
    }
}
