use crate::parser::Expr;
use crate::token::{Literal, TokenType};
use crate::RuntimeError;

pub struct Interpreter {}

impl Interpreter {
    pub fn new() -> Self {
        Self {}
    }

    pub fn interpret(&self, expression: Expr) -> Result<(), RuntimeError> {
        let value = self.evaluate(expression)?;
        println!("{}", value.as_string());
        Ok(())
    }

    fn evaluate(&self, expression: Expr) -> Result<Literal, RuntimeError> {
        match expression {
            Expr::GroupingExpr(g) => self.evaluate(g.expression),
            Expr::BinaryExpr(b) => self.eval_binary(*b),
            Expr::UnaryExpr(u) => self.eval_unary(*u),
            Expr::LitExpr(l) => Ok(l),
        }
    }

    fn eval_binary(&self, b: crate::parser::Binary) -> Result<Literal, RuntimeError> {
        let left = self.evaluate(b.left)?;
        let right = self.evaluate(b.right)?;

        // perform arithmetic, comparison / string concatenation
        match (&left, &right) {
            (Literal::Number(left_num), Literal::Number(right_num)) => match b.operator.ttype {
                TokenType::Minus => return Ok(Literal::Number(left_num - right_num)),
                TokenType::Plus => return Ok(Literal::Number(left_num + right_num)),
                TokenType::Slash => {
                    if right_num != &0.0 {
                        return Ok(Literal::Number(left_num / right_num));
                    } else {
                        return Err(RuntimeError {
                            token: b.operator,
                            message: "Attempted division by zero".to_string(),
                        });
                    }
                }
                TokenType::Star => return Ok(Literal::Number(left_num * right_num)),
                TokenType::Greater => return Ok(Literal::Bool(left_num > right_num)),
                TokenType::GreaterEqual => return Ok(Literal::Bool(left_num >= right_num)),
                TokenType::Less => return Ok(Literal::Bool(left_num < right_num)),
                TokenType::LessEqual => return Ok(Literal::Bool(left_num <= right_num)),
                TokenType::BangEqual => return Ok(Literal::Bool(!self.is_equal(left, right))),
                TokenType::EqualEqual => return Ok(Literal::Bool(self.is_equal(left, right))),
                _ => {
                    return Err(RuntimeError {
                        token: b.operator,
                        message: "Invalid operator used with two numbers".to_string(),
                    })
                }
            },
            (Literal::String(left_str), Literal::String(right_str)) => {
                match b.operator.ttype {
                    TokenType::Plus => {
                        return Ok(Literal::String(left_str.to_owned() + right_str.as_str()))
                    }
                    TokenType::EqualEqual => return Ok(Literal::Bool(self.is_equal(left, right))),
                    TokenType::BangEqual => return Ok(Literal::Bool(!self.is_equal(left, right))),
                    _ => {
                        return Err(RuntimeError {
                            token: b.operator,
                            message: "Invalid operator used with two strings".to_string(),
                        })
                    }
                }
                // implicit conversion of Numbers to Strings for concatenation or comparison
            }
            (Literal::String(left_str), Literal::Number(right_num)) => match b.operator.ttype {
                TokenType::Plus => {
                    return Ok(Literal::String(
                        left_str.to_owned() + right_num.to_string().as_str(),
                    ))
                }
                TokenType::EqualEqual => {
                    return Ok(Literal::Bool(
                        self.is_equal(left, Literal::String(right_num.to_string())),
                    ))
                }
                TokenType::BangEqual => {
                    return Ok(Literal::Bool(
                        !self.is_equal(left, Literal::String(right_num.to_string())),
                    ))
                }
                _ => {
                    return Err(RuntimeError {
                        token: b.operator,
                        message: "Invalid operator used with a string and a number".to_string(),
                    })
                }
            },
            (Literal::Number(left_num), Literal::String(right_str)) => match b.operator.ttype {
                TokenType::Plus => {
                    return Ok(Literal::String(left_num.to_string() + right_str.as_str()))
                }
                TokenType::EqualEqual => {
                    return Ok(Literal::Bool(
                        self.is_equal(Literal::String(left_num.to_string()), right),
                    ))
                }
                TokenType::BangEqual => {
                    return Ok(Literal::Bool(
                        !self.is_equal(Literal::String(left_num.to_string()), right),
                    ))
                }
                _ => {
                    return Err(RuntimeError {
                        token: b.operator,
                        message: "Invalid operator used with a number and a string".to_string(),
                    })
                }
            },
            _ => Err(RuntimeError {
                token: b.operator,
                message: "Operands must be two numbers or two strings.".to_string(),
            }),
        }
    }

    fn eval_unary(&self, u: crate::parser::Unary) -> Result<Literal, RuntimeError> {
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
            return true;
        } else if let Literal::Null = left {
            return false;
        } else {
            return left == right;
        }
    }
}
