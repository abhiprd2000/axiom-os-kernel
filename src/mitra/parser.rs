use super::lexer::Token;
use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone)]
pub enum AstNode {
    Trust {
        name: String,
        value: String,
    },
    TrustedData {
        name: String,
        content: String,
    },
    Verify {
        name: String,
    },
    Spawn {
        pid: u64,
    },
    Send {
        from: u64,
        to: u64,
        msg: String,
    },
    If {
        condition: String,
        body: Vec<AstNode>,
    },
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }

    fn advance(&mut self) -> Token {
        let t = self.tokens.get(self.pos).cloned().unwrap_or(Token::Eof);
        self.pos += 1;
        t
    }

    fn skip_newlines(&mut self) {
        while self.peek() == &Token::Newline {
            self.advance();
        }
    }

    pub fn parse(&mut self) -> Vec<AstNode> {
        let mut nodes = Vec::new();
        loop {
            self.skip_newlines();
            match self.peek().clone() {
                Token::Eof => break,
                Token::Trust => {
                    self.advance();
                    let name = match self.advance() {
                        Token::Ident(s) => s,
                        _ => String::from("unknown"),
                    };
                    self.advance(); // colon
                    let value = match self.advance() {
                        Token::StringLit(s) => s,
                        Token::Ident(s) => s,
                        _ => String::from(""),
                    };
                    nodes.push(AstNode::Trust { name, value });
                }
                Token::Trusted => {
                    self.advance();
                    let name = match self.advance() {
                        Token::Ident(s) => s,
                        _ => String::from("unknown"),
                    };
                    self.advance(); // equals
                    let content = match self.advance() {
                        Token::StringLit(s) => s,
                        Token::Ident(s) => s,
                        _ => String::from(""),
                    };
                    nodes.push(AstNode::TrustedData { name, content });
                }
                Token::Verify => {
                    self.advance();
                    let name = match self.advance() {
                        Token::Ident(s) => s,
                        _ => String::from("unknown"),
                    };
                    nodes.push(AstNode::Verify { name });
                }
                Token::Spawn => {
                    self.advance();
                    let pid = match self.advance() {
                        Token::Number(n) => n,
                        _ => 0,
                    };
                    nodes.push(AstNode::Spawn { pid });
                }
                Token::Send => {
                    self.advance();
                    let from = match self.advance() {
                        Token::Number(n) => n,
                        _ => 0,
                    };
                    self.advance(); // arrow
                    let to = match self.advance() {
                        Token::Number(n) => n,
                        _ => 0,
                    };
                    let msg = match self.advance() {
                        Token::StringLit(s) => s,
                        _ => String::from(""),
                    };
                    nodes.push(AstNode::Send { from, to, msg });
                }
                _ => {
                    self.advance();
                }
            }
        }
        nodes
    }
}
