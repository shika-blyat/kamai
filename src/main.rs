mod ast;
mod typechecker;

use crate::{
    ast::{Expr, Literal, Type},
    typechecker::TypeChecker,
};
use std::collections::HashMap;

fn main() {
    let mut ctx = HashMap::new();
    let mut tchecker = TypeChecker::new();
    ctx.insert(
        "id".to_string(),
        Type::TFun {
            from: Box::new(Type::TVar("a1".to_string())),
            to: Box::new(Type::TVar("a1".to_string())),
        },
    );
    let call = Expr::ECall {
        func: Box::new(Expr::EVar("id".to_string())),
        arg: Box::new(Expr::ELit(Literal::LInt(15.0))),
    };
    let call2 = Expr::ECall {
        func: Box::new(Expr::EVar("id".to_string())),
        arg: Box::new(Expr::ELit(Literal::LBool(true))),
    };
    println!(
        "{:#?}",
        tchecker.infer(
            &Expr::EFun {
                param: "x".to_string(),
                body: Box::new(Expr::EVar("x".to_string()))
            },
            &mut ctx
        )
    );
    println!("{:#?}", tchecker.infer(&call, &mut ctx));
    println!("{:#?}", tchecker.infer(&call2, &mut ctx));
}
