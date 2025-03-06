use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    abstract_domains::abstract_domain::AbstractDomain,
    parser::ast::{ArithmeticExp, Operator},
    state::State,
};

pub enum Node<D: AbstractDomain> {
    Internal {
        value: RefCell<D>,
        operator: Operator,
        left: Rc<Node<D>>,
        right: Rc<Node<D>>,
    },
    VarLeaf {
        value: RefCell<D>,
    },
    ConstantLeaf {
        value: D,
    },
}

impl<D: AbstractDomain> Node<D> {
    pub fn build<'a>(
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
                left.forward_analysis();
                right.forward_analysis();
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
            Node::VarLeaf { value } => {
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
            | Node::VarLeaf { value } => *value.borrow(),
        }
    }

    fn inner_pretty_print(&self, indent: String, last: bool) {
        let node_type = match self {
            Node::Internal {
                value: _,
                operator,
                left: _,
                right: _,
            } => match operator {
                Operator::Add => "+".to_string(),
                Operator::Sub => "-".to_string(),
                Operator::Mul => "*".to_string(),
                Operator::Div => "/".to_string(),
            },
            Node::ConstantLeaf { value: _ } => "Const".to_string(),
            Node::VarLeaf { value: _ } => "Var".to_string(),
        };

        println!(
            "{indent}{node_type} {}",
            <D as Into<String>>::into(self.get_value()),
        );

        let mut new_indent = format!("{indent}|  ");
        if last {
            new_indent = format!("{indent}   ");
        }

        match self {
            Node::Internal {
                value: _,
                operator: _,
                left,
                right,
            } => {
                left.inner_pretty_print(new_indent.clone(), false);
                right.inner_pretty_print(new_indent, true);
            }
            _ => (),
        }
    }

    pub fn pretty_print(&self) {
        self.inner_pretty_print(
            "".to_string(),
            matches!(self, Node::Internal {
                value: _,
                operator: _,
                left: _,
                right: _
            }),
        );
    }
}
