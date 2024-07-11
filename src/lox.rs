use crate::exit;
use crate::interpreter::Interpreter;
use crate::io;
use crate::parser::Parser;
use crate::stdin;
use crate::stdout;
use crate::File;
use crate::Rc;
use crate::RuntimeError;
use crate::Scanner;
use std::error::Error;
use std::io::{Read, Write};

pub struct Lox {
    args: Rc<[String]>,
    interpreter: Interpreter,
    had_error: bool,
    had_runtime_error: bool,
}

impl Lox {
    pub fn new(args: Rc<[String]>) -> Result<Self, io::Error> {
        // decide to run a script or trigger prompt

        let mut i = Self {
            args,
            interpreter: Interpreter::new(),
            had_error: false,
            had_runtime_error: false,
        };

        let len = i.args.len();

        match len {
            2 => {
                i.run_file(i.args[1].clone())?;
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

    fn run_file(&mut self, path: String) -> Result<String, io::Error> {
        // read contents of file and run it
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Lox::run(self, contents.as_str());

        if self.had_error {
            exit(65);
        } else if self.had_runtime_error {
            exit(70);
        }

        Ok(contents)
    }

    fn run_prompt(&mut self) -> Result<String, io::Error> {
        loop {
            let mut input = String::new();

            print!("> ");

            stdout().flush().unwrap();
            stdin().read_line(&mut input).expect("Failed to read input");

            self.run(input.as_str());

            self.had_error = false;
        }
    }

    fn run(&mut self, source: &str) {
        let mut scanner = Scanner::new(String::from(source));
        let result = scanner.scan_tokens();

        match result {
            Err(err) => {
                self.error(err);
            }
            Ok(tokens) => {
                // for t in tokens {
                //     println!("{}", t);
                // }

                let mut parser = Parser::new(tokens.clone());
                let result = parser.parse();

                if let Ok(stmts) = result {
                    // println!("{:#?}", stmts);
                    let interpret_result = self.interpreter.interpret(stmts);

                    if let Err(err) = interpret_result {
                        self.runtime_error(err);
                    }
                } else if let Err(err) = result {
                    self.error(err)
                }
            }
        }
    }

    fn error<T>(&mut self, err: T)
    where
        T: Error,
    {
        self.report(err);
    }

    fn report<T>(&mut self, err: T)
    where
        T: Error,
    {
        println!("{err}");
        self.had_error = true
    }

    fn runtime_error(&mut self, err: RuntimeError) {
        println!("{err}");
        self.had_runtime_error = true;
    }
}
