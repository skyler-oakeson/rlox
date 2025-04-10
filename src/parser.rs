use crate::expression::{Bin, Cond, Expr, Grp, Lit, Un};
use crate::marcher::Marcher;
use crate::token::{Token, TokenType};

/*                    Grammer for lox
 * --------------------------------------------------------
 * expression -> ternary;
 * ternary    -> equality ? expression : expression;
 * equality   -> comparison ( ("=" | "!=") comparison )*;
 * comparison -> term ( (">" | ">=" | "<" | "<=") term )*;
 * term       -> factor ( ("*" | "/") factor)*;
 * factor     -> unary ( ("+" | "-") unary)*;
 * unary      -> ("!" | "-") unary | primary
 * primary    -> NUMBER | STRING | "true" | "false" | "nil" | "(" expression ")";
 */

pub struct Parser {
    tokens: Marcher<Token>,
}

pub fn parse(tokens: &Vec<Token>) -> Box<dyn Expr> {
    let mut parser = Parser::new(tokens);
    parser.expression()
}

impl Parser {
    pub fn new(tokens: &Vec<Token>) -> Self {
        Parser {
            tokens: Marcher::new(tokens.to_vec()),
        }
    }

    fn expression(&mut self) -> Box<dyn Expr> {
        let mut expr = self.ternary();
        while self
            .tokens
            .advance_if(|t| t.token_type == TokenType::Comma)
            .is_some()
        {
            expr = self.equality();
        }
        expr
    }

    fn ternary(&mut self) -> Box<dyn Expr> {
        let mut expr: Box<dyn Expr> = self.equality();
        if self
            .tokens
            .advance_if(|t| t.token_type == TokenType::Question)
            .is_some()
        {
            expr = Box::new(Cond {
                cond: expr,
                cons: self.expression(),
                alt: {
                    if self
                        .tokens
                        .advance_if(|t| t.token_type == TokenType::Colon)
                        .is_none()
                    {
                        panic!("No alternate condition provided")
                    }
                    self.expression()
                },
            })
        }
        expr
    }

    fn equality(&mut self) -> Box<dyn Expr> {
        let mut expr: Box<dyn Expr> = self.comparison();
        while let Some(op) = self.tokens.advance_if(|t| {
            t.token_type == TokenType::BangEqual || t.token_type == TokenType::EqualEqual
        }) {
            expr = Box::new(Bin {
                left: expr,
                operator: op.clone(),
                right: self.comparison(),
            })
        }

        expr
    }

    fn comparison(&mut self) -> Box<dyn Expr> {
        let mut expr: Box<dyn Expr> = self.term();

        while let Some(op) = self.tokens.advance_if(|t| {
            t.token_type == TokenType::Greater
                || t.token_type == TokenType::GreaterEqual
                || t.token_type == TokenType::LessEqual
                || t.token_type == TokenType::Less
        }) {
            expr = Box::new(Bin {
                left: expr,
                operator: op.clone(),
                right: self.term(),
            })
        }
        expr
    }

    fn term(&mut self) -> Box<dyn Expr> {
        let mut expr: Box<dyn Expr> = self.factor();
        while let Some(op) = self
            .tokens
            .advance_if(|t| t.token_type == TokenType::Plus || t.token_type == TokenType::Minus)
        {
            expr = Box::new(Bin {
                left: expr,
                operator: op.clone(),
                right: self.factor(),
            })
        }
        expr
    }

    fn factor(&mut self) -> Box<dyn Expr> {
        let mut expr: Box<dyn Expr> = self.unary();
        while let Some(op) = self
            .tokens
            .advance_if(|t| t.token_type == TokenType::Slash || t.token_type == TokenType::Star)
        {
            expr = Box::new(Bin {
                left: expr,
                operator: op.clone(),
                right: self.unary(),
            });
        }
        expr
    }

    fn unary(&mut self) -> Box<dyn Expr> {
        if let Some(op) = self
            .tokens
            .advance_if(|t| t.token_type == TokenType::Bang || t.token_type == TokenType::Minus)
        {
            let expr = Box::new(Un {
                operator: op.clone(),
                right: self.unary(),
            });
            return expr;
        };

        self.primary()
    }

    fn primary(&mut self) -> Box<dyn Expr> {
        let mut expr: Box<dyn Expr> = Box::new(Lit { value: None });
        if let Some(t) = self.tokens.advance_if(|t| {
            t.token_type == TokenType::True
                || t.token_type == TokenType::Nil
                || t.token_type == TokenType::String
                || t.token_type == TokenType::Number
                || t.token_type == TokenType::LeftParen
        }) {
            match &t.token_type {
                TokenType::True => {
                    expr = Box::new(Lit {
                        value: Some(Box::new(true)),
                    });
                }
                TokenType::Nil => {
                    expr = Box::new(Lit { value: None });
                }
                TokenType::String => {
                    expr = Box::new(Lit {
                        value: Some(Box::new(t.literal.clone().unwrap().as_string())),
                    });
                }
                TokenType::Number => {
                    expr = Box::new(Lit {
                        value: Some(Box::new(t.literal.clone().unwrap().as_number())),
                    });
                }
                TokenType::LeftParen => {
                    expr = Box::new(Grp {
                        expression: self.expression(),
                    });
                    // Ensure there is a closing paren and consume it
                    if self
                        .tokens
                        .advance_if(|t| t.token_type == TokenType::RightParen)
                        .is_none()
                    {
                        panic!("Invalid token to start an expression.")
                    };
                }
                _ => {}
            }
        } else {
            panic!("Invalid token to start an expression.")
        };

        expr
    }
}
