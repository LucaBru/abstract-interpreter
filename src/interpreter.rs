use std::{
    collections::{BTreeMap, HashMap, HashSet},
    env, usize,
};

use crate::{
    abstract_domains::abstract_domain::AbstractDomain,
    parser::ast::{ArithmeticExp, Assignment, BooleanExp, Operator, Position, Statement},
    propagation_algo::propagation_algo::PropagationAlgorithm,
    state::State,
};

pub type Invariant<'a, D> = State<'a, D>;

pub type ProgramInvariants<'a, D> = BTreeMap<Position, Invariant<'a, D>>;

pub struct Interpreter<'a, D: AbstractDomain> {
    program: &'a Statement<'a>,
    initial_state: State<'a, D>,
    widening_thresholds: HashSet<i64>,
    narrowing_steps: usize,
    invariants: ProgramInvariants<'a, D>,
}

impl<'a, D: AbstractDomain> Interpreter<'a, D> {
    pub fn build(
        program: &'a Statement<'a>,
        given_vars: HashMap<&'a str, &str>,
    ) -> Interpreter<'a, D> {
        D::init();

        let narrowing_steps = env::var("NARROWING_STEPS")
            .unwrap_or("0".to_string())
            .parse()
            .unwrap_or(0_usize);
        dbg!(narrowing_steps);

        let mut consts = HashSet::new();
        program.extract_constant(&mut consts);
        dbg!(&consts);

        let mut vars = HashSet::new();
        program.extract_vars(&mut vars);
        let mut vars: HashMap<&'a str, D> = vars.into_iter().map(|var| (var, D::top())).collect();
        given_vars.iter().for_each(|(var, value)| {
            vars.insert(var, D::try_from(value).unwrap_or(D::top()));
        });

        let initial_state = State::new(vars);
        dbg!(&initial_state);

        Interpreter {
            program,
            widening_thresholds: consts,
            invariants: BTreeMap::new(),
            initial_state,
            narrowing_steps,
        }
    }

    pub fn interpret(&mut self) -> ProgramInvariants<'a, D> {
        let program = self.program;
        let initial_state = self.initial_state.clone();
        let last_state = self.statement_eval(program, &initial_state);
        self.invariants.insert(
            Position {
                line: usize::MAX,
                clm: usize::MAX,
            },
            last_state,
        );
        self.invariants.clone()
    }

    pub fn aexp_eval(exp: &ArithmeticExp<'a>, state: &State<'a, D>) -> D {
        match exp {
            ArithmeticExp::Variable(var) => state.lookup(var).clone(),
            ArithmeticExp::Integer(x) => D::constant_abstraction(*x),
            ArithmeticExp::BinaryOperation { lhs, operator, rhs } => {
                let lhs_value = Self::aexp_eval(lhs, state);
                let rhs_value = Self::aexp_eval(rhs, state);
                match operator {
                    Operator::Add => lhs_value + rhs_value,
                    Operator::Sub => lhs_value - rhs_value,
                    Operator::Mul => lhs_value * rhs_value,
                    Operator::Div => lhs_value / rhs_value,
                }
            }
        }
    }

    fn bexp_eval(exp: &BooleanExp<'a>, state: &State<'a, D>) -> State<'a, D> {
        /*
        - propagation algorithm for aexp cond. ==> possible variable refinement
        - multiple  conditionals can be aggregated using &,| operators
        - 2 possible strategies:
            - calculate each cond (now the bexp is an aggregation (&, |) of states), aggregate the states and, given the resulting state re-eval the expr until a fixpoint is reached
            - what about nested aggregators? (A & B) | C -> programmatically easier to find a fixpoint for each aggregator operators
         */
        match exp {
            BooleanExp::Boolean(true) => state.clone(),
            BooleanExp::Boolean(false) => State::bottom(),
            BooleanExp::ArithmeticCondition(cond) => {
                let algo = PropagationAlgorithm::build(cond, state);
                algo.local_iterations()
            }
            BooleanExp::And { lhs, rhs } => {
                let mut fixpoint = false;
                let mut x = state.clone();
                while !fixpoint {
                    let current = Self::bexp_eval(lhs, &x).intersection(&Self::bexp_eval(rhs, &x));
                    fixpoint = current == x || current == State::bottom();
                    x = current;
                }
                x
            }
            BooleanExp::Or { lhs, rhs } => {
                let mut fixpoint = false;
                let mut x = state.clone();
                while !fixpoint {
                    let current = Self::bexp_eval(lhs, &x).union(&Self::bexp_eval(rhs, &x));
                    fixpoint = current == x || current == State::bottom();
                    x = current;
                }
                x
            }
        }
    }

    fn statement_eval(&mut self, stmt: &Statement<'a>, state: &State<'a, D>) -> State<'a, D> {
        if *state == State::bottom() {
            return State::bottom();
        }
        match stmt {
            Statement::Skip => state.clone(),
            Statement::Assignment(Assignment { var, value }) => {
                let mut updated_state = state.clone();
                updated_state.update(&var, Self::aexp_eval(value, state));
                updated_state
            }
            Statement::Composition { lhs, rhs } => {
                let state = self.statement_eval(lhs, state);
                self.statement_eval(rhs, &state)
            }
            Statement::Conditional {
                guard,
                true_branch,
                false_branch,
            } => {
                let t = self.statement_eval(true_branch, &Self::bexp_eval(guard, state));
                let f =
                    self.statement_eval(false_branch, &Self::bexp_eval(&!*guard.clone(), state));

                t.union(&f)
            }
            Statement::While { line, guard, body } => {
                let mut fixpoint = false;
                let mut x = state.clone();
                let mut iter = vec![];

                while !fixpoint {
                    let body_eval = self.statement_eval(body, &Self::bexp_eval(guard, &x));
                    let current = x.widening(&state.union(&body_eval), &self.widening_thresholds);
                    fixpoint = x == current;
                    iter.push(x);
                    x = current;
                }
                println!("Seeking loop invariant");
                dbg_iterations(&iter);

                let mut narrowing_iter = vec![];
                let mut steps = 0;
                fixpoint = false;
                while !fixpoint && steps < self.narrowing_steps {
                    let body_eval = self.statement_eval(body, &Self::bexp_eval(guard, &x));
                    let current = x.narrowing(&state.union(&body_eval));
                    fixpoint = current == x;
                    narrowing_iter.push(x);
                    x = current;
                    steps += 1;
                }
                println!("Refine loop invariant with narrowing");
                dbg_iterations(&narrowing_iter);

                self.invariants.insert(line.clone(), x.clone());
                dbg!(&!*guard.clone(), &x, Self::bexp_eval(&!*guard.clone(), &x));
                Self::bexp_eval(&!*guard.clone(), &x)
            }
        }
    }
}

fn dbg_iterations<'a, D: AbstractDomain>(v: &Vec<State<'a, D>>) {
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
                .reduce(|acc, e| format!("{acc}\t{e}"))
                .unwrap();
            format!("{var} -> {}", values)
        })
        .reduce(|acc, e| format!("{acc}\n{e}"))
        .unwrap();

    println!("{vars}");
}
