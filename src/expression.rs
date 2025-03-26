use crate::token::{Literal, Token};
use std::fmt::Display;

pub trait Expr: Display {}
pub struct Bin {
    pub left: Box<dyn Expr>,
    pub operator: Token,
    pub right: Box<dyn Expr>,
}
impl Expr for Bin {}
impl Display for Bin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} {} {})", self.operator, self.left, self.right)
    }
}

pub struct Grp {
    pub expression: Box<dyn Expr>,
}
impl Expr for Grp {}
impl Display for Grp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(grp {})", self.expression)
    }
}

pub struct Lit {
    pub value: Option<Literal>,
}
impl Expr for Lit {}
impl Display for Lit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.value {
            Some(val) => write!(f, "{}", val),
            None => write!(f, "nil"),
        }
    }
}

pub struct Un {
    pub operator: Token,
    pub right: Box<dyn Expr>,
}
impl Expr for Un {}
impl Display for Un {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} {})", self.operator, self.right)
    }
}
