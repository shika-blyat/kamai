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
    errors::{ErrorReason, ParserError},
};

fn into_insensitive_internal(
    tokens: &[Token],
    layout_level: usize,
) -> Result<(usize, Vec<Token>), ParserError> {
    // Todo: mutate an unique vector instead of constructing new ones at
    // each recursion and even appending them to the existing one.
    let mut new_tokens = vec![];
    let mut k = 0;
    while k < tokens.len() {
        let tok = &tokens[k];
        match tok.kind {
            TokenKind::Where => {
                new_tokens.push(tok.clone());
                if tokens[k + 1].char_level <= layout_level {
                    new_tokens.push(Token::new(TokenKind::LBracket, 0, 0, 0, String::new()));
                    new_tokens.push(Token::new(TokenKind::RBracket, 0, 0, 0, String::new()));
                } else {
                    new_tokens.push(Token::new(TokenKind::LBracket, 0, 0, 0, String::new()));
                    let (new_idx, mut where_block) =
                        into_insensitive_internal(&tokens[k + 1..], tokens[k + 1].char_level)?;
                    k += new_idx;
                    new_tokens.append(&mut where_block);
                    new_tokens.push(Token::new(TokenKind::RBracket, 0, 0, 0, String::new()));
                }
            }
            TokenKind::Equal => {
                if tokens[k + 1].char_level < layout_level {
                    dbg!(layout_level);
                    dbg!(&tokens[k]);
                    dbg!(&tokens[k + 1]);
                    return Err(ParserError::new(
                        ErrorReason::EmptyFunction,
                        tok.line,
                        tok.char_level,
                        1,
                    ));
                } else {
                    new_tokens.push(tok.clone());
                    new_tokens.push(Token::new(TokenKind::LBracket, 0, 0, 0, String::new()));
                    let (new_idx, mut where_block) =
                        into_insensitive_internal(&tokens[k + 1..], tokens[k + 1].char_level)?;
                    k += new_idx;
                    new_tokens.append(&mut where_block);
                    new_tokens.push(Token::new(TokenKind::RBracket, 0, 0, 0, String::new()));
                }
            }
            _ => {
                if tokens[k + 1].char_level == layout_level && tokens[k + 1].kind != TokenKind::EOF
                {
                    new_tokens.push(Token::new(TokenKind::Semicolon, 0, 0, 0, String::new()));
                } else if tokens[k + 1].char_level < layout_level {
                }
                new_tokens.push(tok.clone());
                if tokens[k + 1].kind == TokenKind::EOF {
                    return Ok((k, new_tokens));
                }
            }
        }
        dbg!(k);
        k += 1;
    }
    Ok((k, new_tokens))
}

/// Refers to the module level documentation for an explanation of the purpose of this function
pub fn into_insensitive(tokens: &[Token]) -> Result<Vec<Token>, ParserError> {
    let (final_idx, new_tokens) = into_insensitive_internal(tokens, 1)?;
    if final_idx < tokens.len() {
        return Err(ParserError::new(
            ErrorReason::LayoutError,
            tokens[final_idx].line,
            tokens[final_idx].char_level,
            1,
        ));
    }
    Ok(new_tokens)
}
