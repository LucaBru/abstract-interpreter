use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    abstract_domains::abstract_domain::{AbstractDomain, IntervalBound},
    interpreter::Interpreter,
    parser::ast::{ArithmeticCondition, ArithmeticExp, BooleanExp, ConditionOperator, Operator},
    state::State,
};

pub enum Node<'a, D: AbstractDomain> {
    Internal {
        value: RefCell<D>,
        operator: Operator,
        left: Rc<Node<'a, D>>,
        right: Rc<Node<'a, D>>,
    },
    VarLeaf {
        var: &'a str,
        value: RefCell<D>,
    },
    ConstantLeaf {
        value: D,
    },
}

impl<'a, D: AbstractDomain> Node<'a, D> {
    pub fn build(
        exp: &ArithmeticExp<'a>,
        state: &State<'a, D>,
        var_leafs: &mut HashMap<&'a str, Rc<Self>>,
    ) -> Rc<Self> {
        match exp {
            ArithmeticExp::Integer(c) => Rc::new(Node::ConstantLeaf {
                value: D::constant_abstraction(*c),
            }),
            ArithmeticExp::Variable(var) => {
                let node = Rc::new(Node::VarLeaf {
                    var,
                    value: RefCell::new(*state.lookup(var)),
                });
                var_leafs.insert(var, Rc::clone(&node));
                node
            }
            ArithmeticExp::BinaryOperation { lhs, operator, rhs } => Rc::new(Node::Internal {
                value: RefCell::new(D::top()),
                operator: *operator,
                left: Self::build(lhs, state, var_leafs),
                right: Self::build(rhs, state, var_leafs),
            }),
        }
    }

    pub fn forward_analysis(&self) {
        match self {
            Node::Internal {
                value,
                operator,
                left,
                right,
            } => {
                let op = match operator {
                    Operator::Add => D::add,
                    Operator::Sub => D::sub,
                    Operator::Mul => D::mul,
                    Operator::Div => D::div,
                };
                *value.borrow_mut() = op(left.get_value(), right.get_value());
            }
            _ => (),
        }
    }

    pub fn backward_analysis(&self, refinement: D) -> bool {
        match self {
            Node::Internal {
                value,
                operator,
                left,
                right,
            } => {
                *value.borrow_mut() = refinement;

                let refs = D::backward_arithmetic_operator(
                    left.get_value(),
                    right.get_value(),
                    *value.borrow(),
                    *operator,
                );

                left.backward_analysis(refs[0]) && right.backward_analysis(refs[1])
            }
            Node::ConstantLeaf { value } => {
                refinement.intersection_abstraction(value) != D::bottom()
            }
            Node::VarLeaf { var: _, value } => {
                let n = refinement.intersection_abstraction(&value.borrow());
                if n != D::bottom() {
                    *value.borrow_mut() = refinement;
                }
                n != D::bottom()
            }
        }
    }

    pub fn get_value(&self) -> D {
        match self {
            Node::ConstantLeaf { value } => *value,
            Node::Internal {
                value,
                operator: _,
                left: _,
                right: _,
            }
            | Node::VarLeaf { var: _, value } => *value.borrow(),
        }
    }
}
