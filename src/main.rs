#![feature(or_patterns)]
// TODO write the parser
// TODO Add error handling for lexing

#[macro_use]
extern crate lazy_static;

mod errors;
mod syntax;
mod utils;

use codespan_reporting::{
    files::SimpleFile,
    term::{
        self,
        termcolor::{ColorChoice, StandardStream},
    },
};
use logos::Logos;

use errors::syntax_err::SyntaxErr;

use syntax::{
    ast::{Expr, Node},
    insensitive_layout::*,
    parser::Parser,
    tokens::{Token, TokenKind},
};

fn parse<'a>(code: &'a str) -> Result<Node<Expr<'a>>, SyntaxErr<'a>> {
    let lex = TokenKind::lexer(code);
    let block_tokens = block_inference(lex.spanned().map(|t| Token::from_tuple(t)))?;
    println!("{:#?}", block_tokens);
    Parser::new(block_tokens.into_iter()).operation()
}
fn main() {
    let code = "
    a * 2 + 3
    ";
    let expr = parse(code);
    match expr {
        Ok(expr) => {
            println!("{:#?}", expr);
        }
        Err(e) => {
            let file = SimpleFile::new("main.ka", code);
            let writer = StandardStream::stderr(ColorChoice::Always);
            let config = codespan_reporting::term::Config::default();
            term::emit(&mut writer.lock(), &config, &file, &e.into())
                .expect("Failed to write on stdout");
        }
    }
}
