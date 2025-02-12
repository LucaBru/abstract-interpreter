#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    Assignment(Assignment),
    Skip,
    Composition {
        lhs: Box<Statement>,
        rhs: Box<Statement>,
    },
    Conditional {
        guard: Box<BooleanExp>,
        true_branch: Box<Statement>,
        false_branch: Box<Statement>,
    },
    While {
        guard: Box<BooleanExp>,
        body: Box<Statement>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Assignment {
    pub var: String,
    pub value: Box<ArithmeticExp>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ArithmeticExp {
    Integer(i64),
    Variable(String),
    BinaryOperation {
        lhs: Box<ArithmeticExp>,
        operator: Operator,
        rhs: Box<ArithmeticExp>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ArithmeticCondition {
    pub lhs: Box<ArithmeticExp>,
    pub operator: ConditionOperator,
    pub rhs: Box<ArithmeticExp>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum BooleanExp {
    Boolean(bool),
    ArithmeticCondition(ArithmeticCondition),
    And {
        lhs: Box<BooleanExp>,
        rhs: Box<BooleanExp>,
    },
    Or {
        lhs: Box<BooleanExp>,
        rhs: Box<BooleanExp>,
    },
    Not(Box<BooleanExp>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum ConditionOperator {
    Equal,
    StrictlyGreater,
    StrictlyLess,
    GreaterOrEqual,
}
