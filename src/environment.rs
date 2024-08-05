use crate::RuntimeBreak;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::{
    token::{Literal, Token},
    RuntimeError,
};

#[derive(Debug)]
pub struct Environment {
    enclosing: Option<Rc<RefCell<Environment>>>,
    values: HashMap<String, Literal>,
}

impl Environment {
    // global environment will pass in None for enclosing
    pub fn new(enclosing: Option<Rc<RefCell<Environment>>>) -> Self {
        if let Some(enc) = enclosing {
            Self {
                enclosing: Some(enc),
                values: HashMap::new(),
            }
        } else {
            Self {
                enclosing: None,
                values: HashMap::new(),
            }
        }
    }

    pub fn define(&mut self, name: String, value: Literal) {
        self.values.insert(name, value);
    }

    // can't create a new variable
    pub fn assign(&mut self, name: Token, value: Literal) -> Result<(), RuntimeBreak> {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme, value);
            Ok(())
        } else if let Some(ref mut enc) = self.enclosing {
            enc.borrow_mut().assign(name, value)
        } else {
            Err(RuntimeBreak::RuntimeErrorBreak(RuntimeError {
                token: name.clone(),
                message: format!("Undefined variable '{}'.", &name.lexeme),
            }))
        }
    }

    pub fn get(&self, name: Token) -> Result<Literal, RuntimeError> {
        if self.values.contains_key(&name.lexeme) {
            Ok(self.values.get(&name.lexeme).unwrap().clone())
        }
        // recursively search for the variable in enclosing environment
        else if let Some(ref enc) = self.enclosing {
            enc.borrow().get(name)
        } else {
            Err(RuntimeError {
                token: name.clone(),
                message: format!("Undefined variable '{}'.", &name.lexeme),
            })
        }
    }
}
