use crate::token::Token;
use std::any::Any;
use std::fmt::{Debug, Display};

pub trait AnyDebug: Any + Debug {}
impl<T> AnyDebug for T where T: Any + Debug {}

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

pub struct Cond {
    pub cond: Box<dyn Expr>,
    pub cons: Box<dyn Expr>,
    pub alt: Box<dyn Expr>,
}
impl Expr for Cond {}
impl Display for Cond {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} ? {} : {})", self.cond, self.cons, self.alt)
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
    pub value: Option<Box<dyn AnyDebug>>,
}
impl Expr for Lit {}
impl Display for Lit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.value {
            Some(val) => write!(f, "{:?}", val),
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
