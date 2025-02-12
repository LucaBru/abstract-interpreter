use std::collections::HashMap;

use crate::abstract_domains::abstract_domain::AbstractDomain;

#[derive(Clone)]
pub struct State<'a, D: AbstractDomain> {
    vars: HashMap<&'a str, D>,
}

impl<'a, 'b, D: AbstractDomain> State<'a, D> {
    pub fn update(&mut self, var: &'b str, value: D) {
        let x = self.vars.get_mut(var).unwrap();
        *x = value;
    }

    pub fn union(&self, other: &Self) -> Self {
        //TODO: implement union variable wise
        panic!()
    }
}
