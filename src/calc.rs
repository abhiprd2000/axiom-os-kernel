extern crate alloc;
use alloc::vec::Vec;
#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number(f64),
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
}

fn tokenize(input: &str) -> Result<Vec<Token>, &'static str> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();
    while let Some(&ch) = chars.peek() {
        match ch {
            ' ' | '\t' => {
                chars.next();
            }
            '0'..='9' | '.' => {
                let mut s = alloc::string::String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_digit() || c == '.' {
                        s.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Number(s.parse::<f64>().map_err(|_| "Bad number")?));
            }
            '+' => {
                tokens.push(Token::Plus);
                chars.next();
            }
            '-' => {
                tokens.push(Token::Minus);
                chars.next();
            }
            '*' => {
                tokens.push(Token::Star);
                chars.next();
            }
            '/' => {
                tokens.push(Token::Slash);
                chars.next();
            }
            '(' => {
                tokens.push(Token::LParen);
                chars.next();
            }
            ')' => {
                tokens.push(Token::RParen);
                chars.next();
            }
            _ => return Err("Unknown character"),
        }
    }
    Ok(tokens)
}

struct Parser {
    tokens: alloc::vec::Vec<Token>,
    pos: usize,
}
impl Parser {
    fn new(tokens: alloc::vec::Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }
    fn consume(&mut self) -> Option<Token> {
        let t = self.tokens.get(self.pos).cloned();
        self.pos += 1;
        t
    }
    fn expr(&mut self) -> Result<f64, &'static str> {
        let mut v = self.term()?;
        loop {
            match self.peek() {
                Some(Token::Plus) => {
                    self.consume();
                    v += self.term()?;
                }
                Some(Token::Minus) => {
                    self.consume();
                    v -= self.term()?;
                }
                _ => break,
            }
        }
        Ok(v)
    }
    fn term(&mut self) -> Result<f64, &'static str> {
        let mut v = self.factor()?;
        loop {
            match self.peek() {
                Some(Token::Star) => {
                    self.consume();
                    v *= self.factor()?;
                }
                Some(Token::Slash) => {
                    self.consume();
                    let r = self.factor()?;
                    if r == 0.0 {
                        return Err("Division by zero");
                    }
                    v /= r;
                }
                _ => break,
            }
        }
        Ok(v)
    }
    fn factor(&mut self) -> Result<f64, &'static str> {
        match self.consume() {
            Some(Token::Number(n)) => Ok(n),
            Some(Token::Minus) => Ok(-self.factor()?),
            Some(Token::LParen) => {
                let v = self.expr()?;
                match self.consume() {
                    Some(Token::RParen) => Ok(v),
                    _ => Err("Missing )"),
                }
            }
            _ => Err("Expected number or ("),
        }
    }
}

pub fn evaluate(input: &str) -> Result<f64, &'static str> {
    if input.trim().is_empty() {
        return Err("Empty expression");
    }
    let tokens = tokenize(input)?;
    let mut p = Parser::new(tokens);
    let v = p.expr()?;
    if p.peek().is_some() {
        return Err("Unexpected characters at end");
    }
    Ok(v)
}

pub fn format_result(v: f64) -> alloc::string::String {
    if v == (v as i64) as f64 {
        alloc::format!("{}", v as i64)
    } else {
        let s = alloc::format!("{:.6}", v);
        alloc::string::String::from(s.trim_end_matches('0').trim_end_matches('.'))
    }
}
