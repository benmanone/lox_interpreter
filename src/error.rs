use std::fmt::{Debug, Display};

#[derive(Debug)]
pub struct LoxError {
    pub line: u32,
    pub location: u32,
    pub message: String,
}

impl Display for LoxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Error at {} : {}: {}",
            self.line, self.location, self.message
        )
    }
}

impl std::error::Error for LoxError {}
