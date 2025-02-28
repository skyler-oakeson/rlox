use std::process::exit;

pub struct Error {
    message: String,
    code: i32,
    error_type: String,
}

impl Error {
    pub fn new(message: String, error_type: String, code: i32) -> Self {
        Error {
            message,
            error_type,
            code,
        }
    }

    fn report(&self) {
        println!("[{0}][{1}] {2}", self.code, self.error_type, self.message);
    }

    pub fn exit(&self) {
        self.report();
        exit(self.code);
    }
}

pub struct UsageError {
    error: Error,
}
impl UsageError {
    // 1: Too many args.
    // 2: Inavlid file path.

    pub fn throw(context: Vec<String>, code: i32) {
        let message = match code {
            1 => {
                let args = context.join(" ");
                format!("Too many args. {0}", args)
            }
            2 => {
                format!("Invalid file path: {0}", context.get(2).unwrap())
            }
            _ => {
                format!("Invalid error code.")
            }
        };
        let err = UsageError {
            error: Error::new(message, "UsageError".to_string(), 1),
        };

        err.error.exit()
    }
}
