use crate::error_fmt::Error;
use crate::map;
use crate::token::{Literal, Token, TokenType};
use crate::S;
use std::collections::hash_map::HashMap;

type Lexop = fn(&mut Scanner);

pub struct Scanner {
    source: Vec<u8>,
    start: usize,
    col: usize,
    line: usize,
    tokens: Vec<Token>,
    errors: Vec<Error>,
    keywords: HashMap<String, TokenType>,
    lex_func: HashMap<char, Lexop>,
}

const DO_NOTHING: Lexop = |_s| {};

impl Default for Scanner {
    fn default() -> Scanner {
        Scanner {
            source: Vec::new(),
            tokens: Vec::new(),
            errors: Vec::new(),
            start: 0,
            col: 0,
            line: 1,
            keywords: map![
                {S!("and"), TokenType::And},
                {S!("class"), TokenType::Class},
                {S!("else"), TokenType::Else},
                {S!("false"), TokenType::False},
                {S!("fun"), TokenType::Fun},
                {S!("for"), TokenType::For},
                {S!("if"), TokenType::If},
                {S!("nil"), TokenType::Nil},
                {S!("or"), TokenType::Or},
                {S!("print"), TokenType::Print},
                {S!("return"), TokenType::Return},
                {S!("super"), TokenType::Super},
                {S!("this"), TokenType::This},
                {S!("true"), TokenType::True},
                {S!("var"), TokenType::Var},
                {S!("while"), TokenType::While},
                {S!("eof"), TokenType::Eof}
            ],
            lex_func: map![
                { '{', Self::left_brace as Lexop },
                { '}', Self::right_brace as Lexop },
                { '(', Self::left_paren as Lexop },
                { ')', Self::right_paren as Lexop },
                { ',', Self::comma as Lexop },
                { '.', Self::dot as Lexop },
                { '-', Self::minus as Lexop },
                { '+', Self::plus as Lexop },
                { ';', Self::semicolon as Lexop },
                { '*', Self::star as Lexop },
                { '"', Self::string as Lexop },
                { ' ', |_s| { } },
                { '\r', DO_NOTHING },
                { '\t', DO_NOTHING },
                { '\n', |s| { s.line += 1 } },
                { '!', Self::bang as Lexop },
                { '=', Self::equal as Lexop },
                { '>', Self::greater as Lexop },
                { '<', Self::lesser as Lexop },
                { '/', Self::slash as Lexop }
            ],
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

        // Scan one lexeme at a time until reaching end
        while !self.is_end() {
            self.start = self.col;
            self.scan_lexeme();
        }
        self.tokens.clone()
    }

    fn scan_lexeme(&mut self) {
        let c = *self.advance().unwrap() as char;
        match self.lex_func.get(&c) {
            Some(fun) => fun(self),
            None => {
                if c.is_digit(10) {
                    self.number()
                } else if c.is_ascii_alphabetic() {
                    self.identifier()
                } else {
                    self.add_error("Unexpected character.".to_string())
                }
            }
        }
    }

    fn right_brace(&mut self) {
        self.add_token(TokenType::RightBrace);
    }

    fn left_brace(&mut self) {
        self.add_token(TokenType::LeftBrace);
    }

    fn right_paren(&mut self) {
        self.add_token(TokenType::RightParen)
    }

    fn left_paren(&mut self) {
        self.add_token(TokenType::LeftParen)
    }

    fn comma(&mut self) {
        self.add_token(TokenType::Comma)
    }

    fn dot(&mut self) {
        self.add_token(TokenType::Dot)
    }

    fn minus(&mut self) {
        self.add_token(TokenType::Minus)
    }

    fn plus(&mut self) {
        self.add_token(TokenType::Plus)
    }

    fn semicolon(&mut self) {
        self.add_token(TokenType::Semicolon)
    }

    fn star(&mut self) {
        self.add_token(TokenType::Star)
    }

    fn bang(&mut self) {
        let token = if self.advance_if('=') {
            TokenType::BangEqual
        } else {
            TokenType::Bang
        };
        self.add_token(token)
    }

    fn equal(&mut self) {
        let token = if self.advance_if('=') {
            TokenType::EqualEqual
        } else {
            TokenType::Equal
        };
        self.add_token(token)
    }

    fn greater(&mut self) {
        let token = if self.advance_if('=') {
            TokenType::GreaterEqual
        } else {
            TokenType::Greater
        };
        self.add_token(token)
    }

    fn lesser(&mut self) {
        let token = if self.advance_if('=') {
            TokenType::LessEqual
        } else {
            TokenType::Less
        };
        self.add_token(token)
    }

    fn slash(&mut self) {
        if self.advance_if('/') {
            self.comment();
        } else if self.advance_if('*') {
            self.block_comment();
        } else {
            self.add_token(TokenType::Slash)
        };
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

    fn comment(&mut self) {
        let _ = self.advance_until(|s, c| {
            if c == '\n' {
                s.line += 1;
                Ok(true)
            } else {
                Ok(false)
            }
        });
    }

    fn block_comment(&mut self) {
        let res = self.advance_until(|s, c| {
            if c == '\n' {
                s.line += 1;
                Ok(false)
            } else if c == '*' && s.peek(true).is_some_and(|x| (*x as char) == '/') {
                s.advance();
                Ok(s.advance_if('/'))
            } else if c == '/' && s.peek(true).is_some_and(|x| (*x as char) == '*') {
                s.advance();
                s.advance();
                s.block_comment();
                Ok(false)
            } else if s.peek(false).is_none() {
                Err("Unterminated block comment.".to_string())
            } else {
                Ok(false)
            }
        });
        if res.is_err() {
            self.add_error(res.unwrap_err())
        }
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
            if s.peek(true).is_none() && c != '"' {
                Err("Unterminated string.".to_string())
            } else {
                // Advances past the second quote
                Ok(s.advance_if('"'))
            }
        });

        match res {
            Err(message) => self.add_error(message),
            Ok(_) => {
                // + 1 and -1 to cut quotes off
                let string = String::from_utf8(Vec::from_iter(
                    self.source[self.start + 1..self.col - 1].iter().cloned(),
                ))
                .unwrap();
                self.add_token_literal(TokenType::String, Some(Literal::String(string)));
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
            "Lexical Error: ".to_string() + &message,
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
            (TokenType::String, "the \ncinema too\n"),
            (TokenType::Dot, ""),
        ];
        let literal_string = "\"I\" \"waited\" var \"in\" and \"the \ncinema too\n\".".to_string();
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

    #[test]
    fn test_errors() {
        let error = Error {
            message: "Lexical Error: Unexpected character.".to_string(),
            text: "".to_string(),
            line: 1,
            col: 1,
        };

        let error2 = Error {
            message: "Lexical Error: Unterminated string.".to_string(),
            text: "".to_string(),
            line: 1,
            col: 7,
        };

        let error_string = "~ \"test ".to_string();
        let errors = scan_tokens(error_string).unwrap_err();
        assert_eq!(error.message, errors[0].message);
        assert_eq!(error.line, errors[0].line);
        assert_eq!(error.col, errors[0].col);
        assert_eq!(error2.message, errors[1].message);
        assert_eq!(error2.line, errors[1].line);
        assert_eq!(error2.col, errors[1].col);
    }
}
