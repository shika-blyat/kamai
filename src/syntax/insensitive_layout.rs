use std::convert::{TryFrom, TryInto};

use super::tokens::{Token, TokenKind};

#[derive(Debug)]
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
impl ContextOpener {
    pub fn can_be_closed_by(&self, _tok: &TokenKind) -> bool {
        true
    }
}

#[derive(Debug)]
pub struct Context {
    pub opener: ContextOpener,
    pub column: usize,
}

impl Context {
    pub fn is_closed_by(&self, tok: &TokenKind, column: usize) -> bool {
        let less_indented = match self.opener {
            ContextOpener::Equal => self.column > column,
            _ => self.column >= column,
        };
        less_indented && self.opener.can_be_closed_by(tok)
    }
}

pub struct Contexts {
    pub stack: Vec<Context>,
}
impl Contexts {
    pub fn close_contexts(&mut self, vec: &mut Vec<Token>, tok: &TokenKind, column: usize) {
        while let Some(ctx) = self.stack.last() {
            if ctx.is_closed_by(tok, column) {
                self.stack.pop();
                vec.push(Token {
                    kind: TokenKind::RBrace,
                    span: 0..0,
                })
            } else {
                break;
            }
        }
    }
    pub fn push(&mut self, ctx: Context) {
        self.stack.push(ctx);
    }
}
pub fn into_insensitive<'a>(
    tokens: impl IntoIterator<Item = Token<'a>>,
) -> Result<Vec<Token<'a>>, String> {
    let mut result_vec = vec![];
    let mut iterator = tokens.into_iter().peekable();
    let mut contexts = Contexts { stack: vec![] };
    let mut last_newline = 0;
    while let Some(Token { kind, span }) = iterator.next() {
        contexts.close_contexts(&mut result_vec, &kind, span.start - last_newline);
        match kind {
            TokenKind::Newline => {
                last_newline = span.start;
            }
            kind @ (TokenKind::Then | TokenKind::Else | TokenKind::Eq) => {
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
            kind => result_vec.push(Token { span, kind }),
        }
    }
    for _ in contexts.stack {
        result_vec.push(Token {
            span: 0..0,
            kind: TokenKind::RBrace,
        });
    }
    Ok(result_vec)
}
