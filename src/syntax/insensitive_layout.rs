use std::convert::{TryFrom, TryInto};

use super::tokens::{Token, TokenKind};

#[derive(Debug, PartialEq)]
pub enum ContextOpener {
    Then,
    Else,
    Equal,
}
impl TryFrom<&'_ TokenKind<'_>> for ContextOpener {
    type Error = ();
    fn try_from(tok: &'_ TokenKind) -> Result<Self, ()> {
        Ok(match tok {
            TokenKind::Eq => ContextOpener::Equal,
            TokenKind::Else => ContextOpener::Else,
            TokenKind::Then => ContextOpener::Then,
            _ => return Err(()),
        })
    }
}

#[derive(Debug)]
pub struct Context {
    pub opener: ContextOpener,
    pub newline: usize,
    pub column: usize,
}

impl Context {
    pub fn is_closed_by(&self, tok: &TokenKind, start: usize, newline: usize) -> bool {
        let column = start - newline;
        let less_indented = match self.opener {
            ContextOpener::Equal => self.column > column,
            _ => self.column >= column,
        };
        less_indented
            || self.opener == ContextOpener::Then
                && tok == &TokenKind::Else
                && self.newline == newline
    }
}

pub struct Contexts {
    pub stack: Vec<Context>,
}
impl Contexts {
    pub fn close_contexts(
        &mut self,
        vec: &mut Vec<Token>,
        tok: &TokenKind,
        start: usize,
        newline: usize,
        can_close_instr: bool,
    ) {
        while let Some(ctx) = self.stack.last() {
            if ctx.is_closed_by(tok, start, newline) {
                if can_close_instr {
                    vec.push(Token {
                        kind: TokenKind::Semicolon,
                        span: 0..0,
                    });
                }
                vec.push(Token {
                    kind: TokenKind::RBrace,
                    span: 0..0,
                });
                if let ContextOpener::Else | ContextOpener::Equal = ctx.opener {
                    vec.push(Token {
                        kind: TokenKind::Semicolon,
                        span: 0..0,
                    });
                }
                self.stack.pop();
            } else {
                break;
            }
        }
    }
    pub fn push(&mut self, ctx: Context) {
        self.stack.push(ctx);
    }
}
fn can_be_after_semicolon(tok: &TokenKind) -> Result<bool, String> {
    match tok {
        TokenKind::Then => Ok(false),
        TokenKind::Op(_) => Ok(false), //todo handle ambiguity between `+` and `-` unary/binary by raising an error
        _ => Ok(true),
    }
}
// todo fix ranges used by inserted semicolons/braces
pub fn into_insensitive<'a>(
    tokens: impl IntoIterator<Item = Token<'a>>,
) -> Result<Vec<Token<'a>>, String> {
    let mut result_vec = vec![];
    let mut iterator = tokens.into_iter().peekable();
    let mut contexts = Contexts { stack: vec![] };
    let mut last_newline = 0;
    let mut can_close_instr = false;
    while let Some(Token { kind, span }) = iterator.next() {
        contexts.close_contexts(
            &mut result_vec,
            &kind,
            span.start,
            last_newline,
            can_close_instr,
        );
        match kind {
            TokenKind::Newline => {
                last_newline = span.start;
                match iterator.peek() {
                    Some(tok) if !(can_be_after_semicolon(&tok.kind)? && can_close_instr) => (),
                    _ => {
                        can_close_instr = false;
                        result_vec.push(Token {
                            span: 0..0,
                            kind: TokenKind::Semicolon,
                        });
                    }
                }
            }
            kind @ (TokenKind::Then | TokenKind::Else | TokenKind::Eq) => {
                can_close_instr = false;
                let column = match &kind {
                    TokenKind::Eq => match iterator.peek() {
                        Some(Token { span, .. }) => span.start - last_newline,
                        None => span.end - last_newline,
                    },
                    _ => span.start - last_newline,
                };
                contexts.push(Context {
                    opener: (&kind).try_into().unwrap(),
                    column,
                    newline: last_newline,
                });
                result_vec.push(Token {
                    span: span.clone(),
                    kind,
                });
                result_vec.push(Token {
                    span,
                    kind: TokenKind::LBrace,
                });
            }
            TokenKind::Op(_) => {
                can_close_instr = true;
                result_vec.push(Token { span, kind })
            }
            kind => {
                can_close_instr = true;
                result_vec.push(Token { span, kind })
            }
        }
    }
    for _ in contexts.stack {
        result_vec.push(Token {
            span: 0..0,
            kind: TokenKind::Semicolon,
        });
        result_vec.push(Token {
            span: 0..0,
            kind: TokenKind::RBrace,
        });
        result_vec.push(Token {
            span: 0..0,
            kind: TokenKind::Semicolon,
        });
    }
    Ok(result_vec)
}
