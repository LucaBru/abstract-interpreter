use std::collections::HashMap;

use abstract_domains::interval::Interval;
use grammar::StatementParser;
use interpreter::Interpreter;
use lalrpop_util::lalrpop_mod;
use lexer::Lexer;

mod abstract_domains;
mod ast;
mod interpreter;
mod lexer;
mod propagation_algo;
mod state;
mod tokens;

lalrpop_mod!(grammar);

/* fn get_line_number(text: &str, byte_offset: usize) -> Option<usize> {
    if byte_offset > text.len() {
        return None; // Return None if the byte offset is out of bounds.
    }

    // Split the text into lines and keep track of cumulative byte offsets.
    let mut cumulative_offset = 0;

    for (line_number, line) in text.lines().enumerate() {
        let line_length = line.len() + 1; // +1 for the newline character.

        if cumulative_offset + line_length > byte_offset {
            return Some(line_number + 1); // Line numbers are 1-based.
        }

        cumulative_offset += line_length;
    }

    None // This should not happen if the input is valid.
}
 */
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
    //println!("{}", get_line_number(&source_code, 18).unwrap());

    let initial_vars = HashMap::new();
    let mut interpreter: Interpreter<'_, Interval> = Interpreter::build(&program, initial_vars);
    let invariants = interpreter.interpret();

    println!("{:#?}", invariants)
}
