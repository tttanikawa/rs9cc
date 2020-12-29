use std::{env, unimplemented};

fn search_split_idx(s: &str) -> Option<usize> {
    s.find(|c: char| !c.is_ascii_digit())
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

    let idx = search_split_idx(input.as_str()).unwrap_or(input.len());
    let (d, mut remain) = input.split_at(idx);
    println!("  mov rax, {}", d);

    while let Some(c) = remain.chars().nth(0) {
        remain = &remain[1..];
        let idx = search_split_idx(remain).unwrap_or(remain.len());
        let s = remain.split_at(idx);
        match c {
            '+' => {
                println!("  add rax, {}", s.0);
                remain = s.1;
            }
            '-' => {
                println!("  sub rax, {}", s.0);
                remain = s.1;
            }
            _ => unimplemented!(),
        }
    }

    println!("  ret");
    return;
}
