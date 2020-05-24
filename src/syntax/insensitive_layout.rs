use std::io::{self, Write};

use crate::{
    errors::syntax_err::*,
    syntax::tokens::{Token, TokenKind},
};

pub fn block_inference<'a>(
    tokens: impl IntoIterator<Item = Token<'a>>,
) -> Result<Vec<Token<'a>>, SyntaxErr<'a>> {
    let mut iter = tokens.into_iter().peekable();
    let mut result_vec = vec![];
    let mut context_stack: Vec<usize> = vec![];
    let mut can_close_instr = false;
    let mut last_newline = 0;
    while let Some(Token { kind, span }) = iter.next() {
        match kind {
            TokenKind::Newline => {
                last_newline = span.start;
                if let Some(Token { span, kind }) = iter.peek() {
                    let start_next = span.start - last_newline;
                    while let Some(n) = context_stack.last() {
                        if *n >= start_next {
                            context_stack.pop();
                            can_close_instr = false;
                            result_vec.push(Token {
                                kind: TokenKind::RBrace,
                                span: start_next..start_next,
                            });
                            result_vec.push(Token {
                                kind: TokenKind::Semicolon,
                                span: span.clone(),
                            })
                        } else {
                            break;
                        }
                    }
                    if can_close_instr {
                        match kind {
                            TokenKind::Op(_) | TokenKind::Then | TokenKind::Else => (),
                            _ => {
                                result_vec.push(Token {
                                    kind: TokenKind::Semicolon,
                                    span: span.clone(),
                                });
                            }
                        }
                    }
                } else {
                    if can_close_instr {
                        result_vec.push(Token {
                            kind: TokenKind::Semicolon,
                            span: span.clone(),
                        });
                    }
                }
            }
            TokenKind::Eq => {
                let Token {
                    span: last_span, ..
                } = result_vec.last().ok_or_else(|| SyntaxErr {
                    kind: SyntaxErrKind::UnexpectedToken(TokenKind::Eq),
                    span: span.clone(),
                    expected: Expected::Item,
                    note: Some("Maybe you meant to declare an item?"),
                })?;
                context_stack.push(last_span.end - last_newline);
                let start = span.start;
                result_vec.push(Token { kind, span });
                result_vec.push(Token {
                    kind: TokenKind::LBrace,
                    span: start..start,
                });
            }
            TokenKind::Op(_)
            | TokenKind::If
            | TokenKind::Else
            | TokenKind::Then
            | TokenKind::LBrace
            | TokenKind::Semicolon => {
                can_close_instr = false;
                result_vec.push(Token { kind, span })
            }
            _ => {
                can_close_instr = true;
                result_vec.push(Token { kind, span });
            }
        }
    }
    if result_vec.len() != 0 {
        let end = result_vec.last().unwrap().span.end;
        for _ in context_stack {
            result_vec.push(Token {
                kind: TokenKind::RBrace,
                span: end..end,
            });
        }
    }
    if can_close_instr {
        result_vec.push(Token {
            kind: TokenKind::Semicolon,
            span: result_vec.last().unwrap().span.clone(),
        });
    }
    io::stdout().flush().unwrap();
    Ok(result_vec)
}

mod test {
    #![allow(unused_imports)]
    use super::*;
    use logos::Logos;
    #[test]
    fn func_decl() {
        let vec = vec![
            Token {
                kind: TokenKind::Ident("a"),
                span: 1..2,
            },
            Token {
                kind: TokenKind::Eq,
                span: 3..4,
            },
            Token {
                kind: TokenKind::LBrace,
                span: 3..3,
            },
            Token {
                kind: TokenKind::Number(5),
                span: 5..6,
            },
            Token {
                kind: TokenKind::Semicolon,
                span: 11..13,
            },
            Token {
                kind: TokenKind::If,
                span: 11..13,
            },
            Token {
                kind: TokenKind::Number(2),
                span: 14..15,
            },
            Token {
                kind: TokenKind::Then,
                span: 20..24,
            },
            Token {
                kind: TokenKind::Number(4),
                span: 25..26,
            },
            Token {
                kind: TokenKind::Else,
                span: 31..35,
            },
            Token {
                kind: TokenKind::Number(2),
                span: 36..37,
            },
            Token {
                kind: TokenKind::RBrace,
                span: 1..1,
            },
            Token {
                kind: TokenKind::Semicolon,
                span: 38..39,
            },
            Token {
                kind: TokenKind::Ident("a"),
                span: 38..39,
            },
            Token {
                kind: TokenKind::Eq,
                span: 40..41,
            },
            Token {
                kind: TokenKind::LBrace,
                span: 40..40,
            },
            Token {
                kind: TokenKind::Number(3),
                span: 42..43,
            },
            Token {
                kind: TokenKind::Op("+"),
                span: 44..45,
            },
            Token {
                kind: TokenKind::Number(2),
                span: 46..47,
            },
            Token {
                kind: TokenKind::Op("*"),
                span: 48..49,
            },
            Token {
                kind: TokenKind::Number(3),
                span: 50..51,
            },
            Token {
                kind: TokenKind::RBrace,
                span: 2..2,
            },
            Token {
                kind: TokenKind::Semicolon,
                span: 53..54,
            },
            Token {
                kind: TokenKind::Op("-"),
                span: 53..54,
            },
            Token {
                kind: TokenKind::Number(24),
                span: 55..57,
            },
            Token {
                kind: TokenKind::Semicolon,
                span: 58..59,
            },
            Token {
                kind: TokenKind::Number(5),
                span: 58..59,
            },
        ];
        let code = "
a = 5
    if 2
    then 4
    else 2
a = 3 + 2 * 3
 - 24
5";
        let lex = TokenKind::lexer(code);
        let result = block_inference(lex.spanned().map(|t| Token::from_tuple(t))).unwrap();
        for (t1, t2) in vec.into_iter().zip(result.into_iter()) {
            assert_eq!(t1, t2);
        }
    }
    #[test]
    fn one_line_if() {
        let vec = vec![
            Token {
                kind: TokenKind::Ident("a"),
                span: 5..6,
            },
            Token {
                kind: TokenKind::Eq,
                span: 7..8,
            },
            Token {
                kind: TokenKind::LBrace,
                span: 7..7,
            },
            Token {
                kind: TokenKind::If,
                span: 9..11,
            },
            Token {
                kind: TokenKind::Number(2),
                span: 12..13,
            },
            Token {
                kind: TokenKind::Then,
                span: 14..18,
            },
            Token {
                kind: TokenKind::Number(3),
                span: 19..20,
            },
            Token {
                kind: TokenKind::Else,
                span: 21..25,
            },
            Token {
                kind: TokenKind::Number(4),
                span: 26..27,
            },
            Token {
                kind: TokenKind::Semicolon,
                span: 36..37,
            },
            Token {
                kind: TokenKind::Number(2),
                span: 36..37,
            },
            Token {
                kind: TokenKind::RBrace,
                span: 6..6,
            },
            Token {
                kind: TokenKind::Semicolon,
                span: 43..44,
            },
            Token {
                kind: TokenKind::Number(3),
                span: 43..44,
            },
            Token {
                kind: TokenKind::Semicolon,
                span: 44..45,
            },
        ];
        let code = "
    a = if 2 then 3 else 4
        2
     3
    ";
        let lex = TokenKind::lexer(code);
        let result = block_inference(lex.spanned().map(|t| Token::from_tuple(t))).unwrap();
        for (t1, t2) in vec.into_iter().zip(result.into_iter()) {
            assert_eq!(t1, t2);
        }
    }
    #[test]
    fn nested_func() {
        let vec = vec![
            Token {
                kind: TokenKind::Ident("a"),
                span: 5..6,
            },
            Token {
                kind: TokenKind::Eq,
                span: 7..8,
            },
            Token {
                kind: TokenKind::LBrace,
                span: 7..7,
            },
            Token {
                kind: TokenKind::Number(5),
                span: 9..10,
            },
            Token {
                kind: TokenKind::Semicolon,
                span: 19..20,
            },
            Token {
                kind: TokenKind::Ident("a"),
                span: 19..20,
            },
            Token {
                kind: TokenKind::Eq,
                span: 21..22,
            },
            Token {
                kind: TokenKind::LBrace,
                span: 21..21,
            },
            Token {
                kind: TokenKind::Number(8),
                span: 23..24,
            },
            Token {
                kind: TokenKind::Op("*"),
                span: 25..26,
            },
            Token {
                kind: TokenKind::Number(2),
                span: 27..28,
            },
            Token {
                kind: TokenKind::Semicolon,
                span: 39..40,
            },
            Token {
                kind: TokenKind::Number(5),
                span: 39..40,
            },
            Token {
                kind: TokenKind::RBrace,
                span: 9..9,
            },
            Token {
                kind: TokenKind::Semicolon,
                span: 49..50,
            },
            Token {
                kind: TokenKind::Number(2),
                span: 49..50,
            },
            Token {
                kind: TokenKind::Semicolon,
                span: 50..51,
            },
            Token {
                kind: TokenKind::RBrace,
                span: 51..51,
            },
            Token {
                kind: TokenKind::Semicolon,
                span: 51..51,
            },
        ];
        let code = "
    a = 5
        a = 8 * 2
          5
        2
    ";
        let lex = TokenKind::lexer(code);
        let result = block_inference(lex.spanned().map(|t| Token::from_tuple(t))).unwrap();
        for (t1, t2) in vec.into_iter().zip(result.into_iter()) {
            assert_eq!(t1, t2);
        }
    }
}
