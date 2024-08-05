use crate::token::Literal;
use std::fmt::Display;

use crate::token::{Token, TokenType};

#[derive(Debug)]
pub struct LoxError {
    pub line: u32,
    pub message: String,
}

impl Display for LoxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[line {}] Error while scanning: {}",
            self.line, self.message
        )
    }
}

impl std::error::Error for LoxError {}

#[derive(Debug)]
pub struct ParseError {
    pub token: Token,
    pub message: String,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.token.ttype == TokenType::Eof {
            write!(
                f,
                "Syntax error: Line {} at end: {}",
                self.token.line, self.message
            )
        } else {
            write!(
                f,
                "Syntax error: Line {} at '{}': {}",
                self.token.line, self.token.lexeme, self.message
            )
        }
    }
}

impl std::error::Error for ParseError {}

#[derive(Debug)]
pub enum RuntimeBreak {
    RuntimeErrorBreak(RuntimeError),
    ReturnBreak(ReturnError),
}

impl Display for RuntimeBreak {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeBreak::RuntimeErrorBreak(re) => {
                write!(
                    f,
                    "Runtime error at {:?}: {} [line {}]",
                    re.token.ttype, re.message, re.token.line
                )
            }
            RuntimeBreak::ReturnBreak(re) => {
                write!(f, "Value returned: {:#?}", re.value)
            }
        }
    }
}

impl std::error::Error for RuntimeBreak {}

#[derive(Debug)]
pub struct RuntimeError {
    pub token: Token,
    pub message: String,
}

#[derive(Debug)]
pub struct ReturnError {
    pub value: Literal,
}
