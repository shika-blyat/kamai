use std::{fmt, ops::Range};

use logos::Logos;

#[derive(Debug, Clone)]
pub struct Token<'a> {
    pub kind: TokenKind<'a>,
    pub span: Range<usize>,
}
#[derive(Logos, Debug, Clone, PartialEq)]
pub enum TokenKind<'a> {
    #[regex("[0-9]+", |lex| lex.slice().parse())]
    Number(i64),

    #[regex("(true|false)", |lex| lex.slice().parse())]
    Bool(bool),

    #[regex(r"[A-z_][\w_]*")]
    Ident(&'a str),

    #[token("()")]
    Unit,

    #[token("(")]
    LParen,

    #[token(")")]
    RParen,

    #[token("{")]
    LBrace,

    #[token("}")]
    RBrace,

    #[token(";")]
    Semicolon,

    #[token("fn")]
    Fn,

    #[token("continue")]
    Continue,

    #[token("break")]
    Break,

    #[token("return")]
    Return,

    #[token("if")]
    If,

    #[token("then")]
    Then,

    #[token("else")]
    Else,

    #[token("while")]
    While,

    #[regex("\n+")]
    Newline,

    #[token("=")]
    Eq,

    #[regex(r"(\+|-|\*|/|&&|\|\||<=|>=|>|<|==|!=|!)")]
    Op(&'a str),

    #[regex(r"[ \f\t]+", logos::skip)]
    #[error]
    Error,
}

impl<'a> fmt::Display for TokenKind<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Number(n) => write!(f, "{}", n),
            TokenKind::Ident(s) => write!(f, "{}", s),
            TokenKind::Bool(b) => write!(f, "{}", b),
            TokenKind::Unit => write!(f, "()"),
            TokenKind::LParen => write!(f, "("),
            TokenKind::RParen => write!(f, ")"),
            TokenKind::LBrace => write!(f, "{{"),
            TokenKind::RBrace => write!(f, "}}"),
            TokenKind::Fn => write!(f, "fn"),
            TokenKind::Continue => write!(f, "continue"),
            TokenKind::Break => write!(f, "break"),
            TokenKind::Return => write!(f, "return"),
            TokenKind::If => write!(f, "if"),
            TokenKind::Then => write!(f, "then"),
            TokenKind::Else => write!(f, "else"),
            TokenKind::While => write!(f, "while"),
            TokenKind::Eq => write!(f, "="),
            TokenKind::Op(s) => write!(f, "{}", s),
            TokenKind::Error => write!(f, "Error"),
            TokenKind::Semicolon => write!(f, ";"),
            TokenKind::Newline => Ok(()),
        }
    }
}
