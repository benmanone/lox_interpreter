use crate::token::*;
use crate::HashMap;
use crate::LoxError;
use crate::Rc;

pub struct Scanner {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: u32,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        Scanner {
            source,
            tokens: vec![],
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(&mut self) -> Result<&Vec<Token>, LoxError> {
        while !self.is_at_end() {
            // beginning of next token
            self.start = self.current;
            self.scan_token()?;
        }

        self.tokens.push(Token::new(
            TokenType::Eof,
            String::new(),
            Literal::Null,
            self.line,
        ));

        Ok(&self.tokens)
    }

    pub fn scan_token(&mut self) -> Result<(), LoxError> {
        let c = self.advance();

        match c {
            '(' => {
                self.add_token(TokenType::LeftParen);
                Ok(())
            }
            ')' => {
                self.add_token(TokenType::RightParen);
                Ok(())
            }
            '{' => {
                self.add_token(TokenType::LeftBrace);
                Ok(())
            }
            '}' => {
                self.add_token(TokenType::RightBrace);
                Ok(())
            }
            ',' => {
                self.add_token(TokenType::Comma);
                Ok(())
            }
            '.' => {
                self.add_token(TokenType::Dot);
                Ok(())
            }
            '-' => {
                self.add_token(TokenType::Minus);
                Ok(())
            }
            '+' => {
                self.add_token(TokenType::Plus);
                Ok(())
            }
            ';' => {
                self.add_token(TokenType::Semicolon);
                Ok(())
            }
            '*' => {
                self.add_token(TokenType::Star);
                Ok(())
            }
            // if the next token is =, change the tokentype
            '!' => {
                let is_equals = self.matches('=');
                self.add_token(if is_equals {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                });
                Ok(())
            }
            '=' => {
                let is_equals = self.matches('=');
                self.add_token(if is_equals {
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                });
                Ok(())
            }
            '<' => {
                let is_equals = self.matches('=');
                self.add_token(if is_equals {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                });
                Ok(())
            }
            '>' => {
                let is_equals = self.matches('=');
                self.add_token(if is_equals {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                });
                Ok(())
            }
            // if a second / is found, consume characters until end of line is reached
            // this is necessary as / can signify either division or a comment (//)
            // comments are meaningless so the loop skips over them until the beginning of the next lexeme
            '/' => {
                if self.matches('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                // block comments /* */
                } else if self.matches('*') {
                    while self.peek() != '*' && self.peek_next() != '/' {
                        if self.is_at_end() {
                            return Err(LoxError {
                                line: self.line,
                                message: String::from("Unclosed block comment."),
                            });
                        } else {
                            self.advance();
                        }
                    }
                    // consume final two characters...
                    self.advance();
                    self.advance();
                } else {
                    self.add_token(TokenType::Slash);
                }
                Ok(())
            }
            // ignore whitespace
            ' ' => Ok(()),
            '\r' => Ok(()),
            '\t' => Ok(()),
            '\n' => {
                self.line += 1;
                Ok(())
            }
            '"' => self.string(),
            _ => {
                if c.is_ascii_digit() {
                    {
                        self.number()?;
                        Ok(())
                    }
                } else if c.is_alphabetic() {
                    {
                        self.identifier();
                        Ok(())
                    }
                } else {
                    Err(LoxError {
                        line: self.line,
                        message: String::from("Unexpected character."),
                    })
                }
            }
        }
    }

    pub fn identifier(&mut self) {
        let keywords: HashMap<String, TokenType> = HashMap::from([
            (String::from("and"), TokenType::And),
            (String::from("class"), TokenType::Class),
            (String::from("else"), TokenType::Else),
            (String::from("false"), TokenType::False),
            (String::from("for"), TokenType::For),
            (String::from("fun"), TokenType::Fun),
            (String::from("if"), TokenType::If),
            (String::from("nil"), TokenType::Nil),
            (String::from("or"), TokenType::Or),
            (String::from("print"), TokenType::Print),
            (String::from("return"), TokenType::Return),
            (String::from("super"), TokenType::Super),
            (String::from("this"), TokenType::This),
            (String::from("true"), TokenType::True),
            (String::from("var"), TokenType::Var),
            (String::from("while"), TokenType::While),
        ]);

        while self.peek().is_alphanumeric() {
            self.advance();
        }

        let text = self.source.to_string()[self.start..self.current].to_string();
        let ttype = keywords.get(&text).unwrap_or(&TokenType::Identifier);

        self.add_token(*ttype);
    }

    pub fn number(&mut self) -> Result<(), LoxError> {
        while self.peek().is_ascii_digit() {
            self.advance();
        } // check it is a valid floating point
        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            // consume .
            self.advance();
            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        let try_num = self.source.to_string()[self.start..self.current]
            .to_string()
            .parse();

        if let Ok(num) = try_num {
            self.add_token_literal(TokenType::Number, Literal::Number(num));
            Ok(())
        } else {
            Err(LoxError {
                line: self.line,
                message: "No number".to_string(),
            })
        }
    }

    pub fn string(&mut self) -> Result<(), LoxError> {
        // consume characters until the final "
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }
        if self.is_at_end() {
            return Err(LoxError {
                line: self.line,
                message: String::from("Unterminated string."),
            });
        }
        // encapsulate the closing "
        self.advance();

        // trim quotes from string value
        let value = String::from(&self.source)[self.start + 1..self.current - 1].to_string();
        self.add_token_literal(TokenType::String, Literal::String(value));
        Ok(())
    }

    // consumes character on condition
    pub fn matches(&mut self, expected: char) -> bool {
        if self.is_at_end()
            || self
                .source
                .chars()
                .nth(self.current)
                .is_some_and(|c| c != expected)
        {
            false
        } else {
            self.current += 1;
            true
        }
    }

    // advance() without consuming character
    pub fn peek(&mut self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source.chars().collect::<Rc<[char]>>()[self.current]
        }
    }

    // lookahead twice
    pub fn peek_next(&mut self) -> char {
        // if the next character is at least the final character
        if self.current + 1 >= self.source.len() {
            '\0'
        } else {
            self.source.chars().collect::<Rc<[char]>>()[self.current + 1]
        }
    }

    pub fn is_at_end(&self) -> bool {
        // check if current position is at the end of the source string
        self.current >= self.source.len()
    }

    pub fn advance(&mut self) -> char {
        self.current += 1;
        self.source
            .chars()
            .nth(self.current - 1)
            .expect("Failed to advance while scanning")
    }

    pub fn add_token(&mut self, ttype: TokenType) {
        self.add_token_literal(ttype, Literal::Null);
    }

    pub fn add_token_literal(&mut self, ttype: TokenType, literal: Literal) {
        let text = &self.source[self.start..self.current];
        self.tokens
            .push(Token::new(ttype, String::from(text), literal, self.line))
    }
}
