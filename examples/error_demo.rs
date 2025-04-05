use borf::error_reporting::{create_enhanced_report, print_error_message};
use std::io::{self, Write};
fn main() {
    // Process command-line arguments, if any
    let file_path = std::env::args().nth(1);

    // If a file path was provided, parse that file
    if let Some(path) = file_path {
        parse_file(&path);
    } else {
        // Otherwise, run interactive mode
        interactive_mode();
    }
}

fn parse_file(path: &str) {
    println!("Parsing file: {}", path);

    match borf::parse_file_with_enhanced_errors(path) {
        Ok(module) => {
            println!("✅ Successfully parsed module: {}", module.name);
            println!("Module contains {} declarations", module.declarations.len());
        }
        Err(error) => {
            // Display a rich error with context, suggestions, and source code
            println!("\n🚫 Parsing failed with error:\n");

            // Option 1: Use miette's fancy formatting (best for terminal)
            let report = create_enhanced_report(error.clone());
            println!("{:?}", report);

            // Option 2: Use our custom colored formatting
            println!("\nAlternative error format:");
            print_error_message(&error);
        }
    }
}

fn interactive_mode() {
    println!("📝 Borf Interactive Parser with Enhanced Error Reporting");
    println!("Type Borf code to parse it, or type 'exit' to quit");
    println!("-----------------------------------------------------");

    let mut input = String::new();
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        print!("> ");
        stdout.flush().unwrap();

        input.clear();
        if stdin.read_line(&mut input).unwrap() == 0 || input.trim() == "exit" {
            break;
        }

        // Skip empty lines
        if input.trim().is_empty() {
            continue;
        }

        // Simple module wrapper for snippets
        let wrapped_input = format!("@Test: {{\n{}\n}}", input);

        match borf::error_reporting::parse_string_to_module_with_enhanced_errors(
            &wrapped_input,
            Some("interactive".to_string()),
        ) {
            Ok(_module) => {
                println!("✅ Successfully parsed!");
            }
            Err(error) => {
                // Display error
                println!("\n🚫 Parsing failed:\n");

                // Option 1: Use miette's fancy formatting
                let report = create_enhanced_report(error.clone());
                println!("{:?}", report);

                // Option 2: Use custom colored formatting
                println!("\nAlternative error format:");
                print_error_message(&error);
            }
        }

        println!(); // Add blank line between interactions
    }
}
