#[test]
fn simple_tokens() {
    use crate::parser::lexer::{Lexer, Token, TokenElem};
    let mut lexer = Lexer::new(" ab_cd_e ".to_string());
    assert_eq!(
        lexer.tokenize().unwrap(),
        vec![Token::new(
            TokenElem::Identifier("ab_cd_e".to_string()),
            1..8,
            "ab_cd_e".to_string(),
        )],
    );
    let mut lexer = Lexer::new(" () ".to_string());
    assert_eq!(
        lexer.tokenize().unwrap(),
        vec![Token::new(TokenElem::Unit, 1..3, "()".to_string())]
    );
    let mut lexer = Lexer::new(" 15 ".to_string());
    assert_eq!(
        lexer.tokenize().unwrap(),
        vec![Token::new(TokenElem::Int(15), 1..3, "15".to_string())],
    );

    let mut lexer = Lexer::new(" = ".to_string());
    assert_eq!(
        lexer.tokenize().unwrap(),
        vec![Token::new(TokenElem::Equal, 1..2, "=".to_string())],
    );

    let mut lexer = Lexer::new(" ; ".to_string());
    assert_eq!(
        lexer.tokenize().unwrap(),
        vec![Token::new(TokenElem::Semicolon, 1..2, ";".to_string())],
    );

    let mut lexer = Lexer::new(" (5) ".to_string());
    assert_eq!(
        lexer.tokenize().unwrap(),
        vec![Token::new(
            TokenElem::ParenthesisPair(vec![Token::new(TokenElem::Int(5), 2..3, "5".to_string())]),
            1..3,
            "(5)".to_string(),
        )],
    );

    let mut lexer = Lexer::new(" { 5; 15 } ".to_string());
    assert_eq!(
        lexer.tokenize().unwrap(),
        vec![Token::new(
            TokenElem::BracketPair(vec![
                Token::new(TokenElem::Int(5), 3..4, "5".to_string()),
                Token::new(TokenElem::Semicolon, 4..5, ";".to_string()),
                Token::new(TokenElem::Int(15), 6..8, "15".to_string())
            ]),
            1..9,
            "{ 5; 15 }".to_string(),
        )],
    );

    let mut lexer = Lexer::new(" +/*- ".to_string());
    assert_eq!(
        lexer.tokenize().unwrap(),
        vec![
            Token::new(TokenElem::Op("+".to_string()), 1..2, "+".to_string()),
            Token::new(TokenElem::Op("/".to_string()), 2..3, "/".to_string()),
            Token::new(TokenElem::Op("*".to_string()), 3..4, "*".to_string()),
            Token::new(TokenElem::Op("-".to_string()), 4..5, "-".to_string()),
        ],
    );
}

#[test]
fn unmatching() {
    use crate::parser::lexer::{Lexer, LexerError};
    let mut lexer = Lexer::new(" 1 + 2)".to_string());
    assert_eq!(
        lexer.tokenize(),
        Err(LexerError::new(
            "Unmatched closing parenthesis".to_string(),
            6..7,
        ))
    );
    let mut lexer = Lexer::new(" 1 + 2}".to_string());
    assert_eq!(
        lexer.tokenize(),
        Err(LexerError::new(
            "Unmatched closing bracket".to_string(),
            6..7,
        ))
    )
}
