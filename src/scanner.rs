use crate::error_fmt::Error;
use crate::token::{Literal, Token, TokenType};
use std::collections::hash_map::HashMap;

pub struct Scanner {
    source: Vec<u8>,
    start: usize,
    col: usize,
    line: usize,
    tokens: Vec<Token>,
    errors: Vec<Error>,
    keywords: HashMap<String, TokenType>,
}

impl Default for Scanner {
    fn default() -> Scanner {
        Scanner {
            source: Vec::new(),
            tokens: Vec::new(),
            errors: Vec::new(),
            start: 0,
            col: 0,
            line: 1,
            keywords: vec![
                ("and", TokenType::And),
                ("class", TokenType::Class),
                ("else", TokenType::Else),
                ("false", TokenType::False),
                ("fun", TokenType::Fun),
                ("for", TokenType::For),
                ("if", TokenType::If),
                ("nil", TokenType::Nil),
                ("or", TokenType::Or),
                ("print", TokenType::Print),
                ("return", TokenType::Return),
                ("super", TokenType::Super),
                ("this", TokenType::This),
                ("true", TokenType::True),
                ("var", TokenType::Var),
                ("while", TokenType::While),
                ("eof", TokenType::Eof),
            ].into_iter().map(|(k, v)| (String::from(k), v)).collect()
        }
    }
}


pub fn scan_tokens(input: String) -> Result<Vec<Token>, Vec<Error>> {
    let mut scanner = Scanner::default();
    scanner.scan_tokens(input);
    if scanner.has_errors() {
        Err(scanner.errors)
    } else {
        Ok(scanner.tokens)
    }
}

impl Scanner {

    pub fn scan_tokens(&mut self, input: String) -> Vec<Token> {
        self.source = input.into_bytes();

        while !self.is_end() {
            self.start = self.col;
            self.scan_lexeme();
        }
        self.tokens.clone()
    }

    fn scan_lexeme(&mut self) {
        let c = *self.advance().unwrap() as char;

        if !c.is_ascii() {
            self.add_error();
            self.advance_until(|_, c| c.is_ascii())
        }

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
            ' ' => {}
            '\r' => {}
            '\n' => self.line += 1,
            '\t' => {}
            '!' => {
                let token = if self.advance_if('=') {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                };
                self.add_token(token)
            }
            '=' => {
                let token = if self.advance_if('=') {
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                };
                self.add_token(token)
            }
            '>' => {
                let token = if self.advance_if('=') {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                };
                self.add_token(token)
            }
            '<' => {
                let token = if self.advance_if('=') {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                };
                self.add_token(token)
            }
            '/' => {
                if self.advance_if('/') {
                    self.advance_until(|_s, c| if c == '\n' { true } else { false });
                } else {
                    self.add_token(TokenType::Slash)
                };
            }
            _ => {
                if c.is_digit(10) {
                    self.number()
                } else if c.is_ascii_alphabetic() {
                    self.identifier()
                } else {
                    self.add_error()
                }
            }
        }
    }

    fn advance_until(&mut self, mut until: impl FnMut(&mut Scanner, char) -> bool) {
        while !match self.peek(false) {
            Some(val) => {
                let c = *val as char;
                until(self, c)
            }
            None => true,
        } {
            self.advance();
        }
    }

    fn number(&mut self) {
        self.advance_until(|s, c| match c.is_digit(10) {
            true => false,
            false => {
                let mut stop = true;
                if c == '.' {
                    let next = s.peek(true);
                    let res = next.is_some_and(|n| (*n as char).is_digit(10));
                    match res {
                        true => stop = false,
                        false => stop = true,
                    }
                };
                stop
            }
        });
        let num = String::from_utf8(Vec::from_iter(
            self.source[self.start..self.col].iter().cloned(),
        ))
        .unwrap()
        .parse::<f64>()
        .unwrap();
        self.add_token_literal(TokenType::Number, Some(Literal::Number(num)))
    }

    fn identifier(&mut self) {
        self.advance_until(|_, c| {
            !c.is_alphanumeric()
        });
        
        let identifier = String::from_utf8(Vec::from_iter(
            self.source[self.start..self.col].iter().cloned(),
        )).unwrap();

        match self.keywords.get(&identifier) {
            Some(tt) => self.add_token(tt.clone()),
            None => { self.add_token_literal(TokenType::Identifier, Some(Literal::Identifier(identifier))) },
        };
    }

    fn is_end(&self) -> bool {
        self.col >= self.source.len()
    }

    fn peek(&self, one_extra: bool) -> Option<&u8> {
        self.source.get(self.col + one_extra as usize)
    }

    fn advance_if(&mut self, expected: char) -> bool {
        let did_match = match self.peek(false) {
            Some(c) => *c as char == expected,
            None => false,
        };

        if did_match {
            self.advance();
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
    fn test_peek() {
        let mut scanner = Scanner::default();
        scanner.source = "123".to_string().into_bytes();

        assert_eq!('1', *scanner.peek(false).unwrap() as char);
        assert_eq!('1', *scanner.peek(false).unwrap() as char);
        assert_ne!('2', *scanner.peek(false).unwrap() as char);
        assert_ne!(
            *scanner.advance().unwrap() as char,
            *scanner.peek(false).unwrap() as char
        );
    }

    #[test]
    fn test_advance_until() {
        let mut scanner = Scanner::default();
        scanner.source = "123".to_string().into_bytes();
        // Should advance until the end of the string
        scanner.advance_until(|_s, c| if c.is_digit(10) { false } else { true });
        assert_eq!(scanner.advance(), None)
    }

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
    fn test_scan_number_literals() {
        let tokens = [TokenType::Number];
        let literal_string = "12".to_string();
        let literal_tokens: Vec<Token> =
            scan_tokens(literal_string).expect("literal_string has an invalid literal");
        for i in 0..tokens.len() {
            println!("{}", literal_tokens[i]);
            assert_eq!(tokens[i], literal_tokens[i].token_type)
        }
    }

    #[test]
    fn test_scan_string_literals() {}

    #[test]
    fn test_scan_identifier_literals() {}

    #[test]
    fn test_advance_if() {
        let mut scanner = Scanner::default();
        scanner.source = "123".to_string().into_bytes();
        assert_eq!(scanner.advance_if('1'), true);
        assert_eq!(scanner.advance_if('3'), false);
        assert_eq!(scanner.advance_if('2'), true);
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
            TokenType::Greater,
        ];
        let single_or_double_string = "\t! >= ==!= < <= >\n".to_string();
        let single_or_double_tokens = scan_tokens(single_or_double_string).expect(
            "test_scan_single_or_double_char_tokens has an invalid single_or_double_string",
        );
        for i in 0..tokens.len() {
            assert_eq!(tokens[i], single_or_double_tokens[i].token_type)
        }
    }
}
