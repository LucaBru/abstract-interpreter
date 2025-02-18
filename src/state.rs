use std::collections::{HashMap, HashSet};

use crate::{abstract_domains::abstract_domain::AbstractDomain, ast::Statement};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct State<'a, D: AbstractDomain> {
    vars: HashMap<&'a str, D>,
}

impl<'a, 'b, D: AbstractDomain> State<'a, D> {
    pub fn initialize(program: &'a Statement, vars_initial_values: HashMap<&'b str, D>) -> Self {
        let vars = program.extract_vars();
        let vars = vars
            .clone()
            .into_iter()
            .map(|v| (v, vars_initial_values.get(v).unwrap_or(&D::top()).clone()))
            .collect();
        State { vars }
    }

    pub fn update(&mut self, var: &'a str, value: D) {
        if self.vars.contains_key(var) {
            self.vars.insert(var, value);
        }
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

    pub fn bottom() -> Self {
        State {
            vars: HashMap::new(),
        }
    }

    pub fn widening(&self, rhs: &Self, widening_thresholds: &HashSet<i64>) -> Self {
        if self.vars.is_empty() {
            return rhs.clone();
        } else if rhs.vars.is_empty() {
            return self.clone();
        }
        assert!(self.vars.keys().all(|var| rhs.vars.contains_key(var)));
        let vars = self
            .vars
            .iter()
            .map(|(var, value)| (*var, value.widening(rhs.lookup(var), widening_thresholds)))
            .collect();
        State { vars }
    }

    pub fn narrowing(&self, rhs: &Self) -> Self {
        if self.vars.is_empty() {
            return rhs.clone();
        } else if rhs.vars.is_empty() {
            return self.clone();
        }
        assert!(self.vars.keys().all(|var| rhs.vars.contains_key(var)));
        let vars = self
            .vars
            .iter()
            .map(|(var, value)| (*var, value.narrowing(rhs.lookup(var))))
            .collect();
        State { vars }
    }

    pub fn vars(&self) -> HashSet<&'a str> {
        self.vars.iter().map(|(key, _)| *key).collect()
    }
}
