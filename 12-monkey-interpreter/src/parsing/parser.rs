use crate::lexing::{Lexer, Token};
use crate::parsing::{
    Block, Expr, InfixOp, Operation, ParserError, Precedence, PrefixOp, Statement,
};

pub struct Parser<'a> {
    tokens: std::iter::Peekable<Lexer<'a>>,
}

impl<'a> Iterator for Parser<'a> {
    type Item = Result<Statement, ParserError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.tokens.peek().is_some().then_some(self.parse_statement())
    }
}

impl<'a> Parser<'a> {
    pub fn new(lexer: Lexer<'a>) -> Self {
        let tokens = lexer.into_iter().peekable();
        Self { tokens }
    }

    fn parse_statement(&mut self) -> Result<Statement, ParserError> {
        let stmt = match *self.peek_token()? {
            Token::Let => self.parse_let_statement(),
            Token::Return => self.parse_return_statement(),
            _ => self.parse_expr_statement(),
        };

        stmt.map_err(|e| {
            // Advance until next semicolon
            while let Ok(token) = self.get_token() {
                if token == Token::Semicolon {
                    break;
                }
            }
            e
        })
    }

    fn parse_let_statement(&mut self) -> Result<Statement, ParserError> {
        // Advance 'let' token
        self.get_token()?;
        // Parse identifier
        let iden = self.consume_identifier()?;
        // Parse expected "="
        self.consume_expected(Token::Assign)?;
        // Parse expression
        let expr = self.parse_expression(Precedence::Lowest)?;
        // Parse expected semicolon
        self.consume_expected(Token::Semicolon)?;

        Ok(Statement::Let { iden, expr })
    }

    fn parse_return_statement(&mut self) -> Result<Statement, ParserError> {
        // Advance 'return' token
        self.get_token()?;
        // Parse expression
        let expr = self.parse_expression(Precedence::Lowest)?;
        // Parse semicolon
        self.consume_expected(Token::Semicolon)?;

        Ok(Statement::Return { expr })
    }

    fn parse_expr_statement(&mut self) -> Result<Statement, ParserError> {
        // Parse expression statement
        let expr = self.parse_expression(Precedence::Lowest)?;
        // Parse (optional) semicolon
        let _ = self.consume_expected(Token::Semicolon);

        Ok(Statement::Expression { expr })
    }

    fn parse_expression(&mut self, precedence: Precedence) -> Result<Expr, ParserError> {
        // Parse single token and prefix expressions
        let mut expr = match self.get_token()? {
            Token::Identifier(s) => Expr::Identifier(s),
            Token::Int(n) => Expr::Int(n.parse()?),
            Token::True => Expr::Bool(true),
            Token::False => Expr::Bool(false),
            Token::Bang => Expr::Prefix {
                op: PrefixOp::Not,
                expr: self.parse_expression(Precedence::Prefix)?.boxed(),
            },
            Token::Minus => Expr::Prefix {
                op: PrefixOp::Negative,
                expr: self.parse_expression(Precedence::Prefix)?.boxed(),
            },
            Token::LeftParen => {
                let expr = self.parse_expression(Precedence::Lowest)?;
                self.consume_expected(Token::RightParen)?;
                expr
            }
            Token::If => {
                self.consume_expected(Token::LeftParen)?;
                let condition = self.parse_expression(Precedence::Lowest)?.boxed();
                self.consume_expected(Token::RightParen)?;
                let then_block = self.parse_block()?;
                let else_block = (self.peek_token()? == &Token::Else).then_some({
                    self.get_token().unwrap();
                    self.parse_block()?
                });
                Expr::IfCondition { condition, then_block, else_block }
            }
            Token::Function => {
                self.consume_expected(Token::LeftParen)?;
                let mut params = vec![];
                while !self.consume_expected(Token::RightParen).is_ok() {
                    params.push(self.consume_identifier()?);
                    if self.consume_expected(Token::RightParen).is_ok() {
                        break;
                    }
                    self.consume_expected(Token::Comma)?;
                }
                let body = self.parse_block()?;
                Expr::FnExpr { params, body }
            }
            t => return Err(ParserError::UnexpectedToken(t)),
        };

        // Check if an infix or postfix expression can be parsed
        while let Some(token) = self.tokens.peek() {
            // First, check if it a function call
            if token == &Token::LeftParen {
                self.get_token().unwrap();
                let mut args = vec![];
                while !self.consume_expected(Token::RightParen).is_ok() {
                    args.push(self.parse_expression(Precedence::Lowest)?);
                    if self.consume_expected(Token::RightParen).is_ok() {
                        break;
                    }
                    self.consume_expected(Token::Comma)?;
                }
                expr = Expr::FnCall { callable: expr.boxed(), args };
                continue;
            }

            // Else, check for a binary infix expression
            let op = match token {
                Token::Plus => InfixOp::Add,
                Token::Minus => InfixOp::Sub,
                Token::Asterisk => InfixOp::Mult,
                Token::Slash => InfixOp::Div,
                Token::Eq => InfixOp::Eq,
                Token::NotEq => InfixOp::NotEq,
                Token::Lt => InfixOp::Lt,
                Token::Gt => InfixOp::Gt,
                Token::Lte => InfixOp::Lte,
                Token::Gte => InfixOp::Gte,
                _ => break,
            };

            if precedence >= op.precedence() {
                break;
            }

            self.get_token().unwrap();
            let rh = self.parse_expression(op.precedence())?.boxed();
            expr = Expr::Infix { op, lh: expr.boxed(), rh };
        }

        Ok(expr)
    }

    fn parse_block(&mut self) -> Result<Block, ParserError> {
        self.consume_expected(Token::LeftBrace)?;
        let mut statements = vec![];
        while self.peek_token()? != &Token::RightBrace {
            statements.push(self.parse_statement()?);
        }
        self.consume_expected(Token::RightBrace)?;
        Ok(Block(statements))
    }

    fn get_token(&mut self) -> Result<Token, ParserError> {
        self.tokens.next().ok_or(ParserError::UnexpectedEof)
    }

    fn peek_token(&mut self) -> Result<&Token, ParserError> {
        self.tokens.peek().ok_or(ParserError::UnexpectedEof)
    }

    fn consume_identifier(&mut self) -> Result<String, ParserError> {
        match self.get_token()? {
            Token::Identifier(iden) => Ok(iden),
            _ => Err(ParserError::MissingIdentifier),
        }
    }

    fn consume_expected(&mut self, expected: Token) -> Result<Token, ParserError> {
        self.tokens.next_if_eq(&expected).ok_or(ParserError::MissingToken(expected))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexing::Lexer;
    use crate::parsing::{Block, Expr::*, InfixOp::*};

    type Ast = Vec<Result<Statement, ParserError>>;

    #[test]
    fn test_parse_statements() {
        let input = "let x = 5;
let y = true;
x + 4;
return foobar;";
        let lexer = Lexer::new(input);
        let actual: Ast = Parser::new(lexer).collect();
        let expected = vec![
            Ok(Statement::Let { iden: "x".to_string(), expr: Int(5) }),
            Ok(Statement::Let { iden: "y".to_string(), expr: Bool(true) }),
            Ok(Statement::Expression {
                expr: Infix {
                    op: Add,
                    lh: Identifier("x".to_string()).boxed(),
                    rh: Int(4).boxed(),
                },
            }),
            Ok(Statement::Return { expr: Identifier("foobar".to_string()) }),
        ];
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_parse_basic_expr() {
        let input = "let y = (foobar + 5) * 2;";
        let lexer = Lexer::new(input);
        let actual: Ast = Parser::new(lexer).collect();

        let a = Infix { op: Add, lh: Identifier("foobar".to_string()).boxed(), rh: Int(5).boxed() };
        let b = Infix { op: Mult, lh: a.boxed(), rh: Int(2).boxed() };
        let expected = vec![Ok(Statement::Let { iden: "y".to_string(), expr: b })];
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_missing_identifier() {
        let input = "let (x + y)";
        let lexer = Lexer::new(input);
        let actual: Ast = Parser::new(lexer).collect();
        let expected = vec![Err(ParserError::MissingIdentifier)];
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_display_statements() {
        let actual: Vec<_> = [
            Statement::Let {
                iden: "x".to_string(),
                expr: Infix { op: Add, lh: Int(4).boxed(), rh: Int(7).boxed() },
            },
            Statement::Return { expr: Identifier("x".to_string()) },
        ]
        .into_iter()
        .map(|v| v.to_string())
        .collect();
        let expected = vec!["let x = (4 + 7);", "return x;"];
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_parse_operator_precedence() {
        let input = "4 * foo / 5 + - 2 * (bar / 3 - 1)";
        let lexer = Lexer::new(input);
        let ast: Ast = Parser::new(lexer).collect();
        let actual: Vec<_> = ast.into_iter().map(|v| v.and_then(|s| Ok(s.to_string()))).collect();
        let expected = vec![Ok("(((4 * foo) / 5) + ((-2) * ((bar / 3) - 1)))".to_string())];

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_parse_if_else_expr() {
        let input = "let max_value = if (x >= y) { x } else { y };";
        let lexer = Lexer::new(input);
        let actual: Ast = Parser::new(lexer).collect();

        let expected = vec![Ok(Statement::Let {
            iden: "max_value".to_string(),
            expr: IfCondition {
                condition: Infix {
                    op: Gte,
                    lh: Identifier("x".to_string()).boxed(),
                    rh: Identifier("y".to_string()).boxed(),
                }
                .boxed(),
                then_block: Block(vec![Statement::Expression {
                    expr: Identifier("x".to_string()),
                }]),
                else_block: Some(Block(vec![Statement::Expression {
                    expr: Identifier("y".to_string()),
                }])),
            },
        })];
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_parse_fn_expr() {
        let input = "let add = fn(x, y) {
        return x + y;
    };";
        let lexer = Lexer::new(input);
        let actual: Ast = Parser::new(lexer).collect();

        let expr = Infix {
            op: Add,
            lh: Identifier("x".to_string()).boxed(),
            rh: Identifier("y".to_string()).boxed(),
        };
        let body = Block(vec![Statement::Return { expr }]);
        let params = vec!["x".to_string(), "y".to_string()];
        let fn_expr = FnExpr { params, body };
        let expected = vec![Ok(Statement::Let { iden: "add".to_string(), expr: fn_expr })];

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_parse_fn_expr_params() {
        let input = "fn() {};
fn(a) {};
fn(x, y, z) {};";
        let lexer = Lexer::new(input);
        let actual: Ast = Parser::new(lexer).collect();

        let expected = vec![
            Ok(Statement::Expression { expr: FnExpr { params: vec![], body: Block(vec![]) } }),
            Ok(Statement::Expression {
                expr: FnExpr { params: vec!["a".to_string()], body: Block(vec![]) },
            }),
            Ok(Statement::Expression {
                expr: FnExpr {
                    params: vec!["x".to_string(), "y".to_string(), "z".to_string()],
                    body: Block(vec![]),
                },
            }),
        ];

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_parse_call_expr() {
        let input = "let sum = add(x, 2);
myfunc(2 / 5, 3 * (y + 4));
(fn(x, y) { x * y })(4, 5);";
        let lexer = Lexer::new(input);
        let actual: Ast = Parser::new(lexer).collect();

        let expr = FnCall {
            callable: Identifier("add".to_string()).boxed(),
            args: vec![Identifier("x".to_string()), Int(2)],
        };
        let fn_call_1 = Statement::Let { iden: "sum".to_string(), expr };

        let callable = Identifier("myfunc".to_string()).boxed();
        let add = Infix { op: Add, lh: Identifier("y".to_string()).boxed(), rh: Int(4).boxed() };
        let mult = Infix { op: Mult, lh: Int(3).boxed(), rh: add.boxed() };
        let args = vec![Infix { op: Div, lh: Int(2).boxed(), rh: Int(5).boxed() }, mult];
        let fn_call_2 = Statement::Expression { expr: FnCall { callable, args } };

        let expr = Infix {
            op: Mult,
            lh: Identifier("x".to_string()).boxed(),
            rh: Identifier("y".to_string()).boxed(),
        };
        let callable = FnExpr {
            params: vec!["x".to_string(), "y".to_string()],
            body: Block(vec![Statement::Expression { expr }]),
        }
        .boxed();
        let args = vec![Int(4), Int(5)];
        let fn_call_3 = Statement::Expression { expr: FnCall { callable, args } };

        let expected = vec![Ok(fn_call_1), Ok(fn_call_2), Ok(fn_call_3)];

        assert_eq!(expected, actual);
    }
}
