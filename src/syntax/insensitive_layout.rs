// TODO track the column of the last instruction and use it to disambiguate and restrict layout's rule
use std::convert::{TryFrom, TryInto};

use crate::{
    errors::syntax_err::SyntaxErr,
    syntax::tokens::{Token, TokenKind},
};

pub struct Layout<'a, I: IntoIterator<Item = Token<'a>>> {
    tokens: I,
}
impl<'a, I> Layout<'a, I>
where
    I: IntoIterator<Item = Token<'a>>,
{
    pub fn new(tokens: I) -> Self {
        Self { tokens }
    }
    pub fn into_insensitive(self) -> Result<Vec<Token<'a>>, SyntaxErr<'a>> {
        let mut result_vec = vec![];
        let mut contexts = Contexts { stack: vec![] };
        let mut layout_state = LayoutState {
            last_newline: 0,
            can_close_instr: false,
            just_closed: true,
            last_opened_instr_column: 0,
        };
        let mut iterator = self.tokens.into_iter().peekable();
        while let Some(tok) = iterator.next() {
            contexts.close_contexts(&mut result_vec, &tok, &mut layout_state);
            let Token { kind, span } = tok;
            match &kind {
                TokenKind::Newline => {
                    layout_state.last_newline = span.start;
                    if iterator.peek().is_none()
                        || (Self::can_be_after_semicolon(&iterator.peek().unwrap().kind)?
                            && layout_state.can_close_instr)
                    {
                        layout_state.can_close_instr = false;
                        layout_state.just_closed = true;
                        result_vec.push(Token {
                            span: span.end..span.end,
                            kind: TokenKind::Semicolon,
                        });
                    }
                }
                TokenKind::Then | TokenKind::Else | TokenKind::Eq => {
                    let column = match &kind {
                        TokenKind::Eq => match iterator.peek() {
                            Some(Token { span, .. }) => span.start - layout_state.last_newline,
                            None => span.end - layout_state.last_newline,
                        },
                        _ => span.start - layout_state.last_newline,
                    };
                    Self::update_layout(
                        &mut layout_state,
                        Token {
                            kind: kind.clone(),
                            span: span.clone(),
                        },
                    );
                    contexts.push(Context {
                        opener: (&kind).try_into().expect(
                            format!("Failed to convert to ContextOpener a {:#?}", kind).as_str(),
                        ),
                        column,
                        newline: layout_state.last_newline,
                    });
                    let span = span.end..span.end;
                    result_vec.push(Token {
                        span: span.clone(),
                        kind,
                    });
                    result_vec.push(Token {
                        span,
                        kind: TokenKind::LBrace,
                    });
                }
                _ => {
                    let tok = Token { span, kind };
                    Self::update_layout(&mut layout_state, tok.clone());
                    result_vec.push(tok)
                }
            }
        }
        for _ in contexts.stack {
            result_vec.push(Token {
                span: std::usize::MAX..std::usize::MAX,
                kind: TokenKind::Semicolon,
            });
            result_vec.push(Token {
                span: std::usize::MAX..std::usize::MAX,
                kind: TokenKind::RBrace,
            });
            result_vec.push(Token {
                span: std::usize::MAX..std::usize::MAX,
                kind: TokenKind::Semicolon,
            });
        }
        Ok(result_vec)
    }
    fn update_layout(layout: &mut LayoutState, tok: Token) {
        layout.can_close_instr = Self::can_close_instr(&tok.kind);
        if layout.just_closed {
            layout.last_opened_instr_column = tok.span.start - layout.last_newline;
            layout.just_closed = false;
        }
    }
    fn can_be_after_semicolon(tok: &TokenKind) -> Result<bool, SyntaxErr<'a>> {
        match tok {
            TokenKind::Then => Ok(false),
            TokenKind::Op(_) => Ok(false), //todo handle ambiguity between `+` and `-` unary/binary by raising an error
            _ => Ok(true),
        }
    }
    fn can_close_instr(tok: &TokenKind) -> bool {
        debug_assert_ne!(tok, &TokenKind::Newline);
        match tok {
            TokenKind::Op(_)
            | TokenKind::Eq
            | TokenKind::Else
            | TokenKind::Then
            | TokenKind::Semicolon => false,
            _ => true,
        }
    }
}

pub struct LayoutState {
    pub just_closed: bool,
    pub can_close_instr: bool,
    pub last_opened_instr_column: usize,
    pub last_newline: usize,
}
impl LayoutState {}

pub struct Contexts {
    pub stack: Vec<Context>,
}
impl Contexts {
    /// closes all contexts that can be closed by the given tokn, and inserts the right braces/semicolon as needed
    pub fn close_contexts(
        &mut self,
        vec: &mut Vec<Token>,
        tok: &Token,
        layout_state: &mut LayoutState,
    ) {
        while let Some(ctx) = self.stack.last() {
            if ctx.is_closed_by(&tok.kind, tok.span.start, layout_state.last_newline) {
                // if the instr can be closed and we push a `}`, we should push an enclosing `;`
                // However, it's usually not the case, because there was already a newline before the token triggering the closing of the instruction.
                let span = tok.span.end..tok.span.end;
                if layout_state.can_close_instr {
                    vec.push(Token {
                        kind: TokenKind::Semicolon,
                        span: span.clone(),
                    });
                }
                vec.push(Token {
                    kind: TokenKind::RBrace,
                    span: span.clone(),
                });
                // After a right brace, we insert a `;`, because it's just easier for us in most cases
                vec.push(Token {
                    kind: TokenKind::Semicolon,
                    span,
                });
                layout_state.just_closed = true;
                layout_state.can_close_instr = false;
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

#[allow(unused_imports)]
mod test {
    use super::*;
    use logos::Logos;
    #[test]
    fn complex_case() {
        let tokens = vec![
            TokenKind::Ident("a"),
            TokenKind::Eq,
            TokenKind::LBrace,
            TokenKind::If,
            TokenKind::Number(2),
            TokenKind::Then,
            TokenKind::LBrace,
            TokenKind::If,
            TokenKind::Number(2),
            TokenKind::Then,
            TokenKind::LBrace,
            TokenKind::Number(3),
            TokenKind::Semicolon,
            TokenKind::RBrace,
            TokenKind::Semicolon,
            TokenKind::Else,
            TokenKind::LBrace,
            TokenKind::If,
            TokenKind::Ident("True"),
            TokenKind::Then,
            TokenKind::LBrace,
            TokenKind::Number(4),
            TokenKind::Semicolon,
            TokenKind::RBrace,
            TokenKind::Semicolon,
            TokenKind::Else,
            TokenKind::LBrace,
            TokenKind::Number(4),
            TokenKind::Semicolon,
            TokenKind::RBrace,
            TokenKind::Semicolon,
            TokenKind::RBrace,
            TokenKind::Semicolon,
            TokenKind::RBrace,
            TokenKind::Semicolon,
            TokenKind::Else,
            TokenKind::LBrace,
            TokenKind::Ident("a"),
            TokenKind::Eq,
            TokenKind::LBrace,
            TokenKind::Number(3),
            TokenKind::Semicolon,
            TokenKind::Number(24),
            TokenKind::Semicolon,
            TokenKind::RBrace,
            TokenKind::Semicolon,
            TokenKind::Number(5),
            TokenKind::Semicolon,
            TokenKind::RBrace,
            TokenKind::Semicolon,
            TokenKind::Number(5),
            TokenKind::Semicolon,
            TokenKind::RBrace,
            TokenKind::Semicolon,
        ];
        let code = "a = if 2 
        then
          if 2 then 3 else if True then 4 else 4
        else a = 3
                 24
             5
        5";
        let test_tokens: Vec<Token<'_>> = TokenKind::lexer(code)
            .spanned()
            .into_iter()
            .map(|(kind, span)| Token { kind, span })
            .collect();
        let vec: Vec<TokenKind<'_>> = Layout::new(test_tokens)
            .into_insensitive()
            .unwrap()
            .into_iter()
            .map(|Token { kind, .. }| kind)
            .collect();
        for (tok1, tok2) in vec.into_iter().zip(tokens.into_iter()) {
            assert_eq!(tok1, tok2)
        }
    }
}
