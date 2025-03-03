use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    abstract_domains::abstract_domain::{AbstractDomain, IntervalBound},
    parser::ast::{ArithmeticCondition, ArithmeticExp, BooleanExp, ConditionOperator, Operator},
    state::State,
};

#[derive(Debug, Clone)]
pub enum Op {
    Arithmetic(Operator),
    Cond(ConditionOperator),
    And,
    Or,
}

#[derive(Debug)]
pub struct Node<D: AbstractDomain> {
    pub value: RefCell<D>,
    pub operator: Option<Op>,
    pub children: Vec<Rc<Node<D>>>,
}

impl<D: AbstractDomain> Node<D> {
    pub fn build<'a>(
        exp: &BooleanExp<'a>,
        state: &State<'a, D>,
        var_leafs: &mut HashMap<&'a str, Rc<Self>>,
    ) -> Rc<Self> {
        match exp {
            BooleanExp::And { lhs, rhs } => Rc::new(Node {
                value: RefCell::new(D::top()),
                operator: Some(Op::And),
                children: vec![
                    Self::build(lhs, state, var_leafs),
                    Self::build(rhs, state, var_leafs),
                ],
            }),
            BooleanExp::Or { lhs, rhs } => Rc::new(Node {
                value: RefCell::new(D::top()),
                operator: Some(Op::Or),
                children: vec![
                    Self::build(lhs, state, var_leafs),
                    Self::build(rhs, state, var_leafs),
                ],
            }),
            BooleanExp::ArithmeticCondition(ArithmeticCondition { lhs, operator }) => {
                Rc::new(Node {
                    value: RefCell::new(D::top()),
                    operator: Some(Op::Cond(*operator)),
                    children: vec![Self::build_from_aexp(lhs, state, var_leafs)],
                })
            }
            BooleanExp::Boolean(x) if *x == true => Rc::new(Node {
                value: RefCell::new(D::top()),
                operator: None,
                children: vec![],
            }),
            _ => Rc::new(Node {
                value: RefCell::new(D::bottom()),
                operator: None,
                children: vec![],
            }),
        }
    }

    fn build_from_aexp<'a>(
        exp: &ArithmeticExp<'a>,
        state: &State<'a, D>,
        var_leafs: &mut HashMap<&'a str, Rc<Self>>,
    ) -> Rc<Self> {
        match exp {
            ArithmeticExp::Integer(c) => Rc::new(Node {
                value: RefCell::new(D::constant_abstraction(*c)),
                operator: None,
                children: vec![],
            }),
            ArithmeticExp::Variable(var) => {
                let leaf = var_leafs.get(var);
                if leaf.is_some() {
                    return Rc::clone(leaf.unwrap());
                }

                let node = Rc::new(Node {
                    value: RefCell::new(*state.lookup(var)),
                    operator: None,
                    children: vec![],
                });

                var_leafs.insert(var, Rc::clone(&node));
                node
            }
            ArithmeticExp::BinaryOperation { lhs, operator, rhs } => Rc::new(Node {
                value: RefCell::new(D::top()),
                operator: Some(Op::Arithmetic(*operator)),
                children: vec![
                    Self::build_from_aexp(lhs, state, var_leafs),
                    Self::build_from_aexp(rhs, state, var_leafs),
                ],
            }),
        }
    }

    pub fn forward_analysis(&self) {
        self.children
            .iter()
            .for_each(|child| child.forward_analysis());

        if self.operator.is_none() {
            return;
        }
        *self.value.borrow_mut() = match self.operator.clone().unwrap() {
            Op::Or => self.children[0]
                .value
                .borrow()
                .union_abstraction(&self.children[1].value.borrow()),
            Op::And => self.children[0]
                .value
                .borrow()
                .intersection_abstraction(&self.children[1].value.borrow()),
            Op::Arithmetic(operator) => match operator {
                Operator::Add => {
                    *self.children[0].value.borrow() + *self.children[1].value.borrow()
                }
                Operator::Sub => {
                    *self.children[0].value.borrow() - *self.children[1].value.borrow()
                }
                Operator::Mul => {
                    *self.children[0].value.borrow() * *self.children[1].value.borrow()
                }
                Operator::Div => {
                    *self.children[0].value.borrow() / *self.children[1].value.borrow()
                }
            },
            Op::Cond(operator) => {
                let slice = match operator {
                    ConditionOperator::Equal => D::constant_abstraction(0),
                    /*
                    ----------- 0 -------------
                            [       ]
                                [   ]
                        [   ]

                    Given intv x
                    1. intersect x with > 0
                    2. intersect x with < 0
                    3. 1. U 2.
                    */
                    ConditionOperator::NotEqual => [
                        D::interval_abstraction(IntervalBound::NegInf, IntervalBound::Num(-1)),
                        D::interval_abstraction(IntervalBound::Num(1), IntervalBound::PosInf),
                    ]
                    .map(|intv| intv.intersection_abstraction(&self.value.borrow()))
                    .into_iter()
                    .reduce(|acc, e| acc.union_abstraction(&e))
                    .unwrap(),
                    ConditionOperator::StrictlyLess => {
                        D::interval_abstraction(IntervalBound::NegInf, IntervalBound::Num(-1))
                    }
                    ConditionOperator::GreaterOrEqual => {
                        D::interval_abstraction(IntervalBound::Num(0), IntervalBound::PosInf)
                    }
                };
                self.children[0]
                    .value
                    .borrow()
                    .intersection_abstraction(&slice)
            }
        };
    }

    pub fn backward_analysis(&self) {
        if self.operator.is_none() {
            return;
        }

        match self.operator.clone().unwrap() {
            Op::And | Op::Or | Op::Cond(_) => self.children.iter().for_each(|child| {
                *child.value.borrow_mut() = *self.value.borrow();
                child.backward_analysis();
            }),
            Op::Arithmetic(op) => {
                let operands_refinement = D::backward_arithmetic_operator(
                    *self.children[0].value.borrow(),
                    *self.children[1].value.borrow(),
                    *self.value.borrow(),
                    op,
                );

                *self.children[0].value.borrow_mut() = operands_refinement[0];
                self.children[0].backward_analysis();

                *self.children[1].value.borrow_mut() = operands_refinement[1];
                self.children[1].backward_analysis();
            }
        }
    }

    pub fn pretty_print(&self, indent: String, last: bool) {
        let op = match self.operator.is_none() {
            true => "Leaf".to_string(),
            _ => {
                let op = match self.operator.clone().unwrap() {
                    Op::And => "&",
                    Op::Or => "or",
                    Op::Arithmetic(x) => match x {
                        Operator::Add => "+",
                        Operator::Sub => "-",
                        Operator::Mul => "*",
                        Operator::Div => "/",
                    },
                    Op::Cond(x) => match x {
                        ConditionOperator::Equal => "=",
                        ConditionOperator::GreaterOrEqual => ">=",
                        ConditionOperator::NotEqual => "!=",
                        ConditionOperator::StrictlyLess => "<",
                    },
                };
                op.to_string()
            }
        };
        println!(
            "{indent}{op} {}",
            <D as Into<String>>::into(*self.value.borrow(),),
        );

        let mut new_indent = format!("{indent}|  ");
        if last {
            new_indent = format!("{indent}   ");
        }
        self.children.iter().enumerate().for_each(|(idx, child)| {
            child.pretty_print(new_indent.clone(), idx == self.children.len())
        });
    }
}
