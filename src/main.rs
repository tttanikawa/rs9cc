use std::collections::VecDeque;
use std::env;

#[derive(PartialEq, Debug)]
enum NodeKind {
    ADD,
    SUB,
    MUL,
    DIV,
    NUM,
}

#[derive(Debug)]
struct Node<T> {
    kind: NodeKind,
    lhs: Box<Option<Node<T>>>,
    rhs: Box<Option<Node<T>>>,
    val: Option<T>,
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

fn split_digit(s: &str) -> usize {
    s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len())
}

#[derive(PartialEq, Debug)]
enum TokenKind {
    Reserved,
    NUM,
    EOF,
}

#[derive(Debug)]
struct Token {
    kind: TokenKind,
    string: String,
}

impl Token {
    fn new(kind: TokenKind, string: String) -> Self {
        Token { kind, string }
    }
}

struct Tokenizer {}

impl Tokenizer {
    fn new() -> Self {
        Tokenizer {}
    }

    fn tokenize(&self, input: String) -> VecDeque<Token> {
        let mut tokens = VecDeque::new();
        let mut char_count = 0usize;
        let mut remain = input.as_str();
        while let Some(next) = remain.chars().nth(0) {
            match next {
                ' ' => {
                    let t = remain.trim_start();
                    char_count += remain.len() - t.len();
                    remain = t;
                }
                '+' | '-' | '*' | '/' | '(' | ')' => {
                    tokens.push_back(Token::new(TokenKind::Reserved, next.to_string()));
                    let (_, t) = remain.split_at(1);
                    char_count += 1;
                    remain = t;
                }
                _ if next.is_ascii_digit() => {
                    let idx = split_digit(remain);
                    let (s1, s2) = remain.split_at(idx);
                    tokens.push_back(Token::new(TokenKind::NUM, s1.to_string()));
                    char_count += idx;
                    remain = s2;
                }
                _ => {
                    eprintln!("{}", input);
                    eprintln!(
                        "{}^ Tokenizer error: invalid character {} (column {})",
                        " ".repeat(char_count),
                        next,
                        char_count
                    );
                    panic!();
                }
            }
        }
        tokens.push_back(Token::new(TokenKind::EOF, '\0'.to_string()));
        tokens
    }
}

struct ASTBuilder {
    tokens: VecDeque<Token>,
}

impl ASTBuilder {
    fn new(tokens: VecDeque<Token>) -> Self {
        ASTBuilder { tokens }
    }

    fn consume(&mut self, op: char) -> bool {
        if let Some(token) = self.tokens.front() {
            if token.kind != TokenKind::Reserved || !token.string.starts_with(op) {
                return false;
            }
            self.tokens.pop_front();
            return true;
        } else {
            panic!("Tokenizer consume() error: tokens don't exist");
        }
    }

    fn expect(&mut self, op: char) {
        if let Some(token) = self.tokens.pop_front() {
            if token.kind != TokenKind::Reserved || !token.string.starts_with(op) {
                panic!("Tokenizer expect() error: not {}", op)
            }
        } else {
            panic!("Tokenizer expect() error: tokens don't exist");
        }
    }

    fn expect_number(&mut self) -> i64 {
        if let Some(token) = self.tokens.pop_front() {
            if token.kind != TokenKind::NUM {
                panic!("Tokenizer expect_number() error: not numbers");
            }
            return token.string.parse().unwrap();
        } else {
            panic!("Tokenizer expect_number() error: tokens don't exist");
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
        let mut node = self.mul();
        loop {
            if self.consume('+') {
                node = Box::new(Some(Node::new(NodeKind::ADD, node, self.mul())));
            } else if self.consume('-') {
                node = Box::new(Some(Node::new(NodeKind::SUB, node, self.mul())));
            } else {
                return node;
            }
        }
    }

    fn mul(&mut self) -> Box<Option<Node<i64>>> {
        let mut node = self.primary();
        loop {
            if self.consume('*') {
                node = Box::new(Some(Node::new(NodeKind::MUL, node, self.primary())));
            } else if self.consume('/') {
                node = Box::new(Some(Node::new(NodeKind::DIV, node, self.primary())));
            } else {
                return node;
            }
        }
    }

    fn primary(&mut self) -> Box<Option<Node<i64>>> {
        if self.consume('(') {
            let node = self.expr();
            self.expect(')');
            return node;
        }

        return Box::new(Some(Node::new_num(self.expect_number())));
    }

    fn parse(&mut self) -> Box<Option<Node<i64>>> {
        self.expr()
    }

    fn gen(&self, node: Box<Option<Node<i64>>>) {
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
                _ => panic!("Invalid node kind: {:?}", node.kind),
            }

            println!("  push rax");
        }
    }
}

fn main() {
    if env::args().len() != 2 {
        eprintln!("Invalid argments number: {}", env::args().len());
        return;
    }

    let input = env::args().nth(1).unwrap();

    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");

    let tokenizer = Tokenizer::new();
    let tokens = tokenizer.tokenize(input);
    // dbg!(&tokens);

    let mut builder = ASTBuilder::new(tokens);
    let ast = builder.parse();
    builder.gen(ast);

    println!("  pop rax");
    println!("  ret");
    return;
}
