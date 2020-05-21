#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Operator {
    sym: &'static str,
    assoc: Assoc,
    fixity: Fixity,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Assoc {
    Right,
    Left,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Fixity {
    Prefix,
    Infix,
}
