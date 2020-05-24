#![feature(or_patterns)]
// TODO write the parser
// TODO Add error handling for lexing

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

use syntax::{
    insensitive_layout::*,
    tokens::{Token, TokenKind},
};

fn main() {
    let code = "
    a = 5
        a = 8 * 2
          5
        2
    ";
    let lex = TokenKind::lexer(code);
    let block_tokens = block_inference(lex.spanned().map(|t| Token::from_tuple(t)));
    match block_tokens {
        Ok(tokens) => {
            let tokens: Vec<&TokenKind<'_>> =
                tokens.iter().map(|Token { kind, .. }| kind).collect();
            pretty_print_tokens(tokens.as_slice());
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
