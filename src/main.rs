mod ast;
mod tokenize;
mod util;

use std::env;

use ast::ASTBuilder;
use tokenize::Tokenizer;

fn main() {
    if env::args().len() != 2 {
        eprintln!("Invalid argments number: {}", env::args().len());
        return;
    }

    let input = env::args().nth(1).unwrap();

    let tokenizer = Tokenizer::new();
    let tokens = tokenizer.tokenize(input);
    // dbg!(&tokens);

    let mut builder = ASTBuilder::new(tokens);
    let ast = builder.parse();

    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");

    println!("  push rbp");
    println!("  mov rbp, rsp");
    println!("  sub rsp, 208");

    for code in ast.into_iter() {
        builder.gen(code);
        println!("  pop rax");
    }

    println!("  mov rsp, rbp");
    println!("  pop rbp");
    println!("  ret");
    return;
}
