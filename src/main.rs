use std::{
    collections::HashMap,
    env,
    fs::{self},
};

use abstract_domains::interval::Interval;
use grammar::StatementParser;
use interpreter::Interpreter;
use lalrpop_util::lalrpop_mod;
use lexer::Lexer;
use regex::Regex;

mod abstract_domains;
mod ast;
mod interpreter;
mod lexer;
mod propagation_algo;
mod state;
mod tokens;

lalrpop_mod!(grammar);

fn main() {
    let args: Vec<String> = env::args().collect();

    let file = args[1].as_str();

    let source_code = std::fs::read_to_string(file).unwrap();
    let lexer = Lexer::new(&source_code);
    let program = StatementParser::new().parse(&source_code, lexer).unwrap();

    let re = Regex::new(r"\bassume\b|\s+").unwrap();
    let assumptions = source_code
        .lines()
        .filter(|line| line.contains("assume") && !line.contains("#"))
        .map(|line| re.replace_all(line.trim(), "").to_string())
        .collect::<Vec<_>>()
        .join("; ");

    let given_vars = assumptions
        .split(';')
        .map(|assignment| {
            let mut parts = assignment.split(":=");
            (parts.next().unwrap(), parts.next().unwrap_or_default())
        })
        .collect::<HashMap<&str, &str>>();

    let mut interpreter: Interpreter<'_, Interval> = Interpreter::build(&program, given_vars);
    let mut invariants = interpreter.interpret();
    let (_, program_invariant) = invariants.pop_last().unwrap();

    let mut interpreted_code = source_code.clone();
    invariants.into_iter().rev().for_each(|(line, inv)| {
        interpreted_code.insert_str(line, format!("# LOOP INVARIANT{inv}\n").as_str())
    });

    interpreted_code =
        format!("# PROGRAM ANALYSIS:\n\n{interpreted_code}\n\n# {program_invariant}");

    fs::write(format!("analysis_{file}"), interpreted_code.as_str()).expect("Unable to write file");
}
