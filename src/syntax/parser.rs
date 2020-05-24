use std::{
    convert::{TryFrom, TryInto},
    iter::Peekable,
};

use super::ast::*;
use super::tokens::{Token, TokenKind};
use crate::{errors::syntax_err::*, utils::merge_ranges};
pub struct Parser<'a, I: Iterator<Item = Token<'a>>> {
    tokens: Peekable<I>,
}

macro_rules! empty_tok {
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

enum ShuntingYardState {
    ExpectedOperand,
    ExpectedOp,
}

impl<'a, I: Iterator<Item = Token<'a>>> Parser<'a, I> {
    pub fn new(iter: I) -> Self {
        Self {
            tokens: iter.peekable(),
        }
    }
    fn expr(&mut self) -> Result<Node<Expr<'a>>, SyntaxErr<'a>> {
        match self.atom() {
            Some(v) => Ok(v),
            None => self
                .parenthesized_expr()?
                .ok_or_else(|| self.unexpected_err(Expected::Expr)),
        }
    }
    fn operation(&mut self) -> Result<Node<Expr<'a>>, SyntaxErr<'a>> {
        let mut op_or_expr_vec = vec![];
        loop {
            match self.expr() {
                Ok(e) => op_or_expr_vec.push(OpOrExpr::Expr(e)),
                Err(_) => match self.operator() {
                    Some(op) => op_or_expr_vec.push(OpOrExpr::Op(op)),
                    None => break,
                },
            }
        }
        Self::shunting_yard(op_or_expr_vec)
    }
    fn insert_bin_op(ast: &mut Vec<Node<Expr<'a>>>, op: &Operator) {
        let right = ast.pop().unwrap();
        let left = ast.pop().unwrap();
        ast.push(Node {
            span: merge_ranges(&left.span, &right.span),
            value: Expr::Binary(op.clone().try_into().unwrap(), left.into(), right.into()),
        })
    }
    fn shunting_yard(tokens: Vec<OpOrExpr<'a>>) -> Result<Node<Expr<'a>>, SyntaxErr<'a>> {
        // finalize shunting yard
        let mut op_stack: Vec<Node<Operator<'a>>> = vec![];
        let mut ast: Vec<Node<Expr<'a>>> = vec![];
        for tok in tokens.into_iter() {
            match tok {
                OpOrExpr::Expr(e) => ast.push(e),
                OpOrExpr::Op(Node { value: op, span }) => {
                    while let Some(Node {
                        value: last_op,
                        span: _,
                    }) = op_stack.last()
                    {
                        if last_op.precedence() > op.precedence()
                            || (last_op.precedence() >= op.precedence() && last_op.assoc.is_left())
                        {
                            Self::insert_bin_op(&mut ast, last_op)
                        }
                    }
                    op_stack.push(Node { span, value: op });
                }
            }
        }
        Ok(ast.into_iter().next().unwrap())
    }
    fn operator(&mut self) -> Option<Node<Operator<'a>>> {
        self.op().map(|Token { kind, span }| match kind {
            TokenKind::Op(sym) => Node {
                value: match sym {
                    "+" | "-" | "*" | "/" | "&&" | "||" | "<" | "<=" | ">" | ">=" | "==" | "!=" => {
                        Operator {
                            sym,
                            assoc: Assoc::Left,
                            fixity: Fixity::Infix,
                        }
                    }
                    "!" => Operator {
                        sym,
                        assoc: Assoc::Left,
                        fixity: Fixity::Prefix,
                    },
                    _ => unreachable!(),
                },
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
            .or_else(|| self.call())
            .or_else(|| self.expr_ident())
    }
    fn call(&mut self) -> Option<Node<Expr<'a>>> {
        let f = self.expr_ident().map(|Node { span, value }| Node {
            span: span,
            value: Box::new(value),
        })?;
        let mut args = vec![];
        while let Ok(Node { span, value }) = self.expr() {
            args.push(Node {
                span: span,
                value: Box::new(value),
            });
        }
        if args.is_empty() {
            Some(Node {
                span: f.span.clone(),
                value: Expr::EmptyCall(f),
            })
        } else {
            let mut args_iter = args.into_iter();
            let first_arg = args_iter.next().unwrap();
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
    fn parenthesized_expr(&mut self) -> Result<Option<Node<Expr<'a>>>, SyntaxErr<'a>> {
        match self.lparen() {
            Some(_) => {
                let e = self.expr().map(|e| Some(e))?;
                self.rparen();
                Ok(e)
            }
            None => Ok(None),
        }
    }
    empty_tok!(op, TokenKind::Op(_));
    empty_tok!(num, TokenKind::Number(_));
    empty_tok!(bool, TokenKind::Bool(_));
    empty_tok!(ident, TokenKind::Ident(_));
    empty_tok!(if_, TokenKind::If);
    empty_tok!(else_, TokenKind::Else);
    empty_tok!(then, TokenKind::Then);
    empty_tok!(return_, TokenKind::Return);
    empty_tok!(eq, TokenKind::Eq);
    empty_tok!(semicolon, TokenKind::Semicolon);
    empty_tok!(unit, TokenKind::Unit);
    empty_tok!(lbrace, TokenKind::LBrace);
    empty_tok!(rbrace, TokenKind::RBrace);
    empty_tok!(lparen, TokenKind::LParen);
    empty_tok!(rparen, TokenKind::RParen);
    pub fn unexpected_err(&mut self, expected: Expected) -> SyntaxErr<'a> {
        let (span, kind) = match self.peek() {
            Some(_) => {
                let Token { kind, span } = self.next().unwrap();
                (span, SyntaxErrKind::UnexpectedToken(kind.clone()))
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
            Some(_) => {
                let Token { span, .. } = self.next().unwrap();
                (span, SyntaxErrKind::Unclosed(delimiter))
            }
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
    pub fn peek(&mut self) -> Option<&Token<'a>> {
        self.tokens.peek()
    }
    pub fn next(&mut self) -> Option<Token<'a>> {
        self.tokens.next()
    }
}

enum OpOrExpr<'a> {
    Expr(Node<Expr<'a>>),
    Op(Node<Operator<'a>>),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Operator<'a> {
    sym: &'a str,
    assoc: Assoc,
    fixity: Fixity,
}

impl<'a> Operator<'a> {
    pub fn precedence(&self) -> usize {
        match self.sym {
            "+" | "-" => match self.fixity {
                Fixity::Prefix => 50,
                Fixity::Infix => 10,
            },
            "!" => 50,
            "*" | "/" => 20,
            "&&" | "||" => 5,
            ">=" | ">" | "<" | "<=" => 15,
            "==" | "!=" => 2,
            _ => unreachable!(),
        }
    }
}
impl<'a> TryFrom<Operator<'a>> for BinOp {
    type Error = ();
    fn try_from(op: Operator<'a>) -> Result<BinOp, ()> {
        Ok(match op.sym {
            "+" => match op.fixity {
                Fixity::Prefix => return Err(()),
                Fixity::Infix => BinOp::Add,
            },
            "-" => match op.fixity {
                Fixity::Prefix => return Err(()),
                Fixity::Infix => BinOp::Sub,
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
    Infix,
}
