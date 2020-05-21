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

    #[regex(r"[\t ]+", |lex| lex.slice().chars().fold(0, |counter, c| if c == ' ' { counter + 1} else {counter + 4}), priority = 2)]
    Space(usize),

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
            Token::Fn => write!(f, "fn"),
            Token::Continue => write!(f, "continue"),
            Token::Break => write!(f, "break"),
            Token::Return => write!(f, "return"),
            Token::If => write!(f, "if"),
            Token::Then => write!(f, "then"),
            Token::Else => write!(f, "else"),
            Token::While => write!(f, "while"),
            Token::Eq => write!(f, "="),
            Token::Op(s) => write!(f, "{}", s),
            Token::Error => write!(f, "Error"),
            Token::Space(_) | Token::Newline => Ok(()),
        }
    }
}
