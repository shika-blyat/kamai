#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Expr {
    EVar(String),
    ELit(Literal),
    ECall {
        func: Box<Expr>,
        arg: Box<Expr>,
    },
    EFun {
        param: String,
        body: Box<Expr>,
    },
    ELet {
        name: String,
        body: Box<Expr>,
    },
    ECond {
        cond: Box<Expr>,
        true_branch: Box<Expr>,
        false_branch: Box<Expr>,
    },
    EOperation{
        operation: Vec<OpTerm>,
    },
    EEmpty,
}
#[derive(Debug, Clone)]
pub enum OpTerm{
    Op(String),
    Term(Expr),
}
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Literal {
    LInt(f64),
    LBool(bool),
}
#[derive(Clone, Debug, PartialEq)]
pub enum Type {
    TNamed(String),
    TVar(String),
    TFun { from: Box<Type>, to: Box<Type> },
}
