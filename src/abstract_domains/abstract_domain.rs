use std::ops::{Add, Div, Mul, Sub};

use crate::ast::{ArithmeticCondition, Assignment};

pub trait AbstractDomain:
    PartialOrd
    + Clone
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + Sized
{
    fn top() -> Self;
    fn bottom() -> Self;
    fn assignment_abstraction(&self, a: Assignment) -> Self;
    fn arithmetic_cond_abstraction(&self, c: ArithmeticCondition) -> Self;
    fn union_abstraction(&self, other: &Self) -> Self;
    fn intersection_abstraction(&self, other: &Self) -> Self;
    fn constant_abstraction(c: i64) -> Self;
}
