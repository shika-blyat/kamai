#![feature(or_patterns)]

mod errors;
mod syntax;
mod utils;

use std::{
    io::{self, Write},
    ops::Range,
};

use logos::{Lexer, Logos};

use errors::syntax_err;
use syntax::tokens::Token;

fn pretty_print_tokens<'a>(tokens: &[&'_ Token<'a>]) {
    let mut indent_level = 0;
    for tok in tokens {
        match tok {
            Token::RBrace => {
                indent_level -= 1;
                let indentation = " ".repeat(indent_level * 4);
                print!("\n{}}}\n{}", indentation, indentation,);
            }
            Token::LBrace => {
                indent_level += 1;
                print!("{{\n{}", " ".repeat(indent_level * 4));
            }
            t => print!("{} ", t),
        }
    }
}
#[derive(PartialEq, Debug, Clone, Copy)]
enum OpenTok {
    Eq,
    ThenElse,
    __Newline,
}
impl<'a> From<&'a Token<'a>> for OpenTok {
    fn from(t: &'a Token) -> Self {
        match t {
            Token::Eq => Self::Eq,
            Token::Then | Token::Else => Self::ThenElse,
            Token::Newline => OpenTok::__Newline,
            _ => panic!("Cannot build an OpenTok from a {:#?}", t),
        }
    }
}

fn into_insensitive<'a>(tokens: Lexer<'a, Token<'a>>) -> Vec<(Token<'a>, Range<usize>)> {
    let mut result_vec = vec![];
    let mut context_stack: Vec<(usize, OpenTok)> = vec![];
    let mut last_newline = 0;
    let mut iter = tokens.into_iter().spanned().peekable();
    while let Some((tok, span)) = iter.next() {
        match tok {
            Token::Space(_) => (),
            t @ (Token::Eq | Token::Then | Token::Else | Token::Newline) => {
                if let Some((Token::Space(spaces), span)) = iter.peek() {
                    while let Some((n, open_tok)) = context_stack.last() {
                        if spaces <= n && (open_tok == &OpenTok::from(&t) || t == Token::Newline) {
                            result_vec.push((Token::RBrace, span.start..span.start));
                            context_stack.pop();
                        } else {
                            break;
                        }
                    }
                    let (_, span) = iter.next().unwrap();
                    if t == Token::Newline {
                        last_newline = span.start;
                        continue;
                    }
                }
                context_stack.push((span.start - last_newline, OpenTok::from(&t)));
                result_vec.push((t, span.clone()));
                result_vec.push((Token::LBrace, span.end..span.end));
            }
            t => result_vec.push((t, span)),
        }
    }
    if result_vec.len() != 0 {
        let end = result_vec.last().unwrap().1.end;
        for _ in context_stack {
            result_vec.push((Token::RBrace, end..end));
        }
    }
    io::stdout().flush().unwrap();
    result_vec
}

fn main() {
    //FIXME fix this:
    let code = "a = if 2
    then
      if 2 then 3
    else a = 3
             24
    5";
    let lex = Token::lexer(code);
    let vec = into_insensitive(lex);
    println!("{:#?}", vec);
    let tokens: Vec<&Token<'_>> = vec.iter().map(|(t, _)| t).collect();
    pretty_print_tokens(tokens.as_slice());
}
#[test]
fn test_braces() {
    let vec = vec![
        (Token::Ident("a"), 5..6),
        (Token::Eq, 7..8),
        (Token::LBrace, 8..8),
        (Token::Then, 9..13),
        (Token::LBrace, 13..13),
        (Token::Number(2), 14..15),
        (Token::RBrace, 20..20),
        (Token::Else, 16..20),
        (Token::LBrace, 20..20),
        (Token::Number(3), 21..22),
        (Token::RBrace, 23..23),
        (Token::RBrace, 23..23),
    ];
    assert_eq!(
        vec,
        into_insensitive(Token::lexer(
            "
    a = then 2 else 3
    ",
        )),
    );
    /*let vec2 = vec![
        (Token::Ident("a"), 0..1),
        (Token::Eq, 2..3),
        (Token::LBrace, 3..3),
        (Token::If, 4..6),
        (Token::Number(2), 7..8),
        (Token::Then, 13..17),
        (Token::LBrace, 17..17),
        (Token::If, 24..26),
        (Token::Number(2), 27..28),
        (Token::RBrace, 33..33),
        (Token::Then, 29..33),
        (Token::LBrace, 33..33),
        (Token::Number(3), 34..35),
        (Token::RBrace, 36..36),
        (Token::Else, 40..44),
        (Token::LBrace, 44..44),
        (Token::Ident("a"), 45..46),
        (Token::Eq, 47..48),
        (Token::LBrace, 48..48),
        (Token::Number(3), 49..50),
        (Token::Number(24), 64..66),
        (Token::RBrace, 67..67),
        (Token::RBrace, 67..67),
        (Token::Number(5), 71..72),
        (Token::RBrace, 72..72),
    ];
    assert_eq!(
        into_insensitive(Token::lexer(
            "a = if 2
    then
      if 2 then 3
    else a = 3
             24
    5"
        )),
        vec2
    );*/
}
