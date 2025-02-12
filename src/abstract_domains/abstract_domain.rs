use crate::ast::{ArithmeticCondition, Assignment};

pub trait AbstractDomain: PartialOrd {
    fn top() -> Self;
    fn bottom() -> Self;
    fn assignment_abstraction(&self, a: Assignment) -> Self;
    fn arithmetic_cond_abstraction(&self, c: ArithmeticCondition) -> Self;
    fn union_abstraction(&self, other: &Self) -> Self;
    fn intersection_abstraction(&self, other: &Self) -> Self;
}
