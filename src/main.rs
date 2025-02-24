use std::{
    collections::{BTreeMap, HashMap},
    env,
    fs::{self},
    path::Path,
};

use abstract_domains::{abstract_domain::AbstractDomain, interval::Interval};
use grammar::StatementParser;
use interpreter::{Interpreter, ProgramInvariants};
use lalrpop_util::lalrpop_mod;
use regex::Regex;

mod abstract_domains;
mod interpreter;
mod parser;
mod propagation_algo;
mod state;

lalrpop_mod!(grammar, "/parser/grammar.rs");

fn main() {
    let args: Vec<String> = env::args().collect();

    let file = args[1].as_str();

    let source_code = std::fs::read_to_string(file).unwrap();
    let lexer = parser::lexer::Lexer::new(&source_code);
    let program = StatementParser::new().parse(&source_code, lexer).unwrap();

    dbg!(&program);

    let re = Regex::new(r"\bassume\b|\s+").unwrap();
    let assumptions = source_code
        .clone()
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

    let mut interpreter = Interpreter::<Interval>::build(&program, given_vars);
    let invariants = interpreter.interpret();

    let output_file = Path::new(file).with_extension("analysis");
    fs::write(
        output_file,
        decorate_code_with_analysis(source_code.clone(), invariants),
    )
    .expect("Unable to write file");
}

fn decorate_code_with_analysis<'a, D: AbstractDomain>(
    source_code: String,
    mut invariants: ProgramInvariants<'a, D>,
) -> String {
    // Extract last invariant safely
    let program_inv = invariants
        .pop_last()
        .map(|(_, inv)| format!("\n# {}", inv))
        .unwrap_or_else(|| String::from("\n# No invariant found"));

    let mut code_analysis: Vec<_> = source_code.lines().collect();
    code_analysis.push(&program_inv);

    let invariants: BTreeMap<_, _> = invariants
        .into_iter()
        .map(|(pos, inv)| {
            let tabs = " ".repeat(pos.clm);
            (pos, format!("{tabs}# LOOP INVARIANT: {inv}"))
        })
        .collect();

    // Insert invariants in reverse order to preserve correct line positions
    for (pos, inv) in invariants.iter().rev() {
        println!("Writing inv {inv} at line {}", pos.line);
        code_analysis.insert(pos.line, inv);
    }
    code_analysis.join("\n")
}
