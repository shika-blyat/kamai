#[test]
fn simple_operations() {
    use crate::parser::{
        ast::{Expr, Literal, Op},
        parse::Parser,
    };
    let ast = Parser::new("1 + 2 * 3".to_string())
        .unwrap()
        .expr()
        .unwrap();
    assert_eq!(
        ast,
        Expr::BinOp {
            op: Op::Add,
            left: Box::new(Expr::Val(Literal::Int(1))),
            right: Box::new(Expr::BinOp {
                op: Op::Mul,
                left: Box::new(Expr::Val(Literal::Int(2))),
                right: Box::new(Expr::Val(Literal::Int(3)))
            })
        }
    );
    let ast = Parser::new("(1 + 2) * ()".to_string())
        .unwrap()
        .expr()
        .unwrap();
    assert_eq!(
        ast,
        Expr::BinOp {
            op: Op::Mul,
            left: Box::new(Expr::BinOp {
                op: Op::Add,
                left: Box::new(Expr::Val(Literal::Int(1))),
                right: Box::new(Expr::Val(Literal::Int(2)))
            }),
            right: Box::new(Expr::Val(Literal::Unit))
        }
    );
}

#[test]
fn func_declaration() {
    use crate::parser::{
        ast::{Expr, Literal, Op},
        parse::Parser,
    };
    use std::collections::HashMap;
    let map = Parser::new("a = 1 + 2 * 3;".to_string())
        .unwrap()
        .parse()
        .unwrap();
    let ast = Expr::BinOp {
        op: Op::Add,
        left: Box::new(Expr::Val(Literal::Int(1))),
        right: Box::new(Expr::BinOp {
            op: Op::Mul,
            left: Box::new(Expr::Val(Literal::Int(2))),
            right: Box::new(Expr::Val(Literal::Int(3))),
        }),
    };
    let mut hashmap = HashMap::new();
    hashmap.insert("a".to_string(), ast);
    assert_eq!(map, hashmap);
}
