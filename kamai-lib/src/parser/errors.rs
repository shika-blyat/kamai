use std::ops::Range;

/// Struct representing a lexer error, holding the [reason][LErrorReason],
/// and informations about where the error happened

#[derive(Debug)]
pub struct ParserError {
    reason: ErrorReason,
    range_size: usize,
    line: usize,
    c: usize,
}

#[derive(Debug)]
pub enum ErrorReason {
    LayoutError,
    EmptyFunction,
    NumOverflow(String),
    NumUnderflow(String),
    UnexpectedChar(char),
    /// Internal Compiler Error, containg the error code associated to the ICE if known
    ICE(Option<i32>, Option<String>),
}

impl ParserError {
    pub fn new(reason: ErrorReason, line: usize, c: usize, range_size: usize) -> Self {
        Self {
            line,
            c,
            range_size,
            reason,
        }
    }
}
