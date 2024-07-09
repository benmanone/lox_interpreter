use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum Literal {
    String(String),
    Number(f32),
    Bool(bool),
    Null,
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

#[derive(Debug, Clone)]
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
