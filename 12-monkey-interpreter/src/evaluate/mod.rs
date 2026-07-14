#![allow(dead_code)]

use std::collections::HashMap;
use std::fmt;

use crate::{
    evaluate::Object::Returned,
    parsing::{Expr, InfixOp, PrefixOp, Statement},
};

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum EvalError {
    #[error("identifier not found: {0}")]
    UnknownIdentifier(String),
    #[error("can't apply '{0}' on {1}")]
    InvalidPrefixOp(PrefixOp, Object),
    #[error("can't apply '{0}' on {1} and {2}")]
    InvalidInfixOp(InfixOp, Object, Object),
}

// TODO: Separate Returned variant into its own type
// to avoid returning it from the public facing eval fn
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Object {
    Null,
    Int(i64),
    Bool(bool),
    Returned(Box<Object>),
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Object::*;
        let out = match self {
            Null => "null",
            Int(n) => &n.to_string(),
            Bool(b) => &b.to_string(),
            Returned(v) => &v.to_string(),
        };
        write!(f, "{}", out)?;
        Ok(())
    }
}

type Ctx = HashMap<String, Object>;

// pub struct Scope {
//     context: Ctx,
// }

// impl Scope {
//     pub fn new() -> Self {
//         let context = HashMap::new();
//         Self { context }
//     }
// }

const NULL: Object = Object::Null;
const TRUE: Object = Object::Bool(true);
const FALSE: Object = Object::Bool(false);

pub fn eval(
    statements: impl IntoIterator<Item = Statement>,
    ctx: &mut Ctx,
) -> Result<Object, EvalError> {
    let mut last_expr = NULL;
    for result in statements {
        match result {
            Statement::Let { iden, expr } => {
                let expr = eval_expr(expr, ctx)?;
                let _ = ctx.insert(iden, expr);
            }
            Statement::Expression { expr } => {
                last_expr = eval_expr(expr, ctx)?;
            }
            Statement::Return { expr } => return Ok(unwrap_returned(eval_expr(expr, ctx)?)),
        }
    }

    Ok(unwrap_returned(last_expr))
}

fn eval_block(
    statements: impl IntoIterator<Item = Statement>,
    ctx: &mut Ctx,
) -> Result<Object, EvalError> {
    let mut last_expr = NULL;
    for result in statements {
        match result {
            Statement::Let { iden, expr } => {
                let expr = unwrap_returned(eval_expr(expr, ctx)?);
                let _ = ctx.insert(iden, expr);
            }
            Statement::Expression { expr } => {
                let expr = eval_expr(expr, ctx)?;
                match expr {
                    Returned(_) => return Ok(expr),
                    _ => last_expr = expr,
                };
            }
            Statement::Return { expr } => {
                return Ok(Object::Returned(eval_expr(expr, ctx)?.into()));
            }
        }
    }

    Ok(last_expr)
}

fn eval_expr(expr: Expr, ctx: &mut Ctx) -> Result<Object, EvalError> {
    let obj = match expr {
        Expr::Int(n) => Object::Int(n),
        Expr::Bool(b) => bool_obj(b),
        Expr::Identifier(k) => match ctx.get(&k) {
            Some(v) => v.to_owned(),
            None => return Err(EvalError::UnknownIdentifier(k)),
        },
        Expr::Prefix { op, expr } => {
            let obj = eval_expr(*expr, ctx)?;
            match op {
                PrefixOp::Not => match obj {
                    Object::Null => FALSE,
                    Object::Bool(b) if b => FALSE,
                    Object::Int(n) if n != 0 => FALSE,
                    _ => TRUE,
                },
                PrefixOp::Negative => match obj {
                    Object::Int(n) => Object::Int(-n),
                    _ => return Err(EvalError::InvalidPrefixOp(PrefixOp::Negative, obj)),
                },
            }
        }
        Expr::Infix { op, lh, rh } => {
            let lh = eval_expr(*lh, ctx)?;
            let rh = eval_expr(*rh, ctx)?;
            match op {
                InfixOp::Eq => bool_obj(lh == rh),
                InfixOp::NotEq => bool_obj(lh != rh),
                InfixOp::Add => match (&lh, &rh) {
                    (&Object::Int(a), &Object::Int(b)) => Object::Int(a + b),
                    _ => return Err(EvalError::InvalidInfixOp(InfixOp::Add, lh, rh)),
                },
                InfixOp::Sub => match (&lh, &rh) {
                    (&Object::Int(a), &Object::Int(b)) => Object::Int(a - b),
                    _ => return Err(EvalError::InvalidInfixOp(InfixOp::Sub, lh, rh)),
                },
                InfixOp::Mult => match (&lh, &rh) {
                    (&Object::Int(a), &Object::Int(b)) => Object::Int(a * b),
                    _ => return Err(EvalError::InvalidInfixOp(InfixOp::Mult, lh, rh)),
                },
                InfixOp::Div => match (&lh, &rh) {
                    (&Object::Int(a), &Object::Int(b)) => Object::Int(a / b),
                    _ => return Err(EvalError::InvalidInfixOp(InfixOp::Div, lh, rh)),
                },
                // TODO: maybe implement with PartialOrd?
                InfixOp::Gt => match (&lh, &rh) {
                    (&Object::Int(a), &Object::Int(b)) => bool_obj(a > b),
                    _ => return Err(EvalError::InvalidInfixOp(InfixOp::Gt, lh, rh)),
                },
                InfixOp::Lt => match (&lh, &rh) {
                    (&Object::Int(a), &Object::Int(b)) => bool_obj(a < b),
                    _ => return Err(EvalError::InvalidInfixOp(InfixOp::Lt, lh, rh)),
                },
                InfixOp::Gte => match (&lh, &rh) {
                    (&Object::Int(a), &Object::Int(b)) => bool_obj(a >= b),
                    _ => return Err(EvalError::InvalidInfixOp(InfixOp::Gte, lh, rh)),
                },
                InfixOp::Lte => match (&lh, &rh) {
                    (&Object::Int(a), &Object::Int(b)) => bool_obj(a <= b),
                    _ => return Err(EvalError::InvalidInfixOp(InfixOp::Lte, lh, rh)),
                },
            }
        }
        Expr::IfCondition { condition, then_block, else_block } => {
            let condition = eval_expr(*condition, ctx)?;
            if condition == FALSE || condition == NULL {
                match else_block {
                    Some(block) => eval_block(block, ctx)?,
                    None => NULL,
                }
            } else {
                eval_block(then_block, ctx)?
            }
        }
        // Expr::FnExpr { params, body } => todo!(),
        // Expr::FnCall { callable, args } => todo!(),
        _ => todo!(),
    };
    Ok(obj)
}

#[inline]
fn bool_obj(condition: bool) -> Object {
    if condition { TRUE } else { FALSE }
}

#[inline]
fn unwrap_returned(mut obj: Object) -> Object {
    while let Object::Returned(v) = obj {
        obj = *v;
    }
    obj
}

#[cfg(test)]
mod tests {
    use super::Object::*;
    use super::*;
    use crate::lexing::Lexer;
    use crate::parsing::Parser;

    fn test_eval(input: &str, expected: Result<Object, EvalError>) {
        let mut scope = HashMap::new();
        let lexer = Lexer::new(&input);
        let ast = Parser::new(lexer).parse().expect("parsing should not fail");
        let result = eval(ast, &mut scope);
        assert_eq!(result, expected, "for input: {}", input);
    }

    #[test]
    fn eval_integer_expr() {
        let pairs = [
            ("5", Int(5)),
            ("10", Int(10)),
            ("-5", Int(-5)),
            ("-10", Int(-10)),
            ("5 + 5 + 5 + 5 - 10", Int(10)),
            ("2 * 2 * 2 * 2 * 2", Int(32)),
            ("-50 + 100 + -50", Int(0)),
            ("5 * 2 + 10", Int(20)),
            ("5 + 2 * 10", Int(25)),
            ("20 + 2 * -10", Int(0)),
            ("50 / 2 * 2 + 10", Int(60)),
            ("2 * (5 + 10)", Int(30)),
            ("3 * 3 * 3 + 10", Int(37)),
            ("3 * (3 * 3) + 10", Int(37)),
            ("(5 + 10 * 2 + 15 / 3) * 2 + -10", Int(50)),
        ];
        for (input, expected) in pairs {
            test_eval(input, Ok(expected));
        }
    }

    #[test]
    fn eval_boolean_expr() {
        let pairs = [
            ("true", TRUE),
            ("false", FALSE),
            ("1 < 2", TRUE),
            ("1 > 2", FALSE),
            ("1 < 1", FALSE),
            ("1 > 1", FALSE),
            ("1 == 1", TRUE),
            ("1 != 1", FALSE),
            ("1 == 2", FALSE),
            ("1 != 2", TRUE),
            ("true == true", TRUE),
            ("false == false", TRUE),
            ("true == false", FALSE),
            ("true != false", TRUE),
            ("false != true", TRUE),
            ("(1 < 2) == true", TRUE),
            ("(1 < 2) == false", FALSE),
            ("(1 > 2) == true", FALSE),
            ("(1 > 2) == false", TRUE),
        ];
        for (input, expected) in pairs {
            test_eval(input, Ok(expected));
        }
    }

    #[test]
    fn eval_bang_expr() {
        let pairs = [
            ("!true", FALSE),
            ("!false", TRUE),
            ("!5", FALSE),
            ("!!true", TRUE),
            ("!!false", FALSE),
            ("!!5", TRUE),
        ];
        for (input, expected) in pairs {
            test_eval(input, Ok(expected));
        }
    }

    #[test]
    fn eval_if_expr() {
        let pairs = [
            ("if (true) { 10 }", Int(10)),
            ("if (false) { 10 }", NULL),
            ("if (1) { 10 }", Int(10)),
            ("if (1 < 2) { 10 }", Int(10)),
            ("if (1 > 2) { 10 }", NULL),
            ("if (1 > 2) { 10 } else { 20 }", Int(20)),
            ("if (1 < 2) { 10 } else { 20 }", Int(10)),
        ];
        for (input, expected) in pairs {
            test_eval(input, Ok(expected));
        }
    }

    #[test]
    fn eval_return_stmts() {
        let pairs = [
            ("return 10;", Int(10)),
            ("return 10; 9;", Int(10)),
            ("return 2 * 5; 9;", Int(10)),
            ("9; return 2 * 5; 9;", Int(10)),
            ("if (10 > 1) { return 10; 40; }", Int(10)),
            (
                "
          if (10 > 1) {
            if (10 > 1) {
              return 10;
            }
            return 1;
          }",
                Int(10),
            ),
            //   (
            //       "
            // let f = fn(x) {
            //   return x;
            //   x + 10;
            // };
            // f(10);",
            //       Int(10),
            //   ),
            //   (
            //       "
            // let f = fn(x) {
            //    let result = x + 10;
            //    return result;
            //    return 10;
            // };
            // f(10);",
            //       Int(10),
            //   ),
        ];
        for (input, expected) in pairs {
            test_eval(input, Ok(expected));
        }
    }

    #[test]
    fn eval_errors() {
        let pairs = [
            ("5 + true;", EvalError::InvalidInfixOp(InfixOp::Add, Object::Int(5), TRUE)),
            ("5 + true; 5;", EvalError::InvalidInfixOp(InfixOp::Add, Object::Int(5), TRUE)),
            ("-true", EvalError::InvalidPrefixOp(PrefixOp::Negative, TRUE)),
            ("true + false;", EvalError::InvalidInfixOp(InfixOp::Add, TRUE, FALSE)),
            ("true + false + true + false;", EvalError::InvalidInfixOp(InfixOp::Add, TRUE, FALSE)),
            ("5; true - false; 5", EvalError::InvalidInfixOp(InfixOp::Sub, TRUE, FALSE)),
            ("if (10 > 1) { true / false; }", EvalError::InvalidInfixOp(InfixOp::Div, TRUE, FALSE)),
            (
                "
            if (10 > 1) {
              if (10 > 1) {
                return true + false;
              }

              return 1;
            }
            ",
                EvalError::InvalidInfixOp(InfixOp::Add, TRUE, FALSE),
            ),
            ("foobar", EvalError::UnknownIdentifier("foobar".to_string())),
        ];

        for (input, expected) in pairs {
            test_eval(input, Err(expected));
        }
    }
}
