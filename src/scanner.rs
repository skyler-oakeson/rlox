use crate::error_fmt::report_errors;
use crate::error_fmt::Error;
use crate::map;
use crate::token::{Literal, Token, TokenType};
use crate::S;
use std::collections::hash_map::HashMap;

type Lexop = fn(&mut Scanner);
const DO_NOTHING: Lexop = |_s| {};

pub struct Scanner {
    col: usize,
    errors: Vec<Error>,
    keywords: HashMap<String, TokenType>,
    lex_func: HashMap<char, Lexop>,
    line: usize,
    start: usize,
    source: Vec<u8>,
    tokens: Vec<Token>,
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
            keywords: map![
                { S!("and"), TokenType::And },
                { S!("class"), TokenType::Class },
                { S!("else"), TokenType::Else },
                { S!("false"), TokenType::False },
                { S!("fun"), TokenType::Fun },
                { S!("for"), TokenType::For },
                { S!("if"), TokenType::If },
                { S!("nil"), TokenType::Nil },
                { S!("or"), TokenType::Or },
                { S!("print"), TokenType::Print },
                { S!("return"), TokenType::Return },
                { S!("super"), TokenType::Super },
                { S!("this"), TokenType::This },
                { S!("true"), TokenType::True },
                { S!("var"), TokenType::Var },
                { S!("while"), TokenType::While },
                { S!("eof"), TokenType::Eof }
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
                { ' ', DO_NOTHING },
                { '\r', DO_NOTHING },
                { '\t', DO_NOTHING },
                { '\n', |s| { s.line += 1 } },
                { '!', Self::bang as Lexop },
                { '=', Self::equal as Lexop },
                { '>', Self::greater as Lexop },
                { '<', Self::lesser as Lexop },
                { '/', Self::slash as Lexop },
                { '?', Self::question as Lexop },
                { ':', Self::colon as Lexop }
            ],
        }
    }
}

pub fn scan_tokens(input: &String) -> Vec<Token> {
    let mut scanner = Scanner::default();
    scanner.scan_tokens(input.clone());
    if scanner.has_errors() {
        report_errors(&scanner.errors);
    }
    scanner.tokens
}

impl Scanner {
    fn add_error(&mut self, message: String) {
        let line =
            String::from_utf8(self.source.clone()).unwrap_or(S!("Invalid UTF8 chars in source."));
        self.errors.push(Error::new(
            S!("Lexical Error: ") + &message,
            S!(line),
            self.line.clone(),
            self.col.clone(),
        ))
    }

    fn add_token(&mut self, token_type: TokenType) {
        self.add_token_literal(token_type, None)
    }

    fn add_token_literal(&mut self, token_type: TokenType, literal: Option<Literal>) {
        let lexeme = match token_type {
            TokenType::String | TokenType::Number | TokenType::Identifier => {
                literal.clone().unwrap().to_string()
            }
            _ => String::from_utf8(Vec::from_iter(
                self.source[self.start..self.col].iter().cloned(),
            ))
            .unwrap(),
        };
        self.tokens
            .push(Token::new(token_type, lexeme, literal, self.line, self.col))
    }

    fn advance(&mut self) -> Option<&u8> {
        let c = self.source.get(self.col);
        self.col += 1;
        c
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

    fn bang(&mut self) {
        let token = if self.advance_if('=') {
            TokenType::BangEqual
        } else {
            TokenType::Bang
        };
        self.add_token(token)
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
                Err(S!("Unterminated block comment."))
            } else {
                Ok(false)
            }
        });
        if res.is_err() {
            self.add_error(res.unwrap_err())
        }
    }

    fn colon(&mut self) {
        self.add_token(TokenType::Colon)
    }

    fn comma(&mut self) {
        self.add_token(TokenType::Comma)
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

    fn dot(&mut self) {
        self.add_token(TokenType::Dot)
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

    fn has_errors(&self) -> bool {
        self.errors.len() != 0
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

    fn is_end(&self) -> bool {
        self.col >= self.source.len()
    }

    fn left_brace(&mut self) {
        self.add_token(TokenType::LeftBrace);
    }

    fn left_paren(&mut self) {
        self.add_token(TokenType::LeftParen)
    }

    fn lesser(&mut self) {
        let token = if self.advance_if('=') {
            TokenType::LessEqual
        } else {
            TokenType::Less
        };
        self.add_token(token)
    }

    fn minus(&mut self) {
        self.add_token(TokenType::Minus)
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

    fn peek(&self, one_extra: bool) -> Option<&u8> {
        self.source.get(self.col + one_extra as usize)
    }

    fn plus(&mut self) {
        self.add_token(TokenType::Plus)
    }

    fn right_brace(&mut self) {
        self.add_token(TokenType::RightBrace);
    }

    fn right_paren(&mut self) {
        self.add_token(TokenType::RightParen)
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
                    self.add_error(S!("Unexpected character."))
                }
            }
        }
    }

    pub fn scan_tokens(&mut self, input: String) -> Vec<Token> {
        self.source = input.into_bytes();

        // Scan one lexeme at a time until reaching end
        while !self.is_end() {
            self.start = self.col;
            self.scan_lexeme();
        }

        self.tokens.clone()
    }

    fn semicolon(&mut self) {
        self.add_token(TokenType::Semicolon)
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

    fn star(&mut self) {
        self.add_token(TokenType::Star)
    }

    fn string(&mut self) {
        let res = self.advance_until(|s, c| {
            if c == '\n' {
                s.line += 1
            };
            if s.peek(true).is_none() && c != '"' {
                Err(S!("Unterminated string."))
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

    fn question(&mut self) {
        self.add_token(TokenType::Question)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peek() {
        let mut scanner = Scanner::default();
        scanner.source = S!("123").into_bytes();

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
        scanner.source = S!("123").into_bytes();
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
        let single_char_string = S!("\t() {},.-+; *\n");
        let single_char_tokens: Vec<Token> = scan_tokens(&single_char_string);
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
        let literal_string = S!("12.3 12..3");
        let literal_tokens: Vec<Token> = scan_tokens(&literal_string);
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
        let literal_string = S!("\"I\" \"waited\" var \"in\" and \"the \ncinema too\n\".");
        let literal_tokens: Vec<Token> = scan_tokens(&literal_string);
        for i in 0..tokens.len() {
            assert_eq!(tokens[i].0, literal_tokens[i].token_type);
            assert_eq!(
                tokens[i].1,
                literal_tokens[i]
                    .literal
                    .to_owned()
                    .unwrap_or(Literal::String(S!("")))
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
        let literal_string = S!("and class else false fun for if nil or print return super this true var while eof test THIS Let");
        let literal_tokens: Vec<Token> = scan_tokens(&literal_string);
        for i in 0..tokens.len() {
            assert_eq!(tokens[i].0, literal_tokens[i].token_type);
            assert_eq!(
                tokens[i].1,
                literal_tokens[i]
                    .literal
                    .to_owned()
                    .unwrap_or(Literal::Identifier(S!("")))
                    .as_identifier()
                    .unwrap()
            )
        }
    }

    #[test]
    fn test_advance_if() {
        let mut scanner = Scanner::default();
        scanner.source = S!("123").into_bytes();
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
        let single_or_double_string = S!("\t! >= ==!= < <= >\n");
        let single_or_double_tokens = scan_tokens(&single_or_double_string);
        for i in 0..tokens.len() {
            assert_eq!(tokens[i], single_or_double_tokens[i].token_type)
        }
    }

    #[test]
    fn test_errors() {
        let error = Error {
            message: S!("Lexical Error: Unexpected character."),
            text: S!(""),
            line: 1,
            col: 1,
        };

        let error2 = Error {
            message: S!("Lexical Error: Unterminated string."),
            text: S!(""),
            line: 1,
            col: 7,
        };

        let error_string = S!("~ \"test ");
        let mut scanner = Scanner::default();
        scanner.scan_tokens(error_string);
        assert_eq!(error.message, scanner.errors[0].message);
        assert_eq!(error.line, scanner.errors[0].line);
        assert_eq!(error.col, scanner.errors[0].col);
        assert_eq!(error2.message, scanner.errors[1].message);
        assert_eq!(error2.line, scanner.errors[1].line);
        assert_eq!(error2.col, scanner.errors[1].col);
    }
}
