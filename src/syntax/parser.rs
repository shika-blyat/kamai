// todo write some tests
// todo cleanup the code

use std::{convert::TryFrom, iter::Peekable};

use super::ast::*;
use super::{
    optable::OPTABLE,
    shunting_yard::*,
    tokens::{Token, TokenKind},
};
use crate::errors::syntax_err::*;

macro_rules! tok {
    ($name: ident, $token: pat) => {
        fn $name(&mut self) -> Option<Token<'a>> {
            let Token { kind, .. } = self.peek()?;
            match kind {
                $token => self.next(),
                _ => None,
            }
        }
    };
}

pub struct Parser<'a, I: Iterator<Item = Token<'a>>> {
    tokens: Peekable<I>,
    pub errors: Vec<SyntaxErr<'a>>,
}
#[allow(dead_code)]
impl<'a, I: Iterator<Item = Token<'a>>> Parser<'a, I> {
    pub fn new(iter: I) -> Self {
        Self {
            tokens: iter.peekable(),
            errors: vec![],
        }
    }
    pub fn expr(&mut self) -> Result<Node<Expr<'a>>, SyntaxErr<'a>> {
        let mut op_or_expr_vec = vec![];
        while let Some(val) = self
            .atom()
            .map(|e| OpOrExpr::Expr(e))
            .or_else(|| self.operator().map(|op| OpOrExpr::Op(op)))
        {
            op_or_expr_vec.push(val);
        }
        shunting_yard(op_or_expr_vec)
    }
    fn operator(&mut self) -> Option<Node<Operator<'a>>> {
        self.op().map(|Token { kind, span }| match kind {
            TokenKind::Op(sym) => Node {
                value: *OPTABLE.get(sym).unwrap(),
                span,
            },
            _ => unreachable!(),
        })
    }
    fn atom(&mut self) -> Option<Node<Expr<'a>>> {
        self.num()
            .or_else(|| self.bool())
            .or_else(|| self.unit())
            .map(|Token { kind, span }| Node {
                value: match kind {
                    TokenKind::Unit => Expr::Literal(Literal::Unit),
                    TokenKind::Bool(b) => Expr::Literal(Literal::Bool(b)),
                    TokenKind::Number(n) => Expr::Literal(Literal::Num(n)),
                    _ => unreachable!(),
                },
                span,
            })
            .or_else(|| self.expr_ident())
            .or_else(|| self.parenthesized_expr())
    }
    fn expr_ident(&mut self) -> Option<Node<Expr<'a>>> {
        self.ident().map(|Token { kind, span }| match kind {
            TokenKind::Ident(s) => Node {
                value: Expr::Ident(s),
                span,
            },
            _ => unreachable!(),
        })
    }
    fn parenthesized_expr(&mut self) -> Option<Node<Expr<'a>>> {
        let Token { span, kind } = self.lparen()?;
        let e = self
            .expr()
            .map(|e| Node {
                span: e.span.clone(),
                value: Expr::Parenthesized(e.into_boxed()),
            })
            .ok()?;
        self.rparen().or_else(|| {
            // todo There could be more information here, but i don't have the variants yet to express them, i should probably fix this at some point
            self.errors.push(SyntaxErr {
                span,
                kind: SyntaxErrKind::UnexpectedToken(kind),
                expected: Expected::None,
                note: None,
            });
            self.restore();
            None
        });
        Some(e)
    }
    tok!(op, TokenKind::Op(_));
    tok!(num, TokenKind::Number(_));
    tok!(bool, TokenKind::Bool(_));
    tok!(ident, TokenKind::Ident(_));
    tok!(lparen, TokenKind::LParen);
    tok!(rparen, TokenKind::RParen);
    tok!(unit, TokenKind::Unit);
    tok!(if_, TokenKind::If);
    tok!(else_, TokenKind::Else);
    tok!(then, TokenKind::Then);
    tok!(return_, TokenKind::Return);
    tok!(eq, TokenKind::Eq);
    tok!(semicolon, TokenKind::Semicolon);
    tok!(rbrace, TokenKind::RBrace);
    tok!(lbrace, TokenKind::LBrace);

    fn restore(&mut self) {
        loop {
            match self.next() {
                Some(Token {
                    kind: TokenKind::Semicolon | TokenKind::RBrace,
                    ..
                })
                | None => break,
                _ => (),
            }
        }
    }
    fn peek(&mut self) -> Option<&Token<'a>> {
        self.tokens.peek()
    }
    fn next(&mut self) -> Option<Token<'a>> {
        self.tokens.next()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Operator<'a> {
    pub sym: &'a str,
    pub fixity: Fixity,
    pub prec: u8,
}

impl<'a> Operator<'a> {
    pub fn is_infix(&self) -> bool {
        self.fixity != Fixity::Prefix
    }
    pub fn into_prefix(self) -> Option<Self> {
        match self.fixity {
            Fixity::Prefix => Some(self),
            Fixity::Infix(_) => Some(Self {
                fixity: Fixity::Prefix,
                prec: match self.sym {
                    "+" | "-" => 25,
                    _ => return None,
                },
                ..self
            }),
        }
    }
    pub fn is_left_assoc(&self) -> bool {
        match self.fixity {
            Fixity::Infix(assoc) => assoc.is_left(),
            _ => false,
        }
    }
}
impl<'a> TryFrom<Operator<'a>> for BinOp {
    type Error = ();
    fn try_from(op: Operator<'a>) -> Result<BinOp, ()> {
        Ok(match op.sym {
            "+" => match op.fixity {
                Fixity::Prefix => return Err(()),
                Fixity::Infix(_) => BinOp::Add,
            },
            "-" => match op.fixity {
                Fixity::Prefix => return Err(()),
                Fixity::Infix(_) => BinOp::Sub,
            },
            "*" => BinOp::Mul,
            "/" => BinOp::Div,
            "&&" => BinOp::And,
            "||" => BinOp::Or,
            ">=" => BinOp::GTE,
            ">" => BinOp::GT,
            "<" => BinOp::LT,
            "<=" => BinOp::LTE,
            "==" => BinOp::EqEq,
            "!=" => BinOp::NotEq,
            _ => unreachable!(),
        })
    }
}
impl<'a> TryFrom<Operator<'a>> for UnOp {
    type Error = ();
    fn try_from(op: Operator<'a>) -> Result<UnOp, ()> {
        Ok(match op.sym {
            "+" => match op.fixity {
                Fixity::Prefix => UnOp::Pos,
                Fixity::Infix(_) => return Err(()),
            },
            "-" => match op.fixity {
                Fixity::Prefix => UnOp::Neg,
                Fixity::Infix(_) => return Err(()),
            },
            "!" => UnOp::Neg,
            _ => return Err(()),
        })
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Assoc {
    Right,
    Left,
}
impl Assoc {
    pub fn is_left(&self) -> bool {
        self == &Assoc::Left
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Fixity {
    Prefix,
    Infix(Assoc),
}
/*
mod tests {
    use super::*;
    fn simple_expr()
}*/
