// src/parser.rs

use crate::ast::{Expr, Type};

pub struct Parser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() {
            let ch = self.input[self.pos..].chars().next().unwrap();
            if ch.is_whitespace() {
                self.pos += ch.len_utf8();
            } else if ch == '#' {
                while self.pos < self.input.len() && !self.input[self.pos..].starts_with('\n') {
                    self.pos += 1;
                }
            } else {
                break;
            }
        }
    }

    fn peek(&mut self) -> Option<&str> {
        self.skip_whitespace();
        if self.pos >= self.input.len() { return None; }
        let start = self.pos;
        let mut end = start;
        let first_char = self.input[start..].chars().next()?;
        
        if first_char.is_numeric() {
            let mut has_dot = false;
            while end < self.input.len() {
                let next_char = self.input[end..].chars().next()?;
                if next_char.is_numeric() {
                    end += next_char.len_utf8();
                } else if next_char == '.' && !has_dot {
                    has_dot = true;
                    end += next_char.len_utf8();
                } else {
                    break;
                }
            }
        } else if first_char.is_alphabetic() || first_char == '_' {
            while end < self.input.len() {
                let next_char = self.input[end..].chars().next()?;
                if next_char.is_alphanumeric() || next_char == '_' {
                    end += next_char.len_utf8();
                } else {
                    break;
                }
            }
        } else if self.input[start..].starts_with("=>") {
            end += 2;
        } else {
            end += first_char.len_utf8();
        }
        
        Some(&self.input[start..end])
    }

    fn consume(&mut self, expected: &str) -> Result<(), String> {
        if self.peek() == Some(expected) {
            self.pos += expected.len();
            Ok(())
        } else {
            Err(format!("Expected '{}', found {:?}", expected, self.peek()))
        }
    }

    fn consume_ident(&mut self) -> Result<String, String> {
        let token = self.peek().ok_or("Expected identifier")?.to_string();
        if token == "as" || token == "in" || token == "match" || token == "left" || token == "right" {
            return Err(format!("Keyword '{}' cannot be used as identifier", token));
        }
        self.pos += token.len();
        Ok(token)
    }

    pub fn parse(&mut self) -> Result<Expr, String> {
        self.parse_expr()
    }

    fn parse_expr(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_primary()?;
        while let Some(next) = self.peek() {
            let first_char = next.chars().next().unwrap();
            if next == "(" || first_char.is_numeric() || (first_char.is_alphabetic() && next != "in" && next != "as" && next != "}" && next != "," && next != "right") {
                let arg = self.parse_primary()?;
                expr = Expr::App { func: Box::new(expr), arg: Box::new(arg) };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        let token = self.peek().ok_or("Unexpected EOF")?;
        match token {
            "let" => self.parse_let(),
            "\\" | "λ" => self.parse_lambda(),
            "unpack" => self.parse_unpack(),
            "match" => self.parse_match(),
            "sub" => { self.consume("sub")?; Ok(Expr::Sub(Box::new(self.parse_primary()?), Box::new(self.parse_primary()?))) }
            "add" => { self.consume("add")?; Ok(Expr::Add(Box::new(self.parse_primary()?), Box::new(self.parse_primary()?))) }
            "mul" => { self.consume("mul")?; Ok(Expr::Mul(Box::new(self.parse_primary()?), Box::new(self.parse_primary()?))) } // NEW
            "div" => { self.consume("div")?; Ok(Expr::Div(Box::new(self.parse_primary()?), Box::new(self.parse_primary()?))) } // NEW
            "eq"  => { self.consume("eq")?; Ok(Expr::Eq { left: Box::new(self.parse_primary()?), right: Box::new(self.parse_primary()?) }) }
            "left" => { self.consume("left")?; Ok(Expr::Left(Box::new(self.parse_primary()?), Type::Int)) }
            "right" => { self.consume("right")?; Ok(Expr::Right(Box::new(self.parse_primary()?), Type::Int)) }
            "(" => {
                self.consume("(")?;
                let e = self.parse_expr()?;
                if self.peek() == Some(",") {
                    self.consume(",")?;
                    let e2 = self.parse_expr()?;
                    self.consume(")")?;
                    Ok(Expr::MkPair(Box::new(e), Box::new(e2)))
                } else {
                    self.consume(")")?;
                    Ok(e)
                }
            }
            _ => {
                let first_char = token.chars().next().unwrap();
                if first_char.is_numeric() {
                    if token.contains('.') {
                        let f: f64 = token.parse().map_err(|_| format!("Invalid float format: {}", token))?;
                        self.pos += token.len();
                        Ok(Expr::FloatLiteral(f))
                    } else {
                        let n: i64 = token.parse().map_err(|_| format!("Invalid integer format: {}", token))?;
                        self.pos += token.len();
                        Ok(Expr::IntLiteral(n))
                    }
                } else {
                    let name = self.consume_ident()?;
                    Ok(Expr::Variable(name))
                }
            }
        }
    }

    fn parse_let(&mut self) -> Result<Expr, String> {
        self.consume("let")?;
        let name = self.consume_ident()?;
        self.consume("=")?;
        let val = Box::new(self.parse_expr()?);
        self.consume("in")?;
        let body = Box::new(self.parse_expr()?);
        
        Ok(Expr::App { 
            func: Box::new(Expr::Lambda { param: name, param_type: Type::Int, body }), 
            arg: val 
        })
    }

    fn parse_lambda(&mut self) -> Result<Expr, String> {
        let token_str = self.peek().ok_or("Expected lambda start")?.to_string();
        self.consume(&token_str)?;
        let param = self.consume_ident()?;
        self.consume(".")?;
        let body = Box::new(self.parse_expr()?);
        
        let param_type = Type::Pair(Box::new(Type::Int), Box::new(Type::Int));
        Ok(Expr::Lambda { param, param_type, body })
    }

    fn parse_unpack(&mut self) -> Result<Expr, String> {
        self.consume("unpack")?;
        let pair_expr = Box::new(self.parse_primary()?);
        self.consume("as")?;
        let var1 = self.consume_ident()?;
        self.consume(",")?;
        let var2 = self.consume_ident()?;
        self.consume("in")?;
        let body = Box::new(self.parse_expr()?);
        Ok(Expr::Unpack { pair: pair_expr, var1, var2, body })
    }

    fn parse_match(&mut self) -> Result<Expr, String> {
        self.consume("match")?;
        let expr = Box::new(self.parse_expr()?);
        self.consume("{")?;
        self.consume("left")?;
        let left_var = self.consume_ident()?;
        self.consume("=>")?;
        let left_body = Box::new(self.parse_expr()?);
        if self.peek() == Some(",") { self.consume(",")?; }
        self.consume("right")?;
        let right_var = self.consume_ident()?;
        self.consume("=>")?;
        let right_body = Box::new(self.parse_expr()?);
        self.consume("}")?;
        Ok(Expr::Match { expr, left_var, left_body, right_var, right_body })
    }
}