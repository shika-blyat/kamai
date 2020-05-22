use std::fmt;

use logos::Logos;

#[derive(Logos, Debug, PartialEq)]
pub enum Token<'a> {
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

impl<'a> fmt::Display for Token<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Number(n) => write!(f, "{}", n),
            Token::Ident(s) => write!(f, "{}", s),
            Token::Bool(b) => write!(f, "{}", b),
            Token::Unit => write!(f, "()"),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::LBrace => write!(f, "{{"),
            Token::RBrace => write!(f, "}}"),
            Token::Return => write!(f, "return"),
            Token::If => write!(f, "if"),
            Token::Then => write!(f, "then"),
            Token::Else => write!(f, "else"),
            Token::Eq => write!(f, "="),
            Token::Semicolon => write!(f, ";"),
            Token::Op(s) => write!(f, "{}", s),
            Token::Error => write!(f, "Error"),
            Token::Newline => write!(f, "\n"),
        }
    }
}
