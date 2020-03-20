use std::ops::Range;
#[derive(Clone, Debug)]
pub enum Expr {
    Lambda {
        param: String,
        body: Box<Expr>,
    },
    Val(Literal),
    Call {
        fun: Box<Expr>,
        arg: Box<Expr>,
    },
    BracketExpr {
        left: Box<Expr>,
        right: Box<Expr>,
    },
    BinOp {
        op: Op,
        left: Box<Expr>,
        right: Box<Expr>,
    },
}
#[derive(Debug)]
pub enum OpTerm {
    Op { op: Op, precedence: usize },
    Expr(Expr),
}
#[derive(Clone, Debug)]
pub enum Op {
    Add,
    Mul,
    Semicolon,
}
#[derive(Clone, Debug)]
pub enum Literal {
    Int(isize),
    Identifier(String),
    Unit,
}
#[derive(Debug)]
pub enum ParserReason {
    Expected(String),
    IncorrectToken(String),
}
#[derive(Debug)]
pub struct ParserError {
    pub reason: ParserReason,
    pub range: Range<usize>,
}
impl ParserError {
    pub fn new(reason: ParserReason, range: Range<usize>) -> Self {
        ParserError { reason, range }
    }
}

pub trait ParserResultControlFlow<T, E> {
    fn or_else_savable<O: FnOnce(E) -> Result<T, E>>(self, op: O) -> Result<T, E>;
}

impl<T> ParserResultControlFlow<T, ParserError> for Result<T, ParserError> {
    fn or_else_savable<O: FnOnce(ParserError) -> Result<T, ParserError>>(
        self,
        op: O,
    ) -> Result<T, ParserError> {
        match self {
            Ok(t) => Ok(t),
            Err(e) => match e.reason {
                ParserReason::IncorrectToken(_) => Err(e),
                _ => op(e),
            },
        }
    }
}
