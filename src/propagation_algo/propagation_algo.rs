use std::{collections::HashMap, rc::Rc};

use crate::{
    abstract_domains::abstract_domain::AbstractDomain,
    parser::ast::BooleanExp,
    state::State,
};

use super::node::Node;

pub struct PropagationAlgorithm<'a, 'b, D: AbstractDomain> {
    tree: Rc<Node<D>>,
    state: &'b State<'a, D>,
    var_leafs: HashMap<&'a str, Rc<Node<D>>>,
}

impl<'a, 'b, D: AbstractDomain> PropagationAlgorithm<'a, 'b, D> {
    pub fn build(exp: &BooleanExp<'a>, state: &'b State<'a, D>) -> Self {
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
        println!("LOCAL ITERATIONS");
        let mut fixpoint = false;
        while !fixpoint {
            self.tree.forward_analysis();
            let prev: HashMap<&str, D> = clone_var_leafs();
            println!("After forward");
            self.tree
                .pretty_print("".to_string(), self.tree.children.len() == 1);

            self.tree.backward_analysis();
            println!("After backward");
            self.tree
                .pretty_print("".to_string(), self.tree.children.len() == 1);

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
