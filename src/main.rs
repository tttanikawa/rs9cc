use std::env;

fn main() {
    if env::args().len() != 2 {
        eprintln!("Invalid argments number: {}", env::args().len());
        return;
    }

    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");
    println!("  mov rax, {}", env::args().nth(1).unwrap());
    println!("  ret");

    return;
}
