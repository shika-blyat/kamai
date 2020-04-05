use std::ops::Range;

/// All token currently existing in the lexer. Is stored in a [Token][Token] object
#[derive(Debug, PartialEq, Clone)]
pub enum TokenKind {
    Num(isize),
    Identifier(String),
    Op(String),
    Equal,
    LParen,
    RParen,
    LBracket,
    RBracket,
    Semicolon,
    Where,
    EOF,
}

/// Token objet. Holds a [TokenKind][TokenKind] and some metadata about the token itself
#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    tok: TokenKind,
    line: usize,
    c: usize,
    size: usize,
    lexeme: String,
}

impl Token {
    pub fn new(tok: TokenKind, line: usize, c: usize, size: usize, lexeme: String) -> Self {
        Self {
            tok,
            line,
            c,
            size,
            lexeme,
        }
    }
}
