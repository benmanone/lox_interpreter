use std::fmt::Display;

#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    String(String),
    Number(f32),
    Bool(bool),
    Null,
}

impl Literal {
    pub fn as_string(&self) -> String {
        match self {
            Literal::String(s) => s.to_owned(),
            Literal::Number(n) => n.to_string(),
            Literal::Bool(b) => b.to_string(),
            Literal::Null => "nil".to_string(),
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
