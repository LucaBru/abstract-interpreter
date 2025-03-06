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
        pos: Position,
        guard: Box<BooleanExp<'a>>,
        body: Box<Statement<'a>>,
    },
}

impl<'a> Statement<'a> {
    pub fn extract_vars(&self, vars: &mut HashSet<&'a str>) {
        match self {
            Statement::Skip => (),
            Statement::Assignment(Assignment { var, value: _ }) => {
                vars.insert(var);
            }
            Statement::Composition { lhs, rhs } => {
                lhs.extract_vars(vars);
                rhs.extract_vars(vars);
            }
            Statement::Conditional {
                guard,
                true_branch: lhs,
                false_branch: rhs,
            } => {
                guard.extract_vars(vars);
                lhs.extract_vars(vars);
                rhs.extract_vars(vars);
            }
            Statement::While {
                pos: _,
                guard,
                body,
            } => {
                guard.extract_vars(vars);
                body.extract_vars(vars);
            }
        }
    }

    pub fn extract_constant(&self, consts: &mut HashSet<i64>) {
        match self {
            Statement::Skip => (),
            Statement::Assignment(Assignment { var: _, value }) => {
                value.extract_constants(consts);
            }
            Statement::Conditional {
                guard,
                true_branch,
                false_branch,
            } => {
                guard.extract_constant(consts);
                true_branch.extract_constant(consts);
                false_branch.extract_constant(consts);
            }
            Statement::Composition { lhs, rhs } => {
                lhs.extract_constant(consts);
                rhs.extract_constant(consts);
            }
            Statement::While {
                pos: _,
                guard,
                body,
            } => {
                guard.extract_constant(consts);
                body.extract_constant(consts);
            }
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
    pub fn extract_constants(&self, consts: &mut HashSet<i64>) {
        match self {
            ArithmeticExp::Integer(x) => {
                consts.insert(*x);
            }
            ArithmeticExp::BinaryOperation {
                lhs,
                operator: _,
                rhs,
            } => {
                lhs.extract_constants(consts);
                rhs.extract_constants(consts);
            }
            _ => (),
        }
    }

    pub fn extract_vars(&self, vars: &mut HashSet<&'a str>) {
        match self {
            ArithmeticExp::Variable(x) => {
                vars.insert(*x);
            }
            ArithmeticExp::Integer(_) => (),
            ArithmeticExp::BinaryOperation {
                lhs,
                operator: _,
                rhs,
            } => {
                lhs.extract_vars(vars);
                rhs.extract_vars(vars);
            }
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
    pub fn extract_constant(&self, consts: &mut HashSet<i64>) {
        match self {
            BooleanExp::ArithmeticCondition(ArithmeticCondition { lhs, operator: _ }) => {
                lhs.extract_constants(consts);
                consts.insert(0);
            }
            BooleanExp::And { lhs, rhs } | BooleanExp::Or { lhs, rhs } => {
                lhs.extract_constant(consts);
                rhs.extract_constant(consts);
            }
            _ => (),
        }
    }

    pub fn extract_vars(&self, vars: &mut HashSet<&'a str>) {
        match self {
            BooleanExp::Boolean(_) => (),
            BooleanExp::ArithmeticCondition(ArithmeticCondition { lhs, operator: _ }) => {
                lhs.extract_vars(vars);
            }
            BooleanExp::And { lhs, rhs } | BooleanExp::Or { lhs, rhs } => {
                lhs.extract_vars(vars);
                rhs.extract_vars(vars)
            }
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
