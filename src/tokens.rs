use logos::Logos;
use std::fmt; // to implement the Display trait later
use std::num::ParseIntError;
use std::str::ParseBoolError;

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

#[derive(Logos, Clone, Debug, PartialEq)]
#[logos(skip r"[ \t\n\f]+", skip r"#.*\n?", error = LexicalError)]
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
    #[token("while")]
    While,
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
    Error,
}

impl<'input> fmt::Display for Token<'input> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
