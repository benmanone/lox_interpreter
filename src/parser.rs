use crate::token::Literal;
use crate::token::TokenType::*;
use crate::token::*;

use crate::error::ParseError;

#[derive(Debug)]
pub enum Expr {
    BinaryExpr(Box<Binary>),
    GroupingExpr(Box<Grouping>),
    UnaryExpr(Box<Unary>),
    LitExpr(Literal),
}

#[derive(Debug)]
pub struct Binary {
    pub left: Expr,
    pub operator: Token,
    pub right: Expr,
}

impl Binary {
    pub fn new(left: Expr, operator: Token, right: Expr) -> Self {
        Self {
            left,
            operator,
            right,
        }
    }
}

#[derive(Debug)]
pub struct Grouping {
    pub expression: Expr,
}

impl Grouping {
    pub fn new(expression: Expr) -> Self {
        Self { expression }
    }
}

#[derive(Debug)]
pub struct Unary {
    pub operator: Token,
    pub right: Expr,
}

impl Unary {
    pub fn new(operator: Token, right: Expr) -> Self {
        Self { operator, right }
    }
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Expr, ParseError> {
        // if let Ok(expr) = self.expression() {
        //     expr
        // } else {
        //     Expr::LitExpr(Literal::Null)
        // }
        self.expression()
    }

    // expression → equality ;
    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.equality()
    }

    // equality → comparison ( ( "!=" | "==" ) comparison )* ;
    // keep looping through child comparison expressions until no more != / == tokens
    fn equality(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.comparison()?;

        while self.matches(&[BangEqual, EqualEqual]) {
            // get whether != or ==
            let operator = self.previous().clone();
            // parse right hand operand
            let right = self.comparison()?;
            // creates a nesting tree as `expr` is used as the left hand operand
            expr = Expr::BinaryExpr(Box::new(Binary::new(expr, operator, right)));
        }

        Ok(expr)
    }

    // comparison → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
    // keep looping through child term expressions until no more >, >=, <, <=
    // otherwise similar to equality
    fn comparison(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.term()?;

        while self.matches(&[Greater, GreaterEqual, Less, LessEqual]) {
            let operator = self.previous().clone();
            let right = self.term()?;
            expr = Expr::BinaryExpr(Box::new(Binary::new(expr, operator, right)));
        }

        Ok(expr)
    }

    // term → factor ( ( "-" | "+" ) factor )* ;
    // keep looping through child factor expressions until no more + / -
    // term comes first due to order of operations
    fn term(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.factor()?;

        while self.matches(&[Plus, Minus]) {
            let operator = self.previous().clone();
            let right = self.factor()?;
            expr = Expr::BinaryExpr(Box::new(Binary::new(expr, operator, right)));
        }

        Ok(expr)
    }

    // factor → unary ( ( "/" | "*" ) unary )* ;
    // keep looping through child unary expressions until no more *, /
    fn factor(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.unary()?;

        while self.matches(&[Star, Slash]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            expr = Expr::BinaryExpr(Box::new(Binary::new(expr, operator, right)));
        }

        Ok(expr)
    }

    // unary → ( "!" | "-" ) unary | primary ;
    // if ! or -, must be unary, recursively call unary to parse operand
    fn unary(&mut self) -> Result<Expr, ParseError> {
        if self.matches(&[Bang, Minus]) {
            let operator = self.previous().clone();
            let right = self.unary()?;

            return Ok(Expr::UnaryExpr(Box::new(Unary::new(operator, right))));
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        if self.matches(&[False]) {
            Ok(Expr::LitExpr(Literal::Bool(false)))
        } else if self.matches(&[True]) {
            return Ok(Expr::LitExpr(Literal::Bool(true)));
        } else if self.matches(&[Nil]) {
            return Ok(Expr::LitExpr(Literal::Null));
        } else if self.matches(&[Number, String]) {
            return Ok(Expr::LitExpr(self.previous().clone().literal));
        }
        // must find a right paren or throw error
        else if self.matches(&[LeftParen]) {
            let expr = self.expression()?;
            self.consume(RightParen, "Expect ) after expression".to_string())?;

            return Ok(Expr::GroupingExpr(Box::new(Grouping::new(expr))));
        } else {
            return Err(ParseError {
                token: self.peek().clone(),
                message: "Expect expression.".to_string(),
            });
        }
    }

    // checks if current token has any of the given types
    fn matches(&mut self, ttypes: &[TokenType]) -> bool {
        for tt in ttypes.iter() {
            if self.check(*tt) {
                self.advance();
                return true;
            }
        }

        false
    }

    fn consume(
        &mut self,
        ttype: TokenType,
        message: std::string::String,
    ) -> Result<&Token, ParseError> {
        if self.check(ttype) {
            return Ok(self.advance());
        } else {
            return Err(self.error(self.peek().clone().clone(), message));
        }
    }

    // returns true if token is of given type
    fn check(&self, ttype: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.peek().ttype == ttype
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().ttype == Eof
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.current).unwrap()
    }

    fn previous(&self) -> &Token {
        self.tokens.get(self.current - 1).unwrap()
    }

    fn error(&self, token: Token, message: std::string::String) -> ParseError {
        ParseError { token, message }
    }

    // discard tokens until at the beginning of the next statement
    fn synchronise(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().ttype == Semicolon {
                break;
            }

            match self.peek().ttype {
                Class => break,
                Fun => break,
                Var => break,
                For => break,
                If => break,
                While => break,
                Print => break,
                Return => break,
                _ => (),
            }

            self.advance();
        }
    }
}
