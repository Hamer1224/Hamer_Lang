use crate::lexer::Token;
use std::fs;

#[derive(Debug)]
pub enum Stmt {
    LocalAssign { name: String, value: f64 },
    ClassDef { name: String, fields: Vec<String> },
    HeapAlloc { var_name: String, class_name: String },
    FieldAssign { path: Vec<String>, value: f64 },
    FieldMath { path: Vec<String>, op: Token, rhs_val: f64 },
    PrintVar(String),
    PrintString(String),
    IfStmt { path: Vec<String>, op: Token, rhs_val: f64, body: Vec<Stmt> },
    ProbIf { chance: f64, body: Vec<Stmt> },
    WhileStmt { path: Vec<String>, op: Token, rhs_val: f64, body: Vec<Stmt> },
    AsmBlock(String),      
    IntelBlock(String),    
    PythonBlock(String),   
    MergeBlock(String),    
}

pub struct Parser { pub tokens: Vec<Token>, pub pos: usize }

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self { Self { tokens, pos: 0 } }
    
    fn advance(&mut self) -> Token {
        let t = self.tokens[self.pos].clone();
        if self.pos < self.tokens.len() - 1 { self.pos += 1; }
        t
    }

    fn peek(&self) -> Token { self.tokens[self.pos].clone() }

    pub fn parse_program(&mut self) -> Vec<Stmt> {
        let mut stmts = Vec::new();
        while self.peek() != Token::EOF {
            stmts.push(self.parse_statement());
        }
        stmts
    }

    fn parse_path(&mut self) -> Vec<String> {
        let mut path = Vec::new();
        if let Token::Identifier(s) = self.peek() {
            self.advance(); path.push(s);
            while self.peek() == Token::Dot {
                self.advance();
                if let Token::Identifier(s) = self.advance() { path.push(s); }
            }
        }
        path
    }

    fn parse_statement(&mut self) -> Stmt {
        match self.peek() {
            // Get <filename>: Modular File Inclusion
            Token::Get => {
                self.advance();
                let filename = if let Token::Identifier(s) = self.advance() { s } else { "lib".into() };
                let path = format!("{}.hmr", filename);
                match fs::read_to_string(&path) {
                    Ok(content) => Stmt::MergeBlock(content),
                    Err(_) => Stmt::AsmBlock(format!("// Error: Could not read {}.hmr", filename)),
                }
            }
            // @ symbols: asm, intel, python
            Token::At => {
                self.advance();
                match self.peek() {
                    Token::Identifier(ref s) if s == "intel" => {
                        self.advance(); if self.peek() == Token::Is { self.advance(); }
                        let mut code = String::new();
                        while self.peek() != Token::Done && self.peek() != Token::EOF {
                            match self.advance() {
                                Token::Identifier(id) => code.push_str(&format!("{} ", id)),
                                Token::Number(n) => code.push_str(&format!("{} ", n)),
                                Token::Comma => code.push_str(", "),
                                Token::LeftBracket => code.push_str("[ "),
                                Token::RightBracket => code.push_str("] "),
                                _ => {}
                            }
                        }
                        if self.peek() == Token::Done { self.advance(); }
                        Stmt::IntelBlock(code)
                    }
                    Token::Identifier(ref s) if s == "python" => {
                        self.advance(); if self.peek() == Token::Is { self.advance(); }
                        let mut script = String::new();
                        while self.peek() != Token::Done {
                            match self.advance() {
                                Token::Identifier(id) => script.push_str(&format!("{} ", id)),
                                Token::StringLit(s) => script.push_str(&format!("\"{}\" ", s)),
                                Token::Number(n) => script.push_str(&format!("{} ", n)),
                                _ => {}
                            }
                        }
                        self.advance(); // done
                        Stmt::PythonBlock(script)
                    }
                    Token::Identifier(ref s) if s == "asm" => {
                        self.advance(); if self.peek() == Token::Is { self.advance(); }
                        let mut code = String::new();
                        while self.peek() != Token::Done {
                            match self.advance() {
                                Token::Identifier(id) => code.push_str(&format!("{} ", id)),
                                Token::Number(n) => code.push_str(&format!("#{} ", n)),
                                Token::Comma => code.push_str(", "),
                                _ => {}
                            }
                        }
                        self.advance(); // done
                        Stmt::AsmBlock(code)
                    }
                    _ => Stmt::AsmBlock("nop".into())
                }
            }
            Token::Local => {
                self.advance();
                let name = if let Token::Identifier(s) = self.advance() { s } else { "".into() };
                if self.peek() == Token::Assign { self.advance(); }
                if self.peek() == Token::New {
                    self.advance();
                    let cn = if let Token::Identifier(s) = self.advance() { s } else { "".into() };
                    Stmt::HeapAlloc { var_name: name, class_name: cn }
                } else {
                    let val = if let Token::Number(n) = self.advance() { n } else { 0.0 };
                    Stmt::LocalAssign { name, value: val }
                }
            }
            Token::Class => {
                self.advance();
                let name = if let Token::Identifier(s) = self.advance() { s } else { "".into() };
                if self.peek() == Token::Is { self.advance(); }
                let mut fields = Vec::new();
                while !matches!(self.peek(), Token::Done | Token::EOF) {
                    if let Token::Identifier(s) = self.advance() { fields.push(s); }
                }
                if self.peek() == Token::Done { self.advance(); }
                Stmt::ClassDef { name, fields }
            }
            Token::Print => {
                self.advance();
                if let Token::StringLit(s) = self.peek() { self.advance(); Stmt::PrintString(s) }
                else {
                    let p = self.parse_path();
                    if !p.is_empty() { Stmt::PrintVar(p[0].clone()) } else { Stmt::AsmBlock("nop".into()) }
                }
            }
            Token::If => {
                self.advance();
                if self.peek() == Token::Quest {
                    self.advance(); // ?
                    while matches!(self.peek(), Token::Less | Token::Percent) { self.advance(); }
                    let chance = if let Token::Number(n) = self.advance() { n } else { 0.0 };
                    while matches!(self.peek(), Token::Greater | Token::Is | Token::Then) { self.advance(); }
                    let mut body = Vec::new();
                    while self.peek() != Token::Done { body.push(self.parse_statement()); }
                    self.advance();
                    Stmt::ProbIf { chance, body }
                } else {
                    let p = self.parse_path(); let op = self.advance();
                    let val = if let Token::Number(n) = self.advance() { n } else { 0.0 };
                    while matches!(self.peek(), Token::Then | Token::Is) { self.advance(); }
                    let mut body = Vec::new();
                    while self.peek() != Token::Done { body.push(self.parse_statement()); }
                    self.advance();
                    Stmt::IfStmt { path: p, op, rhs_val: val, body }
                }
            }
            Token::While => {
                self.advance(); let p = self.parse_path(); let op = self.advance();
                let val = if let Token::Number(n) = self.advance() { n } else { 0.0 };
                while matches!(self.peek(), Token::Do | Token::Is) { self.advance(); }
                let mut body = Vec::new();
                while self.peek() != Token::Done { body.push(self.parse_statement()); }
                self.advance();
                Stmt::WhileStmt { path: p, op, rhs_val: val, body }
            }
            _ => {
                let path = self.parse_path();
                if self.peek() == Token::Assign {
                    self.advance();
                    if let Token::Number(v) = self.peek() { self.advance(); Stmt::FieldAssign { path, value: v } }
                    else {
                        self.advance(); // consume self
                        let op = self.advance();
                        let val = if let Token::Number(v) = self.advance() { v } else { 0.0 };
                        Stmt::FieldMath { path, op, rhs_val: val }
                    }
                } else { self.advance(); Stmt::AsmBlock("nop".into()) }
            }
        }
    }
}