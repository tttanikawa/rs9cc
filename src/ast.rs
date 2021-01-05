use crate::tokenize::{Token, TokenKind};
use std::collections::VecDeque;

#[derive(PartialEq, Debug)]
pub enum NodeKind {
    ADD,
    SUB,
    MUL,
    DIV,
    EQ,
    NE,
    LT,
    LE,
    ASSIGN,
    LVAR,
    RETURN,
    NUM,
}

#[derive(Debug)]
pub struct Node<T> {
    pub kind: NodeKind,
    pub lhs: Box<Option<Node<T>>>,
    pub rhs: Box<Option<Node<T>>>,
    pub val: Option<T>,
    pub offset: Option<usize>,
}

pub struct ASTBuilder {
    pub tokens: VecDeque<Token>,
    pub lvars: Vec<LVar>,
}

pub struct LVar {
    name: String,
    offset: usize,
}

impl<T> Node<T> {
    fn new(kind: NodeKind, lhs: Box<Option<Node<T>>>, rhs: Box<Option<Node<T>>>) -> Self {
        Node::<T> {
            kind,
            lhs,
            rhs,
            val: None,
            offset: None,
        }
    }

    fn new_num(val: T) -> Self {
        Node::<T> {
            kind: NodeKind::NUM,
            lhs: Box::new(None),
            rhs: Box::new(None),
            val: Some(val),
            offset: None,
        }
    }

    fn new_ident(offset: usize) -> Self {
        Node::<T> {
            kind: NodeKind::LVAR,
            lhs: Box::new(None),
            rhs: Box::new(None),
            val: None,
            offset: Some(offset),
        }
    }
}

impl ASTBuilder {
    pub fn new(tokens: VecDeque<Token>) -> Self {
        ASTBuilder {
            tokens,
            lvars: Vec::new(),
        }
    }

    fn find_lvar(&self, tok: &Token) -> Option<&LVar> {
        for lvar in &self.lvars {
            if lvar.name == tok.string {
                return Some(lvar);
            }
        }
        None
    }

    fn consume(&mut self, op: &str) -> bool {
        if let Some(token) = self.tokens.front() {
            if token.kind != TokenKind::Reserved || token.string != op {
                return false;
            }
            self.tokens.pop_front();
            return true;
        } else {
            panic!("ASTBuilder consume() error: tokens don't exist");
        }
    }

    fn consume_ident(&mut self) -> Option<Token> {
        if let Some(token) = self.tokens.front() {
            if token.kind != TokenKind::IDENT {
                return None;
            }
            return Some(self.tokens.pop_front().unwrap());
        }
        None
    }

    fn consume_return(&mut self) -> Option<Token> {
        if let Some(token) = self.tokens.front() {
            if token.kind != TokenKind::RETURN {
                return None;
            }
            return Some(self.tokens.pop_front().unwrap());
        }
        None
    }

    fn expect(&mut self, op: &str) {
        if let Some(token) = self.tokens.pop_front() {
            if token.kind != TokenKind::Reserved || token.string != op {
                panic!("ASTBuilder expect() error: not {}", op)
            }
        } else {
            panic!("ASTBuilder expect() error: tokens don't exist");
        }
    }

    fn expect_number(&mut self) -> i64 {
        if let Some(token) = self.tokens.pop_front() {
            if token.kind != TokenKind::NUM {
                panic!("ASTBuilder expect_number() error: not numbers");
            }
            return token.string.parse().unwrap();
        } else {
            panic!("ASTBuilder expect_number() error: tokens don't exist");
        }
    }

    fn at_eof(&self) -> bool {
        if let Some(token) = self.tokens.front() {
            token.kind == TokenKind::EOF
        } else {
            false
        }
    }

    fn program(&mut self) -> Vec<Box<Option<Node<i64>>>> {
        let mut code = vec![];
        while !self.at_eof() {
            code.push(self.stmt());
        }
        code
    }

    fn stmt(&mut self) -> Box<Option<Node<i64>>> {
        let node = if let Some(_) = self.consume_return() {
            Box::new(Some(Node::new(
                NodeKind::RETURN,
                self.expr(),
                Box::new(None),
            )))
        } else {
            self.expr()
        };

        if !self.consume(";") {
            panic!("ASTBuilder stmt() error: no ;");
        }
        node
    }

    fn expr(&mut self) -> Box<Option<Node<i64>>> {
        self.assign()
    }

    fn assign(&mut self) -> Box<Option<Node<i64>>> {
        let mut node = self.equality();
        if self.consume("=") {
            node = Box::new(Some(Node::new(NodeKind::ASSIGN, node, self.assign())));
        }
        node
    }

    fn equality(&mut self) -> Box<Option<Node<i64>>> {
        let mut node = self.relational();
        loop {
            if self.consume("==") {
                node = Box::new(Some(Node::new(NodeKind::EQ, node, self.relational())));
            } else if self.consume("!=") {
                node = Box::new(Some(Node::new(NodeKind::NE, node, self.relational())));
            } else {
                return node;
            }
        }
    }

    fn relational(&mut self) -> Box<Option<Node<i64>>> {
        let mut node = self.add();
        loop {
            if self.consume("<") {
                node = Box::new(Some(Node::new(NodeKind::LT, node, self.add())));
            } else if self.consume("<=") {
                node = Box::new(Some(Node::new(NodeKind::LE, node, self.add())));
            } else if self.consume(">") {
                node = Box::new(Some(Node::new(NodeKind::LT, self.add(), node)));
            } else if self.consume(">=") {
                node = Box::new(Some(Node::new(NodeKind::LE, self.add(), node)));
            } else {
                return node;
            }
        }
    }

    fn add(&mut self) -> Box<Option<Node<i64>>> {
        let mut node = self.mul();
        loop {
            if self.consume("+") {
                node = Box::new(Some(Node::new(NodeKind::ADD, node, self.mul())));
            } else if self.consume("-") {
                node = Box::new(Some(Node::new(NodeKind::SUB, node, self.mul())));
            } else {
                return node;
            }
        }
    }

    fn mul(&mut self) -> Box<Option<Node<i64>>> {
        let mut node = self.unary();
        loop {
            if self.consume("*") {
                node = Box::new(Some(Node::new(NodeKind::MUL, node, self.unary())));
            } else if self.consume("/") {
                node = Box::new(Some(Node::new(NodeKind::DIV, node, self.unary())));
            } else {
                return node;
            }
        }
    }

    fn unary(&mut self) -> Box<Option<Node<i64>>> {
        if self.consume("+") {
            return self.unary();
        } else if self.consume("-") {
            return Box::new(Some(Node::new(
                NodeKind::SUB,
                Box::new(Some(Node::new_num(0))),
                self.unary(),
            )));
        } else {
            return self.primary();
        }
    }

    fn primary(&mut self) -> Box<Option<Node<i64>>> {
        if self.consume("(") {
            let node = self.expr();
            self.expect(")");
            return node;
        } else if let Some(token) = self.consume_ident() {
            if let Some(lvar) = self.find_lvar(&token) {
                return Box::new(Some(Node::new_ident(lvar.offset)));
            } else {
                let offset = self.lvars.len() * 8;
                self.lvars.push(LVar {
                    name: token.string,
                    offset,
                });
                return Box::new(Some(Node::new_ident(offset)));
            }
        } else {
            return Box::new(Some(Node::new_num(self.expect_number())));
        }
    }

    pub fn parse(&mut self) -> Vec<Box<Option<Node<i64>>>> {
        self.program()
    }

    fn gen_lval(&self, node: &Node<i64>) {
        if node.kind != NodeKind::LVAR {
            panic!("ASTBuilder gen_lval() error: not lval");
        }
        println!("  mov rax, rbp");
        println!("  sub rax, {}", node.offset.unwrap());
        println!("  push rax");
    }

    fn gen_lval_box(&self, node: Box<Option<Node<i64>>>) {
        if let Some(node) = *node {
            self.gen_lval(&node);
        }
    }

    pub fn gen(&self, node: Box<Option<Node<i64>>>) {
        if let Some(node) = *node {
            match node.kind {
                NodeKind::NUM => {
                    println!("  push {}", node.val.unwrap());
                    return;
                }
                NodeKind::LVAR => {
                    self.gen_lval(&node);
                    println!("  pop rax");
                    println!("  mov rax, [rax]");
                    println!("  push rax");
                    return;
                }
                NodeKind::ASSIGN => {
                    self.gen_lval_box(node.lhs);
                    self.gen(node.rhs);
                    println!("  pop rdi");
                    println!("  pop rax");
                    println!("  mov [rax], rdi");
                    println!("  push rdi");
                    return;
                }
                NodeKind::RETURN => {
                    self.gen(node.lhs);
                    println!("  pop rax");
                    println!("  mov rsp, rbp");
                    println!("  pop rbp");
                    println!("  ret");
                    return;
                }
                _ => (),
            }

            self.gen(node.lhs);
            self.gen(node.rhs);

            println!("  pop rdi");
            println!("  pop rax");

            match node.kind {
                NodeKind::ADD => println!("  add rax, rdi"),
                NodeKind::SUB => println!("  sub rax, rdi"),
                NodeKind::MUL => println!("  imul rax, rdi"),
                NodeKind::DIV => {
                    println!("  cqo");
                    println!("  idiv rdi");
                }
                NodeKind::EQ => {
                    println!("  cmp rax, rdi");
                    println!("  sete al");
                    println!("  movzb rax, al");
                }
                NodeKind::NE => {
                    println!("  cmp rax, rdi");
                    println!("  setne al");
                    println!("  movzb rax, al");
                }
                NodeKind::LT => {
                    println!("  cmp rax, rdi");
                    println!("  setl al");
                    println!("  movzb rax, al");
                }
                NodeKind::LE => {
                    println!("  cmp rax, rdi");
                    println!("  setle al");
                    println!("  movzb rax, al");
                }
                _ => panic!("Invalid node kind: {:?}", node.kind),
            }

            println!("  push rax");
        }
    }
}
