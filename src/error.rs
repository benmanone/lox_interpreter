use std::fmt::{Debug, Display};

use crate::token::{Token, TokenType};

#[derive(Debug)]
pub struct LoxError {
    pub line: u32,
    pub location: u32,
    pub message: String,
}

impl Display for LoxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[line {}:{}] Error: {}",
            self.line, self.location, self.message
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
            write!(f, "{} at end  {}", self.token.line, self.message)
        } else {
            write!(f, "{}:  {}", self.token.line, self.message)
        }
    }
}

impl std::error::Error for ParseError {}
