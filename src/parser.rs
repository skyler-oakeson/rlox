use crate::expression::{Bin, Expr, Grp, Lit, Un};
use crate::marcher::Marcher;
use crate::token::{Literal, Token, TokenType};

/*                    Grammer for lox
 * --------------------------------------------------------
 * expression -> equality;
 * equality   -> comparison ( ("=" | "!=") comparison )*;
 * comparison -> term ( (">" | ">=" | "<" | "<=") term )*;
 * term       -> factor ( ("*" | "/") factor)*;
 * factor     -> unary ( ("+" | "-") unary)*;
 * unary      -> ("!" | "-") unary;
 *             | primary
 * primary    -> NUMBER | STRING | "true" | "false" | "nil"
 *             | "(" expression ")";
 */

pub struct Parser {
    tokens: Marcher<Token>,
}

pub fn parse(tokens: Vec<Token>) -> Box<dyn Expr> {
    let mut parser = Parser::new(tokens);
    parser.expression()
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens: Marcher::new(tokens),
        }
    }

    fn expression(&mut self) -> Box<dyn Expr> {
        return self.equality();
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
        let mut lit = Box::new(Lit { value: None });

        if let Some(_tok) = self.tokens.advance_if(|t| t.token_type == TokenType::False) {
            lit.value = Some(Box::new(false));
            return lit;
        }

        if let Some(_tok) = self.tokens.advance_if(|t| t.token_type == TokenType::True) {
            lit.value = Some(Box::new(true));
            return lit;
        }

        if let Some(_tok) = self.tokens.advance_if(|t| t.token_type == TokenType::Nil) {
            lit.value = None;
            return lit;
        }

        if let Some(tok) = self
            .tokens
            .advance_if(|t| t.token_type == TokenType::String)
        {
            lit.value = Some(Box::new(tok.literal.clone().unwrap().as_string()));
            return lit;
        }

        if let Some(tok) = self
            .tokens
            .advance_if(|t| t.token_type == TokenType::Number)
        {
            lit.value = Some(Box::new(tok.literal.clone().unwrap().as_number()));
            return lit;
        }

        if let Some(_tok) = self
            .tokens
            .advance_if(|t| t.token_type == TokenType::LeftParen)
        {
            let grp = Box::new(Grp {
                expression: self.expression(),
            });
            return grp;
        };

        lit
    }
}
