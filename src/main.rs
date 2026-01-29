use std::env;
use std::fs;
use std::process;

mod lexer;
mod parser;
mod generator;

use lexer::Lexer;
use parser::Parser;
use generator::Generator;

fn main() {
    // Collect CLI arguments: hamer <filename>
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        println!("H@mer Compiler v0.1");
        println!("Usage: hamer <file.hmr>");
        process::exit(1);
    }

    let file_path = &args[1];
    
    // 1. Read the H@mer source file
    let input = fs::read_to_string(file_path).expect("Could not read source file");

    println!("[H@mer] Tokenizing...");
    // 2. Lexical Analysis (Tokens)
    let mut lexer = Lexer::new(input);
    let mut tokens = Vec::new();
    loop {
        let token = lexer.next_token();
        if token == lexer::Token::EOF { break; }
        tokens.push(token);
    }

    println!("[H@mer] Parsing AST...");
    // 3. Syntax Analysis (Abstract Syntax Tree)
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program();

    println!("[H@mer] Generating ARM64 Assembly...");
    // 4. Code Generation
    let mut generator = Generator::new();
    let assembly = generator.generate(ast);

    // 5. Output to out.s (Assembly file)
    fs::write("out.s", assembly).expect("Could not write assembly file");
    
    println!("[SUCCESS] compiled {} to out.s", file_path);
    println!("Next steps:");
    println!("  as out.s -o out.o");
    println!("  ld out.o -o hamer_prog");
}