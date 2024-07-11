use crate::environment::Environment;
use crate::parser::Assignment;
use crate::parser::Block;
use crate::parser::Expr;
use crate::parser::Stmt;
use crate::parser::Var;
use crate::parser::Variable;
use crate::token::{Literal, TokenType};
use crate::RuntimeError;

pub struct Interpreter {
    environment: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment: Environment::new(None),
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
            Stmt::VarStmt(var) => self.eval_var_stmt(var),
            Stmt::BlockStmt(block) => self.eval_block(block),
        }
    }

    fn execute_block(
        &mut self,
        statements: Vec<Stmt>,
        env: Environment,
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
            Expr::LitExpr(l) => Ok(l),
        }
    }

    fn eval_block(&mut self, block: Block) -> Result<Literal, RuntimeError> {
        self.execute_block(
            block.statements,
            Environment::new(Some(self.environment.clone())),
        )
    }

    fn eval_assign(&mut self, assignment: Assignment) -> Result<Literal, RuntimeError> {
        let value = self.evaluate(assignment.value)?;
        self.environment.assign(assignment.name, value.clone())?;
        // allows nesting of assign expressions inside other expressions e.g. print a = 2;
        Ok(value)
    }

    fn eval_var(&self, var: Variable) -> Result<Literal, RuntimeError> {
        self.environment.get(var.name)
    }

    fn eval_var_stmt(&mut self, var: Var) -> Result<Literal, RuntimeError> {
        let mut value = Literal::Null;

        if var.initialiser != Expr::LitExpr(Literal::Null) {
            value = self.evaluate(var.initialiser)?;
        }

        self.environment.define(var.name.lexeme, value);
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
                return Ok(Literal::Bool(!self.is_truthy(right)));
            } else {
                // unreachable as is_truthy() matches all types
                return Ok(Literal::Null);
            }
        }

        // unreachable
        Ok(Literal::Null)
    }

    // false and nil are "falsey", everything else is "truthy"
    fn is_truthy(&self, right: Literal) -> bool {
        match right {
            Literal::Bool(b) => b,
            Literal::Null => false,
            _ => true,
        }
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
