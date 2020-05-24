// todo write some tests
// todo cleanup the code

use std::{
    convert::{TryFrom, TryInto},
    iter::Peekable,
};

use super::ast::*;
use super::{
    optable::OPTABLE,
    tokens::{Token, TokenKind},
};
use crate::{errors::syntax_err::*, utils::merge_ranges};

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
            //println!("{:#?}", self.peek());
            match self.atom() {
                Some(e) => {
                    //println!("{:#?}", e);
                    op_or_expr_vec.push(OpOrExpr::Expr(e))
                }
                None => match self.operator() {
                    Some(op) => op_or_expr_vec.push(OpOrExpr::Op(op)),
                    None => {
                        break;
                    }
                },
            }
        }
        Self::shunting_yard(op_or_expr_vec)
    }
    fn insert_bin_op(ast: &mut Vec<Node<Expr<'a>>>, Node { value: op, .. }: &Node<Operator<'a>>) {
        let right = ast.pop().unwrap();
        let left = ast.pop().unwrap();
        ast.push(Node {
            span: merge_ranges(&left.span, &right.span),
            value: Expr::Binary(op.clone().try_into().unwrap(), left.into(), right.into()),
        })
    }
    fn insert_un_op(
        ast: &mut Vec<Node<Expr<'a>>>,
        Node {
            value: op,
            span: op_span,
        }: &Node<Operator<'a>>,
    ) {
        let right = ast.pop().unwrap();
        ast.push(Node {
            span: merge_ranges(op_span, &right.span),
            value: Expr::Unary(op.clone().try_into().unwrap(), right.into()),
        })
    }
    fn shunting_yard(tokens: Vec<OpOrExpr<'a>>) -> Result<Node<Expr<'a>>, SyntaxErr<'a>> {
        let mut op_stack: Vec<Node<Operator<'a>>> = vec![];
        let mut ast: Vec<Node<Expr<'a>>> = vec![];
        let tok_len = tokens.len();
        let mut state = ShuntingYardState::ExpectOperand;
        for (idx, tok) in tokens.into_iter().enumerate() {
            match tok {
                OpOrExpr::Expr(e) => {
                    if let ShuntingYardState::ExpectOp = state {
                        let last = ast.pop().unwrap();
                        ast.push(Node {
                            span: merge_ranges(&last.span, &e.span),
                            value: Expr::Call(last.into_boxed(), e.into_boxed()),
                        })
                    } else {
                        state = ShuntingYardState::ExpectOp;
                        ast.push(e)
                    }
                }
                OpOrExpr::Op(Node {
                    value: mut op,
                    span,
                }) => {
                    if idx == tok_len {
                        return Err(SyntaxErr {
                            span: span.clone(),
                            kind: SyntaxErrKind::UnexpectedToken(TokenKind::Op(op.sym)),
                            expected: Expected::OneOf(vec![
                                Expected::Operator,
                                Expected::Semicolon,
                            ]),
                            note: None,
                        });
                    }
                    if let ShuntingYardState::ExpectOperand = state {
                        match op.into_prefix() {
                            Some(un_op) => op = un_op,
                            None => {
                                return Err(SyntaxErr {
                                    span: span.clone(),
                                    kind: SyntaxErrKind::UnexpectedToken(TokenKind::Op(op.sym)),
                                    expected: Expected::Expr,
                                    note: None,
                                })
                            }
                        }
                    }
                    state = ShuntingYardState::ExpectOperand;
                    while let Some(Node { value: last_op, .. }) = op_stack.last() {
                        if last_op.is_infix() {
                            if last_op.prec > op.prec
                                || (last_op.prec >= op.prec && last_op.is_left_assoc())
                            {
                                if last_op.is_infix() {
                                    Self::insert_bin_op(&mut ast, &op_stack.pop().unwrap())
                                } else {
                                    Self::insert_un_op(&mut ast, &op_stack.pop().unwrap())
                                }
                            } else {
                                break;
                            }
                        }
                    }
                    op_stack.push(Node { span, value: op });
                }
            }
        }
        for i in op_stack.into_iter().rev() {
            if i.value.is_infix() {
                Self::insert_bin_op(&mut ast, &i)
            } else {
                Self::insert_un_op(&mut ast, &i)
            }
        }
        if ast.len() != 1 {
            panic!("An unexpected error occured")
        }
        Ok(ast.into_iter().next().unwrap())
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
enum ShuntingYardState {
    ExpectOperand,
    ExpectOp,
}

#[derive(Debug)]
enum OpOrExpr<'a> {
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
