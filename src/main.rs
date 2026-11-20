use std::env::args;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = args();

    match args.len() {
        1 => {
            run_prompt();
            ExitCode::SUCCESS
        }
        2 => {
            let path = args.nth(1).unwrap();
            run_file(path);
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!("Usage: rlox [script]");
            ExitCode::from(64)
        }
    }
}

fn run_file(path: String) {
    println!("Running from a Path '{}'", path)
}

fn run_prompt() {
    println!("Running REPL")
}