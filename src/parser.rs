use crate::token::Literal;
use crate::token::TokenType::*;
use crate::token::*;

use crate::error::ParseError;

#[derive(Debug, PartialEq, Clone)]
pub enum Stmt {
    ExprStmt(Expr),
    FuncDeclStmt(FuncDecl),
    PrintStmt(Expr),
    ForStmt(Box<For>),
    IfStmt(Box<If>),
    WhileStmt(Box<While>),
    VarDeclStmt(VarDecl),
    ReturnStmt(Return),
    BlockStmt(Block),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    AssignExpr(Box<Assignment>),
    BinaryExpr(Box<Binary>),
    CallExpr(Box<Call>),
    GroupingExpr(Box<Grouping>),
    UnaryExpr(Box<Unary>),
    VarExpr(Box<Variable>),
    LogicExpr(Box<Logic>),
    LitExpr(Literal),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Block {
    pub statements: Vec<Stmt>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Assignment {
    pub name: Token,
    pub value: Expr,
}

#[derive(Debug, PartialEq, Clone)]
// paren is used to find the location of errors related to function calls
pub struct Call {
    pub callee: Expr,
    pub paren: Token,
    pub arguments: Option<Vec<Expr>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Logic {
    pub left: Expr,
    pub operator: Token,
    pub right: Expr,
}

#[derive(Debug, PartialEq, Clone)]
pub struct FuncDecl {
    pub name: Token,
    pub params: Vec<Token>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct If {
    pub condition: Expr,
    pub then_branch: Stmt,
    pub else_branch: Stmt,
}

#[derive(Debug, PartialEq, Clone)]
pub struct For {
    pub initialiser: VarDecl,
    pub condition: Stmt,
    pub increment: Option<Expr>,
    pub body: Stmt,
}

#[derive(Debug, PartialEq, Clone)]
pub struct While {
    pub condition: Expr,
    pub body: Stmt,
}

#[derive(Debug, PartialEq, Clone)]
pub struct VarDecl {
    pub name: Token,
    pub initialiser: Expr,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Return {
    pub keyword: Token,
    pub value: Expr,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Variable {
    pub name: Token,
}

#[derive(Debug, PartialEq, Clone)]
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

#[derive(Debug, PartialEq, Clone)]
pub struct Grouping {
    pub expression: Expr,
}

impl Grouping {
    pub fn new(expression: Expr) -> Self {
        Self { expression }
    }
}

#[derive(Debug, PartialEq, Clone)]
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

    pub fn parse(&mut self) -> Result<Vec<Stmt>, ParseError> {
        // if let Ok(expr) = self.expression() {
        //     expr
        // } else {
        //     Expr::LitExpr(Literal::Null)
        // }
        // self.expression()
        let mut statements: Vec<Stmt> = vec![];

        while !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        Ok(statements)
    }

    fn statement(&mut self) -> Result<Stmt, ParseError> {
        if self.matches(&[Print]) {
            self.print_statement()
        } else if self.matches(&[Return]) {
            self.return_statement()
        } else if self.matches(&[While]) {
            self.while_statement()
        } else if self.matches(&[LeftBrace]) {
            Ok(Stmt::BlockStmt(self.block()?))
        } else if self.matches(&[If]) {
            self.if_statement()
        } else if self.matches(&[For]) {
            self.for_statement()
        } else {
            self.expression_statement()
        }
    }

    // block → "{" declaration* "}" ;
    fn block(&mut self) -> Result<Block, ParseError> {
        let mut statements: Vec<Stmt> = vec![];

        while !self.check(RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        self.consume(RightBrace, "Expect } after block.".to_string())?;
        Ok(Block { statements })
    }

    // whileStmt → "while" "(" expression ")" statement ;
    fn while_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(LeftParen, "Expect ( after 'while'.".to_string())?;
        let condition = self.expression()?;
        self.consume(RightParen, "Expect ) after 'while'.".to_string())?;
        let body = self.statement()?;

        Ok(Stmt::WhileStmt(Box::new(While { condition, body })))
    }

    // ifStmt → "if" "(" expression ")" statement ( "else" statement )? ;
    fn if_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(LeftParen, "Expect ( after if".to_string())?;
        let condition = self.expression()?;
        self.consume(RightParen, "Expect ) after condition".to_string())?;

        let then_branch = self.statement()?;
        // innermost call find the else
        // so else is bound to nearest preceding if
        let else_branch = if self.matches(&[Else]) {
            self.statement()?
        } else {
            Stmt::ExprStmt(Expr::LitExpr(Literal::Null))
        };

        Ok(Stmt::IfStmt(Box::new(If {
            condition,
            then_branch,
            else_branch,
        })))
    }

    // convert a for statement into the equivalent while statement, adding the declaration and increment on either side
    fn for_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(LeftParen, "Expect '(' after for statement".to_string())?;

        let initialiser = if self.matches(&[Semicolon]) {
            None
        } else if self.matches(&[Var]) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        let condition = if !self.check(Semicolon) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(Semicolon, "Expect ; after loop condition".to_string())?;

        let increment = if !self.check(RightParen) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(RightParen, "Expect ) after for clauses".to_string())?;

        let mut body = self.statement()?;

        // adds the increment, e.g. i++, to the end of the body so it gets evaluated
        if let Some(inc) = increment {
            body = Stmt::BlockStmt(Block {
                statements: vec![body, Stmt::ExprStmt(inc)],
            });
        }

        // wraps the body in a while statement
        body = match condition {
            None => Stmt::WhileStmt(Box::new(While {
                condition: Expr::LitExpr(Literal::Bool(true)),
                body,
            })),
            Some(cond) => Stmt::WhileStmt(Box::new(While {
                condition: cond,
                body,
            })),
        };

        // adds the declaration, e.g. var i = 1 to before the while loop
        if let Some(init) = initialiser {
            body = Stmt::BlockStmt(Block {
                statements: vec![init, body],
            });
        };

        Ok(body)
    }

    fn print_statement(&mut self) -> Result<Stmt, ParseError> {
        let value = self.expression()?;
        self.consume(Semicolon, "Expect ';' after value".to_string())?;
        Ok(Stmt::PrintStmt(value))
    }

    fn return_statement(&mut self) -> Result<Stmt, ParseError> {
        let keyword = self.previous().clone();
        let mut value = Expr::LitExpr(Literal::Null);

        // Check if an expression is present
        // Semicolons can't begin expressions
        if !self.check(Semicolon) {
            value = self.expression()?;
        }

        self.consume(Semicolon, "Expect ';' after return value".to_string())?;
        Ok(Stmt::ReturnStmt(Return { keyword, value }))
    }

    // varDecl → "var" IDENTIFIER ( "=" expression )? ";" ;
    fn var_declaration(&mut self) -> Result<Stmt, ParseError> {
        let name = self
            .consume(Identifier, "Expect variable name".to_string())?
            .clone();
        let mut initialiser = Expr::LitExpr(Literal::Null);

        if self.matches(&[Equal]) {
            initialiser = self.expression()?;
        }

        self.consume(
            Semicolon,
            "Expect ; after variable declaration.".to_string(),
        )?;

        Ok(Stmt::VarDeclStmt(VarDecl { name, initialiser }))
    }

    fn function(&mut self, kind: std::string::String) -> Result<Stmt, ParseError> {
        let name = self
            .consume(Identifier, format!("Expect {kind} name"))?
            .clone();
        self.consume(LeftParen, format!("Expect '(' after {kind} name"))?;

        let mut parameters: Vec<Token> = vec![];

        if !self.check(RightParen) {
            loop {
                parameters.push(
                    self.consume(Identifier, "Expect identifier name".to_string())?
                        .clone(),
                );
                if !self.matches(&[Comma]) {
                    break;
                } else if parameters.len() >= 255 {
                    return Err(self.error(
                        self.peek().clone(),
                        "Can't have more than 255 parameters".to_string(),
                    ));
                };
            }
        };

        self.consume(RightParen, "Expect ')' after parameters".to_string())?;

        self.consume(LeftBrace, format!("Expect '{{' before {kind} body"))?;
        let body = self.block()?.statements;

        Ok(Stmt::FuncDeclStmt(FuncDecl {
            name: name.clone(),
            params: parameters,
            body,
        }))
    }

    fn expression_statement(&mut self) -> Result<Stmt, ParseError> {
        let value = self.expression()?;
        self.consume(Semicolon, "Expect ';' after value".to_string())?;
        Ok(Stmt::ExprStmt(value))
    }

    // expression → equality ;
    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.assignment()
    }

    // assignment → IDENTIFIER "=" assignment | logic_or ;
    fn assignment(&mut self) -> Result<Expr, ParseError> {
        // LHS is any expression of higher precedence
        // as all LHSs of assignments are also valid expressions
        let expr = self.or()?;

        if self.matches(&[Equal]) {
            // recursively call the function as assignment is right-associative
            let value = self.assignment()?;

            // only return an assignment if assigning to variable
            if let Expr::VarExpr(var) = expr {
                let name = var.name;
                return Ok(Expr::AssignExpr(Box::new(Assignment { name, value })));
            }
            Err(ParseError {
                token: self.previous().clone(),
                message: "Invalid assignment target.".to_string(),
            })
        } else {
            Ok(expr)
        }
    }

    // logic_or → logic_and ( "or" logic_and )* ;
    fn or(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.and()?;

        while self.matches(&[Or]) {
            let operator = self.previous().clone();
            let right = self.and()?;
            expr = Expr::LogicExpr(Box::new(Logic {
                left: expr,
                operator,
                right,
            }));
        }

        Ok(expr)
    }

    // logic_and → equality ( "and" equality )* ;
    fn and(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.equality()?;

        while self.matches(&[And]) {
            let operator = self.previous().clone();
            let right = self.equality()?;
            expr = Expr::LogicExpr(Box::new(Logic {
                left: expr,
                operator,
                right,
            }));
        }

        Ok(expr)
    }

    // declaration → varDecl | statement ;
    fn declaration(&mut self) -> Result<Stmt, ParseError> {
        if self.matches(&[Fun]) {
            self.function("function".to_string())
        } else if self.matches(&[Var]) {
            self.var_declaration()
        } else {
            let stmt_result = self.statement();

            if stmt_result.is_ok() {
                stmt_result
            } else {
                self.synchronise();
                stmt_result
            }
        }
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

    // unary → ( "!" | "-" ) unary | call ;
    // if ! or -, must be unary, recursively call unary to parse operand
    // matches a primary expression followed by any number of function calls
    fn unary(&mut self) -> Result<Expr, ParseError> {
        if self.matches(&[Bang, Minus]) {
            let operator = self.previous().clone();
            let right = self.unary()?;

            return Ok(Expr::UnaryExpr(Box::new(Unary::new(operator, right))));
        }

        self.call()
    }

    // call → primary ( "(" arguments? ")" )* ;
    fn call(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.primary()?;

        // parse call expression with previous expression as callee
        while self.matches(&[LeftParen]) {
            expr = self.arguments(expr)?;
        }

        Ok(expr)
    }

    // arguments → expression ( "," expression )* ;
    // also handles zero-arguments case
    fn arguments(&mut self, callee: Expr) -> Result<Expr, ParseError> {
        let mut args: Vec<Expr> = vec![];

        let try_arguments = if !self.check(RightParen) {
            loop {
                if args.len() >= 255 {
                    break Err(self.error(
                        self.peek().clone(),
                        "Can't have more than 255 arguments".to_string(),
                    ));
                }
                args.push(self.expression()?);
                if !self.matches(&[Comma]) {
                    break Ok(Some(args));
                }
            }
        } else {
            Ok(None)
        };

        let paren = self.consume(RightParen, "Expect ')' after arguments".to_string())?;

        if let Ok(arguments) = try_arguments {
            Ok(Expr::CallExpr(Box::new(Call {
                callee,
                paren: paren.clone(),
                arguments,
            })))
        } else {
            Err(try_arguments.unwrap_err())
        }
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
        } else if self.matches(&[Identifier]) {
            return Ok(Expr::VarExpr(Box::new(Variable {
                name: self.previous().clone(),
            })));
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

    // checks if current token has any of the given types before advancing
    fn matches(&mut self, ttypes: &[TokenType]) -> bool {
        for tt in ttypes.iter() {
            if self.check(*tt) {
                self.advance();
                return true;
            }
        }

        false
    }

    // advance if of certain type, otherwise throw error
    fn consume(
        &mut self,
        ttype: TokenType,
        message: std::string::String,
    ) -> Result<&Token, ParseError> {
        if self.check(ttype) {
            return Ok(self.advance());
        } else {
            return Err(self.error(self.peek().clone(), message));
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
                TokenType::Class => break,
                TokenType::Fun => break,
                TokenType::Var => break,
                TokenType::For => break,
                TokenType::If => break,
                TokenType::While => break,
                TokenType::Print => break,
                TokenType::Return => break,
                _ => (),
            }

            self.advance();
        }
    }
}
