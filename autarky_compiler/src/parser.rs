// src/parser.rs

use crate::ast::{Expr, Type};
use std::iter::Peekable;
use std::vec::IntoIter;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Ident(String), Int(i64),
    Lambda, Colon, Dot, Plus, Minus, LParen, RParen,
    Arrow, EqualGreater, Pipe, Comma,
    IntKw, PairKw, EitherKw, ArrayKw,
    MkPair, Unpack, IntoKw, InKw,
    Left, Right, Match, With,
    ArrayAlloc, ArraySwap,
}

pub struct Parser<'a> {
    input: &'a str,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Parser { input }
    }

    fn lex(&self) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut chars = self.input.chars().peekable();

        while let Some(&c) = chars.peek() {
            match c {
                ' ' | '\n' | '\t' | '\r' => { chars.next(); }
                '\\' => { tokens.push(Token::Lambda); chars.next(); }
                ':' => { tokens.push(Token::Colon); chars.next(); }
                '.' => { tokens.push(Token::Dot); chars.next(); }
                '+' => { tokens.push(Token::Plus); chars.next(); }
                '-' => { 
                    chars.next();
                    if let Some(&'>') = chars.peek() {
                        tokens.push(Token::Arrow); chars.next();
                    } else {
                        tokens.push(Token::Minus);
                    }
                }
                '(' => { tokens.push(Token::LParen); chars.next(); }
                ')' => { tokens.push(Token::RParen); chars.next(); }
                '=' => {
                    chars.next();
                    if let Some(&'>') = chars.peek() {
                        tokens.push(Token::EqualGreater); chars.next();
                    }
                }
                '|' => { tokens.push(Token::Pipe); chars.next(); }
                ',' => { tokens.push(Token::Comma); chars.next(); }
                _ if c.is_ascii_digit() => {
                    let mut num = String::new();
                    while let Some(&d) = chars.peek() {
                        if d.is_ascii_digit() { num.push(chars.next().unwrap()); } else { break; }
                    }
                    tokens.push(Token::Int(num.parse().unwrap()));
                }
                _ if c.is_alphabetic() || c == '_' => {
                    let mut ident = String::new();
                    while let Some(&ch) = chars.peek() {
                        if ch.is_alphanumeric() || ch == '_' { ident.push(chars.next().unwrap()); } else { break; }
                    }
                    match ident.as_str() {
                        "Int" => tokens.push(Token::IntKw),
                        "Pair" => tokens.push(Token::PairKw),
                        "Either" => tokens.push(Token::EitherKw),
                        "Array" => tokens.push(Token::ArrayKw),
                        "mkpair" => tokens.push(Token::MkPair),
                        "unpack" => tokens.push(Token::Unpack),
                        "into" => tokens.push(Token::IntoKw),
                        "in" => tokens.push(Token::InKw),
                        "Left" => tokens.push(Token::Left),
                        "Right" => tokens.push(Token::Right),
                        "match" => tokens.push(Token::Match),
                        "with" => tokens.push(Token::With),
                        "array_alloc" => tokens.push(Token::ArrayAlloc),
                        "array_swap" => tokens.push(Token::ArraySwap),
                        _ => tokens.push(Token::Ident(ident)),
                    }
                }
                _ => { chars.next(); } // Skip unknown
            }
        }
        tokens
    }

    pub fn parse(&mut self) -> Result<Expr, String> {
        let tokens = self.lex();
        if tokens.is_empty() { return Err("Empty file".to_string()); }
        let mut iter = tokens.into_iter().peekable();
        self.parse_expr(&mut iter)
    }

    fn parse_type(&self, iter: &mut Peekable<IntoIter<Token>>) -> Result<Type, String> {
        match iter.next() {
            Some(Token::IntKw) => Ok(Type::Int),
            Some(Token::PairKw) => Ok(Type::Pair(Box::new(self.parse_type(iter)?), Box::new(self.parse_type(iter)?))),
            Some(Token::EitherKw) => Ok(Type::Either(Box::new(self.parse_type(iter)?), Box::new(self.parse_type(iter)?))),
            Some(Token::ArrayKw) => Ok(Type::Array(Box::new(self.parse_type(iter)?))),
            Some(Token::LParen) => {
                let t1 = self.parse_type(iter)?;
                if let Some(Token::Arrow) = iter.next() {
                    let t2 = self.parse_type(iter)?;
                    if let Some(Token::RParen) = iter.next() {
                        return Ok(Type::Func(Box::new(t1), Box::new(t2)));
                    }
                }
                Err("Expected function type format: (Type -> Type)".to_string())
            }
            _ => Err("Invalid Type".to_string())
        }
    }

    fn parse_expr(&self, iter: &mut Peekable<IntoIter<Token>>) -> Result<Expr, String> {
        match iter.peek() {
            Some(Token::Lambda) => {
                iter.next(); // Consume '\'
                let param = match iter.next() { Some(Token::Ident(name)) => name, _ => return Err("Expected identifier".to_string()) };
                if iter.next() != Some(Token::Colon) { return Err("Expected ':'".to_string()); }
                let param_type = self.parse_type(iter)?;
                if iter.next() != Some(Token::Dot) { return Err("Expected '.'".to_string()); }
                let body = self.parse_expr(iter)?;
                Ok(Expr::Lambda { param, param_type, body: Box::new(body) })
            }
            Some(Token::Unpack) => {
                iter.next(); // Consume unpack
                let pair = self.parse_expr(iter)?;
                if iter.next() != Some(Token::IntoKw) { return Err("Expected 'into'".to_string()); }
                let var1 = match iter.next() { Some(Token::Ident(n)) => n, _ => return Err("Expected var1".to_string()) };
                if iter.next() != Some(Token::Comma) { return Err("Expected ','".to_string()); }
                let var2 = match iter.next() { Some(Token::Ident(n)) => n, _ => return Err("Expected var2".to_string()) };
                if iter.next() != Some(Token::InKw) { return Err("Expected 'in'".to_string()); }
                let body = self.parse_expr(iter)?;
                Ok(Expr::Unpack { pair: Box::new(pair), var1, var2, body: Box::new(body) })
            }
            Some(Token::Match) => {
                iter.next();
                let expr = self.parse_expr(iter)?;
                if iter.next() != Some(Token::With) { return Err("Expected 'with'".to_string()); }
                if iter.next() != Some(Token::Left) { return Err("Expected 'Left'".to_string()); }
                let left_var = match iter.next() { Some(Token::Ident(n)) => n, _ => return Err("Expected left var".to_string()) };
                if iter.next() != Some(Token::EqualGreater) { return Err("Expected '=>'".to_string()); }
                let left_body = self.parse_expr(iter)?;
                if iter.next() != Some(Token::Pipe) { return Err("Expected '|'".to_string()); }
                if iter.next() != Some(Token::Right) { return Err("Expected 'Right'".to_string()); }
                let right_var = match iter.next() { Some(Token::Ident(n)) => n, _ => return Err("Expected right var".to_string()) };
                if iter.next() != Some(Token::EqualGreater) { return Err("Expected '=>'".to_string()); }
                let right_body = self.parse_expr(iter)?;
                Ok(Expr::Match { expr: Box::new(expr), left_var, left_body: Box::new(left_body), right_var, right_body: Box::new(right_body) })
            }
            _ => self.parse_add_sub(iter)
        }
    }

    fn parse_add_sub(&self, iter: &mut Peekable<IntoIter<Token>>) -> Result<Expr, String> {
        let mut left = self.parse_app(iter)?;
        while let Some(tok) = iter.peek() {
            match tok {
                Token::Plus => { iter.next(); left = Expr::Add(Box::new(left), Box::new(self.parse_app(iter)?)); }
                Token::Minus => { iter.next(); left = Expr::Sub(Box::new(left), Box::new(self.parse_app(iter)?)); }
                _ => break,
            }
        }
        Ok(left)
    }

    fn parse_app(&self, iter: &mut Peekable<IntoIter<Token>>) -> Result<Expr, String> {
        let mut left = self.parse_primary(iter)?;
        // If the next token is a primary starter, it's an application
        while let Some(tok) = iter.peek() {
            match tok {
                Token::Int(_) | Token::Ident(_) | Token::LParen | Token::MkPair | Token::Left | Token::Right | Token::ArrayAlloc | Token::ArraySwap => {
                    let right = self.parse_primary(iter)?;
                    left = Expr::App { func: Box::new(left), arg: Box::new(right) };
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn parse_primary(&self, iter: &mut Peekable<IntoIter<Token>>) -> Result<Expr, String> {
        match iter.next() {
            Some(Token::Int(n)) => Ok(Expr::IntLiteral(n)),
            Some(Token::Ident(name)) => Ok(Expr::Variable(name)),
            Some(Token::LParen) => {
                let expr = self.parse_expr(iter)?;
                if iter.next() != Some(Token::RParen) { return Err("Expected ')'".to_string()); }
                Ok(expr)
            }
            Some(Token::MkPair) => Ok(Expr::MkPair(Box::new(self.parse_primary(iter)?), Box::new(self.parse_primary(iter)?))),
            Some(Token::Left) => {
                let ty = self.parse_type(iter)?;
                Ok(Expr::Left(Box::new(self.parse_primary(iter)?), ty))
            }
            Some(Token::Right) => {
                let ty = self.parse_type(iter)?;
                Ok(Expr::Right(Box::new(self.parse_primary(iter)?), ty))
            }
            Some(Token::ArrayAlloc) => Ok(Expr::ArrayAlloc { size: Box::new(self.parse_primary(iter)?), init_val: Box::new(self.parse_primary(iter)?) }),
            Some(Token::ArraySwap) => Ok(Expr::ArraySwap { 
                array: Box::new(self.parse_primary(iter)?), 
                index: Box::new(self.parse_primary(iter)?), 
                new_val: Box::new(self.parse_primary(iter)?) 
            }),
            _ => Err("Unexpected token in primary expression".to_string()),
        }
    }
}