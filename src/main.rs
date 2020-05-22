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

fn into_insensitive<'a>(
    tokens: impl IntoIterator<Item = (Token<'a>, Range<usize>)>,
) -> Vec<(Token<'a>, Range<usize>)> {
    let mut result_vec = vec![];
    let mut context_stack: Vec<usize> = vec![];
    let mut last_newline = 0;
    let mut iter = tokens.into_iter().peekable();
    while let Some((tok, span)) = iter.next() {
        match tok {
            Token::Newline => {
                last_newline = span.start;
                if let Some((_, span)) = iter.peek() {
                    let start_next = span.start - last_newline;
                    while let Some(n) = context_stack.last() {
                        if *n >= start_next {
                            context_stack.pop();
                            result_vec.push((Token::RBrace, start_next..start_next));
                        } else {
                            break;
                        }
                    }
                }
            }
            t @ Token::Eq => {
                context_stack.push(span.end - last_newline);
                let start = span.start;
                result_vec.push((t, span));
                result_vec.push((Token::LBrace, start..start));
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
    let code = "
a = if 2
    then
      if 2 then 3 else 4
    else 2 
a = 3
    24
5";
    let lex = Token::lexer(code);
    let vec = into_insensitive(lex.spanned());
    // println!("{:#?}", vec);
    let tokens: Vec<&Token<'_>> = vec.iter().map(|(t, _)| t).collect();
    pretty_print_tokens(tokens.as_slice());
}
