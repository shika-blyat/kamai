use std::ops::Range;

/// Struct representing a lexer error, holding the [reason][LErrorReason],
/// and informations about where the error happened
#[derive(Debug)]
pub struct LexerError {
    reason: LErrorReason,
    range_size: usize,
    line: usize,
    c: usize,
}
impl LexerError {
    pub fn new(reason: LErrorReason, line: usize, c: usize, range_size: usize) -> Self {
        Self {
            line,
            c,
            range_size,
            reason,
        }
    }
}
/// An enum representing all lexer error that could possibly happen while lexing.
/// Is stored in a [LexerError][LexerError] object
#[derive(Debug)]
pub enum LErrorReason {
    NumOverflow(String),
    NumUnderflow(String),
    UnexpectedChar(char),
    /// Internal Compiler Error, containg the error code associated to the ICE if known
    ICE(Option<i32>, Option<String>),
}
