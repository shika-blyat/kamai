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
    let code = "a = if 2 
                          then
                            if 2 then 3 else if True then 4 else 4
                          else a = 3
                                   24
                          5";
    let lex = TokenKind::lexer(code).spanned();
    let tokens: Vec<Token<'_>> = lex
        .into_iter()
        .map(|(kind, span)| Token { kind, span })
        .collect();
    let vec = Layout { tokens }.into_insensitive().expect(
        "J'suis en train de faire un proto donc j'utilise except
    ",
    );
    pretty_print_tokens(vec.iter());
}
