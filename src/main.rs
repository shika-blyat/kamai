use std::collections::HashMap;
use std::ops::Deref;

#[derive(Debug, Clone)]
enum Expr<'a> {
    EInt {
        value: f64,
    },
    EVar {
        name: &'a str,
    },
    EFunc {
        param: &'a str,
        body: Box<Expr<'a>>,
    },
    ECall {
        func: Box<Expr<'a>>,
        arg: Box<Expr<'a>>,
    },
    ECond {
        cond: Box<Expr<'a>>,
        true_branch: Box<Expr<'a>>,
        false_branch: Box<Expr<'a>>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum Type {
    TNamed(String),
    TVar(String),
    TFun { from: Box<Type>, to: Box<Type> },
}
pub type Env = HashMap<String, Type>;
pub type Substitution = HashMap<String, Type>;

pub struct Context {
    pub env: Env,
    current: usize,
}
impl Context {
    pub fn new_from_current(&mut self, name: String, new_type: Type) {
        self.env.insert(name, new_type);
    }
}
pub fn apply_subst_to_type(subst: &Substitution, t: Type) -> Type {
    match &t {
        Type::TNamed(_) => t.clone(),
        Type::TVar(name) => match subst.get(&name.clone()) {
            Some(x) => x.clone(),
            None => t.clone(),
        },
        Type::TFun { from, to } => {
            let (from, to) = (
                Box::new(apply_subst_to_type(subst, from.deref().clone()).clone()),
                Box::new(apply_subst_to_type(subst, to.deref().clone()).clone()),
            );
            Type::TFun { from, to }
        }
    }
}

pub fn new_t_var(ctx: &mut Context) -> Type {
    ctx.current += 1;
    Type::TVar((ctx.current).to_string())
}

fn infer<'a>(ctx: &mut Context, expr: Expr<'a>) -> Result<(Type, Substitution), String> {
    match expr {
        Expr::EInt { .. } => Ok((Type::TNamed("Int".to_string()), HashMap::new())),
        Expr::EVar { name } => match ctx.env.get(name) {
            Some(x) => Ok((x.clone(), HashMap::new())),
            None => Err(format!("Use of undeclared variable {}", name)),
        },
        Expr::EFunc { param, body } => {
            let new_type = new_t_var(ctx);
            ctx.new_from_current(param.to_string(), new_type.clone());
            let (body_type, subst) = infer(ctx, body.deref().clone())?;
            let body_type = Box::new(body_type);
            let infered_type = Type::TFun {
                from: Box::new(apply_subst_to_type(&subst, new_type).clone()),
                to: body_type,
            };
            Ok((infered_type, subst))
        }
        _ => unimplemented!(),
    }
}
fn main() {
    println!("Hello, world!");
}
