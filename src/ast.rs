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
    NUM,
}

#[derive(Debug)]
pub struct Node<T> {
    pub kind: NodeKind,
    pub lhs: Box<Option<Node<T>>>,
    pub rhs: Box<Option<Node<T>>>,
    pub val: Option<T>,
}

pub struct ASTBuilder {
    pub tokens: VecDeque<Token>,
}

impl<T> Node<T> {
    fn new(kind: NodeKind, lhs: Box<Option<Node<T>>>, rhs: Box<Option<Node<T>>>) -> Self {
        Node::<T> {
            kind,
            lhs,
            rhs,
            val: None,
        }
    }

    fn new_num(val: T) -> Self {
        Node::<T> {
            kind: NodeKind::NUM,
            lhs: Box::new(None),
            rhs: Box::new(None),
            val: Some(val),
        }
    }
}

impl ASTBuilder {
    pub fn new(tokens: VecDeque<Token>) -> Self {
        ASTBuilder { tokens }
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

    fn expr(&mut self) -> Box<Option<Node<i64>>> {
        self.equality()
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
        }
        return Box::new(Some(Node::new_num(self.expect_number())));
    }

    pub fn parse(&mut self) -> Box<Option<Node<i64>>> {
        self.expr()
    }

    pub fn gen(&self, node: Box<Option<Node<i64>>>) {
        if let Some(node) = *node {
            if node.kind == NodeKind::NUM {
                println!("  push {}", node.val.unwrap());
                return;
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
