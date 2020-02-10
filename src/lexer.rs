use std::{iter::Peekable, ops::Range, str::Chars};
// Variant prefixed by a `K` are keyword
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    KFun,
    KIf,
    KElse,
    KLet,
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    Identifier(String),
    String(String),
    Number(f64),
    Bool(bool),
}

#[derive(Debug, PartialEq)]
pub enum ErrorKind {
    Eof,
    Failure,
    ParsingError(ParserError),
}

#[derive(Debug, PartialEq)]
pub struct ParserError {
    reason: String,
    range: Range<usize>,
}

pub struct Lexer {
    current: usize,
}

impl<'a> Lexer {
    pub fn new() -> Self {
        Self { current: 0 }
    }
    pub fn tokenize(mut self, input: &'a str) -> Result<Vec<Token>, ErrorKind> {
        let mut tokens = vec![];
        let input = input.chars().collect::<Vec<char>>();
        while let Some(c) = self.current(&input) {
            println!("{:#?}", tokens);
            if c.is_ascii_digit() {
                match self.take_num(&input) {
                    Ok(x) => tokens.push(x),
                    Err(e) => match e {
                        ErrorKind::ParsingError(_) => return Err(e),
                        _ => unreachable!(),
                    },
                }
            } else if c.is_alphabetic() {
                tokens.push(
                    self.take_bool(&input)
                        .or_else(|_| self.take_keyword(&input))
                        .or_else(|_| self.take_identifier(&input))
                        .unwrap(),
                );
            } else if c == '\"' {
                tokens.push(self.take_string(&input)?);
            } else if c == '{' {
                self.advance(1);
                tokens.push(Token::LeftBracket);
            } else if c == '}' {
                self.advance(1);
                tokens.push(Token::RightBracket);
            } else if c == '(' {
                self.advance(1);
                tokens.push(Token::LeftParen);
            } else if c == ')' {
                self.advance(1);
                tokens.push(Token::RightParen);
            } else if c.is_whitespace() {
                self.advance(1);
            }
        }
        Ok(tokens)
    }
    fn take_bool(&mut self, input: &Vec<char>) -> Result<Token, ErrorKind> {
        if self.startswith(input, "True") {
            if input.len() > self.current + 4 {
                if !input[self.current + 4].is_ascii_alphanumeric() {
                    self.advance(4);
                    return Ok(Token::Bool(true));
                }
            } else {
                self.advance(4);
                return Ok(Token::Bool(true));
            }
        }
        if self.startswith(input, "False") {
            if input.len() > self.current + 5 {
                if !input[self.current + 5].is_ascii_alphanumeric() {
                    self.advance(5);
                    return Ok(Token::Bool(false));
                }
            } else {
                self.advance(5);
                return Ok(Token::Bool(false));
            }
        }
        Err(ErrorKind::Failure)
    }
    fn take_string(&mut self, input: &Vec<char>) -> Result<Token, ErrorKind> {
        let mut string = String::new();
        self.advance(1);
        while let Some(c) = self.current(&input) {
            if c == '\"' {
                self.advance(1);
                return Ok(Token::String(string));
            } else if c == '\\' {
                self.advance(1);
                match self.current(&input) {
                    Some(c) => match c {
                        'n' => {
                            string.push('\n');
                            self.advance(1);
                            continue;
                        }
                        'r' => {
                            string.push('\r');
                            self.advance(1);
                            continue;
                        }
                        '\"' => {
                            string.push('\"');
                            self.advance(1);
                            continue;
                        }
                        't' => {
                            string.push('\t');
                            self.advance(1);
                            continue;
                        }
                        'u' => {
                            return Err(ErrorKind::ParsingError(ParserError {
                                reason: format!("Unicode escape sequence isn't available yet"),
                                range: self.current - 1..self.current,
                            }))
                        }
                        _ => (),
                    },
                    None => (),
                }
                return Err(ErrorKind::ParsingError(ParserError {
                    reason: format!("Unknown escape sequence"),
                    range: self.current - 1..self.current,
                }));
            } else {
                string.push(c);
                self.advance(1);
            }
        }
        Err(ErrorKind::ParsingError(ParserError {
            reason: format!("Unclosed string delimiter"),
            range: self.current - 1..self.current,
        }))
    }
    fn take_identifier(&mut self, input: &Vec<char>) -> Result<Token, ErrorKind> {
        let mut ident = self.current(input).unwrap().to_string();
        self.advance(1);
        while let Some(c) = self.current(&input) {
            if c.is_ascii_alphanumeric() || c == '_' || c == '\'' {
                ident.push(c);
                self.advance(1);
            } else {
                break;
            }
        }
        Ok(Token::Identifier(ident))
    }
    fn take_keyword(&mut self, input: &Vec<char>) -> Result<Token, ErrorKind> {
        if self.startswith(&input, "let ") {
            self.advance(4);
            return Ok(Token::KLet);
        } else if self.startswith(&input, "fun ") {
            self.advance(4);
            return Ok(Token::KFun);
        } else if self.startswith(&input, "if ") {
            self.advance(3);
            return Ok(Token::KIf);
        } else if input.len() >= self.current + 4 {
            let after_else = input[self.current + 4];
            if self.startswith(&input, "else") && !after_else.is_ascii_alphanumeric() {
                self.advance(4);
                return Ok(Token::KElse);
            }
        } else if self.startswith(&input, "else") {
            self.advance(4);
            return Ok(Token::KElse);
        }
        Err(ErrorKind::Failure)
    }
    fn take_num(&mut self, input: &Vec<char>) -> Result<Token, ErrorKind> {
        let mut num = String::new();
        let mut cptr = 1;
        while let Some(c) = self.current(&input) {
            if c.is_digit(10) {
                num.push(c);
                self.advance(1);
                cptr += 1;
            } else {
                if c.is_alphabetic() {
                    return Err(ErrorKind::ParsingError(ParserError {
                        reason: format!("Invalid digit"),
                        range: self.current..self.current + cptr + 1,
                    }));
                } else {
                    break;
                }
            }
        }
        Ok(Token::Number(num.parse::<f64>().unwrap()))
    }
    fn startswith(&self, input: &Vec<char>, s: &str) -> bool {
        if input.len() < s.len() {
            return false;
        }
        input[self.current..self.current + s.len()]
            .iter()
            .collect::<String>()
            .as_str()
            == s
    }
    fn current(&self, input: &Vec<char>) -> Option<char> {
        if self.current < input.len() {
            Some(input[self.current])
        } else {
            None
        }
    }
    fn next(&self, input: &Vec<char>) -> Option<char> {
        if self.current + 1 < input.len() {
            Some(input[self.current + 1])
        } else {
            None
        }
    }
    fn last(&self, input: &Vec<char>) -> char {
        input[self.current - 1]
    }
    fn advance(&mut self, num: usize) {
        self.current += num;
    }
}
