use std::fmt;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token {
    // Identifiers, literals
    Identifier(String),
    Int(String),
    String(String),

    // Keywords (reserved)
    Let,
    Function,
    If,
    Else,
    Return,
    True,
    False,

    // Operators (single char)
    Assign,
    Plus,
    Minus,
    Asterisk,
    Slash,
    Bang,
    Lt,
    Gt,

    // Delimiters (single char)
    Semicolon,
    Comma,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,

    // Operators (double char)
    Eq,
    NotEq,
    Lte,
    Gte,

    // Errors (emitted to the parser)
    InvalidChar(char),
    InvalidIdentifier(String),
    InvalidEscape(char),
    UnterminatedString,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Token::*;
        let out: &str = match self {
            Identifier(s) => s,
            Int(n) => n,
            String(s) => s,
            Let => "let",
            Function => "fn",
            If => "if",
            Else => "else",
            Return => "return",
            True => "true",
            False => "false",
            Assign => "=",
            Plus => "+",
            Minus => "-",
            Asterisk => "*",
            Slash => "/",
            Bang => "!",
            Lt => "<",
            Gt => ">",
            Semicolon => ";",
            Comma => ",",
            LeftParen => "(",
            RightParen => ")",
            LeftBrace => "{",
            RightBrace => "}",
            Eq => "==",
            Lte => "<=",
            Gte => ">=",
            NotEq => "!=",
            _ => "NA",
        };
        write!(f, "{}", out)?;
        Ok(())
    }
}
