use crate::token::Token;
use std::fmt;
use std::fmt::Formatter;

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Binary(Box<Expr>, Token, Box<Expr>),
    Grouping(Box<Expr>),
    Literal(Literal),
    Ternary(Box<Expr>, Token, Box<Expr>, Box<Expr>),
    Unary(Token, Box<Expr>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    Number(f64),
    String(String),
    Bool(bool),
    Nil,
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Literal::Number(val) => write!(f, "{val}"),
            Literal::String(val) => write!(f, "{val}"),
            Literal::Bool(val) => write!(f, "{val}"),
            Literal::Nil => write!(f, "nil"),
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Binary(left, op, right) => write!(f, "({} {left} {right})", op.lexeme),
            Expr::Grouping(expr) => write!(f, "(group {expr})"),
            Expr::Literal(val) => write!(f, "{val}"),
            Expr::Ternary(cond, op, truthy, falsy) => {
                write!(f, "({} {cond} {truthy} {falsy})", op.lexeme)
            }
            Expr::Unary(op, right) => write!(f, "({} {right})", op.lexeme),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::TokenType;

    #[test]
    fn display_literals() {
        assert_eq!(format!("{}", Literal::Number(123.0)), "123");
        assert_eq!(format!("{}", Literal::Number(45.67)), "45.67");
        assert_eq!(format!("{}", Literal::String("hello".into())), "hello");
        assert_eq!(format!("{}", Literal::Bool(true)), "true");
        assert_eq!(format!("{}", Literal::Bool(false)), "false");
        assert_eq!(format!("{}", Literal::Nil), "nil");
    }

    #[test]
    fn display_ast_example() {
        // -123 * (45.67)
        let expr = Expr::Binary(
            Box::new(Expr::Unary(
                Token::new(TokenType::Minus, "-".into(), 1),
                Box::new(Expr::Literal(Literal::Number(123.0))),
            )),
            Token::new(TokenType::Star, "*".into(), 1),
            Box::new(Expr::Grouping(Box::new(Expr::Literal(Literal::Number(
                45.67,
            ))))),
        );

        assert_eq!(format!("{expr}"), "(* (- 123) (group 45.67))");
    }
}
