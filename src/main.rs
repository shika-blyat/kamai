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

fn parse<'a>(code: &'a str) -> Result<Node<Expr<'a>>, Vec<SyntaxErr<'a>>> {
    let lex = TokenKind::lexer(code);
    let block_tokens =
        block_inference(lex.spanned().map(|t| Token::from_tuple(t))).map_err(|e| vec![e])?;
    println!("{:#?}", block_tokens);
    let mut parser = Parser::new(block_tokens.into_iter());
    match parser.expr() {
        Ok(e) => {
            if parser.errors.is_empty() {
                Ok(e)
            } else {
                Err(parser.errors)
            }
        }
        Err(e) => {
            parser.errors.push(e);
            Err(parser.errors)
        }
    }
}

fn main() {
    //FIXME a * (2 + 3) doens't work, investigate why
    let code = "
    a * (2 + 3)
    ";
    let expr = parse(code);
    match expr {
        Ok(expr) => {
            println!("ast: {:#?}", expr);
        }
        Err(errors) => {
            let file = SimpleFile::new("main.ka", code);
            let writer = StandardStream::stderr(ColorChoice::Always);
            let config = codespan_reporting::term::Config::default();
            for err in errors {
                term::emit(&mut writer.lock(), &config, &file, &err.into())
                    .expect("Failed to write on stdout");
            }
        }
    }
}
