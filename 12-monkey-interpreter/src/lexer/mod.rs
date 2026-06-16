mod token;

use token::Token;

pub struct Lexer<'a> {
    chars: std::iter::Peekable<std::str::Chars<'a>>,
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        // REFACTOR: this doesn't work with Peekable
        // self.chars.skip_while(|ch| ch.is_whitespace());
        while matches!(self.chars.peek(), Some(ch) if ch.is_whitespace()) {
            self.chars.next().unwrap();
        }

        self.chars.next().and_then(|ch| {
            self.try_tokenize_operator(ch)
                .or_else(|| self.try_tokenize_as_illegal(ch))
                .or_else(|| Some(self.tokenize_word(ch)))
        })
    }
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        let chars = input.chars().peekable();
        Self { chars }
    }

    /// Tokenizes a complete valid alphanumeric word
    fn tokenize_word(&mut self, initial: char) -> Token {
        let mut word = String::from(initial);

        while matches!(self.chars.peek(), Some(ch) if ch.is_alphanumeric() || *ch == '_') {
            word.push(self.chars.next().unwrap());
        }

        if word.parse::<i64>().is_ok() {
            return Token::Int(word);
        } else if initial.is_numeric() {
            return Token::Illegal(word);
        }

        match word.as_str() {
            "let" => Token::Let,
            "fn" => Token::Function,
            "if" => Token::If,
            "else" => Token::Else,
            "return" => Token::Return,
            "true" => Token::True,
            "false" => Token::False,
            _ => Token::Identifier(word),
        }
    }

    /// Checks if is not a supported character.
    fn try_tokenize_as_illegal(&self, ch: char) -> Option<Token> {
        if ch.is_alphanumeric() || ch == '_' { None } else { Some(Token::Illegal(ch.to_string())) }
    }

    /// Tries to tokenize an operator.
    fn try_tokenize_operator(&mut self, ch: char) -> Option<Token> {
        self._try_tokenize_double_operator(ch).or(self._try_tokenize_single_operator(ch))
    }

    /// Tries to tokenize known one-character operators.
    /// It should be used after `try_tokenize_double_operator` to avoid misinterpreting a two-character operator.
    fn _try_tokenize_single_operator(&mut self, ch: char) -> Option<Token> {
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
    fn _try_tokenize_double_operator(&mut self, ch: char) -> Option<Token> {
        let next = self.chars.peek()?;
        let token = match (ch, *next) {
            ('=', '=') => Token::Eq,
            ('!', '=') => Token::NotEq,
            ('>', '=') => Token::Gte,
            ('<', '=') => Token::Lte,
            _ => return None,
        };
        self.chars.next();
        Some(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::token::Token::*;

    #[test]
    fn test_input_sum_statement() {
        let input = "let six_seven = 6 + 7;";
        let expected = vec![
            Let,
            Identifier("six_seven".to_string()),
            Assign,
            Int("6".to_string()),
            Plus,
            Int("7".to_string()),
            Semicolon,
        ];
        let actual: Vec<_> = Lexer::new(input).collect();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_input_math_statement() {
        let input = "let result = (7+8-3) / (2*3);";
        let expected = vec![
            Let,
            Identifier("result".into()),
            Assign,
            LeftParen,
            Int("7".to_string()),
            Plus,
            Int("8".to_string()),
            Minus,
            Int("3".to_string()),
            RightParen,
            Slash,
            LeftParen,
            Int("2".to_string()),
            Asterisk,
            Int("3".to_string()),
            RightParen,
            Semicolon,
        ];
        let actual: Vec<_> = Lexer::new(input).collect();
        assert_eq!(actual, expected);
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
            Let,
            Identifier("five".into()),
            Assign,
            Int("5".to_string()),
            Semicolon,
            Let,
            Identifier("ten".into()),
            Assign,
            Int("10".to_string()),
            Semicolon,
            Let,
            Identifier("add".into()),
            Assign,
            Function,
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
            Let,
            Identifier("result".into()),
            Assign,
            Identifier("add".into()),
            LeftParen,
            Identifier("five".into()),
            Comma,
            Identifier("ten".into()),
            RightParen,
            Semicolon,
        ];
        let actual: Vec<_> = Lexer::new(input).collect();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_input_illegal() {
        let input = "[].@";
        let expected = vec![
            Illegal("[".to_string()),
            Illegal("]".to_string()),
            Illegal(".".to_string()),
            Illegal("@".to_string()),
        ];
        let actual: Vec<_> = Lexer::new(input).collect();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_illegal_identifier() {
        let input = "let 23now = 4;";
        let expected =
            vec![Let, Illegal("23now".to_string()), Assign, Int("4".to_string()), Semicolon];
        let actual: Vec<_> = Lexer::new(input).collect();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_input_unicode() {
        let input = "素敵🌠";
        let expected = vec![Identifier("素敵".to_string()), Illegal("🌠".to_string())];
        let actual: Vec<_> = Lexer::new(input).collect();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_input_conditional_bool() {
        let input = "if (5 < 10) {
    return true;
} else {
    return false;
}";
        let expected = vec![
            If,
            LeftParen,
            Int("5".to_string()),
            Lt,
            Int("10".to_string()),
            RightParen,
            LeftBrace,
            Return,
            True,
            Semicolon,
            RightBrace,
            Else,
            LeftBrace,
            Return,
            False,
            Semicolon,
            RightBrace,
        ];
        let actual: Vec<_> = Lexer::new(input).collect();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_input_comparison() {
        let input = "10 == 10; 9 != 10;
5 > 3; 7 >= 5; 2 <= 4; 1 < 9;";
        let expected = vec![
            Int("10".to_string()),
            Eq,
            Int("10".to_string()),
            Semicolon,
            Int("9".to_string()),
            NotEq,
            Int("10".to_string()),
            Semicolon,
            Int("5".to_string()),
            Gt,
            Int("3".to_string()),
            Semicolon,
            Int("7".to_string()),
            Gte,
            Int("5".to_string()),
            Semicolon,
            Int("2".to_string()),
            Lte,
            Int("4".to_string()),
            Semicolon,
            Int("1".to_string()),
            Lt,
            Int("9".to_string()),
            Semicolon,
        ];
        let actual: Vec<_> = Lexer::new(input).collect();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_input_random_characters() {
        let input = "!=/*3-";
        let expected = vec![NotEq, Slash, Asterisk, Int("3".to_string()), Minus];
        let actual: Vec<_> = Lexer::new(input).collect();
        assert_eq!(actual, expected);
    }
}
