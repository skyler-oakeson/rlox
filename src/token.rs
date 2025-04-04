use std::fmt::Display;

#[allow(dead_code)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TokenType {
    // Single character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Semicolon,
    Minus,
    Plus,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier,
    String,
    Number,

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
    Eof,
}

macro_rules! as_variant {
    ($value:expr, $variant:path) => {
        match $value {
            $variant(x) => Some(x),
            _ => None,
        }
    };
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Identifier(String),
    String(String),
    Number(f64),
}

impl Literal {
    pub fn as_number(self) -> Option<f64> {
        as_variant!(self, Literal::Number)
    }

    pub fn as_identifier(self) -> Option<String> {
        as_variant!(self, Literal::Identifier)
    }

    pub fn as_string(self) -> Option<String> {
        as_variant!(self, Literal::String)
    }
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Number(val) => write!(f, "{}", val),
            Literal::Identifier(val) => write!(f, "{}", val),
            Literal::String(val) => write!(f, "{}", val),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: usize,
    pub col: usize,
    pub literal: Option<Literal>,
}

impl Token {
    pub fn new(
        token_type: TokenType,
        lexeme: String,
        literal: Option<Literal>,
        line: usize,
        col: usize,
    ) -> Self {
        Token {
            token_type,
            lexeme,
            literal,
            col,
            line,
        }
    }
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        other.token_type == self.token_type
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.lexeme)
    }
}
