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
            ]
            .into_iter()
            .map(|(k, v)| (String::from(k), v))
            .collect(),
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
            self.add_error("Lexical Error: Unexpected character".to_string());
            let _ = self.advance_until(|_, c| Ok(c.is_ascii()));
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
            '"' => self.string(),
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
                    let _ =
                        self.advance_until(|_s, c| if c == '\n' { Ok(true) } else { Ok(false) });
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
                    self.add_error("Lexical Error: Unexpected character".to_string())
                }
            }
        }
    }

    fn advance_until(
        &mut self,
        mut until: impl FnMut(&mut Scanner, char) -> Result<bool, String>,
    ) -> Result<(), String> {
        while !match self.peek(false) {
            Some(val) => {
                let c = *val as char;
                until(self, c)?
            }
            None => true,
        } {
            self.advance();
        }
        Ok(())
    }

    fn number(&mut self) {
        let _ = self.advance_until(|s, c| match c.is_digit(10) {
            true => Ok(false),
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
                Ok(stop)
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
        let _ = self.advance_until(|_, c| Ok(!c.is_alphanumeric()));

        let identifier = String::from_utf8(Vec::from_iter(
            self.source[self.start..self.col].iter().cloned(),
        ))
        .unwrap();

        match self.keywords.get(&identifier) {
            Some(tt) => self.add_token(tt.clone()),
            None => {
                self.add_token_literal(TokenType::Identifier, Some(Literal::Identifier(identifier)))
            }
        };
    }

    fn string(&mut self) {
        let res = self.advance_until(|s, c| {
            if c == '\n' {
                s.line += 1
            };
            if s.is_end() && c != '"' {
                Err("Unterminated string.".to_string())
            } else {
                Ok(c == '"')
            }
        });

        match res {
            Err(message) => self.add_error(message),
            Ok(_) => {
                // + 1 to start to avoid quote
                let string = String::from_utf8(Vec::from_iter(
                    self.source[self.start + 1..self.col].iter().cloned(),
                ))
                .unwrap();
                self.add_token_literal(TokenType::String, Some(Literal::String(string)));
                // Advace past the second quote
                self.advance();
            }
        }
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

    fn add_error(&mut self, message: String) {
        let line = String::from_utf8(self.source.clone())
            .unwrap_or("Invalid UTF8 chars in source.".to_string());
        self.errors.push(Error::new(
            message,
            line,
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
        let _ = scanner.advance_until(|_s, c| if c.is_digit(10) { Ok(false) } else { Ok(true) });
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
        let tokens = [
            (TokenType::Number, 12.3),
            (TokenType::Number, 12.0),
            (TokenType::Dot, 0.0),
            (TokenType::Dot, 0.0),
            (TokenType::Number, 3.0),
        ];
        let literal_string = "12.3 12..3".to_string();
        let literal_tokens: Vec<Token> =
            scan_tokens(literal_string).expect("literal_string has an invalid literal");
        for i in 0..tokens.len() {
            assert_eq!(tokens[i].0, literal_tokens[i].token_type);
            assert_eq!(
                tokens[i].1,
                literal_tokens[i]
                    .literal
                    .to_owned()
                    .unwrap_or(Literal::Number(0.0))
                    .as_number()
                    .unwrap()
            )
        }
    }

    #[test]
    fn test_scan_string_literals() {
        let tokens = [
            (TokenType::String, "I"),
            (TokenType::String, "waited"),
            (TokenType::Var, ""),
            (TokenType::String, "in"),
            (TokenType::And, ""),
            (TokenType::String, "the cinema too\n"),
            (TokenType::Dot, ""),
        ];
        let literal_string = "\"I\" \"waited\" var \"in\" and \"the cinema too\n\".".to_string();
        let literal_tokens: Vec<Token> =
            scan_tokens(literal_string).expect("literal_string has an invalid literal");
        for i in 0..tokens.len() {
            assert_eq!(tokens[i].0, literal_tokens[i].token_type);
            assert_eq!(
                tokens[i].1,
                literal_tokens[i]
                    .literal
                    .to_owned()
                    .unwrap_or(Literal::String("".to_string()))
                    .as_string()
                    .unwrap()
            )
        }
    }

    #[test]
    fn test_scan_identifier_literals() {
        let tokens = [
            (TokenType::And, ""),
            (TokenType::Class, ""),
            (TokenType::Else, ""),
            (TokenType::False, ""),
            (TokenType::Fun, ""),
            (TokenType::For, ""),
            (TokenType::If, ""),
            (TokenType::Nil, ""),
            (TokenType::Or, ""),
            (TokenType::Print, ""),
            (TokenType::Return, ""),
            (TokenType::Super, ""),
            (TokenType::This, ""),
            (TokenType::True, ""),
            (TokenType::Var, ""),
            (TokenType::While, ""),
            (TokenType::Eof, ""),
            (TokenType::Identifier, "test"),
            (TokenType::Identifier, "THIS"),
            (TokenType::Identifier, "Let"),
        ];
        let literal_string = "and class else false fun for if nil or print return super this true var while eof test THIS Let".to_string();
        let literal_tokens: Vec<Token> =
            scan_tokens(literal_string).expect("literal_string has an invalid literal");
        for i in 0..tokens.len() {
            assert_eq!(tokens[i].0, literal_tokens[i].token_type);
            assert_eq!(
                tokens[i].1,
                literal_tokens[i]
                    .literal
                    .to_owned()
                    .unwrap_or(Literal::Identifier("".to_string()))
                    .as_identifier()
                    .unwrap()
            )
        }
    }

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
