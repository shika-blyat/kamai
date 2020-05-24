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
    pub error_monad: Vec<SyntaxErr<'a>>,
}

#[allow(dead_code)]
impl<'a, I: Iterator<Item = Token<'a>>> Parser<'a, I> {
    pub fn new(iter: I) -> Self {
        Self {
            tokens: iter.peekable(),
            error_monad: vec![],
        }
    }
    pub fn expr(&mut self) -> Result<Node<Expr<'a>>, SyntaxErr<'a>> {
        let mut op_or_expr_vec = vec![];
        loop {
            match self.atom() {
                Some(e) => op_or_expr_vec.push(OpOrExpr::Expr(e)),
                None => match self.operator() {
                    Some(op) => op_or_expr_vec.push(OpOrExpr::Op(op)),
                    None => {
                        break;
                    }
                },
            }
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
            .map(|Token { kind, span }| match kind {
                TokenKind::Unit => Node {
                    value: Expr::Literal(Literal::Unit),
                    span,
                },
                TokenKind::Ident(s) => Node {
                    value: Expr::Ident(s),
                    span,
                },
                TokenKind::Bool(b) => Node {
                    value: Expr::Literal(Literal::Bool(b)),
                    span,
                },
                TokenKind::Number(n) => Node {
                    value: Expr::Literal(Literal::Num(n)),
                    span,
                },
                _ => unreachable!(),
            })
            .or_else(|| self.expr_ident())
            .or_else(|| self.parenthesized_expr())
    }
    pub fn call(&mut self) -> Option<Node<Expr<'a>>> {
        let f = self.expr_ident()?;
        let mut args = vec![];
        while let Ok(Node { span, value }) = self.expr() {
            args.push(Node {
                span: span,
                value: Box::new(value),
            });
        }
        if args.is_empty() {
            None
        } else {
            let mut args_iter = args.into_iter();
            let first_arg = args_iter.next().unwrap();
            let f = Node {
                span: f.span,
                value: Box::new(f.value),
            };
            let mut call = Node {
                span: f.span.start..first_arg.span.end,
                value: Expr::Call(f, first_arg),
            };
            for expr in args_iter {
                let Node { span, value } = call;
                call = Node {
                    span: span.start..expr.span.end,
                    value: Expr::Call(
                        Node {
                            value: Box::new(value),
                            span,
                        },
                        expr,
                    ),
                };
            }
            Some(call)
        }
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
    /// Returns an option if no parenthesis is opened, and a Some(Err(e)) if a parenthesis is opened but not closed
    fn parenthesized_expr(&mut self) -> Option<Node<Expr<'a>>> {
        let Token { span, .. } = self.lparen()?;
        let e = self
            .expr()
            .map(|e| Node {
                span: e.span.clone(),
                value: Expr::Parenthesized(e.into_boxed()),
            })
            .ok()?;
        self.rparen().or_else(|| {
            self.error_monad.push(SyntaxErr {
                span,
                kind: SyntaxErrKind::Unclosed(Delimiter::Paren),
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
    tok!(if_, TokenKind::If);
    tok!(else_, TokenKind::Else);
    tok!(then, TokenKind::Then);
    tok!(return_, TokenKind::Return);
    tok!(eq, TokenKind::Eq);
    tok!(semicolon, TokenKind::Semicolon);
    tok!(unit, TokenKind::Unit);
    tok!(lbrace, TokenKind::LBrace);
    tok!(rbrace, TokenKind::RBrace);
    tok!(lparen, TokenKind::LParen);
    tok!(rparen, TokenKind::RParen);
    pub fn unexpected_err(&mut self, expected: Expected) -> SyntaxErr<'a> {
        let (span, kind) = match self.peek() {
            Some(Token { span, kind }) => {
                (span.clone(), SyntaxErrKind::UnexpectedToken(kind.clone()))
            }
            None => (
                std::usize::MAX..std::usize::MAX,
                SyntaxErrKind::UnexpectedEOF,
            ),
        };
        SyntaxErr {
            kind,
            span,
            expected,
            note: None,
        }
    }
    pub fn unclosed_err(&mut self, delimiter: Delimiter) -> SyntaxErr<'a> {
        let (span, kind) = match self.peek() {
            Some(Token { span, .. }) => (span.clone(), SyntaxErrKind::Unclosed(delimiter)),
            None => (
                std::usize::MAX..std::usize::MAX,
                SyntaxErrKind::UnexpectedEOF,
            ),
        };
        SyntaxErr {
            kind,
            span,
            expected: Expected::None,
            note: None,
        }
    }
    pub fn restore(&mut self) {
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
    pub fn peek(&mut self) -> Option<&Token<'a>> {
        self.tokens.peek()
    }
    pub fn next(&mut self) -> Option<Token<'a>> {
        self.tokens.next()
    }
}

#[derive(Debug)]
pub(super) enum OpOrExpr<'a> {
    Expr(Node<Expr<'a>>),
    Op(Node<Operator<'a>>),
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
    pub fn can_be_prefix(&self) -> bool {
        match (self.sym, self.fixity) {
            (_, Fixity::Prefix) | ("+" | "-", _) => true,
            _ => false,
        }
    }
    pub fn into_prefix(self) -> Option<Self> {
        match self.fixity {
            Fixity::Prefix => Some(self),
            Fixity::Infix(_) => Some(Self {
                fixity: Fixity::Prefix,
                prec: match self.sym {
                    "+" | "-" => 22,
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
    pub fn is_right(&self) -> bool {
        !self.is_left()
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
