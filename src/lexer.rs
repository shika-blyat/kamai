use std::{iter::Peekable, ops::Range, str::Chars};
// Variant prefixed by a `K` are keyword
#[derive(Debug, Clone, PartialEq)]
pub enum Token<'a> {
    KFun,
    KIf,
    KElse,
    KLet,
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    String(&'a str),
    Number(f64),
    Bool(bool),
}

#[derive(Debug, PartialEq)]
pub enum ErrorKind {
    Eof,
    ParsingError(ParserError),
}

#[derive(Debug, PartialEq)]
pub struct ParserError {
    reason: String,
    range: Range<usize>,
}

pub struct Lexer<'a> {
    current: usize,
    input: &'a str,
}
impl<'a> Lexer<'a> {
    pub fn tokenize(&mut self) -> Result<Vec<Token>, ErrorKind> {
        let mut tokens = vec![];
        loop {
            match self.take_token() {
                Ok(x) => tokens.push(x),
                Err(e) => match e {
                    ErrorKind::Eof => break,
                    _ => return Err(e),
                },
            }
        }
        Ok(tokens)
    }
    fn take_token(&mut self) -> Result<Token, ErrorKind> {
        match self.input[.next()] {
            None => return Err(ErrorKind::Eof),
            Some(c) => {
                if c.is_digit(10) {
                    self.take_num(c)
                } else {
                    unreachable!()
                }
            }
        }
    }
    fn take_num(&mut self, first: char) -> Result<Token, ErrorKind> {
        let mut num = first.to_string();
        while let Some(c) = self.input.peek() {
            if c.is_digit(10) {
                num.push(*c);
                self.input().next();
            } else {
                break;
            }
        }
        Ok(Token::Number(num.parse::<f64>().unwrap()))
    }
}
