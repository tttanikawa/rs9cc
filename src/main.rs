use std::collections::VecDeque;
use std::env;

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

struct Tokenizer {
    tokens: VecDeque<Token>,
}

impl Tokenizer {
    fn new() -> Self {
        Tokenizer {
            tokens: VecDeque::new(),
        }
    }

    fn new_token(&mut self, kind: TokenKind, string: String) {
        self.tokens.push_back(Token { kind, string });
    }

    fn tokenize(&mut self, input: String) {
        self.tokens.clear();
        let mut char_count = 0usize;
        let mut remain = input.as_str();
        while let Some(next) = remain.chars().nth(0) {
            match next {
                ' ' => {
                    let t = remain.trim_start();
                    char_count += remain.len() - t.len();
                    remain = t;
                }
                '+' | '-' => {
                    self.new_token(TokenKind::Reserved, next.to_string());
                    let (_, t) = remain.split_at(1);
                    char_count += 1;
                    remain = t;
                }
                _ if next.is_ascii_digit() => {
                    let idx = split_digit(remain);
                    let (s1, s2) = remain.split_at(idx);
                    self.new_token(TokenKind::NUM, s1.to_string());
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
            eprintln!("{}", char_count);
        }
        self.new_token(TokenKind::EOF, '\0'.to_string());
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
}

fn main() {
    if env::args().len() != 2 {
        eprintln!("Invalid argments number: {}", env::args().len());
        return;
    }

    let input = env::args().nth(1).unwrap();

    let mut tokenizer = Tokenizer::new();
    tokenizer.tokenize(input);

    dbg!(&tokenizer.tokens);

    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");

    println!("  mov rax, {}", tokenizer.expect_number());
    while !tokenizer.at_eof() {
        if tokenizer.consume('+') {
            println!("  add rax, {}", tokenizer.expect_number());
        } else {
            tokenizer.expect('-');
            println!("  sub rax, {}", tokenizer.expect_number());
        }
    }
    println!("  ret");
    return;
}
