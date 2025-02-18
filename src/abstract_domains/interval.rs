use core::fmt;
use std::{
    cmp::{Ordering, max, min},
    collections::HashSet,
    ops::{Add, Div, Mul, Sub},
    sync::RwLock,
};

use super::{
    abstract_domain::{AbstractDomain, IntervalBound},
    int::Int,
};

pub static M: RwLock<Int> = RwLock::new(Int::NegInf);
pub static N: RwLock<Int> = RwLock::new(Int::PosInf);

const TOP: Interval = Interval {
    low: Int::NegInf,
    upper: Int::PosInf,
};
const BOTTOM: Interval = Interval {
    low: Int::Num(1),
    upper: Int::Num(0),
};

const ZERO: Interval = Interval {
    low: Int::Num(0),
    upper: Int::Num(0),
};

#[derive(Clone, Copy, Debug, Eq)]
pub struct Interval {
    low: Int,
    upper: Int,
}

impl Interval {
    fn normal_form(low: Int, upper: Int) -> Self {
        let m = *M.read().unwrap();
        let n = *N.read().unwrap();

        if low > upper {
            return Interval::bottom();
        }

        if m > n && low < upper {
            return TOP;
        } else if m > n {
            return Interval { low, upper };
        }

        let low = match low {
            x if x > n => n.clone(),
            x if x < m => Int::NegInf,
            _ => low,
        };

        let upper = match upper {
            x if x < m => m.clone(),
            x if x > n => Int::PosInf,
            _ => upper,
        };

        Interval { low, upper }
    }
}

impl From<[i64; 2]> for Interval {
    fn from(value: [i64; 2]) -> Self {
        Interval {
            low: Int::Num(value[0]),
            upper: Int::Num(value[1]),
        }
    }
}

impl PartialEq for Interval {
    fn eq(&self, other: &Self) -> bool {
        let m = *M.read().unwrap();
        let n = *N.read().unwrap();

        let is_bottom = |intv: &Interval| intv.low > intv.upper;
        let is_top = |intv: &Interval| match (m > n, intv.low, intv.upper) {
            (true, a, b) if a != b && !is_bottom(intv) => true,
            (false, a, b) if a < m && b > n || a == Int::NegInf && b == Int::PosInf => true,
            _ => false,
        };
        if is_bottom(self) && is_bottom(other) || is_top(self) && is_top(other) {
            return true;
        }

        let Interval { low: a, upper: b } = self;
        let Interval { low: c, upper: d } = other;
        if m > n && a == c && b == d {
            return true;
        } else if m > n {
            return false;
        }

        if a == c && b == d || *a < m && *c < n && b == d || a == b && *b > n && *d > n {
            return true;
        }
        false
    }
}

impl Ord for Interval {
    fn cmp(&self, other: &Self) -> Ordering {
        if *self == BOTTOM && *other != BOTTOM || *self != TOP && *other == TOP {
            return Ordering::Less;
        }

        if self == other {
            return Ordering::Equal;
        }

        let Interval { low: a, upper: b } = self;
        let Interval { low: c, upper: d } = other;

        if c < a && b < d {
            return Ordering::Less;
        }

        Ordering::Greater
    }
}

impl PartialOrd for Interval {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Add for Interval {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        if self == BOTTOM || rhs == BOTTOM {
            return BOTTOM;
        }
        if self == TOP || rhs == TOP {
            return TOP;
        }

        let Interval { low: a, upper: b } = self;
        let Interval { low: c, upper: d } = rhs;
        let low = a + c;
        let upper = b + d;
        Self::normal_form(low, upper)
    }
}

impl Sub for Interval {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        if self == BOTTOM || rhs == BOTTOM {
            return BOTTOM;
        }
        if self == TOP || rhs == TOP {
            return TOP;
        }

        let Interval { low: a, upper: b } = self;
        let Interval { low: c, upper: d } = rhs;
        let low = a - d;
        let upper = b - c;
        Self::normal_form(low, upper)
    }
}

impl Mul for Interval {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        if self == BOTTOM || rhs == BOTTOM {
            return BOTTOM;
        }
        if self == ZERO || rhs == ZERO {
            return ZERO;
        }
        if self == TOP || rhs == TOP {
            return TOP;
        }

        let Interval { low: a, upper: b } = self;
        let Interval { low: c, upper: d } = rhs;

        let mut choices = [a * c, a * d, b * c, b * d];
        choices.sort();
        let low = choices[0];
        let upper = choices[3];

        Self::normal_form(low, upper)
    }
}

impl Div for Interval {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        if self == BOTTOM || rhs == BOTTOM {
            return BOTTOM;
        }
        if rhs == ZERO {
            return BOTTOM;
        }

        let Interval { low: a, upper: b } = self;
        let Interval { low: c, upper: d } = rhs;

        if c >= Int::Num(0) {
            let mut choices = [a / c, a / d, b / c, b / d];
            choices.sort();
            Self::normal_form(choices[0], choices[3])
        } else if d <= Int::Num(0) {
            Interval { low: -b, upper: -a } / Interval { low: -d, upper: -c }
        } else {
            (self.clone()
                / Interval {
                    low: c,
                    upper: Int::Num(0),
                })
            .union_abstraction(
                &(self
                    / Interval {
                        low: Int::Num(0),
                        upper: d,
                    }),
            )
        }
    }
}

impl AbstractDomain for Interval {
    fn bottom() -> Self {
        BOTTOM
    }
    fn top() -> Self {
        TOP
    }
    fn intersection_abstraction(&self, other: &Self) -> Self {
        Self::normal_form(max(self.low, other.low), min(self.upper, other.upper))
    }
    fn union_abstraction(&self, other: &Self) -> Self {
        Self::normal_form(min(self.low, other.low), max(self.upper, other.upper))
    }

    fn constant_abstraction(c: i64) -> Self {
        Interval {
            low: Int::Num(c),
            upper: Int::Num(c),
        }
    }

    fn interval_abstraction(low: IntervalBound, upper: IntervalBound) -> Self {
        let low = match low {
            IntervalBound::NegInf => Int::NegInf,
            IntervalBound::Num(x) => Int::Num(x),
            _ => panic!("PosInf found while parsing a concrete interval to an abstract domain"),
        };

        let upper = match upper {
            IntervalBound::PosInf => Int::PosInf,
            IntervalBound::Num(x) => Int::Num(x),
            _ => panic!("NegInf found while parsing a concrete interval to an abstract domain"),
        };

        Self::normal_form(low, upper)
    }

    fn widening(&self, rhs: &Self, thresholds: &HashSet<i64>) -> Self {
        let thresholds: Vec<Int> = thresholds.into_iter().map(|t| Int::Num(*t)).collect();
        let low = match self.low <= rhs.low {
            true => self.low,
            _ => {
                let mut t = Int::NegInf;
                thresholds.iter().for_each(|x| {
                    if *x > t && *x <= rhs.low {
                        t = *x
                    }
                });
                t
            }
        };

        let upper = match self.upper >= rhs.upper {
            true => self.upper,
            _ => {
                let mut t = Int::PosInf;
                thresholds.iter().for_each(|x| {
                    if *x < t && *x >= rhs.upper {
                        t = *x
                    }
                });
                t
            }
        };
        dbg!(self, rhs, Self::normal_form(low, upper));
        Self::normal_form(low, upper)
    }

    fn narrowing(&self, rhs: &Self) -> Self {
        let Interval { low: a, upper: b } = *self;
        let Interval { low: c, upper: d } = *rhs;
        let mut low = a;
        if a == Int::NegInf {
            low = c;
        }
        let mut upper = b;
        if b == Int::PosInf {
            upper = d;
        }

        Interval { low, upper }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BadInterval<'a>(&'a str);

impl<'a> fmt::Display for BadInterval<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid conversion {} -> Interval", self.0)
    }
}

impl<'a> TryFrom<&'a str> for Interval {
    type Error = BadInterval<'a>;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let trimmed = value.trim().trim_start_matches('[').trim_end_matches(']');
        let parts: Vec<_> = trimmed.split(',').map(Int::try_from).collect();

        if parts.len() != 2 {
            return Err(BadInterval(value));
        }

        if parts[0].is_ok_and(|x| x != Int::PosInf) && parts[1].is_ok_and(|x| x != Int::NegInf) {
            return Ok(Interval {
                low: parts[0].unwrap(),
                upper: parts[1].unwrap(),
            });
        }

        Err(BadInterval(value))
    }
}

impl<'a> Into<String> for Interval {
    fn into(self) -> String {
        format! {"[{},{}]", Into::<String>::into(self.low), Into::<String>::into(self.upper)}
    }
}

#[cfg(test)]
mod test {
    use std::ops::{Add, Div, Mul};

    use crate::abstract_domains::{
        int::Int,
        interval::{BOTTOM, TOP, ZERO},
    };

    use super::{Interval, M, N};

    fn set_domain_bounds(m: Int, n: Int) {
        let mut m_lock = M.write().unwrap();
        *m_lock = m;

        let mut n_lock = N.write().unwrap();
        *n_lock = n
    }

    fn singleton(v: i64) -> Interval {
        Interval {
            low: Int::Num(v),
            upper: Int::Num(v),
        }
    }

    fn constant_domain() {
        set_domain_bounds(Int::PosInf, Int::NegInf);
    }

    fn interval_domain() {
        set_domain_bounds(Int::NegInf, Int::PosInf);
    }

    fn restricted_domain(low: i64, upper: i64) {
        set_domain_bounds(Int::Num(low), Int::Num(upper));
    }

    fn minus_inf_to(x: i64) -> Interval {
        Interval {
            low: Int::NegInf,
            upper: Int::Num(x),
        }
    }

    fn x_to_inf(x: i64) -> Interval {
        Interval {
            low: Int::Num(x),
            upper: Int::PosInf,
        }
    }

    #[test]
    fn intv_abs_domain_cmp() {
        constant_domain();
        assert!(BOTTOM <= BOTTOM);
        assert!(TOP <= TOP);
        assert!(singleton(1) <= singleton(1));
        assert_eq!(singleton(1) <= singleton(2), false);

        restricted_domain(-5, 5);
        assert!(minus_inf_to(0) <= [-6, 0].into());
        assert!(TOP <= [-6, 6].into());
        assert_eq!(
            <[i64; 2] as Into<Interval>>::into([1, 4]) <= [3, 5].into(),
            false
        );
    }

    #[test]
    fn intv_abs_domain_eq() {
        constant_domain();
        assert_eq!(BOTTOM, BOTTOM);
        assert_eq!(singleton(1), singleton(1));
        assert_ne!(singleton(1), singleton(2));
        assert_eq!(TOP, [0, 1].into());

        restricted_domain(-5, 5);
        assert!(Interval::eq(&[-3, 2].into(), &[-3, 2].into()));
        assert_eq!(minus_inf_to(0), [-6, 0].into());
        assert_eq!(TOP, [-6, 6].into());
    }

    #[test]
    fn intv_abs_domain_add() {
        constant_domain();
        assert_eq!(BOTTOM + singleton(1), BOTTOM);
        assert_eq!(TOP + singleton(1), TOP);
        assert_eq!(TOP + BOTTOM, BOTTOM);
        assert_eq!(singleton(1) + singleton(2), singleton(3));

        restricted_domain(-5, 5);
        assert_eq!(
            Interval::add([-3, 0].into(), [-2, 5].into()),
            [-5, 5].into()
        );
        assert_eq!(singleton(-1) + [-5, 5].into(), minus_inf_to(4).into());
        assert_eq!(singleton(5) + singleton(1), x_to_inf(5));

        interval_domain();
        assert_eq!(x_to_inf(0) + [-200, -10].into(), x_to_inf(-200))
    }

    #[test]
    fn intv_abs_domain_sub() {
        constant_domain();
        assert_eq!(BOTTOM - TOP, BOTTOM);
        assert_eq!(TOP - TOP, TOP);
        assert_eq!(singleton(0) - singleton(10), singleton(-10));

        restricted_domain(-5, 5);
        assert_eq!(singleton(5) - [0, 5].into(), [0, 5].into());
        assert_eq!(singleton(-5) - [0, 1].into(), minus_inf_to(-5));
        assert_eq!(singleton(-5) - singleton(1), minus_inf_to(-5));

        interval_domain();
        assert_eq!(minus_inf_to(100) - singleton(-10), minus_inf_to(110));
    }

    #[test]
    fn intv_abs_domain_mul() {
        constant_domain();
        assert_eq!(ZERO * TOP, ZERO);
        assert_eq!(ZERO * BOTTOM, BOTTOM);
        assert_eq!(singleton(5) * singleton(2), singleton(10));

        restricted_domain(-5, 5);
        assert_eq!(singleton(5) * singleton(2), x_to_inf(5));
        assert_eq!(Interval::mul([0, 2].into(), [0, 3].into()), x_to_inf(0));
        assert_eq!(singleton(10) * [-1, 1].into(), TOP)
    }

    #[test]
    fn intv_abs_domain_div() {
        constant_domain();
        assert_eq!(BOTTOM / TOP, BOTTOM);
        //[0,0]/[-inf, inf] = [0,0]/[-inf,0] U [0,0]/[0,inf] = [0,0]/[0,inf] = [min(0/0,0/inf), max(0/0,0/inf)] = [0,0]
        assert_eq!(ZERO / TOP, ZERO);
        assert_eq!(TOP / ZERO, BOTTOM);
        assert_eq!(singleton(1) / singleton(2), ZERO);
        assert_eq!(singleton(1) / singleton(1), singleton(1));

        restricted_domain(-5, 5);
        assert_eq!(
            //[1,1] / [-3,0] = [-1,-1]/[0,3] = [-inf, 0]
            singleton(1) / [0, 3].into(),
            x_to_inf(0)
        );
        assert_eq!(
            //[-3,-1]/[-3,0] = [1,3]/[0,3] = [0, inf]
            Interval::div([-3, -1].into(), [-3, 0].into()),
            x_to_inf(0)
        );
        //[-5,-1] / [0,2] = [-inf, inf]
        //assert_eq!(Interval::from("[-5,1]") / "[0,2]".into(), TOP);
    }
}
