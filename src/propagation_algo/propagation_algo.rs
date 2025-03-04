use std::{collections::HashMap, rc::Rc};

use crate::{
    abstract_domains::abstract_domain::{AbstractDomain, IntervalBound},
    parser::ast::{ArithmeticCondition, BooleanExp, ConditionOperator, Operator},
    state::State,
};

// note that constant must not be update ==>
// given x = T ==> x < 0 & true ==> [-inf, -1] & T = [-inf, -1], which is propagated down
//  pay attention to the union case, and how the leafs value are aggregated together
use super::node::Node;

pub struct PropagationAlgorithm<'a, 'b, D: AbstractDomain> {
    tree: Rc<Node<'a, D>>,
    state: &'b State<'a, D>,
    // enough hashset
    var_leafs: HashMap<&'a str, Rc<Node<'a, D>>>,
    cond: ConditionOperator,
}

impl<'a, 'b, D: AbstractDomain> PropagationAlgorithm<'a, 'b, D> {
    pub fn build(exp: &ArithmeticCondition<'a>, state: &'b State<'a, D>) -> Self {
        let mut var_leafs = HashMap::new();
        let tree = Node::build(exp.lhs.as_ref(), state, &mut var_leafs);

        PropagationAlgorithm {
            tree,
            state,
            var_leafs,
            cond: exp.operator,
        }
    }

    pub fn local_iterations(&self) -> State<'a, D> {
        let clone_var_leafs = || -> HashMap<&str, D> {
            self.var_leafs
                .iter()
                .map(|(var, node)| (*var, node.get_value()))
                .collect()
        };

        let slice = match self.cond {
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
            .map(|intv| intv.intersection_abstraction(&self.tree.get_value()))
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

        let mut fixpoint = false;
        let mut satisfiable = true;
        while satisfiable && !fixpoint {
            self.tree.forward_analysis();
            let prev: HashMap<&str, D> = clone_var_leafs();
            satisfiable = self
                .tree
                .backward_analysis(self.tree.get_value().intersection_abstraction(&slice));
            fixpoint = prev == clone_var_leafs();
        }

        if !satisfiable {
            return State::bottom();
        }

        let mut state = self.state.clone();
        self.var_leafs
            .iter()
            .for_each(|(var, node)| state.update(var, node.get_value()));

        state
    }
}
