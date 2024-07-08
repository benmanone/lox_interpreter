use crate::exit;
use crate::io;
use crate::stdin;
use crate::stdout;
use crate::File;
use crate::LoxError;
use crate::Rc;
use crate::Scanner;
use std::io::{Read, Write};

pub struct Interpreter {
    args: Rc<[String]>,
    had_error: bool,
}

impl Interpreter {
    pub fn new(args: Rc<[String]>) -> Result<Self, io::Error> {
        // decide to run a script or trigger prompt

        let mut i = Self {
            args,
            had_error: false,
        };

        let len = i.args.len();

        match len {
            2 => {
                let _run_result = i.run_file(i.args[1].clone())?;
            }
            _ => {
                if len > 2 {
                    println!("Usage: rlox [script]");
                    exit(64);
                } else {
                    let _prompt = &i.run_prompt()?;
                }
            }
        };

        Ok(i)
    }

    pub fn run_file(&mut self, path: String) -> Result<String, io::Error> {
        // read contents of file and run it
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Interpreter::run(self, contents.as_str());

        if self.had_error {
            exit(65);
        }

        Ok(contents)
    }

    pub fn run_prompt(&mut self) -> Result<String, io::Error> {
        loop {
            let mut input = String::new();

            print!("> ");

            stdout().flush().unwrap();
            stdin().read_line(&mut input).expect("Failed to read input");

            self.run(input.as_str());

            self.had_error = false;
        }
    }

    pub fn run(&mut self, source: &str) {
        let mut scanner = Scanner::new(String::from(source));
        let result = scanner.scan_tokens();

        if let Err(e) = result {
            self.error(e);
        } else if let Ok(tokens) = result {
            for t in tokens {
                println!("{}", t);
            }
        }
    }

    pub fn error(&mut self, err: LoxError) {
        self.report(err);
    }

    pub fn report(&mut self, err: LoxError) {
        println!(
            "[line {}] Error {}: {}",
            err.line, err.location, err.message
        );
        self.had_error = true
    }
}
