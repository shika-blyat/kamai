#![feature(trait_alias)]

mod ast;
mod lexer;
mod parser;
mod typechecker;
use lexer::Lexer;

fn main() {
    let lexer = Lexer::new();
    println!("{:#?}", lexer.tokenize(r#"fun a(){ print("abc\nabc")}"#));
}
