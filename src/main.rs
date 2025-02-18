use std::collections::{HashMap, HashSet};

use abstract_domains::{
    int::Int,
    interval::{Interval, M, N},
};
use grammar::StatementParser;
use interpreter::interpreter;
use lalrpop_util::lalrpop_mod;
use lexer::Lexer;
use state::State;

mod abstract_domains;
mod ast;
mod interpreter;
mod lexer;
mod propagation_algo;
mod state;
mod tokens;

lalrpop_mod!(grammar);

fn main() {
    /*         {
        let mut m_lock = M.write().unwrap();
        *m_lock = Int::PosInf;

        let mut n_lock = N.write().unwrap();
        *n_lock = Int::NegInf;
    } */
    let source_code = std::fs::read_to_string("myscript.toy").unwrap();
    let lexer = Lexer::new(&source_code);
    let program = StatementParser::new().parse(&source_code, lexer).unwrap();

    println!("{:?}", program);

    let initial_state: State<'_, Interval> = State::initialize(&program, HashMap::new());
    let syntax_constant = program.extract_constant();
    dbg!(&syntax_constant);
    let invariants = interpreter(&program, &initial_state, &HashSet::new());

    println!("{:#?}", invariants)
}
