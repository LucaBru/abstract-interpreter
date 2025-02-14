use std::collections::HashMap;

use abstract_domains::interval::Interval;
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
    let source_code = std::fs::read_to_string("myscript.toy").unwrap();
    let lexer = Lexer::new(&source_code);
    let program = StatementParser::new().parse(&source_code, lexer).unwrap();

    println!("{:?}", program);

    let initial_state: State<'_, Interval> = State::initialize(&program, HashMap::new());
    let invariants = interpreter(&program, &initial_state);

    println!("{:#?}", invariants)
}
