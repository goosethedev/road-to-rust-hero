#![allow(dead_code)]

use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub enum Token {
    // Control items
    Illegal(char),
    Eof,

    // Identifiers, literals, keywords (words)
    Identifier(String),
    Int(i64),
    Keyword(KeywordItem),

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

#[derive(Debug, PartialEq, Eq)]
pub enum KeywordItem {
    Let,
    Function,
    If,
    Else,
    Return,
    True,
    False,
}

impl fmt::Display for KeywordItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use KeywordItem::*;

        let out = match self {
            Let => "let",
            Function => "fn",
            If => "if",
            Else => "else",
            Return => "return",
            True => "true",
            False => "false",
        };
        write!(f, "{}", out)?;
        Ok(())
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Token::*;
        let out: &str = match self {
            Illegal(c) => &c.to_string(),
            Eof => "EOF",
            Identifier(s) => s,
            Int(n) => &n.to_string(),
            Keyword(kw) => &kw.to_string(),
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
