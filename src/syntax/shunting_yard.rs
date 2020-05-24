use std::convert::TryInto;

use super::tokens::TokenKind;
use super::{ast::*, parser::Operator};
use crate::{errors::syntax_err::*, utils::merge_ranges};

#[derive(Debug)]
enum ShuntingYardState {
    ExpectOperand,
    ExpectOp,
}
#[derive(Debug)]
pub(super) enum OpOrExpr<'a> {
    Expr(Node<Expr<'a>>),
    Op(Node<Operator<'a>>),
}

fn insert_bin_op<'a>(ast: &mut Vec<Node<Expr<'a>>>, Node { value: op, .. }: &Node<Operator<'a>>) {
    let right = ast.pop().unwrap();
    let left = ast.pop().unwrap();
    ast.push(Node {
        span: merge_ranges(&left.span, &right.span),
        value: Expr::Binary(op.clone().try_into().unwrap(), left.into(), right.into()),
    })
}

fn insert_un_op<'a>(
    ast: &mut Vec<Node<Expr<'a>>>,
    Node {
        value: op,
        span: op_span,
    }: &Node<Operator<'a>>,
) {
    let right = ast.pop().unwrap();
    ast.push(Node {
        span: merge_ranges(op_span, &right.span),
        value: Expr::Unary(op.clone().try_into().unwrap(), right.into()),
    })
}

pub(super) fn shunting_yard<'a>(
    tokens: Vec<OpOrExpr<'a>>,
) -> Result<Node<Expr<'a>>, SyntaxErr<'a>> {
    let mut op_stack: Vec<Node<Operator<'a>>> = vec![];
    let mut ast: Vec<Node<Expr<'a>>> = vec![];
    let tok_len = tokens.len();
    let mut state = ShuntingYardState::ExpectOperand;
    for (idx, tok) in tokens.into_iter().enumerate() {
        match tok {
            OpOrExpr::Expr(e) => {
                if let ShuntingYardState::ExpectOp = state {
                    let last = ast.pop().unwrap();
                    ast.push(Node {
                        span: merge_ranges(&last.span, &e.span),
                        value: Expr::Call(last.into_boxed(), e.into_boxed()),
                    })
                } else {
                    state = ShuntingYardState::ExpectOp;
                    ast.push(e)
                }
            }
            OpOrExpr::Op(Node {
                value: mut op,
                span,
            }) => {
                if idx == tok_len {
                    return Err(SyntaxErr {
                        span: span.clone(),
                        kind: SyntaxErrKind::UnexpectedToken(TokenKind::Op(op.sym)),
                        expected: Expected::OneOf(vec![Expected::Operator, Expected::Semicolon]),
                        note: None,
                    });
                }
                if let ShuntingYardState::ExpectOperand = state {
                    match op.into_prefix() {
                        Some(un_op) => op = un_op,
                        None => {
                            return Err(SyntaxErr {
                                span: span.clone(),
                                kind: SyntaxErrKind::UnexpectedToken(TokenKind::Op(op.sym)),
                                expected: Expected::Expr,
                                note: None,
                            })
                        }
                    }
                }
                state = ShuntingYardState::ExpectOperand;
                while let Some(Node { value: last_op, .. }) = op_stack.last() {
                    if last_op.is_infix() {
                        if last_op.prec > op.prec
                            || (last_op.prec >= op.prec && last_op.is_left_assoc())
                        {
                            if last_op.is_infix() {
                                insert_bin_op(&mut ast, &op_stack.pop().unwrap())
                            } else {
                                insert_un_op(&mut ast, &op_stack.pop().unwrap())
                            }
                        } else {
                            break;
                        }
                    }
                }
                op_stack.push(Node { span, value: op });
            }
        }
    }
    for i in op_stack.into_iter().rev() {
        if i.value.is_infix() {
            insert_bin_op(&mut ast, &i)
        } else {
            insert_un_op(&mut ast, &i)
        }
    }
    if ast.len() != 1 {
        panic!("An unexpected error occured")
    }
    Ok(ast.into_iter().next().unwrap())
}
