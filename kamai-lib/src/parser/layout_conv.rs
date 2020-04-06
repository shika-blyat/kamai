//! Layout conversion utilities
//!
//!
//! The purpose of this module is to convert a layout (indentation) sensitive stream of tokens
//! into an insensitive one, by adding brackets ( `{`, `}`) and semicolons around the content of functions and where blocks
//! A simple example:
//! ```haskell
//! add x y = z
//!         where z = x + y
//! ```
//! should become
//! ```haskell
//! add x y = {
//!     z;
//!     where {
//!         z = {x + y};
//!     }
//! }
//! ```
use super::{
    ast_tokens::{Token, TokenKind},
    errors::{PErrorReason, ParserError},
};

fn into_insensitive(tokens: Vec<Token>) -> Result<Vec<Token>, ParserError> {
    let mut new_tokens = vec![];
    for tok in tokens {
        match tok {
            _ => unreachable!(),
        }
    }
    Ok(new_tokens)
}
