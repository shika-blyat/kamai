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
pub fn apply_subst_to_type(subst: &Substitution, t: &Type) -> Type {
    match &t {
        Type::TNamed(_) => t.clone(),
        Type::TVar(name) => match subst.get(&name.clone()) {
            Some(x) => x.clone(),
            None => t.clone(),
        },
        Type::TFun { from, to } => {
            let (from, to) = (
                Box::new(apply_subst_to_type(subst, from).clone()),
                Box::new(apply_subst_to_type(subst, to).clone()),
            );
            Type::TFun { from, to }
        }
    }
}

pub fn new_t_var(ctx: &mut Context) -> Type {
    ctx.current += 1;
    Type::TVar((ctx.current).to_string())
}
pub fn apply_subst_to_ctx(subst: &Substitution, ctx: &Context) -> Context {
    let env = ctx
        .env
        .iter()
        .map(|(name, typ)| (name.to_string(), apply_subst_to_type(subst, typ)))
        .collect::<HashMap<String, Type>>();
    Context {
        env,
        current: ctx.current,
    }
}

pub fn compose_subst(s1: Substitution, s2: Substitution) -> Substitution {
    let mut s2 = s2
        .into_iter()
        .map(|(name, typ)| (name, apply_subst_to_type(&s1, &typ)))
        .collect::<HashMap<String, Type>>();
    for (name, typ) in s1 {
        s2.insert(name, typ);
    }
    s2
}
fn unify(t1: &Type, t2: &Type) -> Result<Substitution, String> {
    if t1 == t2 {
        return Ok(HashMap::new());
    }
    match (&t1, &t2) {
        (Type::TVar(n), _) => var_bind(n, t2),
        (_, Type::TVar(n)) => var_bind(n, t1),
        (
            Type::TFun {
                from: from1,
                to: to1,
            },
            Type::TFun {
                from: from2,
                to: to2,
            },
        ) => {
            let s1 = unify(from1, from2)?;
            let s2 = unify(
                &apply_subst_to_type(&s1, from1),
                &apply_subst_to_type(&s1, from1),
            )?;
            Ok(compose_subst(s1, s2))
        }
        _ => Err(format!(
            "Type mismatch: \n Expected {:#?}\n found {:#?}",
            t1, t2
        )),
    }
}

fn var_bind(name: &String, t: &Type) -> Result<Substitution, String> {
    if let Type::TVar(n) = &t {
        if name == n {
            return Ok(HashMap::new());
        }
    }
    if contains(&t, &name) {
        Err(format!("Type {:#?} contains a reference to itself", t))
    } else {
        let mut subst = HashMap::new();
        subst.insert(name.to_string(), t.clone());
        Ok(subst)
    }
}
fn contains(t: &Type, name: &String) -> bool {
    match t {
        Type::TNamed(_) => false,
        Type::TVar(n) => n == name,
        Type::TFun { from, to } => contains(from, name) || contains(to, name),
    }
}
fn infer<'a>(ctx: &mut Context, expr: &Expr<'a>) -> Result<(Type, Substitution), String> {
    match expr {
        Expr::EInt { .. } => Ok((Type::TNamed("Int".to_string()), HashMap::new())),
        Expr::EVar { name } => match ctx.env.get(*name) {
            Some(x) => Ok((x.clone(), HashMap::new())),
            None => Err(format!("Use of undeclared variable {}", name)),
        },
        Expr::EFunc { param, body } => {
            let new_type = new_t_var(ctx);
            ctx.new_from_current(param.to_string(), new_type.clone());
            let (body_type, subst) = infer(ctx, body)?;
            let body_type = Box::new(body_type);
            let infered_type = Type::TFun {
                from: Box::new(apply_subst_to_type(&subst, &new_type).clone()),
                to: body_type,
            };
            Ok((infered_type, subst))
        }
        Expr::ECall { arg, func } => {
            let (func_type, s1) = infer(ctx, func)?;
            let (arg_type, s2) = infer(&mut apply_subst_to_ctx(&s1, ctx), arg)?;
            let new_var = new_t_var(ctx);
            let s3 = compose_subst(s1, s2);
            let s4 = unify(
                &Type::TFun {
                    from: Box::new(arg_type.clone()),
                    to: Box::new(new_var),
                },
                &func_type,
            )?;
            let func_type = apply_subst_to_type(&s4, &func_type);
            let s5 = compose_subst(s3, s4);
            if let Type::TFun { from, to } = func_type {
                let s6 = unify(&apply_subst_to_type(&s5, &from), &arg_type)?;
                let result_subst = compose_subst(s5, s6);
                Ok((apply_subst_to_type(&result_subst, &to), result_subst))
            } else {
                unreachable!()
            }
        }
        _ => unimplemented!(),
    }
}
fn main() {
    let mut env = HashMap::new();
    env.insert(
        "function".to_string(),
        Type::TFun {
            from: Box::new(Type::TNamed("Int".to_string())),
            to: Box::new(Type::TVar("1".to_string())),
        },
    );
    let mut context = Context { env, current: 1 };
    println!(
        "{:#?}",
        infer(
            &mut context,
            &Expr::ECall {
                func: Box::new(Expr::EVar { name: "function" }),
                arg: Box::new(Expr::EInt { value: 15.0 }),
            }
        )
    );
}
