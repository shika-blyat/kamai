#[derive(Clone, PartialEq, Debug)]
pub enum Type {
    Named(String), //Int, Unit etc..
    Var(String),
    Fun { from: Box<Type>, to: Box<Type> },
}
