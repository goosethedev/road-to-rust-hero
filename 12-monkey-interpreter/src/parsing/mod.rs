mod parser;

pub use parser::Parser;

use crate::lexing::Token;
use core::fmt;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ParserError {
    #[error("error parsing integer: {0}")]
    ParseInt(#[from] std::num::ParseIntError),
    // specific for let statements
    #[error("missing identifier")]
    MissingIdentifier,
    // one specific token was expected
    #[error("missing expected: {0}")]
    MissingToken(Token),
    // multiple possibilities of tokens but none met
    #[error("found unexpected: {0}")]
    UnexpectedToken(Token),
    #[error("unexpected end")]
    UnexpectedEof,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Statement {
    Let { iden: String, expr: Expr },
    Return { expr: Expr },
    Expression { expr: Expr },
}

#[derive(Debug, PartialEq, Eq)]
pub struct Block(Vec<Statement>);

impl IntoIterator for Block {
    type Item = Statement;
    type IntoIter = std::vec::IntoIter<Statement>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, PartialEq, Eq)]
pub enum Expr {
    Prefix { op: PrefixOp, expr: Box<Expr> },
    Infix { op: InfixOp, lh: Box<Expr>, rh: Box<Expr> },
    Int(i64),
    Bool(bool),
    Identifier(String),
    IfCondition { condition: Box<Expr>, then_block: Block, else_block: Option<Block> },
    FnExpr { params: Vec<String>, body: Block },
    FnCall { callable: Box<Expr>, args: Vec<Expr> },
}

impl Expr {
    fn boxed(self) -> Box<Expr> {
        Box::new(self)
    }
}

trait Operation {
    fn precedence(&self) -> Precedence;
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PrefixOp {
    Not,      // !
    Negative, // -
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum InfixOp {
    Add,
    Sub,
    Mult,
    Div,
    Eq,
    NotEq,
    Lt,
    Gt,
    Lte,
    Gte,
}

impl Operation for PrefixOp {
    fn precedence(&self) -> Precedence {
        Precedence::Prefix
    }
}

impl Operation for InfixOp {
    fn precedence(&self) -> Precedence {
        use InfixOp::*;
        match self {
            Add => Precedence::Sum,
            Sub => Precedence::Sum,
            Mult => Precedence::Prod,
            Div => Precedence::Prod,
            Eq => Precedence::Eq,
            NotEq => Precedence::Eq,
            Lt => Precedence::LeGt,
            Gt => Precedence::LeGt,
            Lte => Precedence::LeGt,
            Gte => Precedence::LeGt,
        }
    }
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Statement::*;
        let out = match self {
            Let { iden, expr } => format! {"let {iden} = {expr};"},
            Return { expr } => format!("return {expr};"),
            Expression { expr } => expr.to_string(),
        };
        write!(f, "{}", out)?;
        Ok(())
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s: String = self.0.iter().map(|s| format!("\t{s}\n")).collect();
        write!(f, "{{\n{s}}}")?;
        Ok(())
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Expr::*;
        let out = match self {
            Prefix { op, expr } => format!("({op}{expr})"),
            Infix { op, lh, rh } => format!("({lh} {op} {rh})"),
            Int(n) => n.to_string(),
            Bool(b) => b.to_string(),
            Identifier(s) => s.clone(),
            IfCondition { condition, then_block, else_block } => {
                if let Some(else_block) = else_block {
                    format!("if ({condition}) {then_block} else {else_block}")
                } else {
                    format!("if ({condition}) {then_block}")
                }
            }
            FnExpr { params, body } => {
                let params = params.join(", ");
                format!("fn({params}) {body}")
            }
            FnCall { callable, args } => {
                let args: Vec<_> = args.iter().map(|a| a.to_string()).collect();
                let args = args.join(", ");
                format!("{callable}({args})")
            }
        };
        write!(f, "{}", out)?;
        Ok(())
    }
}

impl fmt::Display for PrefixOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let out = match self {
            PrefixOp::Not => "!",
            PrefixOp::Negative => "-",
        };
        write!(f, "{}", out)?;
        Ok(())
    }
}

impl fmt::Display for InfixOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use InfixOp::*;
        let out = match self {
            Add => "+",
            Sub => "-",
            Mult => "*",
            Div => "/",
            Eq => "=",
            NotEq => "!=",
            Lt => "<",
            Gt => ">",
            Lte => "<=",
            Gte => ">=",
        };
        write!(f, "{}", out)?;
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Precedence {
    Lowest = 0,
    Eq,
    LeGt,
    Sum,
    Prod,
    Prefix,
}
