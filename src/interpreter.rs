use std::collections::{BTreeMap, HashMap, HashSet};

use crate::{
    abstract_domains::abstract_domain::AbstractDomain,
    ast::{ArithmeticExp, Assignment, BooleanExp, Operator, Statement},
    propagation_algo::PropagationAlgo,
    state::State,
};

pub type Invariant<'a, D> = State<'a, D>;

pub struct Interpreter<'a, D: AbstractDomain> {
    program: &'a Statement<'a>,
    widening_thresholds: HashSet<i64>,
    invariants: BTreeMap<usize, Invariant<'a, D>>,
    initial_state: State<'a, D>,
}

impl<'a, D: AbstractDomain> Interpreter<'a, D> {
    pub fn build(
        program: &'a Statement<'a>,
        given_vars: HashMap<&'a str, &str>,
    ) -> Interpreter<'a, D> {
        D::init();

        let consts = program.extract_constant();

        let vars = program
            .extract_vars()
            .into_iter()
            .map(|var| {
                let mut value = D::top();
                if given_vars.contains_key(var) {
                    value = D::try_from(given_vars[var]).unwrap_or(D::top());
                }
                (var, value)
            })
            .collect::<HashMap<&str, D>>();

        let initial_state = State::new(vars);
        dbg!(&initial_state);

        Interpreter {
            program,
            widening_thresholds: consts,
            invariants: BTreeMap::new(),
            initial_state,
        }
    }

    pub fn interpret(&mut self) -> BTreeMap<usize, Invariant<'a, D>> {
        let program = self.program;
        let initial_state = self.initial_state.clone();
        let last_state = self.abstract_statement_eval(program, &initial_state);
        self.invariants.insert(usize::MAX, last_state);
        self.invariants.clone()
    }

    fn abstract_aexp_eval(exp: &ArithmeticExp<'a>, state: &State<'a, D>) -> D {
        match exp {
            ArithmeticExp::Variable(var) => state.lookup(var).clone(),
            ArithmeticExp::Integer(x) => D::constant_abstraction(*x),
            ArithmeticExp::BinaryOperation { lhs, operator, rhs } => {
                let lhs_value = Self::abstract_aexp_eval(lhs, state);
                let rhs_value = Self::abstract_aexp_eval(rhs, state);
                match operator {
                    Operator::Add => lhs_value + rhs_value,
                    Operator::Sub => lhs_value - rhs_value,
                    Operator::Mul => lhs_value * rhs_value,
                    Operator::Div => lhs_value / rhs_value,
                }
            }
        }
    }

    fn abstract_bexp_eval(exp: &BooleanExp<'a>, state: &State<'a, D>) -> State<'a, D> {
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

    fn abstract_statement_eval(
        &mut self,
        statement: &Statement<'a>,
        state: &State<'a, D>,
    ) -> State<'a, D> {
        if *state == State::bottom() {
            return State::bottom();
        }
        match statement {
            Statement::Skip => state.clone(),
            Statement::Assignment(Assignment { var, value }) => {
                let mut updated_state = state.clone();
                updated_state.update(&var, Self::abstract_aexp_eval(value, state));
                updated_state
            }
            Statement::Composition { lhs, rhs } => {
                let state = self.abstract_statement_eval(lhs, state);
                let state = self.abstract_statement_eval(rhs, &state);
                state
            }
            Statement::Conditional {
                guard,
                true_branch,
                false_branch,
            } => {
                let t = self
                    .abstract_statement_eval(true_branch, &Self::abstract_bexp_eval(guard, state));

                let f = self.abstract_statement_eval(
                    false_branch,
                    &Self::abstract_bexp_eval(&!*guard.clone(), state),
                );

                t.union(&f)
            }
            Statement::While { line, guard, body } => {
                //Sem[while b do S]R = C[!b]lim F where F(X) = X widened (R U Sem[S]C[b]X)
                //want to compute the lim F = least fixed point starting from bottom
                //nested loops: what about invariant?
                //depends on outer loop state, which can vary at each iteration
                //for sure it is upper bounded from its evaluation given the outer loop invariant as state

                let mut fixpoint = false;
                let mut x = state.clone();
                let mut iter = vec![x.clone()];
                while !fixpoint {
                    let body_eval =
                        self.abstract_statement_eval(body, &Self::abstract_bexp_eval(guard, &x));
                    let current = x.widening(&state.union(&body_eval), &self.widening_thresholds);
                    fixpoint = x == current;
                    x = current;
                    iter.push(x.clone());
                }

                println!("Seeking loop invariant");
                print_as_table(iter);

                let mut narrowing_iter = vec![x.clone()];
                fixpoint = false;
                while !fixpoint {
                    let k =
                        self.abstract_statement_eval(body, &Self::abstract_bexp_eval(guard, &x));
                    let next = x.narrowing(&k);
                    fixpoint = next == x;
                    x = next;
                    narrowing_iter.push(x.clone());
                }

                println!("Refine loop invariant with narrowing");
                print_as_table(narrowing_iter);

                self.invariants.insert(*line, x.clone());

                let post_condition = Self::abstract_bexp_eval(&!*guard.clone(), &x);

                post_condition
            }
        }
    }
}

fn print_as_table<'a, D: AbstractDomain>(v: Vec<State<'a, D>>) {
    if v.is_empty() {
        return;
    }

    let vars = v[0].vars();
    let vars = vars
        .into_iter()
        .map(|var| {
            let values = v
                .iter()
                .map(|s| Into::<String>::into(*s.lookup(var)))
                .reduce(|acc, e| format!("{acc}         {e}"))
                .unwrap();
            format!("{var} -> {}", values)
        })
        .reduce(|acc, e| format!("{acc}\n{e}"))
        .unwrap();

    dbg!(vars);
}
