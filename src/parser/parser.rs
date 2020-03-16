use super::lexer::{Token, TokenElem};
use std::{cell::RefCell, collections::HashMap, ops::Range, rc::Rc};

#[derive(Clone)]
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
#[derive(Clone)]
pub enum Op {
    Add,
    Mul,
}
#[derive(Clone)]
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
    ctx: HashMap<String, Expr>,
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
        self.current += 1;
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
        self.current += 1;
        match elem {
            TokenElem::Int(num) => Ok(Expr::Val(Literal::Int(num))),
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
        self.current += 1;
        match elem {
            TokenElem::Op(op) => match op.as_str() {
                "+" => Ok(Op::Add),
                "*" => Ok(Op::Mul),
                _ => unimplemented!("Unhandled operator {}", op),
            },
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
        self.current += 1;
        match elem {
            TokenElem::Equal => Ok(()),
            _ => Err(ParserError::new(
                ParserReason::Other(format!("Expected `=`")),
                self.current..self.current,
            )),
        }
    }
    pub fn _bin_op(&mut self) -> Result<Expr, ParserError> {
        match self.op() {
            Ok(op) => {
                let func = self.identifier()?;
                let mut params = vec![];
                while let Ok(ident) = self.identifier().or_else(|_| self.int()) {
                    params.push(ident);
                }
                if let Some(p1) = params.get(0) {
                    let mut call = Expr::Call {
                        fun: Rc::new(RefCell::new(func)),
                        arg: Rc::new(RefCell::new(p1.clone())),
                    };
                    for i in params.into_iter().rev() {
                        call = Expr::Call {
                            fun: Rc::new(RefCell::new(call)),
                            arg: Rc::new(RefCell::new(i.clone())),
                        };
                    }
                    return Ok(call);
                } else {
                    return Ok(func);
                }
            }
            Err(_) => unreachable!(),
        }
    }
    pub fn expr(&mut self) -> Result<Expr, ParserError> {
        let func = self.identifier()?;
        let mut params = vec![];
        while let Ok(ident) = self.identifier().or_else(|_| self.int()) {
            params.push(ident);
        }
    }
    pub fn func_decl(&mut self) -> Result<(), ParserError> {
        let func_name = self.identifier()?;
        let mut params = vec![];
        while let Ok(ident) = self.identifier() {
            params.push(ident);
        }
        self.equal()?;

        //self.ctx.insert(func_name, v: V)
        Ok(())
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
