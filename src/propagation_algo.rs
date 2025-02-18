use std::{collections::HashMap, usize};

use crate::{
    abstract_domains::abstract_domain::{AbstractDomain, IntervalBound},
    ast::{ArithmeticCondition, ArithmeticExp, BooleanExp, ConditionOperator, Operator},
    state::State,
};

#[derive(Clone, PartialEq, Debug)]
struct Node<'a, D: AbstractDomain> {
    value: D,
    var: Option<&'a str>,
}

impl<'a, D: AbstractDomain> Node<'a, D> {
    fn backward_update(&mut self, value: D) {
        if self.var.is_some() {
            self.value = value;
        }
    }
}

fn left_child(i: usize) -> usize {
    i * 2 + 1
}

fn right_child(i: usize) -> usize {
    i * 2 + 2
}

#[derive(Debug)]
pub struct PropagationAlgo<'a, D: AbstractDomain> {
    values: HashMap<usize, Node<'a, D>>,
}

impl<'a, D: AbstractDomain> PropagationAlgo<'a, D> {
    pub fn build(exp: &BooleanExp<'a>, state_prop: &State<'a, D>) -> PropagationAlgo<'a, D> {
        PropagationAlgo {
            values: Self::build_from_bexp(exp, state_prop, 0),
        }
    }

    fn build_from_bexp(
        exp: &BooleanExp<'a>,
        state_prop: &State<'a, D>,
        i: usize,
    ) -> HashMap<usize, Node<'a, D>> {
        let mut sub_trees_hashmap = match exp {
            BooleanExp::And { lhs, rhs } | BooleanExp::Or { lhs, rhs } => {
                Self::build_from_bexp(lhs, state_prop, left_child(i))
                    .into_iter()
                    .chain(Self::build_from_bexp(rhs, state_prop, right_child(i)))
                    .collect()
            }
            BooleanExp::ArithmeticCondition(ArithmeticCondition { lhs, operator: _ }) => {
                Self::build_from_aexp(lhs, state_prop, left_child(i))
            }
            _ => HashMap::new(),
        };

        let v = match exp {
            BooleanExp::Boolean(x) if *x == false => D::bottom(),
            _ => D::top(),
        };

        sub_trees_hashmap.insert(i, Node {
            value: v,
            var: None,
        });
        sub_trees_hashmap
    }

    fn build_from_aexp(
        exp: &ArithmeticExp<'a>,
        state: &State<'a, D>,
        i: usize,
    ) -> HashMap<usize, Node<'a, D>> {
        let v = match exp {
            ArithmeticExp::Variable(var) => Node {
                value: state.lookup(var).clone(),
                var: Some(var),
            },
            ArithmeticExp::Integer(c) => Node {
                value: D::constant_abstraction(*c),
                var: None,
            },
            _ => Node {
                value: D::top(),
                var: None,
            },
        };

        let mut sub_tree_hashmap = match exp {
            ArithmeticExp::BinaryOperation {
                lhs,
                operator: _,
                rhs,
            } => Self::build_from_aexp(lhs, state, left_child(i))
                .into_iter()
                .chain(Self::build_from_aexp(rhs, state, right_child(i)))
                .collect(),
            _ => HashMap::new(),
        };
        sub_tree_hashmap.insert(i, v);
        sub_tree_hashmap
    }

    fn forward_bexp(exp: &BooleanExp<'a>, tree: &mut HashMap<usize, Node<D>>, i: usize) {
        match exp {
            BooleanExp::And { lhs, rhs } => {
                Self::forward_bexp(lhs, tree, left_child(i));
                Self::forward_bexp(rhs, tree, right_child(i));
                tree.get_mut(&i).unwrap().value = tree[&left_child(i)]
                    .value
                    .intersection_abstraction(&tree[&(right_child(i))].value);
            }
            BooleanExp::Or { lhs, rhs } => {
                Self::forward_bexp(lhs, tree, left_child(i));
                Self::forward_bexp(rhs, tree, right_child(i));
                tree.get_mut(&i).unwrap().value = tree[&(left_child(i))]
                    .value
                    .union_abstraction(&tree[&(right_child(i))].value);
            }
            BooleanExp::ArithmeticCondition(ArithmeticCondition { lhs, operator }) => {
                /*
                Note that in case of constant domain, arithmetic forward operator are very rough, i.e. they cause precision loss
                e.g. take x = [60, 60]. Given x < 0 we obtain an operator which is equal to
                */
                Self::forward_aexp(lhs, tree, left_child(i));
                let slice = match operator {
                    ConditionOperator::Equal => D::constant_abstraction(0),
                    ConditionOperator::NotEqual => D::top(),
                    ConditionOperator::StrictlyLess => {
                        D::interval_abstraction(IntervalBound::NegInf, IntervalBound::Num(-1))
                    }
                    _ => D::interval_abstraction(IntervalBound::Num(0), IntervalBound::PosInf),
                };

                tree.get_mut(&i).unwrap().value = tree[&(left_child(i))]
                    .value
                    .intersection_abstraction(&slice)
            }
            _ => (),
        };
    }

    fn forward_aexp(exp: &ArithmeticExp<'a>, tree: &mut HashMap<usize, Node<D>>, i: usize) {
        match exp {
            ArithmeticExp::BinaryOperation { lhs, operator, rhs } => {
                Self::forward_aexp(lhs, tree, left_child(i));
                Self::forward_aexp(rhs, tree, right_child(i));
                let forward_op = match operator {
                    Operator::Add => D::add,
                    Operator::Sub => D::sub,
                    Operator::Mul => D::mul,
                    Operator::Div => D::div,
                };
                tree.get_mut(&i).unwrap().value = forward_op(
                    tree[&(left_child(i))].value.clone(),
                    tree[&(right_child(i))].value.clone(),
                );
            }
            _ => (),
        };
    }

    fn backward_bexp(exp: &BooleanExp<'a>, tree: &mut HashMap<usize, Node<D>>, i: usize) {
        match exp {
            BooleanExp::And { lhs, rhs } | BooleanExp::Or { lhs, rhs } => {
                let value = tree[&i].value;
                if let Some(left_node) = tree.get_mut(&(left_child(i))) {
                    left_node.backward_update(value);
                };
                if let Some(right_node) = tree.get_mut(&(left_child(i))) {
                    right_node.backward_update(value);
                };
                Self::backward_bexp(lhs, tree, left_child(i));
                Self::backward_bexp(rhs, tree, right_child(i));
            }
            BooleanExp::ArithmeticCondition(ArithmeticCondition { lhs, operator: _ }) => {
                tree.get_mut(&(left_child(i))).unwrap().value = tree[&i].value;
                Self::backward_aexp(lhs, tree, left_child(i));
            }
            _ => (),
        };
    }

    fn backward_aexp(exp: &ArithmeticExp<'a>, tree: &mut HashMap<usize, Node<D>>, i: usize) {
        match exp {
            ArithmeticExp::BinaryOperation { lhs, operator, rhs } => {
                let refinement = D::backward_arithmetic_operator(
                    tree[&(left_child(i))].value,
                    tree[&(right_child(i))].value,
                    tree[&i].value,
                    *operator,
                );

                tree.get_mut(&(left_child(i)))
                    .unwrap()
                    .backward_update(refinement[0]);
                Self::backward_aexp(lhs, tree, left_child(i));

                tree.get_mut(&(right_child(i)))
                    .unwrap()
                    .backward_update(refinement[1]);
                Self::backward_aexp(rhs, tree, right_child(i));
            }
            _ => (),
        }
    }

    pub fn propagation(&mut self, exp: &BooleanExp<'a>) -> (bool, HashMap<&'a str, D>) {
        let mut fixed_point = false;
        let mut satisfiable = true;
        while !fixed_point && satisfiable {
            let prev = self.values.clone();
            Self::forward_bexp(exp, &mut self.values, 0);

            satisfiable = self.values[&0].value != D::bottom();
            if satisfiable {
                Self::backward_bexp(exp, &mut self.values, 0);
                fixed_point = prev == self.values;
            }
        }

        let updated_vars = self
            .values
            .values()
            .filter(|node| node.var.is_some())
            .map(|node| (node.var.clone().unwrap(), node.value))
            .collect();
        (satisfiable, updated_vars)
    }
}
