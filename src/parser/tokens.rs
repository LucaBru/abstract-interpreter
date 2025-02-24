use logos::{Lexer, Logos, Skip};
use std::fmt;
use std::num::ParseIntError;
use std::str::ParseBoolError;

use super::ast::Position;

#[derive(Default, Debug, Clone, PartialEq)]
pub enum LexicalError {
    InvalidInteger(ParseIntError),
    InvalidBoolean(ParseBoolError),
    #[default]
    InvalidToken,
}

impl From<ParseIntError> for LexicalError {
    fn from(err: ParseIntError) -> Self {
        LexicalError::InvalidInteger(err)
    }
}

impl From<ParseBoolError> for LexicalError {
    fn from(err: ParseBoolError) -> Self {
        LexicalError::InvalidBoolean(err)
    }
}

fn newline_callback<'a>(lex: &mut Lexer<'a, Token<'a>>) -> Skip {
    lex.extras.0 += 1;
    lex.extras.1 = lex.span().end;
    Skip
}

fn get_while_token_pos<'a>(lex: &mut Lexer<'a, Token<'a>>) -> Position {
    Position {
        line: lex.extras.0,
        clm: lex.span().start - lex.extras.1,
    }
}

#[derive(Logos, Clone, Debug, PartialEq)]
#[logos(skip r"[ \t\f]+", skip r"assume.*\n?", skip r"#.*\n?", error = LexicalError)]
#[logos(extras=(usize, usize))]
pub enum Token<'input> {
    #[regex("[_a-zA-Z][_0-9a-zA-Z]*", |lex| lex.slice())]
    Identifier(&'input str),
    #[regex("[0-9]*", |lex| lex.slice().parse())]
    Integer(i64),
    #[token("if")]
    If,
    #[token("then")]
    Then,
    #[token("else")]
    Else,
    #[token("while", get_while_token_pos)]
    While(Position),
    #[token("do")]
    Do,
    #[token("skip")]
    Skip,

    #[token("{")]
    LCurlyBracket,
    #[token("}")]
    RCurlyBracket,

    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token(":=")]
    Assign,
    #[token(";")]
    Semicolon,

    #[token("+")]
    OperatorAdd,
    #[token("-")]
    OperatorSub,
    #[token("*")]
    OperatorMul,
    #[token("/")]
    OperatorDiv,

    #[regex("true|false", |lex| lex.slice().parse())]
    Boolean(bool),
    #[token("=")]
    Equal,
    #[token("<")]
    StrictlyLess,
    #[token("!")]
    Not,
    #[token("&")]
    And,

    #[regex(r"\n", newline_callback)]
    Newline,

    Error,
}

impl<'input> fmt::Display for Token<'input> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
