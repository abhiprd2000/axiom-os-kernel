use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Trust,
    Trusted,
    Verify,
    Spawn,
    Send,
    If,
    Then,
    End,
    Ident(String),
    StringLit(String),
    Number(u64),
    Colon,
    Equals,
    Arrow,
    Newline,
    Eof,
    Unknown(char),
}

pub struct Lexer {
    input: Vec<u8>,
    pos: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.bytes().collect(),
            pos: 0,
        }
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.pos).map(|&b| b as char)
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.input.get(self.pos).map(|&b| b as char);
        self.pos += 1;
        c
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c == ' ' || c == '\t' || c == '\r' {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn read_ident(&mut self) -> String {
        let mut s = String::new();
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                s.push(c);
                self.advance();
            } else {
                break;
            }
        }
        s
    }

    fn read_string(&mut self) -> String {
        self.advance();
        let mut s = String::new();
        while let Some(c) = self.peek() {
            if c == '"' {
                self.advance();
                break;
            }
            s.push(c);
            self.advance();
        }
        s
    }

    fn read_number(&mut self) -> u64 {
        let mut n = 0u64;
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                n = n * 10 + (c as u64 - '0' as u64);
                self.advance();
            } else {
                break;
            }
        }
        n
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            self.skip_whitespace();
            match self.peek() {
                None => {
                    tokens.push(Token::Eof);
                    break;
                }
                Some('\n') => {
                    self.advance();
                    tokens.push(Token::Newline);
                }
                Some(':') => {
                    self.advance();
                    tokens.push(Token::Colon);
                }
                Some('=') => {
                    self.advance();
                    tokens.push(Token::Equals);
                }
                Some('-') => {
                    self.advance();
                    if self.peek() == Some('>') {
                        self.advance();
                        tokens.push(Token::Arrow);
                    }
                }
                Some('"') => {
                    let s = self.read_string();
                    tokens.push(Token::StringLit(s));
                }
                Some(c) if c.is_ascii_digit() => {
                    let n = self.read_number();
                    tokens.push(Token::Number(n));
                }
                Some(c) if c.is_alphabetic() => {
                    let ident = self.read_ident();
                    let tok = match ident.as_str() {
                        "trust" => Token::Trust,
                        "trusted_data" => Token::Trusted,
                        "verify" => Token::Verify,
                        "spawn" => Token::Spawn,
                        "send" => Token::Send,
                        "if" => Token::If,
                        "then" => Token::Then,
                        "end" => Token::End,
                        _ => Token::Ident(ident),
                    };
                    tokens.push(tok);
                }
                Some(c) => {
                    self.advance();
                    tokens.push(Token::Unknown(c));
                }
            }
        }
        tokens
    }
}
