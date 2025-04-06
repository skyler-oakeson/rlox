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
        let matches = vec![
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::LessEqual,
            TokenType::Less,
        ];

        while let Some(op) = self.tokens.advance_if(|t| matches.contains(&t.token_type)) {
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
        let matches = vec![TokenType::Plus, TokenType::Minus];
        while let Some(op) = self.tokens.advance_if(|t| matches.contains(&t.token_type)) {
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
        let matches = vec![TokenType::Slash, TokenType::Star];
        while let Some(op) = self.tokens.advance_if(|t| matches.contains(&t.token_type)) {
            expr = Box::new(Bin {
                left: expr,
                operator: op.clone(),
                right: self.unary(),
            });
        }
        expr
    }

    fn unary(&mut self) -> Box<dyn Expr> {
        let matches = vec![TokenType::Bang, TokenType::Minus];
        if let Some(op) = self.tokens.advance_if(|t| matches.contains(&t.token_type)) {
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
