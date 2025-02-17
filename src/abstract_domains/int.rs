use Int::*;
use std::ops::{Add, Div, Mul, Neg, Sub};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]

pub enum Int {
    NegInf,
    Num(i64),
    PosInf,
}

impl Neg for Int {
    type Output = Self;
    fn neg(self) -> Self::Output {
        match self {
            NegInf => PosInf,
            PosInf => NegInf,
            Num(x) => Num(-x),
        }
    }
}

impl Add for Int {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (NegInf, x) | (x, NegInf) if x != PosInf => NegInf,
            (PosInf, x) | (x, PosInf) if x != NegInf => PosInf,
            (Num(lhs), Num(rhs)) => Num(lhs + rhs),
            _ => panic!("Trying to compute PosInf + NegInf or vice-versa, which is undefined "),
        }
    }
}

impl Sub for Int {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (NegInf, x) | (x, NegInf) if x != NegInf => NegInf,
            (PosInf, x) | (x, PosInf) if x != PosInf => PosInf,
            (Num(lhs), Num(rhs)) => Num(lhs - rhs),
            _ => panic!("Trying to compute PosInf + NegInf or vice-versa, which is undefined "),
        }
    }
}

impl Mul for Int {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (NegInf, NegInf) => PosInf,
            (PosInf, PosInf) => PosInf,
            (NegInf, PosInf) | (PosInf, NegInf) => NegInf,
            (x, _) | (_, x) if x == Num(0) => x,
            (Num(x), NegInf) | (NegInf, Num(x)) if x < 0 => PosInf,
            (Num(x), NegInf) | (NegInf, Num(x)) if x > 0 => NegInf,
            (Num(x), PosInf) | (PosInf, Num(x)) if x > 0 => PosInf,
            (Num(x), PosInf) | (PosInf, Num(x)) if x < 0 => NegInf,
            (Num(lhs), Num(rhs)) => Num(lhs * rhs),
            _ => {
                panic!("Trying to compute PosInf * NegInf or vice-versa, which is resolvable here")
            }
        }
    }
}

impl Div for Int {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        match (self.clone(), rhs) {
            (_, NegInf) | (_, PosInf) | (Num(0), Num(0)) => Num(0),
            (PosInf, Num(0)) => PosInf,
            (NegInf, Num(0)) => NegInf,
            (Num(x), Num(0)) if x > 0 => PosInf,
            (Num(x), Num(0)) if x < 0 => NegInf,
            (PosInf | NegInf, Num(x)) if x > 0 => self,
            (PosInf | NegInf, Num(x)) if x < 0 => -self,
            (Num(lhs), Num(rhs)) => Num(lhs / rhs),
            _ => panic!("Unhandled div pattern"),
        }
    }
}
