mod parser;

use parser::{lexer::Lexer, parse::Parser};

// TODO: typechecking
fn main() {
    let mut lexer = Lexer::new(
        "
            {x + y;}
        "
        .to_string(),
    );
    let tokens = lexer.tokenize();
    println!("{:#?}", tokens);
    let tokens = tokens.unwrap();
    let mut parser = Parser::new(tokens);
    println!("{:#?}", parser.expr())
}
