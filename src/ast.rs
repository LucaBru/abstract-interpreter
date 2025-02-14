use std::ops::{Neg, Not};

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
        guard: Box<BooleanExp<'a>>,
        body: Box<Statement<'a>>,
    },
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
        let lhs = Box::new(ArithmeticExp::BinaryOperation {
            lhs,
            operator: Operator::Sub,
            rhs,
        });
        ArithmeticCondition { lhs, operator }
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
    Not(Box<BooleanExp<'a>>),
}

impl<'a> Not for BooleanExp<'a> {
    type Output = BooleanExp<'a>;
    fn not(self) -> Self::Output {
        match self {
            BooleanExp::Boolean(x) => BooleanExp::Boolean(!x),
            BooleanExp::ArithmeticCondition(mut arithmetic_condition) => {
                arithmetic_condition.operator = -arithmetic_condition.operator;
                BooleanExp::ArithmeticCondition(arithmetic_condition)
            }
            BooleanExp::And { lhs, rhs } => BooleanExp::Or {
                lhs: Box::new(BooleanExp::Not(lhs)),
                rhs: Box::new(BooleanExp::Not(rhs)),
            },

            BooleanExp::Or { lhs, rhs } => BooleanExp::And {
                lhs: Box::new(BooleanExp::Not(lhs)),
                rhs: Box::new(BooleanExp::Not(rhs)),
            },
            BooleanExp::Not(x) => *x,
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
