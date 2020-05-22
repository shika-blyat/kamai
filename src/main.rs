#![feature(or_patterns)]
// TODO return an error instead of panicking in block_inference (the unwrap)
// TODO write tests
// TODO Merge block and semicolon inference into a single pass
// TODO write the parser
// TODO Create the error struct in syntax_err and integrate it with codespan

mod errors;
mod syntax;
mod utils;

use std::{
    io::{self, Write},
    ops::Range,
};

use logos::Logos;
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
            Token::Semicolon => {
                print!("{}\n{}", tok, " ".repeat(indent_level * 4));
            }
            Token::LBrace => {
                indent_level += 1;
                print!("{{\n{}", " ".repeat(indent_level * 4));
            }
            t => print!("{} ", t),
        }
    }
}

fn block_inference<'a>(
    tokens: impl IntoIterator<Item = (Token<'a>, Range<usize>)>,
) -> Vec<(Token<'a>, Range<usize>)> {
    let mut result_vec = vec![];
    let mut context_stack: Vec<usize> = vec![];
    let mut last_newline = 0;
    let mut iter = tokens.into_iter().peekable();
    while let Some((tok, span)) = iter.next() {
        match tok {
            t @ Token::Newline => {
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
                result_vec.push((t, span));
            }
            t @ Token::Eq => {
                let (_, last_span) = result_vec.last().unwrap();
                context_stack.push(last_span.start - last_newline);
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

fn semicolon_inference<'a>(
    v: impl IntoIterator<Item = (Token<'a>, Range<usize>)>,
) -> Vec<(Token<'a>, Range<usize>)> {
    let mut result_vec = vec![];
    let mut can_close_instr = false;
    let mut iter = v.into_iter().peekable();
    while let Some((tok, span)) = iter.next() {
        match tok {
            Token::Newline => {
                if can_close_instr {
                    match iter.peek() {
                        Some((Token::Op(_) | Token::Then | Token::Else, _)) => (),
                        _ => result_vec.push((Token::Semicolon, span)),
                    }
                }
            }
            t
            @
            (Token::Op(_)
            | Token::If
            | Token::Else
            | Token::Then
            | Token::LBrace
            | Token::RBrace) => {
                can_close_instr = false;
                result_vec.push((t, span))
            }
            t => {
                can_close_instr = true;
                result_vec.push((t, span));
            }
        }
    }
    result_vec
}
fn main() {
    let code = "
a = 
    5
    if 2
    then 4
    else 2 
a = 3 + 2 * 3
 - 24 
5";
    let lex = Token::lexer(code);
    let vec = semicolon_inference(block_inference(lex.spanned()));
    let tokens: Vec<&Token<'_>> = vec.iter().map(|(t, _)| t).collect();
    pretty_print_tokens(tokens.as_slice());
}
