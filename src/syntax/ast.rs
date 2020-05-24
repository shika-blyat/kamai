#![allow(dead_code)]
use std::ops::Range;

pub type BoxNode<T> = Node<Box<T>>;
pub type Ident<'a> = &'a str;

#[derive(Debug, PartialEq, Clone)]
pub enum Expr<'a> {
    Literal(Literal),
    Ident(Ident<'a>),
    Parenthesized(BoxNode<Expr<'a>>),
    Unary(UnOp, BoxNode<Expr<'a>>),
    Binary(BinOp, BoxNode<Expr<'a>>, BoxNode<Expr<'a>>),
    Lambda(Ident<'a>, BoxNode<Expr<'a>>),
    Call(BoxNode<Expr<'a>>, BoxNode<Expr<'a>>),
    EmptyCall(BoxNode<Expr<'a>>),
    Block {
        instructions: Vec<Node<Statement<'a>>>,
        returns: bool,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum Statement<'a> {
    Return(Expr<'a>),
    Continue,
    Break(Expr<'a>),
    StmtExpr(Expr<'a>),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    And,
    Or,
    LT,
    LTE,
    GT,
    GTE,
    EqEq,
    NotEq,
}
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum UnOp {
    Pos,
    Neg,
    Not,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Literal {
    Num(i64),
    Bool(bool),
    Unit,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Node<T: Clone> {
    pub value: T,
    pub span: Range<usize>,
}
impl<T: Clone> Node<T> {
    pub fn into_boxed(self) -> Node<Box<T>> {
        Node {
            value: Box::new(self.value),
            span: self.span,
        }
    }
}

impl<T: Clone> From<Node<T>> for BoxNode<T> {
    fn from(Node { span, value }: Node<T>) -> Self {
        Self {
            value: Box::new(value),
            span,
        }
    }
}
