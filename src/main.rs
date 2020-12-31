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
