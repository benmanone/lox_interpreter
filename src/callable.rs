use crate::environment::Environment;
use crate::error::*;
use crate::parser::FuncDecl;
use std::cell::RefCell;
use std::fmt::Display;
use std::rc::Rc;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use crate::interpreter::Interpreter;
use crate::token::Literal;

pub trait Callable {
    fn arity(&self) -> i32;
    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<Literal>,
    ) -> Result<Literal, RuntimeBreak>;
}

#[derive(Debug, PartialEq, Clone)]
pub struct Function {
    declaration: Box<FuncDecl>,
}

impl Function {
    pub fn new(declaration: FuncDecl) -> Self {
        Self {
            declaration: Box::new(declaration),
        }
    }
}

impl Callable for Function {
    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<Literal>,
    ) -> Result<Literal, RuntimeBreak> {
        let env = Rc::new(RefCell::new(Environment::new(Some(
            interpreter.globals.clone(),
        ))));
        for param in self.declaration.params.iter().enumerate() {
            env.borrow_mut().define(
                param.1.lexeme.clone(),
                arguments.get(param.0).unwrap().clone(),
            );
        }

        let block_result = interpreter.execute_block(self.declaration.body.clone(), env);

        if let Err(RuntimeBreak::ReturnBreak(re)) = block_result {
            Ok(re.value)
        } else if let Err(RuntimeBreak::RuntimeErrorBreak(re)) = block_result {
            Err(RuntimeBreak::RuntimeErrorBreak(re))
        } else {
            Ok(Literal::Null)
        }
    }

    fn arity(&self) -> i32 {
        self.declaration.params.len() as i32
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fn {}>", self.declaration.name)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum NativeFunction {
    Clock,
}

impl NativeFunction {
    pub fn clock() -> f32 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f32()
    }
}

impl Callable for NativeFunction {
    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<Literal>,
    ) -> Result<Literal, RuntimeBreak> {
        match self {
            NativeFunction::Clock => Ok(Literal::Number(NativeFunction::clock())),
        }
    }

    fn arity(&self) -> i32 {
        0
    }
}

impl Display for NativeFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<native fn>")
    }
}
