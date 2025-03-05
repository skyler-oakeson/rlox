use std::env;
use std::fs;
use std::io::{stdin, stdout, Write};
use std::process::exit;

mod error_fmt;
mod scanner;
mod token;

use scanner::scan_tokens;
use token::Token;

fn main() {
    let args: Vec<String> = env::args().collect();
    parse_args(args);
}

pub fn parse_args(args: Vec<String>) {
    match args.len() {
        1 => run_prompt(),
        2 => run_file(args.get(1).unwrap()),
        _ => {
            println!("Too many args");
            std::process::exit(-1)
        }
    }
}

fn run_file(path: &str) {
    match fs::read_to_string(path) {
        Ok(s) => s.lines().for_each(|l| run(l.to_string())),
        Err(err) => {
            println!("{:?}", err);
            std::process::exit(-1)
        }
    }
}

fn run_prompt() {
    let input = &mut String::new();
    loop {
        print!("> ");
        Write::flush(&mut stdout()).expect("Flush failed!");
        input.clear();
        let _ = stdin().read_line(input);
        run(input.to_string());
    }
}

fn run(source: String) {
    // Scanning phase
    let tokens: Vec<Token> = match scan_tokens(source) {
        Ok(tokens) => tokens,
        Err(errors) => {
            error_fmt::report_errors(&errors);
            println!("{0} Errors detected in scanning phase.", errors.len());
            exit(-1)
        }
    };

    for token in tokens {
        println!("{}", token)
    }
}
