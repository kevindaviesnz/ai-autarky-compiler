use crate::ast::{Term, Type, Permission};
use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Ident(String), Number(u32), StringLit(String),
    Lambda, Colon, Dot, LParen, RParen, LBracket, RBracket, Bang, Plus, Minus, EqEq, Lin, Pi, IntKw,
    UnitKw, UnitValKw, BoolKw, TrueKw, FalseKw, StringKw,
    PairKw, MkPairKw, UnpackKw, EitherKw, LeftKw, RightKw, MatchKw, WithKw, FatArrow, Bar,      
    IfKw, ThenKw, ElseKw, FixKw, FreeKw, TypeUniv(u32), 
    SplitKw, IntoKw, InKw, MergeKw, AndKw, Comma, ArrayKw, AllocKw, ReadKw, WriteKw, ReadFileKw,
    RecKw, FoldKw, UnfoldKw,
    Eof,
}

pub struct Lexer<'a> { chars: Peekable<Chars<'a>> }

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self { Self { chars: input.chars().peekable() } }
    fn consume_whitespace(&mut self) { while let Some(&c) = self.chars.peek() { if c.is_whitespace() { self.chars.next(); } else { break; } } }
    pub fn next_token(&mut self) -> Token {
        self.consume_whitespace();
        match self.chars.next() {
            Some('\\') => Token::Lambda, Some(':') => Token::Colon, Some('.') => Token::Dot,
            Some('(') => Token::LParen, Some(')') => Token::RParen, Some('[') => Token::LBracket, Some(']') => Token::RBracket,
            Some('!') => Token::Bang, Some(',') => Token::Comma, Some('+') => Token::Plus, Some('-') => Token::Minus, Some('|') => Token::Bar, 
            Some('"') => {
                let mut s = String::new();
                while let Some(&c) = self.chars.peek() { if c == '"' { self.chars.next(); break; } else { s.push(self.chars.next().unwrap()); } }
                Token::StringLit(s)
            }
            Some('=') => { 
                if let Some(&'=') = self.chars.peek() { self.chars.next(); Token::EqEq } else if let Some(&'>') = self.chars.peek() { self.chars.next(); Token::FatArrow } else { panic!("Lexer Error"); }
            }
            Some(c) if c.is_alphabetic() || c == '_' => {
                let mut ident = String::from(c);
                while let Some(&next_c) = self.chars.peek() { if next_c.is_alphanumeric() || next_c == '_' { ident.push(self.chars.next().unwrap()); } else { break; } }
                match ident.as_str() {
                    "Lin" => Token::Lin, "Pi" => Token::Pi, "Int" => Token::IntKw, "String" => Token::StringKw, 
                    "Unit" => Token::UnitKw, "unit" => Token::UnitValKw, 
                    "Bool" => Token::BoolKw, "True" => Token::TrueKw, "False" => Token::FalseKw,
                    "Pair" => Token::PairKw, "mkpair" => Token::MkPairKw, "unpack" => Token::UnpackKw, 
                    "Either" => Token::EitherKw, "Left" => Token::LeftKw, "Right" => Token::RightKw, 
                    "match" => Token::MatchKw, "with" => Token::WithKw, 
                    "Array" => Token::ArrayKw, "alloc" => Token::AllocKw, "read" => Token::ReadKw, "write" => Token::WriteKw, "read_file" => Token::ReadFileKw,
                    "if" => Token::IfKw, "then" => Token::ThenKw, "else" => Token::ElseKw,     
                    "fix" => Token::FixKw, "free" => Token::FreeKw, 
                    "split" => Token::SplitKw, "into" => Token::IntoKw, "in" => Token::InKw, "merge" => Token::MergeKw, "and" => Token::AndKw,
                    "Rec" => Token::RecKw, "fold" => Token::FoldKw, "unfold" => Token::UnfoldKw,
                    s if s.starts_with("Type_") => Token::TypeUniv(s[5..].parse().unwrap()),
                    _ => Token::Ident(ident),
                }
            }
            Some(c) if c.is_ascii_digit() => {
                let mut num_str = String::from(c);
                while let Some(&next_c) = self.chars.peek() { if next_c.is_ascii_digit() { num_str.push(self.chars.next().unwrap()); } else { break; } }
                Token::Number(num_str.parse().unwrap())
            }
            None => Token::Eof,
            Some(c) => panic!("Lexer Error: Unexpected character '{}'", c),
        }
    }
}

pub struct Parser { tokens: Vec<Token>, pos: usize }

impl Parser {
    pub fn new(input: &str) -> Self {
        let mut lexer = Lexer::new(input); let mut tokens = Vec::new();
        loop { let tok = lexer.next_token(); tokens.push(tok.clone()); if tok == Token::Eof { break; } }
        Self { tokens, pos: 0 }
    }
    fn peek(&self) -> &Token { &self.tokens[self.pos] }
    fn advance(&mut self) -> &Token { let tok = &self.tokens[self.pos]; if self.pos < self.tokens.len() - 1 { self.pos += 1; } tok }
    fn expect(&mut self, expected: Token) -> Result<(), String> {
        let current = self.advance().clone();
        if current == expected { Ok(()) } else { Err(format!("Expected {:?}, found {:?}", expected, current)) }
    }

    pub fn parse_type(&mut self) -> Result<Type, String> {
        match self.peek().clone() {
            Token::LParen => { self.advance(); let inner = self.parse_type()?; self.expect(Token::RParen)?; Ok(inner) }
            Token::TypeUniv(n) => { self.advance(); Ok(Type::Universe(n)) }
            Token::IntKw => { self.advance(); Ok(Type::Int) }
            Token::UnitKw => { self.advance(); Ok(Type::Unit) }
            Token::BoolKw => { self.advance(); Ok(Type::Bool) }
            Token::StringKw => { self.advance(); Ok(Type::String) } 
            Token::PairKw => { self.advance(); Ok(Type::Pair(Box::new(self.parse_type()?), Box::new(self.parse_type()?))) }
            Token::EitherKw => { self.advance(); Ok(Type::Either(Box::new(self.parse_type()?), Box::new(self.parse_type()?))) }
            Token::ArrayKw => { self.advance(); Ok(Type::Array(Box::new(self.parse_type()?))) }
            Token::Bang => { self.advance(); Ok(Type::Persistent(Box::new(self.parse_type()?))) }
            Token::Lin => { self.advance(); Ok(Type::Linear(Permission::Full, Box::new(self.parse_type()?))) }
            Token::RecKw => {
                self.advance();
                if let Token::Ident(name) = self.advance().clone() {
                    self.expect(Token::Dot)?;
                    Ok(Type::Rec(name, Box::new(self.parse_type()?)))
                } else { Err("Expected identifier for Rec".to_string()) }
            }
            Token::Ident(name) => { self.advance(); Ok(Type::TVar(name)) }
            Token::Pi => {
                self.advance();
                if let Token::Ident(p) = self.advance().clone() {
                    self.expect(Token::Colon)?; let t1 = self.parse_type()?; self.expect(Token::Dot)?; let t2 = self.parse_type()?; Ok(Type::Pi(p, Box::new(t1), Box::new(t2)))
                } else { Err("Expected identifier".to_string()) }
            }
            _ => Err(format!("Unexpected type token: {:?}", self.peek())),
        }
    }

    pub fn parse_term(&mut self) -> Result<Term, String> {
        match self.peek().clone() {
            Token::Number(n) => { self.advance(); Ok(Term::IntVal(n)) }
            Token::StringLit(s) => { self.advance(); Ok(Term::StringVal(s)) } 
            Token::UnitValKw => { self.advance(); Ok(Term::UnitVal) }
            Token::TrueKw => { self.advance(); Ok(Term::BoolVal(true)) }
            Token::FalseKw => { self.advance(); Ok(Term::BoolVal(false)) }
            Token::Ident(name) => { self.advance(); Ok(Term::Var(name)) }
            Token::LeftKw => { self.advance(); let t = self.parse_term()?; let ty = self.parse_type()?; Ok(Term::Left(Box::new(t), ty)) }
            Token::RightKw => { self.advance(); let ty = self.parse_type()?; let t = self.parse_term()?; Ok(Term::Right(ty, Box::new(t))) }
            Token::AllocKw => { self.advance(); let s = self.parse_term()?; let i = self.parse_term()?; Ok(Term::Alloc(Box::new(s), Box::new(i))) }
            Token::ReadKw => { self.advance(); let a = self.parse_term()?; let i = self.parse_term()?; Ok(Term::Read(Box::new(a), Box::new(i))) }
            Token::WriteKw => { self.advance(); let a = self.parse_term()?; let i = self.parse_term()?; let v = self.parse_term()?; Ok(Term::Write(Box::new(a), Box::new(i), Box::new(v))) }
            Token::ReadFileKw => { self.advance(); Ok(Term::ReadFile(Box::new(self.parse_term()?))) }
            Token::FoldKw => {
                self.advance(); self.expect(Token::LBracket)?; let ty = self.parse_type()?; self.expect(Token::RBracket)?;
                Ok(Term::Fold(ty, Box::new(self.parse_term()?)))
            }
            Token::UnfoldKw => { self.advance(); Ok(Term::Unfold(Box::new(self.parse_term()?))) }
            Token::MatchKw => { 
                self.advance(); let tgt = self.parse_term()?; self.expect(Token::WithKw)?; self.expect(Token::LeftKw)?;
                let id_l = if let Token::Ident(n) = self.advance().clone() { n } else { return Err("Expected id".to_string()) };
                self.expect(Token::FatArrow)?; let b_l = self.parse_term()?; self.expect(Token::Bar)?; self.expect(Token::RightKw)?;
                let id_r = if let Token::Ident(n) = self.advance().clone() { n } else { return Err("Expected id".to_string()) };
                self.expect(Token::FatArrow)?; let b_r = self.parse_term()?; Ok(Term::Match(Box::new(tgt), id_l, Box::new(b_l), id_r, Box::new(b_r)))
            }
            Token::MkPairKw => { self.advance(); Ok(Term::MkPair(Box::new(self.parse_term()?), Box::new(self.parse_term()?))) }
            Token::UnpackKw => { 
                self.advance(); let tgt = self.parse_term()?; self.expect(Token::IntoKw)?;
                if let Token::Ident(a1) = self.advance().clone() {
                    self.expect(Token::Comma)?;
                    if let Token::Ident(a2) = self.advance().clone() {
                        self.expect(Token::InKw)?; Ok(Term::Unpack(Box::new(tgt), a1, a2, Box::new(self.parse_term()?)))
                    } else { Err("Expected alias".to_string()) }
                } else { Err("Expected alias".to_string()) }
            }
            Token::IfKw => { 
                self.advance(); let c = self.parse_term()?; self.expect(Token::ThenKw)?; let t = self.parse_term()?; self.expect(Token::ElseKw)?; let f = self.parse_term()?;
                Ok(Term::If(Box::new(c), Box::new(t), Box::new(f)))
            }
            Token::FixKw => { self.advance(); Ok(Term::Fix(Box::new(self.parse_term()?))) }
            Token::FreeKw => { self.advance(); Ok(Term::Free(Box::new(self.parse_term()?))) }
            Token::SplitKw => {
                self.advance(); 
                if let Token::Ident(tgt) = self.advance().clone() {
                    self.expect(Token::IntoKw)?;
                    if let Token::Ident(a1) = self.advance().clone() {
                        self.expect(Token::Comma)?;
                        if let Token::Ident(a2) = self.advance().clone() { self.expect(Token::InKw)?; Ok(Term::Split(tgt, a1, a2, Box::new(self.parse_term()?))) } else { Err("Err".to_string()) }
                    } else { Err("Err".to_string()) }
                } else { Err("Err".to_string()) }
            }
            Token::MergeKw => {
                self.advance(); 
                if let Token::Ident(a1) = self.advance().clone() {
                    self.expect(Token::AndKw)?;
                    if let Token::Ident(a2) = self.advance().clone() {
                        self.expect(Token::IntoKw)?;
                        if let Token::Ident(tgt) = self.advance().clone() { self.expect(Token::InKw)?; Ok(Term::Merge(a1, a2, tgt, Box::new(self.parse_term()?))) } else { Err("Err".to_string()) }
                    } else { Err("Err".to_string()) }
                } else { Err("Err".to_string()) }
            }
            Token::Lambda => {
                self.advance(); 
                if let Token::Ident(p) = self.advance().clone() { self.expect(Token::Colon)?; let p_ty = self.parse_type()?; self.expect(Token::Dot)?; Ok(Term::Abs(p, p_ty, Box::new(self.parse_term()?))) } else { Err("Err".to_string()) }
            }
            Token::LParen => {
                self.advance(); let t1 = self.parse_term()?;
                if self.peek() == &Token::Plus { self.advance(); let t2 = self.parse_term()?; self.expect(Token::RParen)?; Ok(Term::Add(Box::new(t1), Box::new(t2))) }
                else if self.peek() == &Token::Minus { self.advance(); let t2 = self.parse_term()?; self.expect(Token::RParen)?; Ok(Term::Sub(Box::new(t1), Box::new(t2))) }
                else if self.peek() == &Token::EqEq { self.advance(); let t2 = self.parse_term()?; self.expect(Token::RParen)?; Ok(Term::Eq(Box::new(t1), Box::new(t2))) }
                else if self.peek() == &Token::RParen { self.advance(); Ok(t1) }
                else { let t2 = self.parse_term()?; self.expect(Token::RParen)?; Ok(Term::App(Box::new(t1), Box::new(t2))) }
            }
            Token::EitherKw | Token::PairKw => {
                // If Either or Pair appear in a term position unexpectedly
                Err(format!("Token {:?} found in term position. Did you mean to use it in a type?", self.peek()))
            }
            _ => Err(format!("Unexpected term token: {:?}", self.peek())),
        }
    }
}