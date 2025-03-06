use std::{
    env,
    fs::{self},
    path::Path,
};

use abstract_domains::interval::Interval;
use grammar::StatementParser;
use interpreter::Interpreter;
use lalrpop_util::lalrpop_mod;
use utils::{decorate_code_with_analysis, extract_vars_init};

mod abstract_domains;
mod interpreter;
mod parser;
mod propagation_algo;
mod state;
mod utils;

lalrpop_mod!(grammar, "/parser/grammar.rs");

fn main() {
    let args: Vec<String> = env::args().collect();

    let file = args[1].as_str();

    let source_code = std::fs::read_to_string(file).unwrap();
    let lexer = parser::lexer::Lexer::new(&source_code);
    let program = StatementParser::new().parse(&source_code, lexer).unwrap();

    println!("Program: {:#?}", &program);

    let given_vars = extract_vars_init(&source_code);
    let mut interpreter = Interpreter::<Interval>::build(&program, given_vars);
    let invariants = interpreter.interpret();

    let output_file = Path::new(file).with_extension("analysis");
    fs::write(
        output_file,
        decorate_code_with_analysis(source_code.clone(), invariants),
    )
    .expect("Unable to write file");
}
