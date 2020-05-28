// TODO replace `String`s by a real error type

use std::convert::{TryFrom, TryInto};

use super::tokens::{Token, TokenKind};

pub struct Layout<'a, I: IntoIterator<Item = Token<'a>>> {
    pub tokens: I,
}
impl<'a, I: IntoIterator<Item = Token<'a>>> Layout<'a, I> {
    fn can_be_after_semicolon(tok: &TokenKind) -> Result<bool, String> {
        match tok {
            TokenKind::Then => Ok(false),
            TokenKind::Op(_) => Ok(false), //todo handle ambiguity between `+` and `-` unary/binary by raising an error
            _ => Ok(true),
        }
    }
    fn can_close_instr(tok: &TokenKind) -> bool {
        debug_assert_ne!(tok, &TokenKind::Newline);
        match tok {
            TokenKind::Op(_) | TokenKind::Eq | TokenKind::Else | TokenKind::Then => false,
            _ => true,
        }
    }
    // todo fix ranges used by inserted semicolons/braces
    pub fn into_insensitive(self) -> Result<Vec<Token<'a>>, String> {
        let mut result_vec = vec![];
        let mut contexts = Contexts { stack: vec![] };
        let mut last_newline = 0;
        let mut can_close_instr = false;

        let mut iterator = self.tokens.into_iter().peekable();
        while let Some(tok) = iterator.next() {
            contexts.close_contexts(&mut result_vec, &tok, last_newline, can_close_instr);
            let Token { kind, span } = tok;
            match kind {
                TokenKind::Newline => {
                    last_newline = span.start;
                    match iterator.peek() {
                        // Yes it's ugly, it just does nothing if we're not supposed to close the current instruction, any improvements are welcome :)
                        Some(tok)
                            if !(Self::can_be_after_semicolon(&tok.kind)? && can_close_instr) =>
                        {
                            ()
                        }
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
                    can_close_instr = Self::can_close_instr(&kind);
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
                kind => {
                    can_close_instr = Self::can_close_instr(&kind);
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
}

pub struct Contexts {
    pub stack: Vec<Context>,
}
impl Contexts {
    /// closes all contexts that can be closed by the given tokn, and inserts the right braces/semicolon as needed
    pub fn close_contexts(
        &mut self,
        vec: &mut Vec<Token>,
        tok: &Token,
        newline: usize,
        can_close_instr: bool,
    ) {
        while let Some(ctx) = self.stack.last() {
            if ctx.is_closed_by(&tok.kind, tok.span.start, newline) {
                // if the instr can be closed and we push a `}`, we should push an enclosing `;`
                // However, it's usually not the case, because there was already a newline before the token triggering the closing of the instruction.
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
                // After a right brace, we insert a `;`, because it's just easier for us in most cases
                vec.push(Token {
                    kind: TokenKind::Semicolon,
                    span: 0..0,
                });
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

#[derive(Debug)]
pub struct Context {
    pub opener: ContextOpener,
    pub newline: usize,
    pub column: usize,
}

impl Context {
    /// checks if the given token closes this context
    pub fn is_closed_by(&self, tok: &TokenKind, tok_start: usize, tok_newline: usize) -> bool {
        let tok_column = tok_start - tok_newline;
        let less_indented = match self.opener {
            ContextOpener::Equal => self.column > tok_column,
            _ => self.column >= tok_column,
        };
        less_indented
            || self.opener == ContextOpener::Then // We don't want our one liner `if expr then expr else expr` to be parsed as `if expr then {expr else {expr}}`
                && tok == &TokenKind::Else
                && self.newline == tok_newline
    }
}

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
