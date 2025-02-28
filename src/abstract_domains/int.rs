use Int::*;
use core::fmt;
use std::ops::{Add, Div, Mul, Neg, Sub};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]

pub enum Int {
    NegInf,
    Num(i64),
    PosInf,
}

#[derive(Debug, Clone, Copy)]
pub struct BadInt<'a>(&'a str);

impl<'a> fmt::Display for BadInt<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid conversion: {}: String -> Int", self.0)
    }
}

impl<'a> TryFrom<&'a str> for Int {
    type Error = BadInt<'a>;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        match value.trim() {
            "inf" => Ok(Int::PosInf),
            "-inf" => Ok(Int::NegInf),
            x if x.parse::<i64>().is_ok() => Ok(Int::Num(x.parse::<i64>().unwrap())),
            _ => Err(BadInt(value)),
        }
    }
}

impl Into<String> for Int {
    fn into(self) -> String {
        match self {
            Int::PosInf => "inf".to_string(),
            Int::NegInf => "-inf".to_string(),
            Int::Num(x) => format!("{x}"),
        }
    }
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
            (NegInf, x) if x != NegInf => NegInf,
            (x, NegInf) if x != NegInf => PosInf,
            (PosInf, x) if x != PosInf => PosInf,
            (x, PosInf) if x != PosInf => NegInf,
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
            (_, NegInf | PosInf) | (Num(0), Num(0)) => Num(0),
            (x, Num(0)) if x > Int::Num(0) => PosInf,
            (x, Num(0)) if x < Int::Num(0) => NegInf,
            (Num(lhs), Num(rhs)) => Num(lhs / rhs),
            _ => panic!("Unhandled div pattern"),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::abstract_domains::int::Int;

    #[test]
    fn int_sub() {
        assert_eq!(Int::Num(0) - Int::NegInf, Int::PosInf);
        assert_eq!(Int::Num(0) - Int::PosInf, Int::NegInf);
    }

    #[test]
    fn int_div() {
        assert_eq!(Int::NegInf / Int::Num(0), Int::NegInf);
        assert_eq!(Int::Num(10) / Int::Num(0), Int::PosInf);
        assert_eq!(Int::PosInf / Int::NegInf, Int::Num(0))
    }
}
