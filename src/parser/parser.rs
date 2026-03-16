use crate::lexer::token::Token;
use crate::parser::ast::*;
use crate::parser::ast::InterpolationPart;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "parse error: {}", self.message)
    }
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut stmts = Vec::new();
        while !self.is_at_end() {
            self.skip_semicolons();
            if self.is_at_end() {
                break;
            }
            stmts.push(self.parse_statement()?);
        }
        Ok(stmts)
    }

    fn parse_statement(&mut self) -> Result<Stmt, ParseError> {
        match self.peek() {
            Token::Let => self.parse_let(),
            Token::If => self.parse_if(),
            Token::While => self.parse_while(),
            Token::Repeat => self.parse_repeat(),
            Token::Try => self.parse_try_catch(),
            Token::Return => self.parse_return(),
            Token::Fail => self.parse_fail(),
            Token::Load => self.parse_load(),
            Token::Identifier(_) if self.is_assignment() => self.parse_assignment(),
            _ => self.parse_expr_stmt(),
        }
    }

    fn parse_let(&mut self) -> Result<Stmt, ParseError> {
        self.expect(Token::Let)?;

        let mut is_dyn = false;
        let mut is_pub = false;

        // Parse modifiers: dyn, pub (in any order)
        loop {
            match self.peek() {
                Token::Dyn => { self.advance(); is_dyn = true; }
                Token::Pub => { self.advance(); is_pub = true; }
                _ => break,
            }
        }

        let name = self.expect_identifier()?;

        // Check if this is a function declaration: let name (params) { ... }
        if self.check(&Token::LeftParen) {
            return self.parse_func_decl(name, is_dyn, is_pub);
        }

        // Variable declaration: let name = value  or  let name: Type = value
        let type_annotation = if self.check(&Token::Colon) {
            self.advance();
            Some(self.expect_identifier()?)
        } else {
            None
        };

        self.expect(Token::Assign)?;
        let value = self.parse_expression()?;
        self.consume_semicolon();

        Ok(Stmt::Let {
            name,
            is_dyn,
            is_pub,
            type_annotation,
            value,
        })
    }

    fn parse_func_decl(
        &mut self,
        name: String,
        is_dyn: bool,
        is_pub: bool,
    ) -> Result<Stmt, ParseError> {
        self.expect(Token::LeftParen)?;
        let params = self.parse_params()?;
        self.expect(Token::RightParen)?;
        self.expect(Token::LeftBrace)?;
        let body = self.parse_block()?;
        self.expect(Token::RightBrace)?;
        self.consume_semicolon();

        Ok(Stmt::FuncDecl {
            name,
            is_dyn,
            is_pub,
            params,
            body,
        })
    }

    fn parse_params(&mut self) -> Result<Vec<Param>, ParseError> {
        let mut params = Vec::new();
        if self.check(&Token::RightParen) {
            return Ok(params);
        }

        loop {
            let name = self.expect_identifier()?;
            let type_annotation = if self.check(&Token::Colon) {
                self.advance();
                Some(self.expect_identifier()?)
            } else {
                None
            };
            params.push(Param { name, type_annotation });

            if !self.check(&Token::Comma) {
                break;
            }
            self.advance(); // consume comma
        }

        Ok(params)
    }

    fn parse_if(&mut self) -> Result<Stmt, ParseError> {
        self.expect(Token::If)?;
        self.expect(Token::LeftParen)?;
        let condition = self.parse_expression()?;
        self.expect(Token::RightParen)?;
        self.expect(Token::LeftBrace)?;
        let then_branch = self.parse_block()?;
        self.expect(Token::RightBrace)?;

        let else_branch = if self.check(&Token::Else) {
            self.advance();
            self.expect(Token::LeftBrace)?;
            let block = self.parse_block()?;
            self.expect(Token::RightBrace)?;
            Some(block)
        } else {
            None
        };

        self.consume_semicolon();
        Ok(Stmt::If { condition, then_branch, else_branch })
    }

    fn parse_while(&mut self) -> Result<Stmt, ParseError> {
        self.expect(Token::While)?;
        self.expect(Token::LeftParen)?;
        let condition = self.parse_expression()?;
        self.expect(Token::RightParen)?;
        self.expect(Token::LeftBrace)?;
        let body = self.parse_block()?;
        self.expect(Token::RightBrace)?;
        self.consume_semicolon();

        Ok(Stmt::While { condition, body })
    }

    fn parse_repeat(&mut self) -> Result<Stmt, ParseError> {
        self.expect(Token::Repeat)?;
        self.expect(Token::LeftBrace)?;
        let body = self.parse_block()?;
        self.expect(Token::RightBrace)?;

        if self.check(&Token::While) {
            self.advance();
            self.expect(Token::LeftParen)?;
            let condition = self.parse_expression()?;
            self.expect(Token::RightParen)?;
            self.consume_semicolon();
            Ok(Stmt::RepeatWhile { body, condition })
        } else if self.check(&Token::For) {
            self.advance();
            self.expect(Token::LeftParen)?;
            let var_name = self.expect_identifier()?;
            self.expect(Token::In)?;
            let iterable = self.parse_expression()?;
            self.expect(Token::RightParen)?;
            self.consume_semicolon();
            Ok(Stmt::RepeatFor { var_name, iterable, body })
        } else {
            Err(ParseError {
                message: "expected 'while' or 'for' after repeat block".to_string(),
            })
        }
    }

    fn parse_try_catch(&mut self) -> Result<Stmt, ParseError> {
        self.expect(Token::Try)?;
        self.expect(Token::LeftBrace)?;
        let try_body = self.parse_block()?;
        self.expect(Token::RightBrace)?;
        self.expect(Token::Catch)?;
        self.expect(Token::LeftParen)?;
        let error_name = self.expect_identifier()?;
        self.expect(Token::RightParen)?;
        self.expect(Token::LeftBrace)?;
        let catch_body = self.parse_block()?;
        self.expect(Token::RightBrace)?;
        self.consume_semicolon();

        Ok(Stmt::TryCatch { try_body, error_name, catch_body })
    }

    fn parse_return(&mut self) -> Result<Stmt, ParseError> {
        self.expect(Token::Return)?;
        let value = self.parse_expression()?;
        self.consume_semicolon();
        Ok(Stmt::Return(value))
    }

    fn parse_fail(&mut self) -> Result<Stmt, ParseError> {
        self.expect(Token::Fail)?;
        let value = self.parse_expression()?;
        self.consume_semicolon();
        Ok(Stmt::Fail(value))
    }

    fn parse_load(&mut self) -> Result<Stmt, ParseError> {
        self.expect(Token::Load)?;

        let items = if self.check(&Token::Star) {
            self.advance();
            vec!["*".to_string()]
        } else {
            let mut items = vec![self.expect_identifier()?];
            while self.check(&Token::Comma) {
                self.advance();
                items.push(self.expect_identifier()?);
            }
            items
        };

        self.expect(Token::From)?;
        let path = self.expect_string()?;
        self.consume_semicolon();

        Ok(Stmt::Load { items, path })
    }

    fn parse_assignment(&mut self) -> Result<Stmt, ParseError> {
        let name = self.expect_identifier()?;
        self.expect(Token::Assign)?;
        let value = self.parse_expression()?;
        self.consume_semicolon();
        Ok(Stmt::Assign { name, value })
    }

    fn parse_expr_stmt(&mut self) -> Result<Stmt, ParseError> {
        let expr = self.parse_expression()?;
        self.consume_semicolon();
        Ok(Stmt::ExprStmt(expr))
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut stmts = Vec::new();
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            self.skip_semicolons();
            if self.check(&Token::RightBrace) {
                break;
            }
            stmts.push(self.parse_statement()?);
        }
        Ok(stmts)
    }

    // ── Expression parsing (precedence climbing) ──

    fn parse_expression(&mut self) -> Result<Expr, ParseError> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_and()?;
        while self.check(&Token::PipePipe) || self.check(&Token::Or) {
            self.advance();
            let right = self.parse_and()?;
            left = Expr::Binary {
                left: Box::new(left),
                op: BinOp::Or,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_equality()?;
        while self.check(&Token::AmpAmp) || self.check(&Token::And) {
            self.advance();
            let right = self.parse_equality()?;
            left = Expr::Binary {
                left: Box::new(left),
                op: BinOp::And,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_comparison()?;
        loop {
            if self.check(&Token::EqualEqual) || self.check(&Token::Is) {
                self.advance();
                let right = self.parse_comparison()?;
                left = Expr::Binary {
                    left: Box::new(left),
                    op: BinOp::EqualEqual,
                    right: Box::new(right),
                };
            } else if self.check(&Token::BangEqual) {
                self.advance();
                let right = self.parse_comparison()?;
                left = Expr::Binary {
                    left: Box::new(left),
                    op: BinOp::NotEqual,
                    right: Box::new(right),
                };
            } else if self.check_is_not() {
                self.advance(); // is
                self.advance(); // not
                let right = self.parse_comparison()?;
                left = Expr::Binary {
                    left: Box::new(left),
                    op: BinOp::NotEqual,
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_addition()?;
        loop {
            let op = if self.check(&Token::GreaterThan) {
                self.advance();
                Some(BinOp::Greater)
            } else if self.check(&Token::LessThan) {
                self.advance();
                Some(BinOp::Less)
            } else if self.check(&Token::GreaterEqual) {
                self.advance();
                Some(BinOp::GreaterEqual)
            } else if self.check(&Token::LessEqual) {
                self.advance();
                Some(BinOp::LessEqual)
            } else if self.check_greater_than() {
                self.advance(); // greater
                self.advance(); // than
                Some(BinOp::Greater)
            } else if self.check_lesser_than() {
                self.advance(); // lesser
                self.advance(); // than
                Some(BinOp::Less)
            } else if self.check_greater_or_equal() {
                self.advance(); // greater
                self.advance(); // or
                self.advance(); // equal
                Some(BinOp::GreaterEqual)
            } else if self.check_lesser_or_equal() {
                self.advance(); // lesser
                self.advance(); // or
                self.advance(); // equal
                Some(BinOp::LessEqual)
            } else {
                None
            };

            if let Some(op) = op {
                let right = self.parse_addition()?;
                left = Expr::Binary {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_addition(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_multiplication()?;
        loop {
            let op = if self.check(&Token::Plus) {
                self.advance();
                Some(BinOp::Add)
            } else if self.check(&Token::Minus) {
                self.advance();
                Some(BinOp::Sub)
            } else {
                None
            };

            if let Some(op) = op {
                let right = self.parse_multiplication()?;
                left = Expr::Binary {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_multiplication(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_unary()?;
        loop {
            let op = if self.check(&Token::Star) {
                self.advance();
                Some(BinOp::Mul)
            } else if self.check(&Token::Slash) {
                self.advance();
                Some(BinOp::Div)
            } else if self.check(&Token::Percent) {
                self.advance();
                Some(BinOp::Mod)
            } else {
                None
            };

            if let Some(op) = op {
                let right = self.parse_unary()?;
                left = Expr::Binary {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        if self.check(&Token::Minus) {
            self.advance();
            let expr = self.parse_unary()?;
            return Ok(Expr::Unary {
                op: UnaryOp::Negate,
                expr: Box::new(expr),
            });
        }
        if self.check(&Token::Bang) || self.check(&Token::Not) {
            self.advance();
            let expr = self.parse_unary()?;
            return Ok(Expr::Unary {
                op: UnaryOp::Not,
                expr: Box::new(expr),
            });
        }
        self.parse_call()
    }

    fn parse_call(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_primary()?;

        loop {
            if self.check(&Token::LeftParen) {
                self.advance();
                let args = self.parse_args()?;
                self.expect(Token::RightParen)?;
                expr = Expr::Call {
                    callee: Box::new(expr),
                    args,
                };
            } else if self.check(&Token::Dot) {
                self.advance();
                let field = self.expect_identifier()?;
                if self.check(&Token::LeftParen) {
                    self.advance();
                    let args = self.parse_args()?;
                    self.expect(Token::RightParen)?;
                    expr = Expr::MethodCall {
                        object: Box::new(expr),
                        method: field,
                        args,
                    };
                } else {
                    expr = Expr::FieldAccess {
                        object: Box::new(expr),
                        field,
                    };
                }
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_args(&mut self) -> Result<Vec<Expr>, ParseError> {
        let mut args = Vec::new();
        if self.check(&Token::RightParen) {
            return Ok(args);
        }

        loop {
            args.push(self.parse_expression()?);
            if !self.check(&Token::Comma) {
                break;
            }
            self.advance();
        }

        Ok(args)
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        let token = self.peek().clone();
        match token {
            Token::NumberLit(n, is_float) => {
                self.advance();
                Ok(Expr::NumberLit(n, is_float))
            }
            Token::TextLit(s) => {
                self.advance();
                Ok(Expr::TextLit(s))
            }
            Token::InterpolatedText(parts) => {
                self.advance();
                self.parse_interpolated(parts)
            }
            Token::BoolLit(b) => {
                self.advance();
                Ok(Expr::BoolLit(b))
            }
            Token::Identifier(name) => {
                self.advance();
                Ok(Expr::Identifier(name))
            }
            Token::LeftParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(Token::RightParen)?;
                Ok(expr)
            }
            Token::LeftBracket => {
                self.advance();
                let mut elements = Vec::new();
                if !self.check(&Token::RightBracket) {
                    loop {
                        elements.push(self.parse_expression()?);
                        if !self.check(&Token::Comma) {
                            break;
                        }
                        self.advance();
                    }
                }
                self.expect(Token::RightBracket)?;
                Ok(Expr::ListLit(elements))
            }
            _ => Err(ParseError {
                message: format!("unexpected token {:?}", token),
            }),
        }
    }

    // ── Helpers ──

    fn peek(&self) -> &Token {
        self.tokens.get(self.current).unwrap_or(&Token::Eof)
    }

    fn peek_at(&self, offset: usize) -> &Token {
        self.tokens.get(self.current + offset).unwrap_or(&Token::Eof)
    }

    fn advance(&mut self) -> &Token {
        let token = &self.tokens[self.current];
        self.current += 1;
        token
    }

    fn check(&self, expected: &Token) -> bool {
        std::mem::discriminant(self.peek()) == std::mem::discriminant(expected)
    }

    fn check_is_not(&self) -> bool {
        matches!(self.peek(), Token::Is) && matches!(self.peek_at(1), Token::Not)
    }

    fn check_greater_than(&self) -> bool {
        matches!(self.peek(), Token::Greater) && matches!(self.peek_at(1), Token::Than)
    }

    fn check_lesser_than(&self) -> bool {
        matches!(self.peek(), Token::Lesser) && matches!(self.peek_at(1), Token::Than)
    }

    fn check_greater_or_equal(&self) -> bool {
        matches!(self.peek(), Token::Greater)
            && matches!(self.peek_at(1), Token::Or)
            && matches!(self.peek_at(2), Token::Equal)
    }

    fn check_lesser_or_equal(&self) -> bool {
        matches!(self.peek(), Token::Lesser)
            && matches!(self.peek_at(1), Token::Or)
            && matches!(self.peek_at(2), Token::Equal)
    }

    fn is_assignment(&self) -> bool {
        matches!(self.peek(), Token::Identifier(_)) && matches!(self.peek_at(1), Token::Assign)
    }

    fn expect(&mut self, expected: Token) -> Result<(), ParseError> {
        if self.check(&expected) {
            self.advance();
            Ok(())
        } else {
            Err(ParseError {
                message: format!("expected {:?}, got {:?}", expected, self.peek()),
            })
        }
    }

    fn expect_identifier(&mut self) -> Result<String, ParseError> {
        match self.peek().clone() {
            Token::Identifier(name) => {
                self.advance();
                Ok(name)
            }
            other => Err(ParseError {
                message: format!("expected identifier, got {:?}", other),
            }),
        }
    }

    fn expect_string(&mut self) -> Result<String, ParseError> {
        match self.peek().clone() {
            Token::TextLit(s) => {
                self.advance();
                Ok(s)
            }
            other => Err(ParseError {
                message: format!("expected string, got {:?}", other),
            }),
        }
    }

    fn consume_semicolon(&mut self) {
        if self.check(&Token::Semicolon) {
            self.advance();
        }
    }

    fn skip_semicolons(&mut self) {
        while self.check(&Token::Semicolon) {
            self.advance();
        }
    }

    fn parse_interpolated(&mut self, parts: Vec<(String, String)>) -> Result<Expr, ParseError> {
        let mut iparts = Vec::new();
        for (text, expr_src) in parts {
            if !text.is_empty() {
                iparts.push(InterpolationPart::Text(text));
            }
            if !expr_src.is_empty() {
                // Parse the expression source as a mini program
                let mut scanner = crate::lexer::scanner::Scanner::new(&expr_src);
                let tokens = scanner.scan_tokens().map_err(|e| ParseError {
                    message: format!("in interpolation: {}", e),
                })?;
                let mut parser = Parser::new(tokens);
                let expr = parser.parse_expression()?;
                iparts.push(InterpolationPart::Expr(expr));
            }
        }
        Ok(Expr::Interpolation(iparts))
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek(), Token::Eof)
    }
}
