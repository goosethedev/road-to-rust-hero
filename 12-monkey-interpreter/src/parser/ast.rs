#![allow(dead_code)]

use crate::lexer::{Lexer, Token};
use crate::parser::{
    Block, Expr, InfixOp, Operation, ParserError, Precedence, PrefixOp, Statement,
};

pub struct Ast<'a> {
    tokens: std::iter::Peekable<Lexer<'a>>,
}

impl<'a> Ast<'a> {
    pub fn new(lexer: Lexer<'a>) -> Self {
        let tokens = lexer.into_iter().peekable();
        Self { tokens }
    }
    pub fn parse(&mut self) -> Vec<Result<Statement, ParserError>> {
        let mut statements = vec![];
        while self.tokens.peek().is_some() {
            statements.push(self.parse_statement());
        }
        statements
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
        let Ok(Token::Identifier(iden)) = self.get_token() else {
            return Err(ParserError::MissingIdentifier);
        };
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
            t => return Err(ParserError::UnexpectedToken(t)),
        };

        // Check if an infix expression can be parsed
        while let Some(token) = self.tokens.peek() {
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

    fn consume_expected(&mut self, expected: Token) -> Result<(), ParserError> {
        if *self.peek_token()? == expected {
            self.get_token().unwrap();
            return Ok(());
        }
        Err(ParserError::MissingToken(expected))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{lexer::Lexer, parser::Block};

    #[test]
    fn test_parse_statements() {
        let input = "let x = 5;
let y = true;
x + 4;
return foobar;";
        let lexer = Lexer::new(input);
        let actual = Ast::new(lexer).parse();
        let expected = vec![
            Ok(Statement::Let { iden: "x".to_string(), expr: Expr::Int(5) }),
            Ok(Statement::Let { iden: "y".to_string(), expr: Expr::Bool(true) }),
            Ok(Statement::Expression {
                expr: Expr::Infix {
                    op: InfixOp::Add,
                    lh: Expr::Identifier("x".to_string()).boxed(),
                    rh: Expr::Int(4).boxed(),
                },
            }),
            Ok(Statement::Return { expr: Expr::Identifier("foobar".to_string()) }),
        ];
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_parse_basic_expr() {
        let input = "let y = (foobar + 5) * 2;";
        let lexer = Lexer::new(input);
        let actual = Ast::new(lexer).parse();

        let a = Expr::Infix {
            op: InfixOp::Add,
            lh: Expr::Identifier("foobar".to_string()).boxed(),
            rh: Expr::Int(5).boxed(),
        };
        let b = Expr::Infix { op: InfixOp::Mult, lh: a.boxed(), rh: Expr::Int(2).boxed() };
        let expected = vec![Ok(Statement::Let { iden: "y".to_string(), expr: b })];
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_missing_identifier() {
        let input = "let (x + y)";
        let lexer = Lexer::new(input);
        let actual = Ast::new(lexer).parse();
        let expected = vec![Err(ParserError::MissingIdentifier)];
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_display_statements() {
        let actual: Vec<_> = [
            Statement::Let {
                iden: "x".to_string(),
                expr: Expr::Infix {
                    op: InfixOp::Add,
                    lh: Expr::Int(4).boxed(),
                    rh: Expr::Int(7).boxed(),
                },
            },
            Statement::Return { expr: Expr::Identifier("x".to_string()) },
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
        let ast = Ast::new(lexer).parse();
        let actual: Vec<_> = ast.into_iter().map(|v| v.and_then(|s| Ok(s.to_string()))).collect();
        let expected = vec![Ok("(((4 * foo) / 5) + ((-2) * ((bar / 3) - 1)))".to_string())];

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_parse_if_else_expr() {
        let input = "let max_value = if (x >= y) { x } else { y };";
        let lexer = Lexer::new(input);
        let actual = Ast::new(lexer).parse();

        let expected = vec![Ok(Statement::Let {
            iden: "max_value".to_string(),
            expr: Expr::IfCondition {
                condition: Expr::Infix {
                    op: InfixOp::Gte,
                    lh: Expr::Identifier("x".to_string()).boxed(),
                    rh: Expr::Identifier("y".to_string()).boxed(),
                }
                .boxed(),
                then_block: Block(vec![Statement::Expression {
                    expr: Expr::Identifier("x".to_string()),
                }]),
                else_block: Some(Block(vec![Statement::Expression {
                    expr: Expr::Identifier("y".to_string()),
                }])),
            },
        })];
        assert_eq!(expected, actual);
    }

    //     #[test]
    //     fn test_parse_fn_expr() {
    //         let input = "let add = fn(x, y) {
    //     return x + y;
    // };";
    //         let lexer = Lexer::new(input);
    //         let ast = Ast::new(lexer);

    //         let a = Expr::Sum { lh: Expr::Int(4), rh: Expr::Int(5) };
    //         let b = Expr::Mult { lh: a, rh: Expr::Int(2) };
    //         let expected = vec![Statement::Let { iden: "x", expr: b }];
    //     }
}
