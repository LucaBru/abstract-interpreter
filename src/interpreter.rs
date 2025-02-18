use std::collections::HashSet;

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

fn abstract_bexp_eval<'a, D: AbstractDomain>(
    exp: &BooleanExp<'a>,
    state: &State<'a, D>,
) -> State<'a, D> {
    let mut prop_algo = PropagationAlgo::build(exp, state);

    let (satisfiable, updated_vars) = prop_algo.propagation(exp);

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
    widening_thresholds: &HashSet<i64>,
) -> (State<'a, D>, Vec<Invariant<'a, D>>) {
    if *state == State::bottom() {
        return (State::bottom(), vec![]);
    }
    match statement {
        Statement::Skip => (state.clone(), vec![]),
        Statement::Assignment(Assignment { var, value }) => {
            let mut updated_state = state.clone();
            updated_state.update(&var, abstract_aexp_eval(value, state));
            (updated_state, vec![])
        }
        Statement::Composition { lhs, rhs } => {
            let (state, lhs_invariants) =
                abstract_statement_semantic(lhs, state, widening_thresholds);
            let (state, rhs_invariants) =
                abstract_statement_semantic(rhs, &state, widening_thresholds);
            (state, [lhs_invariants, rhs_invariants].concat())
        }
        Statement::Conditional {
            guard,
            true_branch,
            false_branch,
        } => {
            let (true_state, t) = abstract_statement_semantic(
                true_branch,
                &abstract_bexp_eval(guard, state),
                widening_thresholds,
            );

            let (false_state, f) = abstract_statement_semantic(
                false_branch,
                &abstract_bexp_eval(&!*guard.clone(), state),
                widening_thresholds,
            );

            (true_state.union(&false_state), [t, f].concat())
        }
        Statement::While { guard, body } => {
            //Sem[while b do S]R = C[!b]lim F where F(X) = X widened (R U Sem[S]C[b]X)
            //want to compute the lim F = least fixed point starting from bottom
            //nested loops: what about invariant?
            //depends on outer loop state, which can vary at each iteration
            //for sure it is upper bounded from its evaluation given the outer loop invariant as state
            let f = |x: &State<'a, D>| -> (State<'a, D>, Vec<Invariant<'a, D>>) {
                let (body_state, inner_invs) = abstract_statement_semantic(
                    body,
                    &abstract_bexp_eval(guard, x),
                    widening_thresholds,
                );
                (
                    x.widening(&state.union(&body_state), widening_thresholds),
                    inner_invs,
                )
            };

            let narrowing_refinement = |inv: State<'a, D>| {
                let mut fixpoint = false;
                let mut inv = inv;
                while !fixpoint {
                    let (k, _) = abstract_statement_semantic(
                        body,
                        &abstract_bexp_eval(guard, &inv),
                        widening_thresholds,
                    );
                    let next = inv.narrowing(&k);
                    dbg!(&next);
                    fixpoint = next == inv;
                    inv = next;
                }
                inv
            };

            let mut fixed_point = false;
            let mut inv = state.clone();
            let mut inner_loops_invs = vec![];
            let mut iter = vec![inv.clone()];
            while !fixed_point {
                let (current, inner_loops_invariant) = f(&inv);
                fixed_point = inv == current;
                if fixed_point {
                    inner_loops_invs = inner_loops_invariant;
                } else {
                    inv = current;
                    iter.push(inv.clone());
                }
            }

            inv = narrowing_refinement(inv);

            let post_condition = abstract_bexp_eval(&!*guard.clone(), &inv);
            let snap = SnapShot {
                body,
                pre_cond: state.clone(),
                post_cond: post_condition.clone(),
                iter,
            };

            snap.pretty_print();

            inner_loops_invs.push(inv);

            (post_condition, inner_loops_invs)
        }
    }
}
struct SnapShot<'a, 'b, D: AbstractDomain> {
    body: &'b Statement<'a>,
    pre_cond: State<'a, D>,
    post_cond: State<'a, D>,
    iter: Vec<State<'a, D>>,
}

impl<'a, 'b, D: AbstractDomain> SnapShot<'a, 'b, D> {
    fn pretty_print(self) {
        let format_var_iterations = |var: &'a str| -> String {
            self.iter
                .iter()
                .map(|i| {
                    let value = if i.vars().contains(var) {
                        i.lookup(var)
                    } else {
                        &D::bottom()
                    };
                    format!("{:#?}\t", Into::<String>::into(*value))
                })
                .collect()
        };

        println!("Semantic of loop with body: {:#?}", self.body);
        println!("Pre-condition: {:#?}", self.pre_cond);
        println!("Post-condition: {:#?}", self.post_cond);
        println!("Kleene iterations:");

        self.iter[0].vars().into_iter().for_each(|var| {
            println!("{var}: {}", format_var_iterations(var));
        });

        print!("\n\n");
    }
}

pub fn interpreter<'a, D: AbstractDomain>(
    program: &Statement<'a>,
    state: &State<'a, D>,
    widening_thresholds: &HashSet<i64>,
) -> Vec<Invariant<'a, D>> {
    let (last_state, mut invariants) =
        abstract_statement_semantic(program, state, widening_thresholds);
    invariants.insert(invariants.len(), last_state);
    invariants
}
