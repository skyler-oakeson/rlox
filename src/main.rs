use std::env;
use std::fs;
use std::io::{stdin, stdout, BufRead, Read, Write};

mod errors;
use errors::UsageError;

mod scanner;

fn main() {
    let args: Vec<String> = env::args().collect();
    parse_args(args);
}

pub fn parse_args(args: Vec<String>) {
    match args.len() {
        1 => run_prompt(),
        2 => run_file(args.get(1).unwrap()),
        _ => {
            UsageError::throw(args, 1);
        }
    }
}

fn run_file(path: &str) {
    match fs::read_to_string(path) {
        Ok(s) => {
            let _ = s.lines().map(|l| run(l.to_string()));
        }
        Err(..) => UsageError::throw(env::args().collect(), 2),
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
    println!("Running {source}");
}
