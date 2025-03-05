use crate::error_fmt::Error;
use crate::token::{Literal, Token, TokenType};

pub struct Scanner {
    source: Vec<u8>,
    start: usize,
    col: usize,
    line: usize,
    tokens: Vec<Token>,
    errors: Vec<Error>,
}

pub fn scan_tokens(input: String) -> Result<Vec<Token>, Vec<Error>> {
    let mut scanner = Scanner::new();
    scanner.scan_tokens(input);
    if scanner.has_errors() {
        Err(scanner.errors)
    } else {
        Ok(scanner.tokens)
    }
}

impl Scanner {
    pub fn new() -> Self {
        Scanner {
            source: Vec::new(),
            tokens: Vec::new(),
            errors: Vec::new(),
            start: 0,
            col: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(&mut self, input: String) -> Vec<Token> {
        self.source = input.into_bytes();

        while !self.is_end() {
            self.start = self.col;
            self.scan_lexeme();
        }
        self.tokens.clone()
    }

    #[rustfmt::skip]
    fn scan_lexeme(&mut self) {
        let c = *self.advance().unwrap() as char;
        match c {
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            '-' => self.add_token(TokenType::Minus),
            '+' => self.add_token(TokenType::Plus),
            ';' => self.add_token(TokenType::Semicolon),
            '*' => self.add_token(TokenType::Star),
            ' ' => {},
            '\r' => {},
            '\n' => self.line += 1,
            '\t' => {},
            '!' => { 
                let token = if self.is_next('=') { TokenType::BangEqual } else { TokenType::Bang };
                self.add_token(token)
            },
            '=' => {
                let token = if self.is_next('=') { TokenType::EqualEqual } else { TokenType::Equal };
                self.add_token(token)
            }
            '>' => {
                let token = if self.is_next('=') { TokenType::GreaterEqual } else { TokenType::Greater };
                self.add_token(token)
            }
            '<' => {
                let token = if self.is_next('=') { TokenType::LessEqual } else { TokenType::Less };
                self.add_token(token)
            }
            '/' => {
                if self.is_next('/') {
                    while match self.peek() {
                        Some(val) => { 
                            let c = *val as char;
                            if c == '\n' { false } else { true }
                        }
                        None => { false }
                    } { self.advance(); }
                } else { self.add_token(TokenType::Slash) };
            }
            _ => { 
                //if c.is_digit(10) {
                //    self.number()
                //}
                self.add_error()
            },
        }
    }

    //fn number(&mut self) {
    //   while match self.peek() {
    //       Some(val) => { 
    //           let c = *val as char;
    //           if c.is_digit(10) {
    //               true
    //           } else {
    //               false
    //           }
    //       }
    //       None => { false }
    //   } { self.advance(); }
    //   self.add_token_literal(TokenType::Number, Literal())
    //}

    fn is_end(&self) -> bool {
        self.col >= self.source.len()
    }

    fn peek(&self) -> Option<&u8> {
        self.source.get(self.col + 1)
    }

    fn is_next(&mut self, expected: char) -> bool {
        let did_match = match self.peek() {
            Some(c) => *c as char == expected,
            None => false,
        };

        if did_match {
            self.col += 1
        };

        did_match
    }

    fn advance(&mut self) -> Option<&u8> {
        let c = self.source.get(self.col);
        self.col += 1;
        c
    }

    fn add_token(&mut self, token_type: TokenType) {
        self.add_token_literal(token_type, None)
    }

    fn add_token_literal(&mut self, token_type: TokenType, literal: Option<Literal>) {
        self.tokens
            .push(Token::new(token_type, literal, self.line, self.col))
    }

    fn add_error(&mut self) {
        let message = "Lexical Error: Unexpected character".to_string();
        let text = String::from_utf8(self.source.clone())
            .unwrap_or("Invalid UTF8 chars in source.".to_string());
        self.errors.push(Error::new(
            message,
            text,
            self.line.clone(),
            self.col.clone(),
        ))
    }

    fn has_errors(&self) -> bool {
        self.errors.len() != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_single_char_tokens() {
        let tokens = [
            TokenType::LeftParen,
            TokenType::RightParen,
            TokenType::LeftBrace,
            TokenType::RightBrace,
            TokenType::Comma,
            TokenType::Dot,
            TokenType::Minus,
            TokenType::Plus,
            TokenType::Semicolon,
            TokenType::Star,
        ];
        let single_char_string = "\t() {},.-+; *\n".to_string();
        let single_char_tokens: Vec<Token> = scan_tokens(single_char_string)
            .expect("test_scan_single_char_tokens has an invalid single_char_string");
        for i in 0..tokens.len() {
            assert_eq!(tokens[i], single_char_tokens[i].token_type)
        }
    }

    #[test]
    fn test_is_next() {
        let mut scanner = Scanner::new();
        scanner.source = "123".to_string().into_bytes();
        assert_eq!(scanner.is_next('2'), true);
        assert_eq!(scanner.is_next('1'), false);
        assert_eq!(scanner.is_next('3'), true);
    }

    #[test]
    fn test_scan_single_or_double_char_tokens() {
        let tokens = [
            TokenType::Bang,
            TokenType::GreaterEqual,
            TokenType::EqualEqual,
            TokenType::BangEqual,
            TokenType::Less,
            TokenType::LessEqual,
            TokenType::Greater
        ];
        let single_or_double_string = "\t! >= ==!= < <= >\n".to_string();
        let single_or_double_tokens = scan_tokens(single_or_double_string).expect("test_scan_single_or_double_char_tokens has an invalid single_or_double_string");
        for i in 0..tokens.len() {
            assert_eq!(tokens[i], single_or_double_tokens[i].token_type)
        }
    }
}
