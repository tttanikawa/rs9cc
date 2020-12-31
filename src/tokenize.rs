use crate::util;
use std::collections::VecDeque;

#[derive(PartialEq, Debug)]
pub enum TokenKind {
    Reserved,
    NUM,
    EOF,
}

#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub string: String,
}

pub struct Tokenizer {}

impl Token {
    pub fn new(kind: TokenKind, string: String) -> Self {
        Token { kind, string }
    }
}

impl Tokenizer {
    pub fn new() -> Self {
        Tokenizer {}
    }

    pub fn tokenize(&self, input: String) -> VecDeque<Token> {
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
                    let idx = util::split_digit(remain);
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
