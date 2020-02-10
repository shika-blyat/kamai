use crate::ast::{Expr, Literal, Type};
use std::collections::HashMap;

type Context = HashMap<String, Type>;
type Subst = HashMap<String, Type>;

pub struct TypeChecker {
    current: usize,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self { current: 0 }
    }
    fn infer_literal(lit: &Literal) -> Type {
        match lit {
            Literal::LInt(_) => Type::TNamed("Int".to_string()),
            Literal::LBool(_) => Type::TNamed("Bool".to_string()),
        }
    }
    fn new_typevar(&mut self) -> Type {
        self.current += 1;
        Type::TVar(format!("a{}", self.current))
    }
    pub fn infer(&mut self, expr: &Expr, ctx: &mut Context) -> Result<(Type, Subst), String> {
        match expr {
            Expr::ELit(lit) => Ok((Self::infer_literal(&lit), HashMap::new())),
            Expr::EVar(name) => match ctx.get(name) {
                Some(typ) => Ok((typ.clone(), HashMap::new())),
                None => Err(format!("Use of undeclared variable {}", name)),
            },
            Expr::EFun { param, body } => {
                let tvar = self.new_typevar();
                let mut new_ctx = ctx.clone();
                new_ctx.insert(param.to_string(), tvar.clone());
                let (body_type, subst) = self.infer(body, &mut new_ctx)?;
                let body_type = Box::new(body_type);
                let infered_type = Type::TFun {
                    from: Box::new(apply_subst_to_type(&subst, &tvar)),
                    to: body_type,
                };
                Ok((infered_type, subst))
            }
            Expr::ECall { func, arg } => {
                let (func_type, s1) = self.infer(func, ctx)?;
                let (arg_type, s2) = self.infer(arg, &mut apply_subst_to_ctx(&s1, ctx))?;
                let tvar = self.new_typevar();
                let s3 = compose_subst(s1, s2);
                let s4 = unify(
                    &Type::TFun {
                        from: Box::new(arg_type.clone()),
                        to: Box::new(tvar),
                    },
                    &func_type,
                )?;
                let func_type = apply_subst_to_type(&s4, &func_type);
                let s5 = compose_subst(s3, s4);
                if let Type::TFun { from, to } = &func_type {
                    let s6 = unify(&apply_subst_to_type(&s5, from), &arg_type)?;
                    let result_subst = compose_subst(s5, s6);
                    return Ok((apply_subst_to_type(&result_subst, to), result_subst));
                }
                Err("Internal Error Occured\n func_type was supposed to be a TFun".to_string())
            }
            Expr::ELet { name: _, body } => self.infer(body, ctx),
            /*Expr::ECond{ cond, true_branch, false_branch} => {

            }*/
            _ => unreachable!(),
        }
    }
}

fn apply_subst_to_type(subst: &Subst, t: &Type) -> Type {
    match &t {
        Type::TNamed(_) => t.clone(),
        Type::TVar(name) => match subst.get(name) {
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
fn apply_subst_to_ctx(subst: &Subst, ctx: &Context) -> Context {
    let ctx = ctx
        .iter()
        .map(|(name, typ)| (name.to_string(), apply_subst_to_type(subst, typ)))
        .collect::<HashMap<String, Type>>();
    ctx
}

fn compose_subst(s1: Subst, s2: Subst) -> Subst {
    let mut s2 = s2
        .into_iter()
        .map(|(name, typ)| (name, apply_subst_to_type(&s1, &typ)))
        .collect::<HashMap<String, Type>>();
    for (name, typ) in s1 {
        s2.insert(name, typ);
    }
    s2
}
fn unify(t1: &Type, t2: &Type) -> Result<Subst, String> {
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

fn var_bind(name: &String, t: &Type) -> Result<Subst, String> {
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
