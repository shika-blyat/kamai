use std::ops::Range;

use crate::syntax::tokens::TokenKind;
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
    None,
    OneOf(Vec<Expected>),
}

#[derive(Debug)]
pub struct SyntaxErr<'a> {
    pub span: Range<usize>,
    pub kind: SyntaxErrKind<'a>,
    pub expected: Expected,
    pub note: Option<HelpNote>,
}

#[derive(Debug)]
pub enum SyntaxErrKind<'a> {
    UnexpectedToken(TokenKind<'a>),
    Unclosed(Delimiter),
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
        }
        match note {
            Some(s) => diag = diag.with_notes(vec![s.to_string()]),
            None => (),
        }
        match expected {
            Expected::Item => {
                diag = diag.with_labels(vec![
                    Label::primary((), span).with_message("expected an item declaration")
                ])
            }
            Expected::Expr => {
                diag = diag.with_labels(vec![
                    Label::primary((), span).with_message("expected an expression")
                ])
            }
            Expected::OneOf(_) | Expected::None => (),
        }
        diag
    }
}
