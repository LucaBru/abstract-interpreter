use crate::{
    abstract_domains::abstract_domain::AbstractDomain,
    ast::{ArithmeticExp, Assignment, BooleanExp, Operator, Statement},
    propagation_algo::PropagationAlgo,
    state::State,
};

type Invariant<'a, D> = State<'a, D>;

fn abstract_aexp_eval<'a, D: AbstractDomain>(exp: &ArithmeticExp<'a>, state: &State<'a, D>) -> D {
    match exp {
        ArithmeticExp::Variable(var) => state.lookup(var).clone(),
        ArithmeticExp::Integer(x) => D::constant_abstraction(*x),
        ArithmeticExp::BinaryOperation { lhs, operator, rhs } => {
            let lhs_value = abstract_aexp_eval(lhs, state);
            let rhs_value = abstract_aexp_eval(rhs, state);
            match operator {
                Operator::Add => lhs_value + rhs_value,
                Operator::Sub => lhs_value - rhs_value,
                Operator::Mul => lhs_value * rhs_value,
                Operator::Div => lhs_value / rhs_value,
            }
        }
    }
}

fn abstract_bexp_semantic<'a, D: AbstractDomain>(
    exp: &BooleanExp<'a>,
    state: &State<'a, D>,
) -> State<'a, D> {
    let mut prop_algo = PropagationAlgo::build(exp, state);
    let (satisfiable, updated_vars) = prop_algo.refinement_propagation(exp);

    if !satisfiable {
        return State::bottom();
    }

    let mut state = state.clone();
    updated_vars
        .into_iter()
        .for_each(|(var, value)| state.update(var, value));
    state
}

fn abstract_statement_semantic<'a, D: AbstractDomain>(
    statement: &Statement<'a>,
    state: &State<'a, D>,
) -> State<'a, D> {
    match statement {
        Statement::Skip => state.clone(),
        Statement::Assignment(Assignment { var, value }) => {
            let mut updated_state = state.clone();
            updated_state.update(&var, abstract_aexp_eval(value, state));
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

            let not_guard = !*guard.clone();
            let false_state = abstract_statement_semantic(
                false_branch,
                &abstract_bexp_semantic(&not_guard, state),
            );

            true_state.union(&false_state)
        }
        Statement::While { guard, body } => {
            panic!()
            //TODO: compute least fixed point using chaotic iterations
        }
    }
}

pub fn interpreter<'a, D: AbstractDomain>(
    program: &Statement<'a>,
    state: &State<'a, D>,
) -> Vec<Invariant<'a, D>> {
    //TODO: add loops invariant
    Vec::from([abstract_statement_semantic(program, state)])
}
