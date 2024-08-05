use crate::callable::*;
use crate::environment::*;
use crate::error::*;
use crate::parser::*;
use crate::token::Literal;
use crate::token::TokenType;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Interpreter {
    pub globals: Rc<RefCell<Environment>>,
    pub environment: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new() -> Self {
        let globals = Interpreter::insert_native_functions();

        Self {
            environment: globals.clone(),
            globals,
        }
    }

    fn insert_native_functions() -> Rc<RefCell<Environment>> {
        let globals = Rc::new(RefCell::new(Environment::new(None)));
        globals.borrow_mut().define(
            "clock".to_string(),
            Literal::NativeFunc(NativeFunction::Clock),
        );
        globals
    }

    pub fn interpret(&mut self, stmts: Vec<Stmt>) -> Result<(), RuntimeBreak> {
        for stmt in stmts {
            self.execute(stmt)?;
        }
        Ok(())
    }

    fn execute(&mut self, stmt: Stmt) -> Result<(), RuntimeBreak> {
        match stmt {
            Stmt::ExprStmt(expr) => match self.evaluate(expr) {
                Ok(_l) => Ok(()),
                Err(err) => Err(err),
            },
            Stmt::PrintStmt(expr) => self.eval_print_stmt(expr),
            Stmt::IfStmt(ifstmt) => self.eval_if_stmt(*ifstmt),
            Stmt::WhileStmt(whilestmt) => self.eval_while_stmt(*whilestmt),
            Stmt::VarDeclStmt(var) => self.eval_var_decl_stmt(var),
            Stmt::FuncDeclStmt(func) => self.eval_func_decl_stmt(func),
            Stmt::ReturnStmt(ret) => self.eval_return_stmt(ret),
            Stmt::BlockStmt(block) => self.eval_block(block),
            _ => Ok(()),
        }
    }

    pub fn execute_block(
        &mut self,
        statements: Vec<Stmt>,
        env: Rc<RefCell<Environment>>,
    ) -> Result<(), RuntimeBreak> {
        let previous = Rc::clone(&self.environment);
        self.environment = env;

        for stmt in statements {
            if let Err(e) = self.execute(stmt) {
                self.environment = previous;
                return Err(e);
            }
        }

        self.environment = previous;
        Ok(())
    }

    fn evaluate(&mut self, expression: Expr) -> Result<Literal, RuntimeBreak> {
        match expression {
            Expr::GroupingExpr(g) => self.evaluate(g.expression),
            Expr::BinaryExpr(b) => self.eval_binary(*b),
            Expr::UnaryExpr(u) => self.eval_unary(*u),
            Expr::VarExpr(v) => self.eval_var(*v),
            Expr::AssignExpr(a) => self.eval_assign(*a),
            Expr::LogicExpr(l) => self.eval_logic(*l),
            Expr::CallExpr(c) => self.eval_call(*c),
            Expr::LitExpr(l) => Ok(l),
        }
    }

    fn eval_block(&mut self, block: Block) -> Result<(), RuntimeBreak> {
        self.execute_block(
            block.statements,
            Rc::new(RefCell::new(Environment::new(Some(
                self.environment.clone(),
            )))),
        )
    }

    fn eval_assign(&mut self, assignment: Assignment) -> Result<Literal, RuntimeBreak> {
        let value = self.evaluate(assignment.value)?;
        self.environment
            .borrow_mut()
            .assign(assignment.name, value.clone())?;
        // allows nesting of assign expressions inside other expressions e.g. print a = 2;
        Ok(value)
    }

    fn eval_var(&self, var: Variable) -> Result<Literal, RuntimeBreak> {
        match self.environment.borrow_mut().get(var.name) {
            Ok(l) => Ok(l),
            Err(re) => Err(RuntimeBreak::RuntimeErrorBreak(re)),
        }
    }

    fn eval_logic(&mut self, logic: Logic) -> Result<Literal, RuntimeBreak> {
        let left = self.evaluate(logic.left)?;

        // check to see if it is possible to short circuit (if left is already true in an or statement)
        if (logic.operator.ttype == TokenType::Or && left.is_truthy()) || !left.is_truthy() {
            Ok(left)
        } else {
            self.evaluate(logic.right)
        }
    }

    fn eval_call(&mut self, call: Call) -> Result<Literal, RuntimeBreak> {
        let callee = self.evaluate(call.callee)?;

        let arguments: Result<Vec<Literal>, RuntimeBreak> = if let Some(args) = call.arguments {
            args.iter().map(|a| self.evaluate(a.clone())).collect()
        } else {
            Ok(vec![])
        };

        match callee {
            Literal::Func(f) => match arguments {
                Ok(args) => {
                    if f.arity() != args.len() as i32 {
                        Err(RuntimeBreak::RuntimeErrorBreak(RuntimeError {
                            token: call.paren,
                            message: format!(
                                "Expected {} arguments but got {}",
                                f.arity(),
                                args.len()
                            ),
                        }))
                    } else {
                        f.call(self, args)
                    }
                }
                Err(err) => Err(err),
            },
            Literal::NativeFunc(nf) => match arguments {
                Ok(args) => {
                    if nf.arity() != args.len() as i32 {
                        Err(RuntimeBreak::RuntimeErrorBreak(RuntimeError {
                            token: call.paren,
                            message: format!(
                                "Expected {} arguments but got {}",
                                nf.arity(),
                                args.len()
                            ),
                        }))
                    } else {
                        nf.call(self, args)
                    }
                }
                Err(err) => Err(err),
            },
            _ => Err(RuntimeBreak::RuntimeErrorBreak(RuntimeError {
                token: call.paren,
                message: "Can only call functions and classes".to_string(),
            })),
        }
    }

    fn eval_if_stmt(&mut self, ifstmt: If) -> Result<(), RuntimeBreak> {
        if self.evaluate(ifstmt.condition)?.is_truthy() {
            self.execute(ifstmt.then_branch)
        } else if ifstmt.else_branch != Stmt::ExprStmt(Expr::LitExpr(Literal::Null)) {
            self.execute(ifstmt.else_branch)
        } else {
            Ok(())
        }
    }

    fn eval_while_stmt(&mut self, whilestmt: While) -> Result<(), RuntimeBreak> {
        let condition = whilestmt.condition;

        while self.evaluate(condition.clone())?.is_truthy() {
            self.execute(whilestmt.body.clone())?;
        }
        Ok(())
    }

    fn eval_var_decl_stmt(&mut self, var: VarDecl) -> Result<(), RuntimeBreak> {
        let value = if var.initialiser != Expr::LitExpr(Literal::Null) {
            self.evaluate(var.initialiser)?
        } else {
            Literal::Null
        };

        self.environment.borrow_mut().define(var.name.lexeme, value);
        Ok(())
    }

    fn eval_func_decl_stmt(&mut self, func: FuncDecl) -> Result<(), RuntimeBreak> {
        self.environment
            .borrow_mut()
            .define(func.name.lexeme.clone(), Literal::Func(Function::new(func)));
        Ok(())
    }

    fn eval_return_stmt(&mut self, ret: Return) -> Result<(), RuntimeBreak> {
        let mut value = Literal::Null;
        if ret.value != Expr::LitExpr(Literal::Null) {
            value = self.evaluate(ret.value)?;
        }
        Err(RuntimeBreak::ReturnBreak(ReturnError { value }))
    }

    fn eval_print_stmt(&mut self, expr: Expr) -> Result<(), RuntimeBreak> {
        let value = self.evaluate(expr)?;
        println!("{}", value.as_string());
        Ok(())
    }

    fn eval_binary(&mut self, b: crate::parser::Binary) -> Result<Literal, RuntimeBreak> {
        let left = self.evaluate(b.left)?;
        let right = self.evaluate(b.right)?;

        // perform arithmetic, comparison / string concatenation
        match (&left, &right) {
            (Literal::Number(left_num), Literal::Number(right_num)) => match b.operator.ttype {
                TokenType::Minus => Ok(Literal::Number(left_num - right_num)),
                TokenType::Plus => Ok(Literal::Number(left_num + right_num)),
                TokenType::Slash => {
                    if right_num != &0.0 {
                        Ok(Literal::Number(left_num / right_num))
                    } else {
                        Err(RuntimeBreak::RuntimeErrorBreak(RuntimeError {
                            token: b.operator,
                            message: "Attempted division by zero".to_string(),
                        }))
                    }
                }
                TokenType::Star => Ok(Literal::Number(left_num * right_num)),
                TokenType::Greater => Ok(Literal::Bool(left_num > right_num)),
                TokenType::GreaterEqual => Ok(Literal::Bool(left_num >= right_num)),
                TokenType::Less => Ok(Literal::Bool(left_num < right_num)),
                TokenType::LessEqual => Ok(Literal::Bool(left_num <= right_num)),
                TokenType::EqualEqual => Ok(Literal::Bool(self.is_equal(left, right))),
                TokenType::BangEqual => Ok(Literal::Bool(!self.is_equal(left, right))),
                _ => Err(RuntimeBreak::RuntimeErrorBreak(RuntimeError {
                    token: b.operator,
                    message: "Invalid operator used with two numbers".to_string(),
                })),
            },
            (Literal::String(left_str), Literal::String(right_str)) => {
                match b.operator.ttype {
                    TokenType::Plus => {
                        Ok(Literal::String(left_str.to_owned() + right_str.as_str()))
                    }
                    TokenType::EqualEqual => Ok(Literal::Bool(self.is_equal(left, right))),
                    TokenType::BangEqual => Ok(Literal::Bool(!self.is_equal(left, right))),
                    _ => Err(RuntimeBreak::RuntimeErrorBreak(RuntimeError {
                        token: b.operator,
                        message: "Invalid operator used with two strings".to_string(),
                    })),
                }
                // implicit conversion of Numbers to Strings for concatenation or comparison
            }
            (Literal::String(left_str), Literal::Number(right_num)) => match b.operator.ttype {
                TokenType::Plus => Ok(Literal::String(
                    left_str.to_owned() + right_num.to_string().as_str(),
                )),
                TokenType::EqualEqual => Ok(Literal::Bool(
                    self.is_equal(left, Literal::String(right_num.to_string())),
                )),
                TokenType::BangEqual => Ok(Literal::Bool(
                    !self.is_equal(left, Literal::String(right_num.to_string())),
                )),
                _ => Err(RuntimeBreak::RuntimeErrorBreak(RuntimeError {
                    token: b.operator,
                    message: "Invalid operator used with a string and a number".to_string(),
                })),
            },
            (Literal::Number(left_num), Literal::String(right_str)) => match b.operator.ttype {
                TokenType::Plus => Ok(Literal::String(left_num.to_string() + right_str.as_str())),
                TokenType::EqualEqual => Ok(Literal::Bool(
                    self.is_equal(Literal::String(left_num.to_string()), right),
                )),
                TokenType::BangEqual => Ok(Literal::Bool(
                    !self.is_equal(Literal::String(left_num.to_string()), right),
                )),
                _ => Err(RuntimeBreak::RuntimeErrorBreak(RuntimeError {
                    token: b.operator,
                    message: "Invalid operator used with a number and a string".to_string(),
                })),
            },
            _ => match b.operator.ttype {
                TokenType::EqualEqual => Ok(Literal::Bool(self.is_equal(left, right))),
                TokenType::BangEqual => Ok(Literal::Bool(!self.is_equal(left, right))),
                _ => Err(RuntimeBreak::RuntimeErrorBreak(RuntimeError {
                    token: b.operator,
                    message: "Operands must be two numbers or two strings.".to_string(),
                })),
            },
        }
    }

    fn eval_unary(&mut self, u: crate::parser::Unary) -> Result<Literal, RuntimeBreak> {
        let right = self.evaluate(u.right)?;

        if u.operator.ttype == TokenType::Minus {
            if let Literal::Number(n) = right {
                return Ok(Literal::Number(-n));
            } else {
                return Err(RuntimeBreak::RuntimeErrorBreak(RuntimeError {
                    token: u.operator,
                    message: "Operand must be number".to_string(),
                }));
            }
        } else if u.operator.ttype == TokenType::Bang {
            if let Literal::Bool(_) = right {
                return Ok(Literal::Bool(!right.is_truthy()));
            } else {
                // unreachable as is_truthy() matches all types
                return Ok(Literal::Null);
            }
        }

        // unreachable
        Ok(Literal::Null)
    }

    fn is_equal(&self, left: Literal, right: Literal) -> bool {
        if let (Literal::Null, Literal::Null) = (&left, &right) {
            true
        } else if let Literal::Null = left {
            false
        } else {
            left == right
        }
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}
