use std::collections::HashMap;

use crate::{
    token::{Literal, Token},
    RuntimeError,
};

pub struct Environment {
    values: HashMap<String, Literal>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, value: Literal) {
        self.values.insert(name, value);
        // println!("{:?}", self.values);
    }

    // can't create a new variable
    pub fn assign(&mut self, name: Token, value: Literal) -> Result<(), RuntimeError> {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme, value);
            Ok(())
        } else {
            Err(RuntimeError {
                token: name.clone(),
                message: format!("Undefined variable '{}'.", &name.lexeme),
            })
        }
    }

    pub fn get(&self, name: Token) -> Result<Literal, RuntimeError> {
        if self.values.contains_key(&name.lexeme) {
            // println!("{:?}", self.values);
            Ok(self.values.get(&name.lexeme).unwrap().clone())
        } else {
            // println!("{:?}", self.values);
            Err(RuntimeError {
                token: name.clone(),
                message: format!("Undefined variable '{}'.", &name.lexeme),
            })
        }
    }
}
