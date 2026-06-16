use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub enum Token {
    // Control items
    Illegal(String),

    // Identifiers, literals, keywords
    Identifier(String),
    Int(String),
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
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Token::*;
        let out: &str = match self {
            Illegal(s) => s,
            Identifier(s) => s,
            Int(n) => n,
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
        };
        write!(f, "{}", out)?;
        Ok(())
    }
}
