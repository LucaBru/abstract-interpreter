use std::{
    cell::{LazyCell, OnceCell},
    collections::{HashMap, HashSet},
    sync::{LazyLock, OnceLock},
};

use crate::{
    abstract_domains::abstract_domain::AbstractDomain,
    ast::{Assignment, Statement},
};

#[derive(Clone, Debug)]
pub struct State<'a, D: AbstractDomain> {
    vars: HashMap<&'a str, D>,
}

impl<'a, 'b, D: AbstractDomain> State<'a, D> {
    fn extract_vars(program: &'a Statement) -> HashSet<&'a str> {
        match program {
            Statement::Skip => HashSet::new(),
            Statement::Assignment(Assignment { var, value: _ }) => HashSet::from([&var[..]]),
            Statement::Composition { lhs, rhs }
            | Statement::Conditional {
                guard: _,
                true_branch: lhs,
                false_branch: rhs,
            } => {
                let mut vars = Self::extract_vars(&lhs);
                vars.extend(Self::extract_vars(&rhs));
                vars
            }
            Statement::While { guard: _, body } => Self::extract_vars(&body),
        }
    }

    pub fn initialize(program: &'a Statement, vars_initial_values: HashMap<&'b str, D>) -> Self {
        let vars = Self::extract_vars(program);
        let vars = vars
            .clone()
            .into_iter()
            .map(|v| (v, vars_initial_values.get(v).unwrap_or(&D::top()).clone()))
            .collect();
        State { vars }
    }

    pub fn update(&mut self, var: &'a str, value: D) {
        self.vars.insert(var, value);
    }

    pub fn union(&self, other: &Self) -> Self {
        if self.vars.is_empty() {
            return other.clone();
        } else if other.vars.is_empty() {
            return self.clone();
        }

        let mut r = self.clone();
        other.vars.iter().for_each(|(var, value)| {
            let old_value = r.vars.insert(var, value.clone());
            if old_value.is_some() {
                r.vars
                    .insert(var, old_value.unwrap().union_abstraction(value));
            }
        });
        r
    }

    pub fn lookup(&self, var: &'b str) -> &D {
        self.vars.get(var).unwrap()
    }
}
