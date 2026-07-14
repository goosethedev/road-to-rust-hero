use std::fmt;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::parsing::{Block, Expr, InfixOp, PrefixOp, Statement};

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum EvalError {
    #[error("identifier not found: {0}")]
    UnknownIdentifier(String),
    #[error("can't apply '{0}' on {1}")]
    InvalidPrefixOp(PrefixOp, Object),
    #[error("can't apply '{0}' on {1} and {2}")]
    InvalidInfixOp(InfixOp, Object, Object),
    #[error("incorrect number of arguments: expected {0}, got {1}")]
    ParamSizeMismatch(usize, usize),
    #[error("expression is not callable: {0}")]
    ExprNotCallable(Expr),
}

// TODO: Separate Returned variant into its own type
// to avoid returning it from the public facing eval fn
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Object {
    Null,
    Int(i64),
    Bool(bool),
    FnExpr(Box<FnObject>),
    Returned(Box<Object>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FnObject {
    params: Vec<String>,
    body: Block,
    env: Environment,
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Object::*;
        let out = match self {
            Null => "null",
            Int(n) => &n.to_string(),
            Bool(b) => &b.to_string(),
            FnExpr(e) => &e.to_string(),
            Returned(v) => &v.to_string(),
        };
        write!(f, "{}", out)?;
        Ok(())
    }
}

impl fmt::Display for FnObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let params = self.params.join(", ");
        let body = self.body.to_string();
        let out = format!("fn ({params}) {{{body}}}");
        write!(f, "{}", out)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Environment(Rc<InnerEnv>);

// TODO: Wrap Object in Rc for better mem performance maybe?
#[derive(Debug, Clone, PartialEq, Eq)]
struct InnerEnv {
    outer: Option<Environment>,
    local: RefCell<HashMap<String, Object>>,
}

impl Environment {
    pub fn new() -> Self {
        let env = InnerEnv { outer: None, local: RefCell::new(HashMap::new()) };
        Self(Rc::new(env))
    }

    pub fn with(outer: Environment, local: HashMap<String, Object>) -> Self {
        let env = InnerEnv { outer: Some(outer.clone()), local: RefCell::new(local) };
        Self(Rc::new(env))
    }

    pub fn get(&self, key: &str) -> Option<Object> {
        let env = self.0.clone();
        env.local.borrow().get(key).cloned().or(env.outer.clone().and_then(|outer| outer.get(key)))
    }

    pub fn set(&self, key: String, value: Object) -> Option<Object> {
        self.0.clone().local.borrow_mut().insert(key, value)
    }
}

const NULL: Object = Object::Null;
const TRUE: Object = Object::Bool(true);
const FALSE: Object = Object::Bool(false);

pub fn eval(
    statements: impl IntoIterator<Item = Statement>,
    env: Environment,
) -> Result<Object, EvalError> {
    let mut last_expr = NULL;
    for result in statements {
        match result {
            Statement::Let { iden, expr } => {
                let expr = eval_expr(expr, env.clone())?;
                env.set(iden, expr);
            }
            Statement::Expression { expr } => {
                last_expr = eval_expr(expr, env.clone())?;
            }
            Statement::Return { expr } => {
                return Ok(unwrap_returned(eval_expr(expr, env.clone())?));
            }
        }
    }

    Ok(unwrap_returned(last_expr))
}

fn eval_block(
    statements: impl IntoIterator<Item = Statement>,
    env: Environment,
) -> Result<Object, EvalError> {
    let mut last_expr = NULL;
    for result in statements {
        match result {
            Statement::Let { iden, expr } => {
                let expr = unwrap_returned(eval_expr(expr, env.clone())?);
                env.set(iden, expr);
            }
            Statement::Expression { expr } => {
                let expr = eval_expr(expr, env.clone())?;
                match expr {
                    Object::Returned(_) => return Ok(expr),
                    _ => last_expr = expr,
                };
            }
            Statement::Return { expr } => {
                return Ok(Object::Returned(eval_expr(expr, env.clone())?.into()));
            }
        }
    }

    Ok(last_expr)
}

fn eval_expr(expr: Expr, env: Environment) -> Result<Object, EvalError> {
    let obj = match expr {
        Expr::Int(n) => Object::Int(n),
        Expr::Bool(b) => bool_obj(b),
        Expr::Identifier(k) => env.get(&k).ok_or(EvalError::UnknownIdentifier(k))?,
        Expr::Prefix { op, expr } => {
            let obj = eval_expr(*expr, env)?;
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
            let lh = eval_expr(*lh, env.clone())?;
            let rh = eval_expr(*rh, env.clone())?;
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
            let condition = eval_expr(*condition, env.clone())?;
            if condition == FALSE || condition == NULL {
                match else_block {
                    Some(block) => eval_block(block, env.clone())?,
                    None => NULL,
                }
            } else {
                eval_block(then_block, env.clone())?
            }
        }
        Expr::FnExpr { params, body } => Object::FnExpr(FnObject { params, body, env }.into()),
        Expr::FnCall { callable, args } => {
            let Object::FnExpr(fn_expr) = eval_expr(*callable.clone(), env.clone())? else {
                return Err(EvalError::ExprNotCallable(*callable));
            };

            if fn_expr.params.len() != args.len() {
                return Err(EvalError::ParamSizeMismatch(fn_expr.params.len(), args.len()));
            }

            let block_env = Environment::with(fn_expr.env, HashMap::new());
            for (param, arg) in fn_expr.params.into_iter().zip(args) {
                block_env.set(param, eval_expr(arg, env.clone())?);
            }

            unwrap_returned(eval_block(fn_expr.body, block_env)?)
        }
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
        let env = Environment::new();
        let lexer = Lexer::new(&input);
        let ast = Parser::new(lexer).parse().expect("parsing should not fail");
        let result = eval(ast, env);
        assert_eq!(expected, result, "for input: {}", input);
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
    fn eval_let_stmts() {
        let pairs = [
            ("let a = 5; a;", 5),
            ("let a = 5 * 5; a;", 25),
            ("let a = 5; let b = a; b;", 5),
            ("let a = 5; let b = a; let c = a + b + 5; c;", 15),
        ];
        for (input, expected) in pairs {
            test_eval(input, Ok(Int(expected)));
        }
    }

    #[test]
    fn eval_fn_expr() {
        let pairs = [
            ("let identity = fn(x) { x; }; identity(5);", 5),
            ("let identity = fn(x) { return x; }; identity(5);", 5),
            ("let double = fn(x) { x * 2; }; double(5);", 10),
            ("let add = fn(x, y) { x + y; }; add(5, 5);", 10),
            ("let add = fn(x, y) { x + y; }; add(5 + 5, add(5, 5));", 20),
            ("fn(x) { x; }(5)", 5),
        ];
        for (input, expected) in pairs {
            test_eval(input, Ok(Int(expected)));
        }
    }

    #[test]
    fn eval_enclosing_env() {
        let input = "
let first = 10;
let second = 10;
let third = 10;

let ourFunction = fn(first) {
    let second = 20;

    first + second + third;
};

ourFunction(20) + first + second;";
        test_eval(input, Ok(Int(70)));
    }

    #[test]
    fn eval_closure() {
        let input = "
            let newAdder = fn(x) {
              fn(y) { x + y };
            };

            let addTwo = newAdder(2);
            addTwo(2);";
        test_eval(input, Ok(Int(4)));
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
