use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::stdout;
use std::io::{self, stdin};
use std::process::exit;
use std::rc::Rc;

pub mod token;

pub mod scanner;
use scanner::*;

pub mod error;
use error::*;

pub mod lox;
use lox::*;

pub mod interpreter;

pub mod parser;

fn main() {
    let args: Rc<[String]> = env::args().collect();

    let _int = Lox::new(args);
}
