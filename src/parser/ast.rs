use std::{
    collections::HashSet,
    hash::Hash,
    ops::{Neg, Not},
};

#[derive(Hash, PartialOrd, Ord, Eq, Debug, Clone, PartialEq)]
pub struct Position {
    pub line: usize,
    pub clm: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Statement<'a> {
    Assignment(Assignment<'a>),
    Skip,
    Composition {
        lhs: Box<Statement<'a>>,
        rhs: Box<Statement<'a>>,
    },
    Conditional {
        guard: Box<BooleanExp<'a>>,
        true_branch: Box<Statement<'a>>,
        false_branch: Box<Statement<'a>>,
    },
    While {
        line: Position,
        guard: Box<BooleanExp<'a>>,
        body: Box<Statement<'a>>,
    },
}

impl<'a> Statement<'a> {
    pub fn extract_vars(&self) -> HashSet<&'a str> {
        match self {
            Statement::Skip => HashSet::new(),
            Statement::Assignment(Assignment { var, value: _ }) => HashSet::from([&var[..]]),
            Statement::Composition { lhs, rhs } => lhs
                .extract_vars()
                .into_iter()
                .chain(rhs.extract_vars())
                .collect(),
            Statement::Conditional {
                guard,
                true_branch: lhs,
                false_branch: rhs,
            } => Self::extract_vars(&lhs)
                .into_iter()
                .chain(rhs.extract_vars())
                .chain(guard.extract_vars())
                .collect(),
            Statement::While {
                line: _,
                guard,
                body,
            } => Self::extract_vars(&body)
                .into_iter()
                .chain(guard.extract_vars())
                .collect(),
        }
    }

    pub fn extract_constant(&self) -> HashSet<i64> {
        match self {
            Statement::Skip => HashSet::new(),
            Statement::Assignment(Assignment { var: _, value }) => value.extract_constants(),
            Statement::Conditional {
                guard,
                true_branch,
                false_branch,
            } => guard
                .extract_constant()
                .into_iter()
                .chain(true_branch.extract_constant())
                .chain(false_branch.extract_constant())
                .collect(),
            Statement::Composition { lhs, rhs } => lhs
                .extract_constant()
                .into_iter()
                .chain(rhs.extract_constant())
                .collect(),
            Statement::While {
                line: _,
                guard,
                body,
            } => guard
                .extract_constant()
                .into_iter()
                .chain(body.extract_constant())
                .collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Assignment<'a> {
    pub var: &'a str,
    pub value: Box<ArithmeticExp<'a>>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ArithmeticExp<'a> {
    Integer(i64),
    Variable(&'a str),
    BinaryOperation {
        lhs: Box<ArithmeticExp<'a>>,
        operator: Operator,
        rhs: Box<ArithmeticExp<'a>>,
    },
}

impl<'a> ArithmeticExp<'a> {
    pub fn extract_constants(&self) -> HashSet<i64> {
        match self {
            ArithmeticExp::Integer(x) => HashSet::from([*x]),
            ArithmeticExp::BinaryOperation {
                lhs,
                operator: _,
                rhs,
            } => lhs
                .extract_constants()
                .into_iter()
                .chain(rhs.extract_constants())
                .collect(),
            _ => HashSet::new(),
        }
    }

    pub fn extract_vars(&self) -> HashSet<&'a str> {
        match self {
            ArithmeticExp::Variable(x) => HashSet::from([*x]),
            ArithmeticExp::Integer(_) => HashSet::new(),
            ArithmeticExp::BinaryOperation {
                lhs,
                operator: _,
                rhs,
            } => lhs
                .extract_vars()
                .into_iter()
                .chain(rhs.extract_vars())
                .collect(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ArithmeticCondition<'a> {
    pub lhs: Box<ArithmeticExp<'a>>,
    pub operator: ConditionOperator,
}

impl<'a> ArithmeticCondition<'a> {
    pub fn normal_form(
        lhs: Box<ArithmeticExp<'a>>,
        operator: ConditionOperator,
        rhs: Box<ArithmeticExp<'a>>,
    ) -> Self {
        if *rhs.as_ref() == ArithmeticExp::Integer(0) {
            return ArithmeticCondition { lhs, operator };
        }
        let lhs = Box::new(ArithmeticExp::BinaryOperation {
            lhs,
            operator: Operator::Sub,
            rhs,
        });
        ArithmeticCondition { lhs, operator }
    }
}

impl<'a> Not for ArithmeticCondition<'a> {
    type Output = Self;
    fn not(self) -> Self::Output {
        ArithmeticCondition {
            operator: -self.operator,
            ..self
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum BooleanExp<'a> {
    Boolean(bool),
    ArithmeticCondition(ArithmeticCondition<'a>),
    And {
        lhs: Box<BooleanExp<'a>>,
        rhs: Box<BooleanExp<'a>>,
    },
    Or {
        lhs: Box<BooleanExp<'a>>,
        rhs: Box<BooleanExp<'a>>,
    },
}

impl<'a> Not for BooleanExp<'a> {
    type Output = BooleanExp<'a>;
    fn not(self) -> Self::Output {
        match self {
            BooleanExp::Boolean(x) => BooleanExp::Boolean(!x),
            BooleanExp::ArithmeticCondition(x) => BooleanExp::ArithmeticCondition(!x),
            BooleanExp::And { lhs, rhs } => BooleanExp::Or {
                lhs: Box::new(!*lhs),
                rhs: Box::new(!*rhs),
            },

            BooleanExp::Or { lhs, rhs } => BooleanExp::And {
                lhs: Box::new(!*lhs),
                rhs: Box::new(!*rhs),
            },
        }
    }
}

impl<'a> BooleanExp<'a> {
    pub fn extract_constant(&self) -> HashSet<i64> {
        match self {
            BooleanExp::ArithmeticCondition(ArithmeticCondition { lhs, operator: _ }) => {
                lhs.extract_constants()
            }
            BooleanExp::And { lhs, rhs } | BooleanExp::Or { lhs, rhs } => lhs
                .extract_constant()
                .into_iter()
                .chain(rhs.extract_constant())
                .collect(),
            _ => HashSet::new(),
        }
    }

    pub fn extract_vars(&self) -> HashSet<&'a str> {
        match self {
            BooleanExp::Boolean(_) => HashSet::new(),
            BooleanExp::ArithmeticCondition(ArithmeticCondition { lhs, operator: _ }) => {
                lhs.extract_vars()
            }
            BooleanExp::And { lhs, rhs } | BooleanExp::Or { lhs, rhs } => lhs
                .extract_vars()
                .into_iter()
                .chain(rhs.extract_vars())
                .collect(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ConditionOperator {
    Equal,
    NotEqual,
    StrictlyLess,
    GreaterOrEqual,
}

impl Neg for ConditionOperator {
    type Output = Self;
    fn neg(self) -> Self::Output {
        match self {
            ConditionOperator::Equal => ConditionOperator::NotEqual,
            ConditionOperator::NotEqual => ConditionOperator::Equal,
            ConditionOperator::StrictlyLess => ConditionOperator::GreaterOrEqual,
            ConditionOperator::GreaterOrEqual => ConditionOperator::StrictlyLess,
        }
    }
}
