#![allow(dead_code)]
use std::ops::Range;

pub type BoxNode<T> = Node<Box<T>>;
pub type Ident<'a> = &'a str;

#[derive(Debug, PartialEq, Clone)]
pub enum Expr<'a> {
    Literal(Node<Literal>),
    Ident(Node<Ident<'a>>),
    Unary(UnOp, BoxNode<Expr<'a>>),
    Binary(BinOp, BoxNode<Expr<'a>>, BoxNode<Expr<'a>>),
    Lambda(Ident<'a>, BoxNode<Expr<'a>>),
    Call(BoxNode<Expr<'a>>, Vec<Node<Expr<'a>>>),
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
}
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum UnOp {
    Pos,
    Neg,
    Not,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Literal {
    Num(usize),
    Bool(bool),
    Unit,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Node<T: Clone> {
    value: T,
    span: Range<usize>,
}
