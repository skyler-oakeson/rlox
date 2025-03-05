use std::fmt::Display;

#[derive(Debug)]
pub struct Error {
    pub message: String,
    pub text: String,
    pub line: usize,
    pub col: usize,
}

impl Error {
    pub fn new(message: String, text: String, line: usize, col: usize) -> Self {
        Error {
            message,
            text,
            line,
            col,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{0}\n|\n|{1}. {2}\n|{3}↑ \n",
            self.message,
            self.line,
            self.text.trim(),
            std::iter::repeat(" ")
                .take(self.col + 2)
                .collect::<String>()
        )
    }
}

pub fn report_errors(errors: &Vec<Error>) {
    for error in errors {
        println!("{}", error)
    }
}
