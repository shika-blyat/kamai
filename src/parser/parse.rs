#![allow(clippy::range_plus_one)]
use super::ast::*;
use super::{
    lexer::{Token, TokenElem},
    shunting_yard::shunting_yard,
};
use std::collections::{HashMap, VecDeque};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }
    #[allow(dead_code)]
    pub fn parse(&mut self) -> Result<HashMap<String, Expr>, ParserError> {
        let mut map = HashMap::new();
        while !self.is_empty() {
            let (func_name, function) = self.func_decl()?;
            map.insert(func_name, function);
        }
        Ok(map)
    }
    pub fn atom(&mut self) -> Result<Expr, ParserError> {
        self.int()
            .or_else_savable(|_| self.func_call())
            .or_else_savable(|_| self.brackets())
            .or_else_savable(|_| self.parenthesis())
            .or_else_savable(|_| self.unit())
            .or_else_savable(|_| match self.current() {
                Some(tok) => {
                    let range = tok.range.clone();
                    Err(ParserError::new(
                        ParserReason::IncorrectToken(format!("Unexpected {:#?}", tok)),
                        range,
                    ))
                }
                None => Err(ParserError::new(
                    ParserReason::IncorrectToken("Unexpected EOF".to_string()),
                    self.current..self.current + 1,
                )),
            })
    }
    pub fn func_decl(&mut self) -> Result<(String, Expr), ParserError> {
        let func_name = match self.identifier()? {
            Expr::Identifier(func_name) => func_name,
            _ => unreachable!(),
        };
        let mut params = vec![];
        while let Ok(ident) = self.identifier() {
            match ident {
                Expr::Identifier(ident) => params.push(ident),
                _ => unreachable!(),
            }
        }
        self.equal()?;
        let mut function = self.expr()?;
        self.semicolon()?;
        for param in params {
            function = Expr::Lambda {
                param,
                body: Box::new(function),
            }
        }
        Ok((func_name, function))
    }
    pub fn func_call(&mut self) -> Result<Expr, ParserError> {
        let func_name = self.identifier()?;
        let mut params = VecDeque::new();
        while let Ok(atom) = self.expr() {
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
        match op {
            Err(_) => {
                self.current = begin;
                Ok(shunting_yard(left)?)
            }
            Ok(op) => {
                let right = self.atom().map_err(|_| {
                    ParserError::new(
                        ParserReason::IncorrectToken(format!(
                            "Missing right operand for operator {:#?}",
                            op
                        )),
                        self.current..self.current + 1,
                    )
                })?;
                let precedence = find_op_precedence(&op);
                left.push(OpTerm::Op { op, precedence });
                left.push(OpTerm::Expr(right));
                Ok(self._expr(left)?)
            }
        }
    }
    pub fn expr(&mut self) -> Result<Expr, ParserError> {
        let left = self.atom()?;
        self._expr(vec![OpTerm::Expr(left)])
    }
    pub fn identifier(&mut self) -> Result<Expr, ParserError> {
        let elem = self.current_elem_or_err("Expected identifier".to_string())?;
        match elem {
            TokenElem::Identifier(s) => {
                self.current += 1;
                Ok(Expr::Identifier(s))
            }
            _ => Err(ParserError::new(
                ParserReason::Expected("Expected identifier".to_string()),
                self.current..self.current + 1,
            )),
        }
    }
    fn unit(&mut self) -> Result<Expr, ParserError> {
        let elem = self.current_elem_or_err("Expected unit_type".to_string())?;
        match elem {
            TokenElem::Unit => {
                self.current += 1;
                Ok(Expr::Val(Literal::Unit))
            }
            _ => Err(ParserError::new(
                ParserReason::Expected("Expected unit type".to_string()),
                self.current..self.current + 1,
            )),
        }
    }
    pub fn int(&mut self) -> Result<Expr, ParserError> {
        let elem = self.current_elem_or_err("Expected number".to_string())?;
        match elem {
            TokenElem::Int(num) => {
                self.current += 1;
                Ok(Expr::Val(Literal::Int(num)))
            }
            _ => Err(ParserError::new(
                ParserReason::Expected("Expected number".to_string()),
                self.current..self.current + 1,
            )),
        }
    }
    pub fn op(&mut self) -> Result<Op, ParserError> {
        let elem = self.current_elem_or_err("Expected operator".to_string())?;
        match elem {
            TokenElem::Op(op) => {
                self.current += 1;
                match op.as_str() {
                    "+" => Ok(Op::Add),
                    "*" => Ok(Op::Mul),
                    op => unimplemented!("Unhandled operator {}", op),
                }
            }
            _ => Err(ParserError::new(
                ParserReason::Expected("Expected operator".to_string()),
                self.current..self.current + 1,
            )),
        }
    }
    pub fn semicolon(&mut self) -> Result<(), ParserError> {
        let elem = self.current_elem_or_err("Expected `;`".to_string())?;
        match elem {
            TokenElem::Semicolon => {
                self.current += 1;
                Ok(())
            }
            _ => Err(ParserError::new(
                ParserReason::Expected("Expected `;`".to_string()),
                self.current..self.current + 1,
            )),
        }
    }
    pub fn equal(&mut self) -> Result<(), ParserError> {
        let elem = self.current_elem_or_err("Expected `=`".to_string())?;
        match elem {
            TokenElem::Equal => {
                self.current += 1;
                Ok(())
            }
            _ => Err(ParserError::new(
                ParserReason::Expected("Expected `=`".to_string()),
                self.current..self.current + 1,
            )),
        }
    }
    pub fn brackets(&mut self) -> Result<Expr, ParserError> {
        let elem = self.current_elem_or_err("Expected brackets".to_string())?;
        match elem {
            TokenElem::BracketPair(tokens) => {
                self.current += 1;
                let mut parser = Parser::new(tokens);
                let expr = parser
                    .expr()
                    .or_else_savable(|_| Ok(Expr::Val(Literal::Unit)))?;
                let mut bracket_expr = vec![OpTerm::Expr(expr)];
                while parser.semicolon().is_ok() {
                    match parser.expr() {
                        Ok(expr) => {
                            bracket_expr.push(OpTerm::Op {
                                op: Op::Semicolon,
                                precedence: 0,
                            });
                            bracket_expr.push(OpTerm::Expr(expr));
                        }
                        Err(parser_err) => match parser_err.reason {
                            ParserReason::IncorrectToken(_) => return Err(parser_err),
                            _ => match bracket_expr.last() {
                                Some(OpTerm::Expr(Expr::Val(Literal::Unit))) => (),
                                _ => {
                                    bracket_expr.push(OpTerm::Op {
                                        op: Op::Semicolon,
                                        precedence: 0,
                                    });
                                    bracket_expr.push(OpTerm::Expr(Expr::Val(Literal::Unit)));
                                }
                            },
                        },
                    }
                }
                println!("{:#?}", bracket_expr);
                if parser.is_empty() {
                    Ok(shunting_yard(bracket_expr)?)
                } else {
                    Err(ParserError::new(
                        ParserReason::Expected(format!(
                            "Unexpected {}",
                            parser.tokens[parser.current].lexeme
                        )),
                        self.current + parser.current..self.current + parser.current + 1,
                    ))
                }
            }
            _ => Err(ParserError::new(
                ParserReason::Expected("Expected brackets".to_string()),
                self.current..self.current,
            )),
        }
    }
    pub fn parenthesis(&mut self) -> Result<Expr, ParserError> {
        let elem = self.current_elem_or_err("Expected parenthesis".to_string())?;
        match elem {
            TokenElem::ParenthesisPair(tokens) => {
                self.current += 1;
                let mut parser = Parser::new(tokens);
                let expr = parser.expr();
                if parser.is_empty() {
                    expr
                } else {
                    Err(ParserError::new(
                        ParserReason::Expected(format!(
                            "Unexpected {}",
                            parser.tokens[parser.current].lexeme
                        )),
                        self.current + parser.current..self.current + parser.current + 1,
                    ))
                }
            }
            _ => Err(ParserError::new(
                ParserReason::Expected("Expected parenthesis".to_string()),
                self.current..self.current + 1,
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
    pub fn current_elem_or_err(&self, msg: String) -> Result<TokenElem, ParserError> {
        match self.current() {
            Some(x) => Ok(x.elem),
            None => Err(ParserError::new(
                ParserReason::Expected(msg),
                self.current..self.current,
            )),
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
        Op::Semicolon => 0,
    }
}
