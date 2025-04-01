// Use the library crate
use borf::parser;
use std::env;
use std::fs;

fn main() {
    println!("Hello from Borf binary!");

    // Get command line arguments
    let args: Vec<String> = env::args().collect();

    // If a file path is provided, try to parse it
    if args.len() > 1 {
        let file_path = &args[1];

        // Read the file contents
        match fs::read_to_string(file_path) {
            Ok(input) => {
                println!("Parsing file: {}", file_path);

                // Try to parse the file
                match parser::parse_program(&input) {
                    Ok(defs) => {
                        println!("Successfully parsed {} definition(s)!", defs.len());
                        for (i, def) in defs.iter().enumerate() {
                            match def {
                                parser::BorfDefinition::ACSet(_) => {
                                    println!("  {}. ACSet definition", i + 1)
                                }
                                parser::BorfDefinition::WireDgm(_) => {
                                    println!("  {}. WireDgm definition", i + 1)
                                }
                                parser::BorfDefinition::INet(_) => {
                                    println!("  {}. INet definition", i + 1)
                                }
                                parser::BorfDefinition::PipeExpr(_) => {
                                    println!("  {}. Pipe expression", i + 1)
                                }
                                parser::BorfDefinition::AppExpr(_) => {
                                    println!("  {}. Application expression", i + 1)
                                }
                                parser::BorfDefinition::CompositionExpr(_) => {
                                    println!("  {}. Composition expression", i + 1)
                                }
                            }
                        }
                    }
                    Err(e) => println!("Parse error: {:?}", e),
                }
            }
            Err(e) => println!("Error reading file {}: {}", file_path, e),
        }
    } else {
        println!("Usage: borf <file_path>");
    }
}
