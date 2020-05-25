use std::{fmt, ops::Range};

use logos::Logos;

#[derive(Debug, Clone, PartialEq)]
pub struct Token<'a> {
    pub kind: TokenKind<'a>,
    pub span: Range<usize>,
}
impl<'a> Token<'a> {
    pub fn from_tuple((kind, span): (TokenKind<'a>, Range<usize>)) -> Self {
        Self { kind, span }
    }
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

    #[token("return")]
    Return,

    #[token("if")]
    If,

    #[token("then")]
    Then,

    #[token("else")]
    Else,

    #[token(";")]
    Semicolon,

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
            TokenKind::Return => write!(f, "return"),
            TokenKind::If => write!(f, "if"),
            TokenKind::Then => write!(f, "then"),
            TokenKind::Else => write!(f, "else"),
            TokenKind::Eq => write!(f, "="),
            TokenKind::Semicolon => write!(f, ";"),
            TokenKind::Op(s) => write!(f, "{}", s),
            TokenKind::Error => write!(f, "Error"),
            TokenKind::Newline => write!(f, "\n"),
        }
    }
}

#[allow(dead_code)]
pub fn pretty_print_tokens<'a>(tokens: impl IntoIterator<Item = &'a Token<'a>>) {
    let mut indent_level = 0;
    for tok in tokens {
        match &tok.kind {
            TokenKind::RBrace => {
                indent_level -= 1;
                let indentation = " ".repeat(indent_level * 4);
                print!("\n{}}}\n{}", indentation, indentation,);
            }
            TokenKind::Semicolon => {
                print!("{}\n{}", tok.kind, " ".repeat(indent_level * 4));
            }
            TokenKind::LBrace => {
                indent_level += 1;
                print!("{{\n{}", " ".repeat(indent_level * 4));
            }
            t => print!("{} ", t),
        }
    }
}
