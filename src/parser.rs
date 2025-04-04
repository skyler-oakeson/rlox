use crate::expression::{Bin, Expr, Grp, Lit, Un};
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

pub struct Marcher<T> {
    values: Vec<T>,
    curr: i32,
}

impl<T> Marcher<T>
where
    T: PartialEq,
{
    fn new(values: Vec<T>) -> Self {
        Marcher { values, curr: -1 }
    }

    fn advance(&mut self, ahead: u32) -> Option<&T> {
        let t = self.values.get((self.curr + (ahead) as i32) as usize);
        if t.is_some() {
            self.curr += ahead as i32
        }
        t
    }

    fn previous(&self, behind: u32) -> Option<&T> {
        self.values.get((self.curr - behind as i32) as usize)
    }

    fn peek(&self, ahead: u32) -> Option<&T> {
        self.values.get((self.curr + ahead as i32) as usize)
    }

    fn advance_if(&mut self, predicate: impl FnOnce(&T) -> bool) -> bool {
        let t = match self.peek(1) {
            Some(val) => val,
            None => return false,
        };

        let res = predicate(t);
        res.then(|| self.advance(1));
        res
    }
}

struct Parser {
    tokens: Marcher<Token>,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens: Marcher::new(tokens),
        }
    }

    fn expression(&mut self) -> Box<dyn Expr> {
        return self.equality();
    }

    fn equality(&mut self) -> Box<dyn Expr> {
        let mut expr: Box<dyn Expr> = self.comparison();
        while self.tokens.advance_if(|t| {
            t.token_type == TokenType::BangEqual || t.token_type == TokenType::EqualEqual
        }) {
            let op = self.tokens.previous(1);
            expr = Box::new(Bin {
                left: expr,
                operator: op.unwrap().clone(),
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

        while self.tokens.advance_if(|t| matches.contains(&t.token_type)) {
            let op = self.tokens.previous(1);
            expr = Box::new(Bin {
                left: expr,
                operator: op.unwrap().clone(),
                right: self.term(),
            })
        }
        expr
    }

    fn term(&mut self) -> Box<dyn Expr> {
        let mut expr: Box<dyn Expr> = self.factor();
        let matches = vec![TokenType::Plus, TokenType::Minus];
        while self.tokens.advance_if(|t| matches.contains(&t.token_type)) {
            let op = self.tokens.previous(1);
            expr = Box::new(Bin {
                left: expr,
                operator: op.unwrap().clone(),
                right: self.factor(),
            })
        }
        expr
    }

    fn factor(&mut self) -> Box<dyn Expr> {
        let mut expr: Box<dyn Expr> = self.unary();
        let matches = vec![TokenType::Slash, TokenType::Star];
        while self.tokens.advance_if(|t| matches.contains(&t.token_type)) {
            let op = self.tokens.previous(1);
            expr = Box::new(Bin {
                left: expr,
                operator: op.unwrap().clone(),
                right: self.unary(),
            });
        }
        expr
    }

    fn unary(&mut self) -> Box<dyn Expr> {
        let matches = vec![TokenType::Bang, TokenType::Minus];
        if self.tokens.advance_if(|t| matches.contains(&t.token_type)) {
            let op = self.tokens.previous(1);
            let expr = Box::new(Un {
                operator: op.unwrap().clone(),
                right: self.unary(),
            });
            return expr;
        };

        self.primary()
    }

    fn primary(&mut self) -> Box<dyn Expr> {
        let mut expr = Box::new(Lit { value: None });

        if self.tokens.advance_if(|t| t.token_type == TokenType::False) {
            return Box::new(Lit {
                value: Some(Box::new(false)),
            });
        }

        if self.tokens.advance_if(|t| t.token_type == TokenType::True) {
            return Box::new(Lit {
                value: Some(Box::new(true)),
            });
        }

        if self.tokens.advance_if(|t| t.token_type == TokenType::Nil) {
            return Box::new(Lit { value: None });
        }

        self.tokens.advance_if(|t| {
            let mut res = false;
            if t.token_type == TokenType::String {
                expr = Box::new(Lit {
                    value: Some(Box::new(t.literal.clone().unwrap().as_string())),
                });
                res = true;
            };

            if t.token_type == TokenType::Number {
                expr = Box::new(Lit {
                    value: Some(Box::new(t.literal.clone().unwrap().as_number())),
                });
                res = true;
            };
            res
        });

        if self
            .tokens
            .advance_if(|t| t.token_type == TokenType::LeftParen)
        {
            return Box::new(Grp {
                expression: self.expression(),
            });
        };

        expr
    }
}
