use super::{
    lexer::{Token, TokenElem},
    shunting_yard::shunting_yard,
};
use std::{
    collections::{HashMap, VecDeque},
    ops::Range,
};

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
    BinOp {
        op: Op,
        left: Box<Expr>,
        right: Box<Expr>,
    },
}

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
}
#[derive(Debug)]
pub enum ParserReason {
    UnexpectedEof,
    Other(String),
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
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }
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
            TokenElem::Identifier(s) => {
                self.current += 1;
                Ok(Expr::Val(Literal::Identifier(s)))
            }
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
            TokenElem::Int(num) => {
                self.current += 1;
                Ok(Expr::Val(Literal::Int(num)))
            }
            _ => Err(ParserError::new(
                ParserReason::Other(format!("Expected number")),
                self.current..self.current,
            )),
        }
    }
    pub fn op(&mut self) -> Result<Op, ParserError> {
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
            TokenElem::Op(op) => {
                self.current += 1;
                match op.as_str() {
                    "+" => Ok(Op::Add),
                    "*" => Ok(Op::Mul),
                    _ => unimplemented!("Unhandled operator {}", op),
                }
            }
            _ => Err(ParserError::new(
                ParserReason::Other(format!("Expected operator")),
                self.current..self.current,
            )),
        }
    }
    pub fn equal(&mut self) -> Result<(), ParserError> {
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
            TokenElem::Equal => {
                self.current += 1;
                Ok(())
            }
            _ => Err(ParserError::new(
                ParserReason::Other(format!("Expected `=`")),
                self.current..self.current,
            )),
        }
    }
    pub fn atom(&mut self) -> Result<Expr, ParserError> {
        self.int().or_else(|_| self.func_call())
    }
    pub fn func_call(&mut self) -> Result<Expr, ParserError> {
        let func_name = self.identifier()?;
        let mut params = VecDeque::new();
        while let Ok(atom) = self.identifier() {
            params.push_front(atom);
        }
        if params.is_empty() {
            return Ok(func_name);
        }
        let mut call = Expr::Call {
            fun: Box::new(func_name),
            arg: Box::new(params.pop_back().unwrap()),
        };
        for _ in 0..params.len() {
            call = Expr::Call {
                fun: Box::new(call),
                arg: Box::new(params.pop_back().unwrap()),
            }
        }
        Ok(call)
    }
    pub fn _expr(&mut self, mut left: Vec<OpTerm>) -> Result<Expr, ParserError> {
        let begin = self.current;
        let op = self.op();
        if op.is_err() {
            self.current = begin;
            return Ok(shunting_yard(left)?);
        } else {
            let right = self.atom()?;
            let op = op.unwrap();
            let precedence = find_op_precedence(&op);
            left.push(OpTerm::Op { op, precedence });
            left.push(OpTerm::Expr(right));
            return Ok(self._expr(left)?);
        }
    }
    pub fn expr(&mut self) -> Result<Expr, ParserError> {
        let left = self.atom()?;
        self._expr(vec![OpTerm::Expr(left)])
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

fn find_op_precedence(op: &Op) -> usize {
    match op {
        Op::Add => 5,
        Op::Mul => 10,
    }
}
