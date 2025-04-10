#![allow(dead_code)]

use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub enum Token {
    // Control items
    Illegal(char),
    Eof,

    // Identifiers, literals, keywords
    Identifier(String),
    Int(i64),
    Keyword(KeywordItem),

    // Operators
    Assign,
    Plus,

    // Delimiters
    Semicolon,
    Comma,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
}

#[derive(Debug, PartialEq, Eq)]
pub enum KeywordItem {
    Let,
    Function,
}

impl fmt::Display for KeywordItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let out = match self {
            KeywordItem::Let => "let",
            KeywordItem::Function => "fn",
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
            Semicolon => ";",
            Comma => ",",
            LeftParen => "(",
            RightParen => ")",
            LeftBrace => "{",
            RightBrace => "}",
        };
        write!(f, "{}", out)?;
        Ok(())
    }
}
