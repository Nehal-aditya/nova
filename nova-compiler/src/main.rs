// src/main.rs
// NOVA Compiler - Command-line interface

use nova_compiler::{Lexer, Parser};
use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: nova <file.nv>");
        eprintln!("\nPhase 0: Lexer and Parser only (no code generation)");
        process::exit(1);
    }

    let filename = &args[1];

    // Check for help
    if filename == "--help" || filename == "-h" {
        println!("NOVA Compiler - Phase 0");
        println!();
        println!("Usage: nova <file.nv>");
        println!();
        println!("This is a development build that performs lexical analysis and parsing.");
        println!("It does not yet perform type checking or code generation.");
        println!();
        println!("Examples:");
        println!("  nova hello.nv          -- Parse hello.nv");
        println!("  nova -c <code>         -- Parse code snippet");
        return;
    }

    // Check for direct code parsing
    if filename == "-c" && args.len() > 2 {
        let code = &args[2];
        if let Err(e) = parse_code(code, "<input>") {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
        return;
    }

    // Read file
    let source = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", filename, e);
            process::exit(1);
        }
    };

    if let Err(e) = parse_code(&source, filename) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn parse_code(source: &str, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Lexical analysis
    let mut lexer = Lexer::new(source, 0);
    let tokens = lexer.tokenize()?;

    println!("✓ Lexer: {} tokens", tokens.len());

    // Parsing
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program()?;

    println!("✓ Parser: {} top-level items", program.items.len());

    // Print AST summary
    for (i, item) in program.items.iter().enumerate() {
        match item {
            nova_compiler::TopLevelItem::MissionDecl(m) => {
                println!("  [{}] mission {} : {:?}", i, m.name, m.return_type);
            }
            nova_compiler::TopLevelItem::ParallelMissionDecl(m) => {
                println!("  [{}] parallel mission {} : {:?}", i, m.name, m.return_type);
            }
            nova_compiler::TopLevelItem::ConstellationDecl(c) => {
                println!("  [{}] constellation {}", i, c.name);
            }
            nova_compiler::TopLevelItem::ModelDecl(m) => {
                println!("  [{}] model {} ({} layers)", i, m.name, m.layers.len());
            }
            nova_compiler::TopLevelItem::StructDecl(s) => {
                println!("  [{}] struct {} ({} fields)", i, s.name, s.fields.len());
            }
            nova_compiler::TopLevelItem::EnumDecl(e) => {
                println!("  [{}] enum {} ({} variants)", i, e.name, e.variants.len());
            }
            nova_compiler::TopLevelItem::UnitDecl(u) => {
                println!("  [{}] unit {}", i, u.name);
            }
            nova_compiler::TopLevelItem::TestMission(m) => {
                println!("  [{}] test mission {}", i, m.name);
            }
        }
    }

    println!();
    println!("Parse successful: {}", filename);

    Ok(())
}
