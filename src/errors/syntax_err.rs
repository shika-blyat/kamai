use std::{fmt, ops::Range};

use crate::syntax::{ast::Expr, tokens::TokenKind};
use codespan_reporting::diagnostic::{Diagnostic, Label};

#[derive(Debug)]
pub enum Delimiter {
    Paren,
    Brace,
}
type HelpNote = &'static str;

#[derive(Debug)]
pub enum Expected {
    Item,
    Expr,
    Operator,
    Semicolon,
    None,
    OneOf(Vec<Expected>),
}
impl fmt::Display for Expected {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expected::Item => write!(f, "an item"),
            Expected::Expr => write!(f, "an expression"),
            Expected::Operator => write!(f, "an operator"),
            Expected::Semicolon => write!(f, "a `;`"),
            Expected::OneOf(v) => {
                write!(f, "one of [")?;
                for e in v.iter().take(v.len() - 1) {
                    write!(f, "{}, ", e)?;
                }
                write!(f, " {}]", v.last().unwrap())
            }
            Expected::None => write!(f, ""),
        }
    }
}
#[derive(Debug)]
pub struct SyntaxErr<'a> {
    pub span: Range<usize>,
    pub kind: SyntaxErrKind<'a>,
    pub expected: Expected,
    pub note: Option<HelpNote>,
}

#[derive(Debug)]
pub enum AmbiguousOp {
    Add,
    Sub,
}
impl fmt::Display for AmbiguousOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AmbiguousOp::Add => write!(f, "`+`"),
            AmbiguousOp::Sub => write!(f, "`-`"),
        }
    }
}

#[derive(Debug)]
pub enum SyntaxErrKind<'a> {
    UnexpectedToken(TokenKind<'a>),
    Unclosed(Delimiter),
    UnexpectedExpr(Expr<'a>),
    AmbiguousInstructions(AmbiguousOp),
    UnexpectedEOF,
}
impl<'a> From<SyntaxErr<'a>> for Diagnostic<()> {
    fn from(
        SyntaxErr {
            span,
            kind,
            expected,
            note,
        }: SyntaxErr<'a>,
    ) -> Self {
        let mut diag = Diagnostic::error();
        match kind {
            SyntaxErrKind::UnexpectedToken(t) => {
                diag = diag.with_message(format!("Unexpected {:#?}", t));
            }
            SyntaxErrKind::UnexpectedEOF => {
                diag = diag.with_message(format!("Unexpected EOF"));
            }
            SyntaxErrKind::Unclosed(delimiter) => {
                diag = diag.with_message(format!("Unclosed {:#?}", delimiter));
            }
            SyntaxErrKind::UnexpectedExpr(_) => {
                diag = diag.with_message(format!("Unexpected expression"))
            }
            SyntaxErrKind::AmbiguousInstructions(amb_op) => {
                diag = diag.with_message(format!(
                    "Ambiguity: potentially instruction closer operator {} after newline at the same level than the previous instruction is not allowed", amb_op
                ))
            }
        }
        match note {
            Some(s) => diag = diag.with_notes(vec![s.to_string()]),
            None => (),
        }
        match expected {
            Expected::None => (),
            _ => {
                diag = diag.with_labels(vec![
                    Label::primary((), span).with_message(format!("Expected {}", expected))
                ])
            }
        }
        diag
    }
}
