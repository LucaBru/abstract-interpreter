use std::{
    collections::HashSet,
    fmt::Debug,
    ops::{Add, Div, Mul, Sub},
};

use crate::parser::ast::Operator;

pub enum IntervalBound {
    NegInf,
    Num(i64),
    PosInf,
}

pub trait AbstractDomain:
    PartialOrd
    + Clone
    + Copy
    + Debug
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + for<'a> TryFrom<&'a str>
    + Into<String>
    + Sized
{
    //function called before building the interpreter to allow to abstract domain to initialize some domain specific values
    fn init() {}
    fn top() -> Self;
    fn bottom() -> Self;
    fn lub(&self, other: &Self) -> Self;
    fn glb(&self, other: &Self) -> Self;
    fn constant_abstraction(c: i64) -> Self;
    fn interval_abstraction(low: IntervalBound, upper: IntervalBound) -> Self;
    fn widening_operator() -> Option<impl Fn(&Self, &Self, &HashSet<i64>) -> Self>;
    fn narrowing(&self, rhs: &Self) -> Self {
        self.glb(rhs)
    }

    fn backward_arithmetic_operator(
        lhs: Self,
        rhs: Self,
        result: Self,
        operator: Operator,
    ) -> [Self; 2] {
        match operator {
            Operator::Add => {
                let lhs_ref = lhs.glb(&(result - rhs));
                let rhs_ref = rhs.glb(&(result - lhs));
                [lhs_ref, rhs_ref]
            }
            Operator::Sub => {
                let lhs_ref = lhs.glb(&(result + rhs));
                let rhs_ref = rhs.glb(&(lhs - result));
                [lhs_ref, rhs_ref]
            }
            Operator::Mul => {
                let lhs_ref = lhs.glb(&(result / rhs));
                let rhs_ref = rhs.glb(&(result / lhs));
                [lhs_ref, rhs_ref]
            }
            Operator::Div => {
                let s = result
                    + AbstractDomain::interval_abstraction(
                        IntervalBound::Num(-1),
                        IntervalBound::Num(1),
                    );
                let lhs_ref = lhs.glb(&(s * rhs));
                let rhs_ref = rhs.glb(&(lhs / s).lub(&AbstractDomain::interval_abstraction(
                    IntervalBound::Num(0),
                    IntervalBound::Num(0),
                )));
                [lhs_ref, rhs_ref]
            }
        }
    }
}
