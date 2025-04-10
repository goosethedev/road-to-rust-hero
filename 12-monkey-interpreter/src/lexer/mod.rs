#![allow(dead_code)]
#![allow(unused_variables)]

mod token;

use std::{iter::Peekable, str::Chars};

use token::{KeywordItem, Token};

pub struct Lexer<'a> {
    input: &'a str,
    chars: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        let chars = input.chars().peekable();
        Self { input, chars }
    }

    pub fn execute(&mut self) -> Vec<Token> {
        let mut output = vec![];

        while let Some(ch) = self.chars.next() {
            if ch.is_whitespace() {
                continue;
            };
            let token = self
                .try_tokenize_double_operator(ch)
                .or_else(|| self.try_tokenize_single_operator(ch))
                .or_else(|| self.try_tokenize_as_illegal(ch))
                .unwrap_or_else(|| self.tokenize_word(ch));
            output.push(token);
        }

        output.push(Token::Eof);
        output
    }

    /// Tokenizes a complete alpha-numeric word
    fn tokenize_word(&mut self, initial: char) -> Token {
        let mut word = vec![initial];

        while let Some(ch) = self.chars.peek() {
            if ch.is_alphanumeric() {
                word.push(self.chars.next().unwrap());
            } else {
                break;
            };
        }
        let word: String = word.iter().collect::<String>();

        if let Ok(num) = word.parse::<i64>() {
            return Token::Int(num);
        }

        match word.as_str() {
            "let" => Token::Keyword(KeywordItem::Let),
            "fn" => Token::Keyword(KeywordItem::Function),
            "if" => Token::Keyword(KeywordItem::If),
            "else" => Token::Keyword(KeywordItem::Else),
            "return" => Token::Keyword(KeywordItem::Return),
            "true" => Token::Keyword(KeywordItem::True),
            "false" => Token::Keyword(KeywordItem::False),
            _ => Token::Identifier(word),
        }
    }

    /// Checks if is not a supported character.
    fn try_tokenize_as_illegal(&self, ch: char) -> Option<Token> {
        if ch.is_alphanumeric() || ch == '_' {
            None
        } else {
            Some(Token::Illegal(ch))
        }
    }

    /// Tries to tokenize known one-character operators.
    /// It should be used after `try_tokenize_double_operator` to avoid misinterpreting a two-character operator.
    fn try_tokenize_single_operator(&mut self, ch: char) -> Option<Token> {
        let token = match ch {
            '=' => Token::Assign,
            '+' => Token::Plus,
            '-' => Token::Minus,
            '*' => Token::Asterisk,
            '/' => Token::Slash,
            '!' => Token::Bang,
            '<' => Token::Lt,
            '>' => Token::Gt,
            ';' => Token::Semicolon,
            ',' => Token::Comma,
            '(' => Token::LeftParen,
            ')' => Token::RightParen,
            '{' => Token::LeftBrace,
            '}' => Token::RightBrace,
            _ => return None,
        };
        Some(token)
    }

    /// Tries to tokenize known two-character operators.
    fn try_tokenize_double_operator(&mut self, ch: char) -> Option<Token> {
        let next = self.chars.peek()?;
        let token = match (ch, *next) {
            ('=', '=') => Token::Eq,
            ('!', '=') => Token::NotEq,
            ('>', '=') => Token::Gte,
            ('<', '=') => Token::Lte,
            _ => return None,
        };
        self.chars.next(); // Advance the pointer
        Some(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::token::KeywordItem::*;
    use crate::lexer::token::Token::*;

    #[test]
    fn test_input_sum_statement() {
        let input = "let eleven = 4 + 7;";
        let expected = vec![
            Keyword(Let),
            Identifier("eleven".into()),
            Assign,
            Int(4),
            Plus,
            Int(7),
            Semicolon,
            Eof,
        ];

        let mut lexer = Lexer::new(input);
        assert_eq!(expected, lexer.execute())
    }

    #[test]
    fn test_input_math_statement() {
        let input = "let result = (7+8-3) / (2*3);";
        let expected = vec![
            Keyword(Let),
            Identifier("result".into()),
            Assign,
            LeftParen,
            Int(7),
            Plus,
            Int(8),
            Minus,
            Int(3),
            RightParen,
            Slash,
            LeftParen,
            Int(2),
            Asterisk,
            Int(3),
            RightParen,
            Semicolon,
            Eof,
        ];

        let mut lexer = Lexer::new(input);
        assert_eq!(expected, lexer.execute())
    }

    #[test]
    fn test_input_complete() {
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
            Assign,
            Int(10),
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

        let mut lexer = Lexer::new(input);
        assert_eq!(expected, lexer.execute());
    }

    #[test]
    fn test_input_illegal() {
        let input = "[].@";
        let expected = vec![Illegal('['), Illegal(']'), Illegal('.'), Illegal('@'), Eof];
        let mut lexer = Lexer::new(input);
        assert_eq!(expected, lexer.execute());
    }

    #[test]
    fn test_input_conditional_bool() {
        let input = "if (5 < 10) {
    return true;
} else {
    return false;
}";
        let expected = vec![
            Keyword(If),
            LeftParen,
            Int(5),
            Lt,
            Int(10),
            RightParen,
            LeftBrace,
            Keyword(Return),
            Keyword(True),
            Semicolon,
            RightBrace,
            Keyword(Else),
            LeftBrace,
            Keyword(Return),
            Keyword(False),
            Semicolon,
            RightBrace,
            Eof,
        ];
        let mut lexer = Lexer::new(input);
        assert_eq!(expected, lexer.execute());
    }

    #[test]
    fn test_input_comparison() {
        let input = "10 == 10; 9 != 10;
5 > 3; 7 >= 5; 2 <= 4; 1 < 9;";
        let expected = vec![
            Int(10),
            Eq,
            Int(10),
            Semicolon,
            Int(9),
            NotEq,
            Int(10),
            Semicolon,
            Int(5),
            Gt,
            Int(3),
            Semicolon,
            Int(7),
            Gte,
            Int(5),
            Semicolon,
            Int(2),
            Lte,
            Int(4),
            Semicolon,
            Int(1),
            Lt,
            Int(9),
            Semicolon,
            Eof,
        ];
        let mut lexer = Lexer::new(input);
        assert_eq!(expected, lexer.execute());
    }

    #[test]
    fn test_input_random_characters() {
        let input = "!=/*3-";
        let expected = vec![NotEq, Slash, Asterisk, Int(3), Minus, Eof];
        let mut lexer = Lexer::new(input);
        assert_eq!(expected, lexer.execute());
    }
}
