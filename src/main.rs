mod parser;
mod typechecker;

use parser::{lexer::Lexer, parse::Parser};

// TODO: typechecking
fn main() {
    let s = "
    {x + y;}
";
    let mut lexer = Lexer::new(s.to_string());
    let tokens = lexer.tokenize();
    println!("{:#?}", tokens);
    let tokens = tokens.unwrap();
    let mut parser = Parser::new(s.to_string()).unwrap();
    println!("{:#?}", parser.expr())
}
