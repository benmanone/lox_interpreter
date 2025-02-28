use std::fmt::Display;

use crate::callable::*;

#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    String(String),
    Number(f32),
    Bool(bool),
    Func(Function),
    NativeFunc(NativeFunction),
    Null,
}

impl Literal {
    pub fn as_string(&self) -> String {
        match self {
            Literal::String(s) => s.to_owned(),
            Literal::Number(n) => n.to_string(),
            Literal::Bool(b) => b.to_string(),
            Literal::Func(f) => f.to_string(),
            Literal::NativeFunc(n) => n.to_string(),
            Literal::Null => "nil".to_string(),
        }
    }

    // false and nil are "falsey", everything else is "truthy"
    pub fn is_truthy(&self) -> bool {
        match self {
            Literal::Bool(b) => *b,
            Literal::Null => false,
            _ => true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenType {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Identifier,
    String,
    Number,
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub ttype: TokenType,
    pub lexeme: String,
    pub literal: Literal,
    pub line: u32,
}

impl Token {
    pub fn new(ttype: TokenType, lexeme: String, literal: Literal, line: u32) -> Self {
        Token {
            ttype,
            lexeme,
            literal,
            line,
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Type: {:?}, Value: {}, Literal: {:?}",
            self.ttype, self.lexeme, self.literal
        )
    }
}
