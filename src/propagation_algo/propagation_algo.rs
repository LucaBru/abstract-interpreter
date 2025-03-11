use std::{collections::HashMap, rc::Rc};

use crate::{
    abstract_domains::abstract_domain::{AbstractDomain, IntervalBound},
    parser::ast::{ArithmeticCondition, ConditionOperator},
    state::State,
};

use super::node::Node;

pub struct PropagationAlgorithm<'a, 'b, D: AbstractDomain> {
    tree: Rc<Node<D>>,
    state: &'b State<'a, D>,
    var_leafs: HashMap<&'a str, Rc<Node<D>>>,
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

        let stl = D::interval_abstraction(IntervalBound::NegInf, IntervalBound::Num(-1));
        let gt = D::interval_abstraction(IntervalBound::Num(0), IntervalBound::PosInf);
        let sgt = D::interval_abstraction(IntervalBound::Num(1), IntervalBound::PosInf);

        let slice = &match self.cond {
            ConditionOperator::Equal => D::constant_abstraction(0),
            // eventually discard 0 if it is a bound
            ConditionOperator::NotEqual => stl
                .glb(&self.tree.get_value())
                .lub(&sgt.glb(&self.tree.get_value())),
            ConditionOperator::StrictlyLess => stl,
            ConditionOperator::GreaterOrEqual => gt,
        };

        println!("{:#?}", self.cond);

        let mut fixpoint = false;
        let mut satisfiable = true;
        while satisfiable && !fixpoint {
            self.tree.forward_analysis();

            println!("After forward analysis");
            self.tree.pretty_print();

            let prev: HashMap<&str, D> = clone_var_leafs();
            satisfiable = self
                .tree
                .backward_analysis(self.tree.get_value().glb(slice));

            println!("After backward analysis");
            self.tree.pretty_print();

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
