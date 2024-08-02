use crate::environment::*;
use crate::parser::Assignment;
use crate::parser::Block;
use crate::parser::Expr;
use crate::parser::If;
use crate::parser::Logic;
use crate::parser::Stmt;
use crate::parser::Var;
use crate::parser::Variable;
use crate::parser::While;
use crate::token::{Literal, TokenType};
use crate::RuntimeError;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Interpreter {
    environment: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            // need to use RefCell to legally keep track of data in enclosing environments
            // now clones of `environment` still reference the same data
            environment: Rc::new(RefCell::new(Environment::new(None))),
        }
    }

    pub fn interpret(&mut self, stmts: Vec<Stmt>) -> Result<(), RuntimeError> {
        for stmt in stmts {
            self.execute(stmt)?;
        }
        Ok(())
    }

    fn execute(&mut self, stmt: Stmt) -> Result<Literal, RuntimeError> {
        match stmt {
            Stmt::ExprStmt(expr) => self.evaluate(expr),
            Stmt::PrintStmt(expr) => self.eval_print_stmt(expr),
            Stmt::IfStmt(ifstmt) => self.eval_if_stmt(*ifstmt),
            Stmt::WhileStmt(whilestmt) => self.eval_while_stmt(*whilestmt),
            Stmt::VarStmt(var) => self.eval_var_stmt(var),
            Stmt::BlockStmt(block) => self.eval_block(block),
            _ => Ok(Literal::Null),
        }
    }

    fn execute_block(
        &mut self,
        statements: Vec<Stmt>,
        env: Rc<RefCell<Environment>>,
    ) -> Result<Literal, RuntimeError> {
        let previous = self.environment.clone();
        self.environment = env;

        for stmt in statements {
            self.execute(stmt)?;
        }

        self.environment = previous;
        Ok(Literal::Null)
    }

    fn evaluate(&mut self, expression: Expr) -> Result<Literal, RuntimeError> {
        match expression {
            Expr::GroupingExpr(g) => self.evaluate(g.expression),
            Expr::BinaryExpr(b) => self.eval_binary(*b),
            Expr::UnaryExpr(u) => self.eval_unary(*u),
            Expr::VarExpr(v) => self.eval_var(*v),
            Expr::AssignExpr(a) => self.eval_assign(*a),
            Expr::LogicExpr(l) => self.eval_logic(*l),
            Expr::LitExpr(l) => Ok(l),
        }
    }

    fn eval_block(&mut self, block: Block) -> Result<Literal, RuntimeError> {
        self.execute_block(block.statements, self.environment.clone())
    }

    fn eval_assign(&mut self, assignment: Assignment) -> Result<Literal, RuntimeError> {
        let value = self.evaluate(assignment.value)?;
        self.environment
            .borrow_mut()
            .assign(assignment.name, value.clone())?;
        // allows nesting of assign expressions inside other expressions e.g. print a = 2;
        Ok(value)
    }

    fn eval_var(&self, var: Variable) -> Result<Literal, RuntimeError> {
        self.environment.borrow_mut().get(var.name)
    }

    fn eval_logic(&mut self, logic: Logic) -> Result<Literal, RuntimeError> {
        let left = self.evaluate(logic.left)?;

        // check to see if it is possible to short circuit (if left is already true in an or statement)
        if (logic.operator.ttype == TokenType::Or && left.is_truthy()) || !left.is_truthy() {
            Ok(left)
        } else {
            self.evaluate(logic.right)
        }
    }

    fn eval_if_stmt(&mut self, ifstmt: If) -> Result<Literal, RuntimeError> {
        if self.evaluate(ifstmt.condition)?.is_truthy() {
            self.execute(ifstmt.then_branch)
        } else if ifstmt.else_branch != Stmt::ExprStmt(Expr::LitExpr(Literal::Null)) {
            self.execute(ifstmt.else_branch)
        } else {
            Ok(Literal::Null)
        }
    }

    fn eval_while_stmt(&mut self, whilestmt: While) -> Result<Literal, RuntimeError> {
        let condition = whilestmt.condition;

        while self.evaluate(condition.clone())?.is_truthy() {
            self.execute(whilestmt.body.clone())?;
        }
        Ok(Literal::Null)
    }

    fn eval_var_stmt(&mut self, var: Var) -> Result<Literal, RuntimeError> {
        let mut value = Literal::Null;

        if var.initialiser != Expr::LitExpr(Literal::Null) {
            value = self.evaluate(var.initialiser)?;
        }

        self.environment.borrow_mut().define(var.name.lexeme, value);
        Ok(Literal::Null)
    }

    fn eval_print_stmt(&mut self, expr: Expr) -> Result<Literal, RuntimeError> {
        let value = self.evaluate(expr)?;
        println!("{}", value.as_string());
        Ok(value)
    }

    fn eval_binary(&mut self, b: crate::parser::Binary) -> Result<Literal, RuntimeError> {
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
                        Err(RuntimeError {
                            token: b.operator,
                            message: "Attempted division by zero".to_string(),
                        })
                    }
                }
                TokenType::Star => Ok(Literal::Number(left_num * right_num)),
                TokenType::Greater => Ok(Literal::Bool(left_num > right_num)),
                TokenType::GreaterEqual => Ok(Literal::Bool(left_num >= right_num)),
                TokenType::Less => Ok(Literal::Bool(left_num < right_num)),
                TokenType::LessEqual => Ok(Literal::Bool(left_num <= right_num)),
                TokenType::EqualEqual => Ok(Literal::Bool(self.is_equal(left, right))),
                TokenType::BangEqual => Ok(Literal::Bool(!self.is_equal(left, right))),
                _ => Err(RuntimeError {
                    token: b.operator,
                    message: "Invalid operator used with two numbers".to_string(),
                }),
            },
            (Literal::String(left_str), Literal::String(right_str)) => {
                match b.operator.ttype {
                    TokenType::Plus => {
                        Ok(Literal::String(left_str.to_owned() + right_str.as_str()))
                    }
                    TokenType::EqualEqual => Ok(Literal::Bool(self.is_equal(left, right))),
                    TokenType::BangEqual => Ok(Literal::Bool(!self.is_equal(left, right))),
                    _ => Err(RuntimeError {
                        token: b.operator,
                        message: "Invalid operator used with two strings".to_string(),
                    }),
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
                _ => Err(RuntimeError {
                    token: b.operator,
                    message: "Invalid operator used with a string and a number".to_string(),
                }),
            },
            (Literal::Number(left_num), Literal::String(right_str)) => match b.operator.ttype {
                TokenType::Plus => Ok(Literal::String(left_num.to_string() + right_str.as_str())),
                TokenType::EqualEqual => Ok(Literal::Bool(
                    self.is_equal(Literal::String(left_num.to_string()), right),
                )),
                TokenType::BangEqual => Ok(Literal::Bool(
                    !self.is_equal(Literal::String(left_num.to_string()), right),
                )),
                _ => Err(RuntimeError {
                    token: b.operator,
                    message: "Invalid operator used with a number and a string".to_string(),
                }),
            },
            _ => match b.operator.ttype {
                TokenType::EqualEqual => Ok(Literal::Bool(self.is_equal(left, right))),
                TokenType::BangEqual => Ok(Literal::Bool(!self.is_equal(left, right))),
                _ => Err(RuntimeError {
                    token: b.operator,
                    message: "Operands must be two numbers or two strings.".to_string(),
                }),
            },
        }
    }

    fn eval_unary(&mut self, u: crate::parser::Unary) -> Result<Literal, RuntimeError> {
        let right = self.evaluate(u.right)?;

        if u.operator.ttype == TokenType::Minus {
            if let Literal::Number(n) = right {
                return Ok(Literal::Number(-n));
            } else {
                return Err(RuntimeError {
                    token: u.operator,
                    message: "Operand must be number".to_string(),
                });
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
