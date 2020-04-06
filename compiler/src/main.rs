#![feature(int_error_matching)]
use kamai::parser;

use parser::lexer::Lexer;

fn main() {
    let lexer = Lexer::new(
        "
x = a + b
  where a = 2 + 5
        b = 5",
    );
    let tokens = lexer.tokenize();
    println!("{:#?}", tokens);
}
