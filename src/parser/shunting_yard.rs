use super::parser::{Expr, Op, OpTerm, ParserError};

fn add_infix_op(ast: &mut Vec<Expr>, op: Op) {
    let roperand = ast.pop().unwrap();
    let loperand = ast.pop().unwrap();
    ast.push(Expr::BinOp {
        left: Box::new(loperand),
        op,
        right: Box::new(roperand),
    });
}

pub fn shunting_yard(tokens: Vec<OpTerm>) -> Result<Expr, ParserError> {
    let mut op_stack: Vec<OpTerm> = vec![];
    let mut ast: Vec<Expr> = vec![];
    for i in tokens.into_iter() {
        match i {
            OpTerm::Expr(Expr::Val(val)) => ast.push(Expr::Val(val)),
            OpTerm::Expr(call @ Expr::Call { fun: _, arg: _ }) => ast.push(call),
            OpTerm::Expr(bracket_expr @ Expr::BracketExpr { left: _, right: _ }) => {
                ast.push(bracket_expr)
            }
            OpTerm::Expr(
                bin_op
                @
                Expr::BinOp {
                    op: _,
                    left: _,
                    right: _,
                },
            ) => ast.push(bin_op),
            OpTerm::Op { op, precedence } => {
                while op_stack.last().is_some() {
                    let last = op_stack.last().unwrap();
                    if let OpTerm::Op {
                        op: _,
                        precedence: last_precedence,
                    } = last
                    {
                        if *last_precedence > precedence || (*last_precedence == precedence)
                        // and op is left associative
                        {
                            let operator = match op_stack.pop().unwrap() {
                                OpTerm::Op { op, precedence: _ } => op,
                                _ => unreachable!(),
                            };
                            add_infix_op(&mut ast, operator);
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                op_stack.push(OpTerm::Op { op, precedence });
            }
            //OpTerm::Expr(Expr::Operation(expr)) => ast.push(shunting_yard(expr)?),
            // for parenthesis
            c => panic!("{:#?}", c),
        }
    }
    for i in op_stack.into_iter().rev() {
        if let OpTerm::Op { op, precedence: _ } = i {
            add_infix_op(&mut ast, op);
        }
    }
    Ok(ast.into_iter().nth(0).unwrap())
}
