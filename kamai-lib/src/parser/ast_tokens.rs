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
    pub kind: TokenKind,
    pub line: usize,
    pub char_level: usize,
    pub size: usize,
    pub lexeme: String,
}

impl Token {
    pub fn new(
        kind: TokenKind,
        line: usize,
        char_level: usize,
        size: usize,
        lexeme: String,
    ) -> Self {
        Self {
            kind,
            line,
            char_level,
            size,
            lexeme,
        }
    }
}
