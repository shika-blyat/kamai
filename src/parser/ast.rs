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
}
#[derive(Clone, Debug)]
pub enum Literal {
    Int(isize),
    Identifier(String),
    Unit,
}
#[derive(Debug)]
pub enum ParserReason {
    Custom(String),
}
#[derive(Debug)]
pub struct ParserError {
    reason: ParserReason,
    range: Range<usize>,
}
impl ParserError {
    pub fn new(reason: ParserReason, range: Range<usize>) -> Self {
        ParserError { reason, range }
    }
}
