use super::type_tree::Type;
use crate::parser::ast::{Expr, Literal};
use std::collections::HashMap;

struct Context {
    cntr: usize,
    ctx: HashMap<String, Type>,
}
impl Context {
    pub fn get(&self, ident: &String) -> Option<&Type> {
        self.ctx.get(ident)
    }
    pub fn new_tvar(&mut self) -> Type {
        let tvar = Type::Var(format!("T{}", self.cntr));
        self.cntr += 1;
        tvar
    }
}
struct Subst {
    subst: Option<HashMap<String, Type>>,
}
impl Subst {
    pub fn empty() -> Self {
        Self { subst: None }
    }
}

struct TypeError {
    reason: String,
}
fn infer(expr: Expr, mut ctx: Context) -> Result<(Type, Subst), TypeError> {
    match expr {
        Expr::Val(val) => match val {
            Literal::Int(_) => Ok((Type::Named("Int".to_string()), Subst::empty())),
            Literal::Unit => Ok((Type::Named("()".to_string()), Subst::empty())),
        },
        Expr::Identifier(ident) => match ctx.get(&ident) {
            Some(t) => Ok((t.clone(), Subst::empty())),
            None => Err(TypeError {
                reason: format!("Use of undeclared identifier {}", ident),
            }),
        },
        Expr::Lambda { param, body } => {
            let tvar = ctx.new_tvar();
        }
        _ => unreachable!(),
    }
}
