#![allow(dead_code)]
#![allow(unused_variables)]

mod token;

use token::Token;

pub struct Lexer {}

impl Lexer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn parse(&self, input: &str) -> Vec<Token> {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::token::KeywordItem::*;
    use crate::lexer::token::Token::*;

    #[test]
    fn test_input() {
        let input = "let five = 5;
let ten = 10;

let add = fn(x, y) {
x + y;
};

let result = add(five, ten);";

        let expected = vec![
            Keyword(Let),
            Identifier("five".into()),
            Assign,
            Int(5),
            Semicolon,
            Keyword(Let),
            Identifier("ten".into()),
            Semicolon,
            Keyword(Let),
            Identifier("add".into()),
            Assign,
            Keyword(Function),
            LeftParen,
            Identifier("x".into()),
            Comma,
            Identifier("y".into()),
            RightParen,
            LeftBrace,
            Identifier("x".into()),
            Plus,
            Identifier("y".into()),
            Semicolon,
            RightBrace,
            Semicolon,
            Keyword(Let),
            Identifier("result".into()),
            Assign,
            Identifier("add".into()),
            LeftParen,
            Identifier("five".into()),
            Comma,
            Identifier("ten".into()),
            RightParen,
            Semicolon,
            Eof,
        ];

        let lexer = Lexer::new();
        assert_eq!(expected, lexer.parse(input));
    }
}
