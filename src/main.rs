use std::env;
use std::fs;
use std::io::{stdin, stdout, Write};
use std::process::exit;

mod error_fmt;
mod expression;
mod parser;
mod scanner;
mod token;
mod utils;

use parser::parse;
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

fn run(source: String) {
    // Scanning phase
    let tokens = scan_tokens(source);
    for token in tokens {
        println!("{:?}", token)
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

fn run_file(path: &str) {
    match fs::read_to_string(path) {
        Ok(s) => run(s),
        Err(err) => {
            println!("{}", err.to_string());
            std::process::exit(-1)
        }
    }
}
