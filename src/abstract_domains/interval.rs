use std::{
    cmp::{Ordering, max, min},
    ops::{Add, Div, Mul, Sub},
    sync::RwLock,
};

use super::{abstract_domain::AbstractDomain, int::Int};

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

#[derive(Clone, Debug, Eq)]
pub struct Interval {
    low: Int,
    upper: Int,
}

impl Interval {
    fn normal_form(low: Int, upper: Int) -> Self {
        let m = *M.read().unwrap();
        let n = *N.read().unwrap();

        if m > n && low != upper {
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
    fn arithmetic_cond_abstraction(&self, c: crate::ast::ArithmeticCondition) -> Self {
        panic!()
    }
    fn assignment_abstraction(&self, a: crate::ast::Assignment) -> Self {
        panic!()
    }
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
}
/*
impl BackwardAdd for Interval {
    type Output = Self;
    fn backward_add(self, lhs: Self, rhs: Self) -> Refinement<Self> {
        Refinement {
            lhs: lhs.intersect(self - rhs),
            rhs: rhs.intersect(self - lhs),
        }
    }
}

impl BackwardSub for Interval {
    type Output = Self;
    fn backward_sub(self, lhs: Self, rhs: Self) -> Refinement<Self> {
        Refinement {
            lhs: lhs.intersect(self + rhs),
            rhs: rhs.intersect(lhs - self),
        }
    }
}

impl BackwardMul for Interval {
    type Output = Self;
    fn backward_mul(self, lhs: Self, rhs: Self) -> Refinement<Self> {
        Refinement {
            lhs: lhs.intersect(self / rhs),
            rhs: rhs.intersect(self / lhs),
        }
    }
}

impl BackwardDiv for Interval {
    type Output = Self;
    fn backward_div(self, lhs: Self, rhs: Self) -> Refinement<Self> {
        let one = Interval::from("[-1,1]");
        Refinement {
            lhs: lhs.intersect((self + one) * rhs),
            rhs: rhs.intersect(lhs / (self + one)),
        }
    }
}
*/

#[cfg(test)]
mod test {
    use std::{
        cmp::Ordering,
        ops::{Add, Div, Mul},
    };

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
        dbg!(Interval::partial_cmp(&[-3, 2].into(), &[-5, 2].into()));
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

    /*
    #[test]
    fn intv_abs_domain_abstraction() {
        let conc_intv = ConcreteIntv(I::Num(0), I::Num(10));

        set_domain_bounds(1.into(), 0.into());
        assert_eq!(Interval::intv_abstraction(conc_intv), TOP);

        set_domain_bounds(0.into(), 10.into());
        assert_eq!(Interval::intv_abstraction(conc_intv), "[0,10]".into());

        set_domain_bounds(1.into(), 100.into());
        assert_eq!(Interval::intv_abstraction(conc_intv), "[-inf, 10]".into());

        set_domain_bounds(0.into(), 5.into());
        assert_eq!(Interval::intv_abstraction(conc_intv), "[0, inf]".into());

        set_domain_bounds(1.into(), 2.into());
        assert_eq!(Interval::intv_abstraction(conc_intv), TOP);

        set_domain_bounds(100.into(), 100.into());
        assert_eq!(Interval::intv_abstraction(conc_intv), "[-inf, 100]".into());
        set_domain_bounds((-100).into(), 100.into());
        assert_eq!(
            Interval::intv_abstraction(ConcreteIntv(I::NegInf, I::Num(-200))),
            Interval::from("[-inf,-100]")
        );
        assert_eq!(
            Interval::intv_abstraction(ConcreteIntv(I::Num(2000), I::PosInf)),
            Interval::from("[100, inf]")
        );

        interval_domain();
        assert_eq!(
            Interval::intv_abstraction(ConcreteIntv(I::NegInf, I::Num(0))),
            Interval::from("[-inf,0]")
        );
        assert_eq!(
            Interval::intv_abstraction(ConcreteIntv(I::Num(0), I::PosInf)),
            Interval::from("[0, inf]")
        );
    } */
}
