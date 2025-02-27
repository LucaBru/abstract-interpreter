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
struct Node<D: AbstractDomain> {
    value: RefCell<D>,
    operator: Option<Op>,
    children: Vec<Rc<Node<D>>>,
}

impl<D: AbstractDomain> Node<D> {
    fn build<'a>(
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

    fn forward_analysis(&self) {
        self.children
            .iter()
            .for_each(|child| child.forward_analysis());

        if self.operator.is_none() {
            return;
        }
        dbg!(self.operator.clone().unwrap(), *self.value.borrow());
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
                    dbg!(
                        *self.children[0].value.borrow(),
                        *self.children[1].value.borrow(),
                        *self.children[0].value.borrow() - *self.children[1].value.borrow(),
                    );
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
        dbg!(*self.value.borrow());
    }

    fn backward_analysis(&self) {
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
}

pub struct PropagationAlgorithm<'a, 'b, D: AbstractDomain> {
    tree: Rc<Node<D>>,
    state: &'b State<'a, D>,
    var_leafs: HashMap<&'a str, Rc<Node<D>>>,
}

impl<'a, 'b, D: AbstractDomain> PropagationAlgorithm<'a, 'b, D> {
    pub fn build(exp: &BooleanExp<'a>, state: &'b State<'a, D>) -> Self {
        dbg!(exp);
        let mut var_leafs = HashMap::new();
        let tree = Node::build(exp, state, &mut var_leafs);

        PropagationAlgorithm {
            tree,
            state,
            var_leafs,
        }
    }

    pub fn local_iterations(&self) -> State<'a, D> {
        let clone_var_leafs = || -> HashMap<&str, D> {
            self.var_leafs
                .iter()
                .map(|(var, node)| (*var, *node.value.borrow()))
                .collect()
        };

        let mut fixpoint = false;
        while !fixpoint {
            self.tree.forward_analysis();
            let prev = clone_var_leafs();
            dbg!(&prev);
            self.tree.backward_analysis();
            dbg!(clone_var_leafs());
            fixpoint = prev == clone_var_leafs();
        }

        if *self.tree.value.borrow() == D::bottom() {
            return State::bottom();
        }

        let mut state = self.state.clone();
        self.var_leafs
            .iter()
            .for_each(|(var, node)| state.update(var, *node.value.borrow()));

        state
    }
}
