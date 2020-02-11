use crate::{
    ast::{Expr, Literal},
    lexer::{ParserError, Token},
};

pub trait Parser<T> = Fn(Vec<Token>) -> ParserResult<T>;

pub enum ParserResult<T> {
    Ok(T, Vec<Token>),
    Failure(Vec<Token>),
    Error(ParserError),
    Empty,
}

impl<T> ParserResult<T> {
    pub fn or_else<F: FnOnce(Vec<Token>) -> Self>(self, f: F) -> Self {
        match self {
            Self::Ok(_, _) => self,
            Self::Failure(tokens) => f(tokens),
            Self::Error(_) => self,
        }
    }
    pub fn and_then<F: FnOnce(T, Vec<Token>) -> Self>(self, f: F) -> Self {
        match self {
            Self::Ok(token, tokens) => f(token, tokens),
            Self::Failure(tokens) => Self::Failure(tokens),
            Self::Error(_) => self,
        }
    }
}

pub fn many1<T>(predicate: impl Parser<Expr>) -> impl Parser<Vec<Expr>> {
    move |tokens| {
        let mut result_tokens = vec![];
        match predicate(tokens) {
            ParserResult::Ok(token, mut tokens) => {
                result_tokens.push(token);
                let mut result = predicate(tokens);
                while let ParserResult::Ok(token, rem_tokens) = result {
                    result_tokens.push(token);
                    tokens = rem_tokens;
                    result = predicate(tokens);
                }
                match result {
                    ParserResult::Failure(tokens) => ParserResult::Ok(result_tokens, tokens),
                    ParserResult::Error(e) => ParserResult::Error(e),
                    ParserResult::Empty => ParserResult::Ok(result_tokens, tokens),
                    _ => unreachable!(),
                }
            }
            ParserResult::Error(e) => ParserResult::Error(e),
            ParserResult::Failure(e) => ParserResult::Failure(e),
            ParserResult::Empty =>  ParserResult::Empty,
        }
    }
}

fn atom(tokens: Vec<Token>) -> impl Parser<Expr> {
    |tokens: Vec<Token>| {
        if tokens.is_empty(){
            return ParserResult::Empty;
        }
        match tokens[0]{
            Token::Bool()
        }
    }
}
