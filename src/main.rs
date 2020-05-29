#![feature(or_patterns)]

mod errors;
mod syntax;
mod utils;

use logos::Logos;

use syntax::{
    insensitive_layout::Layout,
    tokens::{Token, TokenKind},
};

fn pretty_print_tokens<'a>(tokens: impl IntoIterator<Item = &'a Token<'a>>) {
    let mut indent_level = 0;
    for tok in tokens {
        match &tok.kind {
            TokenKind::RBrace => {
                indent_level -= 1;
                let indentation = " ".repeat(indent_level * 4);
                print!("\n{}}}\n{}", indentation, indentation,);
            }
            TokenKind::LBrace => {
                indent_level += 1;
                print!("{{\n{}", " ".repeat(indent_level * 4));
            }
            t => print!("{} ", t),
        }
    }
}
fn main() {
    let code = "a = 1 + 2
                            * 3
                          if 2 then a else b";
    let lex = TokenKind::lexer(code).spanned();
    let tokens: Vec<Token<'_>> = lex
        .into_iter()
        .map(|(kind, span)| Token { kind, span })
        .collect();
    let vec = Layout::new(tokens).into_insensitive().unwrap();
    println!(
        "{:#?}",
        vec.iter()
            .map(|Token { kind, .. }| kind)
            .collect::<Vec<&'_ TokenKind<'_>>>()
    );
    pretty_print_tokens(vec.iter());
}
