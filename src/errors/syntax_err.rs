use std::ops::Range;

use crate::syntax::tokens::TokenKind;
use codespan_reporting::diagnostic::{Diagnostic, Label};

#[derive(Debug)]
pub enum SyntaxErrKind<'a> {
    UnexpectedToken(TokenKind<'a>),
}

type HelpNote = &'static str;

#[derive(Debug)]
pub enum Expected {
    Item,
    OneOf(Vec<Expected>),
}

#[derive(Debug)]
pub struct SyntaxErr<'a> {
    pub span: Range<usize>,
    pub kind: SyntaxErrKind<'a>,
    pub expected: Expected,
    pub note: Option<HelpNote>,
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
        match kind {
            SyntaxErrKind::UnexpectedToken(t) => {
                let mut diag = Diagnostic::error().with_message(format!("Unexpected {:#?}", t));
                match note {
                    Some(s) => diag = diag.with_notes(vec![s.to_string()]),
                    None => (),
                }
                match expected {
                    Expected::Item => {
                        diag = diag
                            .with_labels(vec![Label::primary((), span)
                                .with_message("expected an item declaration")])
                    }
                    _ => (),
                }
                diag
            }
        }
    }
}
