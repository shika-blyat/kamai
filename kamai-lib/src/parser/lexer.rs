//! Lexer module
//!
//!
//! The lexer module stores the definition of the [Lexer][Lexer] object, build it from a [&str][str]
//! and then call [tokenize][Lexer::tokenize] when you want to convert the given string into a token vector.

use super::{
    ast_tokens::{Token, TokenKind},
    errors::{ErrorReason, ParserError},
    layout_conv::into_insensitive,
};
use std::{iter::Peekable, num::IntErrorKind, str::Chars};

// Todo: allow lexers to be reused by reinitializing all fields

/// Lexer object, see the module level documentation for more information
pub struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
    code: &'a str,
    current_c: usize,
    line: usize,
    counter: usize,
}

impl<'a> Lexer<'a> {
    /// Build a new `Lexer` object, setting all counters (current char, line, and counter) to 1
    pub fn new(s: &'a str) -> Self {
        Self {
            chars: s.chars().peekable(),
            current_c: 1,
            line: 1,
            code: s,
            counter: 1,
        }
    }
    /// Consume the `Lexer` by building a `Token` vector from the inner string, returning a [ParserError](super::errors::ParserError)
    /// if any error happens
    pub fn tokenize(mut self) -> Result<Vec<Token>, ParserError> {
        let mut tokens = vec![];
        loop {
            let next = self.next();
            if next.is_none() {
                tokens.push(Token::new(
                    TokenKind::EOF,
                    self.line,
                    self.current_c,
                    1,
                    String::new(),
                ));
                return Ok(into_insensitive(&tokens)?);
            }
            let c = next.unwrap();
            if c.is_ascii_alphabetic() {
                tokens.push(self.identifier(c));
                continue;
            } else if c.is_ascii_digit() {
                tokens.push(self.int(c)?);
                continue;
            } else if c.is_whitespace() {
                // updating newline counter is already done in the body of the `next` method
                continue;
            }
            match c {
                '+' | '*' | '/' | '-' => {
                    tokens.push(self.build_tok(TokenKind::Op(c.to_string()), 1, c.to_string()))
                }
                '=' => tokens.push(self.build_tok(TokenKind::Equal, 1, c.to_string())),
                '(' => tokens.push(self.build_tok(TokenKind::LParen, 1, c.to_string())),
                ')' => tokens.push(self.build_tok(TokenKind::RParen, 1, c.to_string())),
                _ => return Err(self.build_err(ErrorReason::UnexpectedChar(c), 1)),
            }
        }
    }
    /// Parse an integer
    fn int(&mut self, first_c: char) -> Result<Token, ParserError> {
        let mut num_s = first_c.to_string();
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                num_s.push(self.next().unwrap());
            } else {
                break;
            }
        }
        let num = match num_s.clone().parse::<isize>() {
            Ok(n) => n,
            Err(e) => match e.kind() {
                IntErrorKind::Empty | IntErrorKind::Zero | IntErrorKind::InvalidDigit => {
                    return Err(self.build_err(
                        ErrorReason::ICE(
                            Some(1001),
                            Some("Unexpected error while lexing num".to_string()),
                        ),
                        1,
                    ))
                }
                IntErrorKind::Overflow => {
                    let num_s_len = num_s.len();
                    return Err(self.build_err(ErrorReason::NumOverflow(num_s), num_s_len));
                }
                IntErrorKind::Underflow => {
                    let num_s_len = num_s.len();
                    return Err(self.build_err(ErrorReason::NumUnderflow(num_s), num_s_len));
                }
                _ => {
                    let reason = ErrorReason::ICE(
                        Some(1001),
                        Some("Unexpected error while lexing num due to unexhaustive int conversion's error handling".to_string()),
                    );
                    return Err(self.build_err(reason, 1));
                }
            },
        };
        Ok(Token::new(
            TokenKind::Num(num),
            self.line,
            self.current_c - num_s.len(),
            num_s.len(),
            num_s,
        ))
    }
    /// Parse an identifier
    fn identifier(&mut self, first_c: char) -> Token {
        let mut ident = first_c.to_string();
        while let Some(c) = self.peek() {
            if c.is_ascii_alphanumeric() {
                ident.push(self.next().unwrap());
            } else {
                break;
            }
        }
        match ident.as_str() {
            "where" => Token::new(TokenKind::Where, self.line, self.current_c - 5, 5, ident),
            _ => Token::new(
                TokenKind::Identifier(ident.clone()),
                self.line,
                self.current_c - ident.len(),
                ident.len(),
                ident,
            ),
        }
    }
    /// build an error from the given `reason` and a range_size representing the range of chars where the error happened.
    /// Assume that self.current_c is at the end of the range_size
    fn build_err(&self, reason: ErrorReason, range_size: usize) -> ParserError {
        ParserError::new(reason, self.line, self.current_c - range_size, range_size)
    }
    /// build a token from the given TokenKind, size and lexeme.
    /// Assume that self.current_c is at the end of the size
    fn build_tok(&self, tok: TokenKind, size: usize, lexeme: String) -> Token {
        Token::new(tok, self.line, self.current_c - size, size, lexeme)
    }
    fn peek(&mut self) -> Option<&char> {
        self.chars.peek()
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = char;
    fn next(&mut self) -> Option<char> {
        let c = self.chars.next();
        match c {
            Some('\n') => {
                self.line += 1;
                self.current_c = 0;
            }
            None => return c,
            _ => (),
        }
        self.current_c += 1;
        self.counter += 1;
        c
    }
}
