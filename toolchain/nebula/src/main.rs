// NOVA Nebula Package Manager
// Main entry point for the nebula CLI tool

use std::env;
use std::path::PathBuf;

mod resolver;

fn print_help() {
    println!("Nebula - NOVA Package Manager");
    println!();
    println!("USAGE:");
    println!("    nebula <COMMAND> [OPTIONS]");
    println!();
    println!("COMMANDS:");
    println!("    new <name>          Create a new NOVA project");
    println!("    build               Build the current project");
    println!("    run                 Build and run the current project");
    println!("    add <constellation> Add a dependency");
    println!("    test                Run tests");
    println!("    clean               Clean build artifacts");
    println!("    help                Show this help message");
    println!();
    println!("EXAMPLES:");
    println!("    nebula new my_project");
    println!("    nebula add cosmos.stats");
    println!("    nebula build");
    println!("    nebula run");
}

fn create_new_project(name: &str) -> Result<(), String> {
    let project_dir = PathBuf::from(name);
    
    if project_dir.exists() {
        return Err(format!("Directory '{}' already exists", name));
    }
    
    std::fs::create_dir_all(&project_dir)
        .map_err(|e| format!("Failed to create directory: {}", e))?;
    
    // Create nova.toml manifest
    let manifest_content = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
authors = [""]
description = ""

[dependencies]
"#,
        name
    );
    
    std::fs::write(project_dir.join("nova.toml"), manifest_content)
        .map_err(|e| format!("Failed to create nova.toml: {}", e))?;
    
    // Create src directory
    let src_dir = project_dir.join("src");
    std::fs::create_dir_all(&src_dir)
        .map_err(|e| format!("Failed to create src directory: {}", e))?;
    
    // Create main.nv
    let main_content = r#"mission main() -> Void {
    transmit("Hello from NOVA!")
}
"#;
    
    std::fs::write(src_dir.join("main.nv"), main_content)
        .map_err(|e| format!("Failed to create main.nv: {}", e))?;
    
    // Create tests directory
    let tests_dir = project_dir.join("tests");
    std::fs::create_dir_all(&tests_dir)
        .map_err(|e| format!("Failed to create tests directory: {}", e))?;
    
    println!("Created NOVA project '{}'", name);
    println!("  - nova.toml (manifest)");
    println!("  - src/main.nv (entry point)");
    println!("  - tests/ (test files)");
    println!();
    println!("Next steps:");
    println!("  cd {}", name);
    println!("  nebula build");
    
    Ok(())
}

fn build_project() -> Result<(), String> {
    let current_dir = env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {}", e))?;
    
    let manifest_path = current_dir.join("nova.toml");
    if !manifest_path.exists() {
        return Err("No nova.toml found. Are you in a NOVA project?".to_string());
    }
    
    println!("Building project...");
    
    // TODO: Integrate with actual compiler pipeline
    // For now, just check that source files exist
    let src_dir = current_dir.join("src");
    if !src_dir.exists() {
        return Err("No src/ directory found".to_string());
    }
    
    let main_nv = src_dir.join("main.nv");
    if !main_nv.exists() {
        return Err("No src/main.nv found".to_string());
    }
    
    println!("Found source files:");
    if let Ok(entries) = std::fs::read_dir(&src_dir) {
        for entry in entries.flatten() {
            if entry.path().extension().map_or(false, |ext| ext == "nv") {
                println!("  - {}", entry.path().display());
            }
        }
    }
    
    println!("Build configuration loaded (compiler integration pending)");
    Ok(())
}

fn run_project() -> Result<(), String> {
    build_project()?;
    println!("Running project...");
    // TODO: Execute compiled binary
    println!("Execution not yet implemented");
    Ok(())
}

fn add_dependency(dep: &str) -> Result<(), String> {
    let current_dir = env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {}", e))?;
    
    let manifest_path = current_dir.join("nova.toml");
    if !manifest_path.exists() {
        return Err("No nova.toml found. Are you in a NOVA project?".to_string());
    }
    
    println!("Adding dependency: {}", dep);
    
    // Read existing manifest
    let mut content = std::fs::read_to_string(&manifest_path)
        .map_err(|e| format!("Failed to read nova.toml: {}", e))?;
    
    // Check if dependency already exists
    if content.contains(&format!("{} =", dep)) {
        return Err(format!("Dependency '{}' already exists", dep));
    }
    
    // Add dependency
    if !content.ends_with('\n') {
        content.push('\n');
    }
    content.push_str(&format!("{} = \"latest\"\n", dep));
    
    std::fs::write(&manifest_path, content)
        .map_err(|e| format!("Failed to write nova.toml: {}", e))?;
    
    println!("Added '{}' to dependencies", dep);
    Ok(())
}

fn clean_project() -> Result<(), String> {
    let current_dir = env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {}", e))?;
    
    let build_dir = current_dir.join("build");
    let target_dir = current_dir.join("target");
    
    if build_dir.exists() {
        std::fs::remove_dir_all(&build_dir)
            .map_err(|e| format!("Failed to remove build/: {}", e))?;
        println!("Removed build/");
    }
    
    if target_dir.exists() {
        std::fs::remove_dir_all(&target_dir)
            .map_err(|e| format!("Failed to remove target/: {}", e))?;
        println!("Removed target/");
    }
    
    println!("Clean complete");
    Ok(())
}

fn run_tests() -> Result<(), String> {
    let current_dir = env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {}", e))?;
    
    let manifest_path = current_dir.join("nova.toml");
    if !manifest_path.exists() {
        return Err("No nova.toml found. Are you in a NOVA project?".to_string());
    }
    
    let tests_dir = current_dir.join("tests");
    if !tests_dir.exists() {
        return Err("No tests/ directory found".to_string());
    }
    
    println!("Running tests...");
    
    let mut test_count = 0;
    if let Ok(entries) = std::fs::read_dir(&tests_dir) {
        for entry in entries.flatten() {
            if entry.path().extension().map_or(false, |ext| ext == "nv") {
                println!("  Found test: {}", entry.path().display());
                test_count += 1;
            }
        }
    }
    
    if test_count == 0 {
        println!("No test files found");
    } else {
        println!("Found {} test file(s)", test_count);
        println!("Test execution not yet implemented");
    }
    
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_help();
        std::process::exit(1);
    }
    
    let command = &args[1];
    
    let result = match command.as_str() {
        "new" => {
            if args.len() < 3 {
                Err("Usage: nebula new <project-name>".to_string())
            } else {
                create_new_project(&args[2])
            }
        }
        "build" => build_project(),
        "run" => run_project(),
        "add" => {
            if args.len() < 3 {
                Err("Usage: nebula add <constellation>".to_string())
            } else {
                add_dependency(&args[2])
            }
        }
        "test" => run_tests(),
        "clean" => clean_project(),
        "help" | "--help" | "-h" => {
            print_help();
            Ok(())
        }
        _ => Err(format!("Unknown command: {}. Use 'nebula help' for usage.", command)),
    };
    
    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}