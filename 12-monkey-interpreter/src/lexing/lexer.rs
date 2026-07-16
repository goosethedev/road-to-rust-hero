use super::Token;

pub struct Lexer<'a> {
    chars: std::iter::Peekable<std::str::Chars<'a>>,
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        // Skip all whitespaces first
        while self.chars.next_if(|ch| ch.is_whitespace()).is_some() {}

        // Try to tokenize from next character
        self.chars.next().and_then(|ch| {
            self.try_tokenize_operator(ch)
                .or_else(|| self.try_tokenize_string(ch))
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
        if !is_valid_literal_char(initial) {
            return Token::InvalidChar(initial);
        }

        let mut word = String::from(initial);

        while matches!(self.chars.peek(), Some(ch) if is_valid_literal_char(*ch)) {
            word.push(self.chars.next().unwrap());
        }

        if word.chars().all(|ch| ch.is_ascii_digit()) {
            return Token::Int(word);
        } else if initial.is_ascii_digit() {
            return Token::InvalidIdentifier(word);
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

    fn try_tokenize_string(&mut self, initial: char) -> Option<Token> {
        (initial == '"').then(|| {
            let mut content = String::new();

            loop {
                match self.chars.peek() {
                    Some('"') => {
                        self.chars.next();
                        return Token::String(content);
                    }
                    // TODO: Add other escape sequences
                    Some('\\') => {
                        self.chars.next();
                        let ch = match self.chars.peek().copied() {
                            Some(ch) if ch == '\\' || ch == '"' => ch,
                            Some('n') => '\n',
                            Some('t') => '\t',
                            Some('r') => '\r',
                            Some(ch) => {
                                // On invalid, advance until next quote and consume it
                                while self.chars.next_if(|ch| *ch != '"').is_some() {}
                                self.chars.next_if(|ch| *ch == '"');
                                return Token::InvalidEscape(ch);
                            }
                            None => return Token::InvalidChar('\\'),
                        };
                        self.chars.next();
                        content.push(ch);
                    }
                    Some(_) => content.push(self.chars.next().unwrap()),
                    None => return Token::UnterminatedString,
                }
            }
        })
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

#[inline]
fn is_valid_literal_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexing::token::Token::*;

    fn test_lexing(input: &str, expected: &[Token]) {
        let result: Vec<_> = Lexer::new(input).collect();
        assert_eq!(expected, &result);
    }

    #[test]
    fn tokenize_input_sum_statement() {
        let input = "let six_seven = 6 + 7;";
        let expected = &[
            Let,
            Identifier("six_seven".to_string()),
            Assign,
            Int("6".to_string()),
            Plus,
            Int("7".to_string()),
            Semicolon,
        ];
        test_lexing(input, expected);
    }

    #[test]
    fn tokenize_input_math_statement() {
        let input = "let result = (7+8-3) / (2*3);";
        let expected = &[
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
        test_lexing(input, expected);
    }

    #[test]
    fn tokenize_input_complete() {
        let input = "let five = 5;
let ten = 10;

let add = fn(x, y) {
x + y;
};

let result = add(five, ten);";

        let expected = &[
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
        test_lexing(input, expected);
    }

    #[test]
    fn tokenize_input_invalid_chars() {
        let input = "^\\.@";
        let expected = &[InvalidChar('^'), InvalidChar('\\'), InvalidChar('.'), InvalidChar('@')];
        test_lexing(input, expected);
    }

    #[test]
    fn tokenize_illegal_identifier() {
        let input = "let 23now = 4;";
        let expected =
            &[Let, InvalidIdentifier("23now".to_string()), Assign, Int("4".to_string()), Semicolon];
        test_lexing(input, expected);
    }

    #[test]
    fn tokenize_input_unicode() {
        let input = "298🥀素敵🌠";
        let expected = &[
            Int("298".to_string()),
            InvalidChar('🥀'),
            Identifier("素敵".to_string()),
            InvalidChar('🌠'),
        ];
        test_lexing(input, expected);
    }

    #[test]
    fn tokenize_input_conditional_bool() {
        let input = "if (5 < 10) {
    return true;
} else {
    return false;
}";
        let expected = &[
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
        test_lexing(input, expected);
    }

    #[test]
    fn tokenize_input_comparison() {
        let input = "10 == 10; 9 != 10;
5 > 3; 7 >= 5; 2 <= 4; 1 < 9;";
        let expected = &[
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
        test_lexing(input, expected);
    }

    #[test]
    fn tokenize_input_random_characters() {
        let input = "!=/*3-";
        let expected = &[NotEq, Slash, Asterisk, Int("3".to_string()), Minus];
        test_lexing(input, expected);
    }

    #[test]
    fn tokenize_strings() {
        let input = r#""foobar" == "tú y yo" "八九寺" >　"🥀🌠""#;
        let expected = &[
            String("foobar".to_string()),
            Eq,
            String("tú y yo".to_string()),
            String("八九寺".to_string()),
            Gt,
            String("🥀🌠".to_string()),
        ];
        test_lexing(input, expected);
    }

    #[test]
    fn tokenize_escape_sequences() {
        let input = r#" "this \" here \\" where"that\h" "this \nthere\t"  "\"#;
        let expected = &[
            String("this \" here \\".to_string()),
            Identifier("where".to_string()),
            InvalidEscape('h'),
            String("this \nthere\t".to_string()),
            InvalidChar('\\'),
        ];
        test_lexing(input, expected);
    }

    #[test]
    fn tokenize_unterminated_string() {
        let input = r#"  "here you go   "#;
        let expected = &[UnterminatedString];
        test_lexing(input, expected);
    }
}
