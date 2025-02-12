use std::collections::HashMap;

use crate::{
    abstract_domains::abstract_domain::AbstractDomain,
    ast::{ArithmeticExp, Assignment, BooleanExp, Statement},
    state::State,
};

type Invariant<'a, D: AbstractDomain> = State<'a, D>;

fn abstract_aexp_semantic<'a, D: AbstractDomain>(exp: &ArithmeticExp, state: &State<'a, D>) -> D {
    panic!()
}

fn abstract_bexp_semantic<'a, D: AbstractDomain>(
    exp: &BooleanExp,
    state: &State<'a, D>,
) -> State<'a, D> {
    //TODO: implement advanced bexp abstract semantic using local iterations
    panic!()
}

fn abstract_statement_semantic<'a, D: AbstractDomain + Clone>(
    statement: &Statement,
    state: &State<'a, D>,
) -> State<'a, D> {
    match statement {
        Statement::Skip => state.clone(),
        Statement::Assignment(Assignment { var, value }) => {
            let mut updated_state = state.clone();
            updated_state.update(&var, abstract_aexp_semantic(value, state));
            updated_state
        }
        Statement::Composition { lhs, rhs } => {
            let state = abstract_statement_semantic(lhs, state);
            abstract_statement_semantic(rhs, &state)
        }
        Statement::Conditional {
            guard,
            true_branch,
            false_branch,
        } => {
            let true_state =
                abstract_statement_semantic(true_branch, &abstract_bexp_semantic(guard, state));

            let false_state =
                abstract_statement_semantic(false_branch, &abstract_bexp_semantic(guard, state));
            true_state.union(&false_state)
        }
        Statement::While { guard, body } => {
            panic!()
            //TODO: compute least fixed point using chaotic iterations
        }
    }
}

pub fn interpreter<'a, D: AbstractDomain + Clone>(
    program: &Statement,
    state: &'a State<D>,
) -> Vec<Invariant<'a, D>> {
    //TODO: add loops invariant
    Vec::from([abstract_statement_semantic(program, state)])
}
