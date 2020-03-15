use super::lexer::{Token, TokenElem};
use std::{cell::RefCell, ops::Range, rc::Rc};

pub enum Expr {
    Lambda {
        param: String,
        body: Rc<RefCell<Expr>>,
    },
    Val(Literal),
    Call {
        fun: Rc<RefCell<Expr>>,
        arg: Rc<RefCell<Expr>>,
    },
    BinOp {
        op: Op,
        arg1: Rc<RefCell<Expr>>,
        arg2: Rc<RefCell<Expr>>,
    },
}
pub enum Op {
    Add,
    Mul,
}
pub enum Literal {
    Int(isize),
    Identifier(String),
}

pub enum ParserReason {
    UnexpectedEof,
    Other(String),
}
struct ParserError {
    reason: ParserReason,
    range: Range<usize>,
}
impl ParserError {
    pub fn new(reason: ParserReason, range: Range<usize>) -> Self {
        ParserError { reason, range }
    }
}
struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn identifier(&mut self) -> Result<Expr, ParserError> {
        let elem = match self.current() {
            Some(x) => x.elem,
            None => {
                return Err(ParserError::new(
                    ParserReason::UnexpectedEof,
                    self.current..self.current,
                ))
            }
        };
        match elem {
            TokenElem::Identifier(s) => Ok(Expr::Val(Literal::Identifier(s))),
            _ => Err(ParserError::new(
                ParserReason::Other(format!("Expected identifier")),
                self.current..self.current,
            )),
        }
    }
    pub fn int(&mut self) -> Result<Expr, ParserError> {
        let elem = match self.current() {
            Some(x) => x.elem,
            None => {
                return Err(ParserError::new(
                    ParserReason::UnexpectedEof,
                    self.current..self.current,
                ))
            }
        };
        match elem {
            TokenElem::Int(num) => Ok(Expr::Val(Literal::Int(num))),
            _ => Err(ParserError::new(
                ParserReason::Other(format!("Expected number")),
                self.current..self.current,
            )),
        }
    }
    pub fn current(&self) -> Option<Token> {
        if self.is_empty() {
            None
        } else {
            Some(self.tokens[self.current].clone())
        }
    }
    pub fn is_empty(&self) -> bool {
        self.current >= self.tokens.len()
    }
}
