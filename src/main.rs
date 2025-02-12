use grammar::StatementParser;
use lalrpop_util::lalrpop_mod;
use lexer::Lexer;

mod abstract_domains;
mod ast;
mod interpreter;
mod lexer;
mod state;
mod tokens;

lalrpop_mod!(grammar);

fn main() {
    let source_code = std::fs::read_to_string("myscript.toy").unwrap();
    let lexer = Lexer::new(&source_code);
    let parser = StatementParser::new();
    let ast = parser.parse(lexer).unwrap();

    println!("{:?}", ast);
}
