mod ast;
mod error;
mod parser;
mod scanner;
mod token;

use crate::error::ErrorReporter;
use crate::parser::Parser;
use crate::scanner::Scanner;
use std::env::args;
use std::io::Write;
use std::process::ExitCode;
use std::{fs, io};

struct Lox {
    reporter: ErrorReporter,
}

impl Lox {
    fn new() -> Self {
        Self {
            reporter: ErrorReporter::new(),
        }
    }

    fn run_file(&mut self, path: String) -> ExitCode {
        let source = fs::read_to_string(path).unwrap();
        self.run(&source);
        if self.reporter.had_error() {
            return ExitCode::from(65);
        }
        ExitCode::SUCCESS
    }

    fn run_prompt(&mut self) {
        let mut line = String::new();

        loop {
            print!("> ");
            io::stdout().flush().unwrap();
            line.clear();

            if io::stdin().read_line(&mut line).unwrap() == 0 {
                break;
            }
            self.run(line.trim());
            self.reporter.reset();
        }
    }

    fn run(&mut self, source: &str) {
        let scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens(&mut self.reporter);
        let mut parser = Parser::new(tokens);

        if let Some(expression) = parser.parse(&mut self.reporter) {
            if self.reporter.had_error() { return; }
            println!("{}", expression);
        }
    }
}

fn main() -> ExitCode {
    let mut lox = Lox::new();
    let mut args = args();

    match args.len() {
        1 => {
            lox.run_prompt();
            ExitCode::SUCCESS
        }
        2 => {
            let path = args.nth(1).unwrap();
            lox.run_file(path)
        }
        _ => {
            eprintln!("Usage: rlox [script]");
            ExitCode::from(64)
        }
    }
}
