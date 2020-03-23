#![allow(clippy::range_plus_one)]
use std::ops::Range;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenElem {
    Int(isize),
    Identifier(String),
    Equal,
    Op(String),
    Semicolon,
    BracketPair(Vec<Token>),
    ParenthesisPair(Vec<Token>),
}
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub elem: TokenElem,
    pub range: Range<usize>,
    pub lexeme: String,
}

impl Token {
    pub fn new(elem: TokenElem, range: Range<usize>, lexeme: String) -> Self {
        Self {
            elem,
            range,
            lexeme,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct LexerError {
    reason: String,
    range: Range<usize>,
}

impl LexerError {
    pub fn new(reason: String, range: Range<usize>) -> Self {
        Self { reason, range }
    }
}

pub struct Lexer {
    code: Vec<char>,
    current: usize,
    is_inside_parenthesis: bool,
    is_inside_brackets: bool,
}

impl Lexer {
    /// Create a new lexer object, taking the code to tokenize in argument
    pub fn new(code: String) -> Self {
        Self {
            code: code.chars().collect(),
            current: 0,
            is_inside_brackets: false,
            is_inside_parenthesis: false,
        }
    }

    // The tokenize method is the core of the lexer logic, and is recursively called when an opening brakcet/parenthesis is encountered
    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexerError> {
        let mut tokens = vec![];
        loop {
            match self.current() {
                Some(c) => {
                    if c.is_ascii_digit() {
                        tokens.push(self.consume_num());
                        continue;
                    } else if c.is_ascii_alphabetic() {
                        tokens.push(self.consume_identifier());
                        continue;
                    } else if c.is_whitespace() {
                        self.current += 1;
                        continue;
                    }
                    match c {
                        '=' => tokens.push(Token::new(
                            TokenElem::Equal,
                            self.current..self.current + 1,
                            "=".to_string(),
                        )),
                        '(' => {
                            self.is_inside_parenthesis = true;
                            tokens.push(self.consume_parenthesis()?)
                        }
                        ')' => {
                            // If the method was recursively called because it encountered an opening bracket, then is_inside_brackets will be true
                            if self.is_inside_parenthesis {
                                self.is_inside_parenthesis = false;
                                return Ok(tokens);
                            } else {
                                return Err(LexerError::new(
                                    "Unmatched closing parenthesis".to_string(),
                                    self.current..self.current + 1,
                                ));
                            }
                        }
                        '{' => {
                            self.is_inside_brackets = true;
                            tokens.push(self.consume_brackets()?)
                        }
                        '}' => {
                            // If the method was recursively called because it encountered an opening bracket, then is_inside_brackets will be true
                            if self.is_inside_brackets {
                                self.is_inside_brackets = false;
                                return Ok(tokens);
                            } else {
                                return Err(LexerError::new(
                                    "Unmatched closing bracket".to_string(),
                                    self.current..self.current + 1,
                                ));
                            }
                        }
                        ';' => tokens.push(Token::new(
                            TokenElem::Semicolon,
                            self.current..self.current + 1,
                            c.to_string(),
                        )),
                        '+' | '-' | '/' | '*' => tokens.push(Token::new(
                            TokenElem::Op(c.to_string()),
                            self.current..self.current + 1,
                            c.to_string(),
                        )),
                        c => {
                            return Err(LexerError::new(
                                format!("Unexpected character {}", c),
                                self.current..self.current + 1,
                            ))
                        }
                    }
                    self.current += 1;
                }
                None => return Ok(tokens),
            }
        }
    }

    pub fn consume_num(&mut self) -> Token {
        let mut num = self.current().unwrap().to_string();
        self.current += 1;
        while let Some(c) = self.current() {
            if c.is_ascii_digit() {
                num.push(c)
            } else {
                break;
            }
            self.current += 1;
        }
        let num_len = num.len();
        let num_c = num.clone();
        /*let num = match num.parse(){
            Ok(num) => num,
            Err(_) =>
        };*/
        Token::new(
            TokenElem::Int(num.parse().unwrap()),
            self.current - num_len..self.current,
            num_c,
        )
    }

    pub fn consume_brackets(&mut self) -> Result<Token, LexerError> {
        let start = self.current;
        self.current += 1;
        Ok(Token::new(
            TokenElem::BracketPair(self.tokenize()?),
            start..self.current,
            self.code[start..self.current + 1]
                .iter()
                .collect::<String>(),
        ))
    }

    pub fn consume_parenthesis(&mut self) -> Result<Token, LexerError> {
        let start = self.current;
        self.current += 1;
        Ok(Token::new(
            TokenElem::ParenthesisPair(self.tokenize()?),
            start..self.current,
            self.code[start..self.current + 1]
                .iter()
                .collect::<String>(),
        ))
    }

    pub fn consume_identifier(&mut self) -> Token {
        let mut ident = self.current().unwrap().to_string();
        self.current += 1;
        while let Some(c) = self.current() {
            if c.is_ascii_alphanumeric() || c == '_' {
                ident.push(c)
            } else {
                break;
            }
            self.current += 1;
        }
        let ident_len = ident.len();
        let ident_c = ident.clone();
        match ident.as_str() {
            _ => Token::new(
                TokenElem::Identifier(ident),
                self.current - ident_len..self.current,
                ident_c,
            ),
        }
    }

    pub fn current(&self) -> Option<char> {
        if self.is_empty() {
            None
        } else {
            Some(self.code[self.current])
        }
    }

    pub fn is_empty(&self) -> bool {
        self.code.len() <= self.current
    }
}
