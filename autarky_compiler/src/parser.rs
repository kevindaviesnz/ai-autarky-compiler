use crate::ast::{Term, Type, Permission};
use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Ident(String),
    Number(u32),
    Lambda,   
    Colon,    
    Dot,      
    LParen,   
    RParen,   
    Bang,
    Plus,     
    Lin,      
    Pi,       
    IntKw,
    UnitKw,    
    UnitValKw, 
    BoolKw,    // NEW
    TrueKw,    // NEW
    FalseKw,   // NEW
    IfKw,      // NEW
    ThenKw,    // NEW
    ElseKw,    // NEW
    FreeKw,    
    TypeUniv(u32), 
    SplitKw,
    IntoKw,
    InKw,
    MergeKw,
    AndKw,
    Comma,
    Eof,
}

pub struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { chars: input.chars().peekable() }
    }

    fn consume_whitespace(&mut self) {
        while let Some(&c) = self.chars.peek() {
            if c.is_whitespace() {
                self.chars.next();
            } else {
                break;
            }
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.consume_whitespace();

        match self.chars.next() {
            Some('\\') => Token::Lambda,
            Some(':') => Token::Colon,
            Some('.') => Token::Dot,
            Some('(') => Token::LParen,
            Some(')') => Token::RParen,
            Some('!') => Token::Bang,
            Some(',') => Token::Comma,
            Some('+') => Token::Plus, 
            Some(c) if c.is_alphabetic() || c == '_' => {
                let mut ident = String::from(c);
                while let Some(&next_c) = self.chars.peek() {
                    if next_c.is_alphanumeric() || next_c == '_' {
                        ident.push(self.chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                match ident.as_str() {
                    "Lin" => Token::Lin,
                    "Pi" => Token::Pi,
                    "Int" => Token::IntKw, 
                    "Unit" => Token::UnitKw, 
                    "unit" => Token::UnitValKw, 
                    "Bool" => Token::BoolKw,     // NEW
                    "True" => Token::TrueKw,     // NEW
                    "False" => Token::FalseKw,   // NEW
                    "if" => Token::IfKw,         // NEW
                    "then" => Token::ThenKw,     // NEW
                    "else" => Token::ElseKw,     // NEW
                    "free" => Token::FreeKw, 
                    "split" => Token::SplitKw,
                    "into" => Token::IntoKw,
                    "in" => Token::InKw,
                    "merge" => Token::MergeKw,
                    "and" => Token::AndKw,
                    s if s.starts_with("Type_") => {
                        let num_str = &s[5..];
                        if let Ok(n) = num_str.parse::<u32>() {
                            Token::TypeUniv(n)
                        } else {
                            Token::Ident(ident)
                        }
                    }
                    _ => Token::Ident(ident),
                }
            }
            Some(c) if c.is_ascii_digit() => {
                let mut num_str = String::from(c);
                while let Some(&next_c) = self.chars.peek() {
                    if next_c.is_ascii_digit() {
                        num_str.push(self.chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                Token::Number(num_str.parse().unwrap())
            }
            None => Token::Eof,
            Some(c) => panic!("Lexer Error: Unexpected character '{}'", c),
        }
    }
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(input: &str) -> Self {
        let mut lexer = Lexer::new(input);
        let mut tokens = Vec::new();
        loop {
            let tok = lexer.next_token();
            tokens.push(tok.clone());
            if tok == Token::Eof {
                break;
            }
        }
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn advance(&mut self) -> &Token {
        let tok = &self.tokens[self.pos];
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
        tok
    }

    fn expect(&mut self, expected: Token) -> Result<(), String> {
        let current = self.advance().clone();
        if current == expected {
            Ok(())
        } else {
            Err(format!("Parser Error: Expected {:?}, found {:?}", expected, current))
        }
    }

    pub fn parse_type(&mut self) -> Result<Type, String> {
        match self.peek().clone() {
            Token::TypeUniv(n) => {
                self.advance();
                Ok(Type::Universe(n))
            }
            Token::IntKw => {
                self.advance();
                Ok(Type::Int)
            }
            Token::UnitKw => {
                self.advance();
                Ok(Type::Unit) 
            }
            Token::BoolKw => {
                self.advance();
                Ok(Type::Bool) // NEW
            }
            Token::Bang => {
                self.advance();
                let inner = self.parse_type()?;
                Ok(Type::Persistent(Box::new(inner)))
            }
            Token::Lin => {
                self.advance();
                let inner = self.parse_type()?;
                Ok(Type::Linear(Permission::Full, Box::new(inner)))
            }
            Token::Pi => {
                self.advance();
                if let Token::Ident(param) = self.advance().clone() {
                    self.expect(Token::Colon)?;
                    let t1 = self.parse_type()?;
                    self.expect(Token::Dot)?;
                    let t2 = self.parse_type()?;
                    Ok(Type::Pi(param, Box::new(t1), Box::new(t2)))
                } else {
                    Err("Expected identifier after Pi".to_string())
                }
            }
            _ => Err(format!("Unexpected token in type: {:?}", self.peek())),
        }
    }

    pub fn parse_term(&mut self) -> Result<Term, String> {
        match self.peek().clone() {
            Token::Number(n) => {
                self.advance();
                Ok(Term::IntVal(n))
            }
            Token::UnitValKw => {
                self.advance();
                Ok(Term::UnitVal) 
            }
            Token::TrueKw => {
                self.advance();
                Ok(Term::BoolVal(true)) // NEW
            }
            Token::FalseKw => {
                self.advance();
                Ok(Term::BoolVal(false)) // NEW
            }
            Token::Ident(name) => {
                self.advance();
                Ok(Term::Var(name))
            }
            Token::IfKw => { // NEW: Parse if condition then true_branch else false_branch
                self.advance(); // consume 'if'
                let condition = self.parse_term()?;
                self.expect(Token::ThenKw)?;
                let true_branch = self.parse_term()?;
                self.expect(Token::ElseKw)?;
                let false_branch = self.parse_term()?;
                Ok(Term::If(Box::new(condition), Box::new(true_branch), Box::new(false_branch)))
            }
            Token::FreeKw => {
                self.advance(); 
                let target = self.parse_term()?;
                Ok(Term::Free(Box::new(target))) 
            }
            Token::SplitKw => {
                self.advance(); 
                if let Token::Ident(target) = self.advance().clone() {
                    self.expect(Token::IntoKw)?;
                    if let Token::Ident(alias1) = self.advance().clone() {
                        self.expect(Token::Comma)?;
                        if let Token::Ident(alias2) = self.advance().clone() {
                            self.expect(Token::InKw)?;
                            let body = self.parse_term()?;
                            Ok(Term::Split(target, alias1, alias2, Box::new(body)))
                        } else { Err("Expected second alias identifier".to_string()) }
                    } else { Err("Expected first alias identifier".to_string()) }
                } else { Err("Expected target identifier to split".to_string()) }
            }
            Token::MergeKw => {
                self.advance(); 
                if let Token::Ident(alias1) = self.advance().clone() {
                    self.expect(Token::AndKw)?;
                    if let Token::Ident(alias2) = self.advance().clone() {
                        self.expect(Token::IntoKw)?;
                        if let Token::Ident(target) = self.advance().clone() {
                            self.expect(Token::InKw)?;
                            let body = self.parse_term()?;
                            Ok(Term::Merge(alias1, alias2, target, Box::new(body)))
                        } else { Err("Expected target identifier".to_string()) }
                    } else { Err("Expected second alias".to_string()) }
                } else { Err("Expected first alias".to_string()) }
            }
            Token::Lambda => {
                self.advance(); 
                if let Token::Ident(param) = self.advance().clone() {
                    self.expect(Token::Colon)?;
                    let param_type = self.parse_type()?;
                    self.expect(Token::Dot)?;
                    let body = self.parse_term()?;
                    Ok(Term::Abs(param, param_type, Box::new(body)))
                } else {
                    Err("Expected identifier after lambda '\\'".to_string())
                }
            }
            Token::LParen => {
                self.advance(); 
                let t1 = self.parse_term()?;
                
                if self.peek() == &Token::Plus {
                    self.advance(); 
                    let t2 = self.parse_term()?;
                    self.expect(Token::RParen)?;
                    Ok(Term::Add(Box::new(t1), Box::new(t2)))
                } else if self.peek() == &Token::RParen {
                    self.advance(); 
                    Ok(t1)
                } else {
                    let t2 = self.parse_term()?;
                    self.expect(Token::RParen)?;
                    Ok(Term::App(Box::new(t1), Box::new(t2)))
                }
            }
            _ => Err(format!("Unexpected token in term: {:?}", self.peek())),
        }
    }
}