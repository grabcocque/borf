// Use the library crate
use borf::parser::{self, TopLevelItem};
use std::env;
use std::fs;
use std::path::Path;

const PRELUDE_PATH: &str = "src/prelude/mod.borf";

fn load_and_parse_prelude() -> Result<Vec<TopLevelItem>, String> {
    println!("Loading prelude from: {}", PRELUDE_PATH);
    let file_path = Path::new(PRELUDE_PATH);

    match fs::read_to_string(file_path) {
        Ok(prelude_content) => {
            println!("Parsing prelude...");
            parser::parse_program(&prelude_content)
                .map_err(|e| format!("Prelude parsing failed: {:?}", e))
        }
        Err(e) => Err(format!(
            "Failed to read prelude file {}: {}",
            file_path.display(),
            e
        )),
    }
}

fn main() {
    println!("Hello from Borf binary!");

    // --- Load and Parse Prelude ---
    let prelude_definitions = match load_and_parse_prelude() {
        Ok(items) => {
            println!("Successfully parsed {} prelude item(s)!", items.len());
            // TODO: Store prelude definitions in a context/environment
            // TODO: Process the export directive `...>>` if needed
            Some(items)
        }
        Err(e) => {
            println!("Error loading prelude: {}", e);
            // Decide if execution should halt if prelude fails
            None
        }
    };

    // --- Process Command Line Argument File ---
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let file_path = &args[1];
        println!("\nProcessing user file: {}", file_path);

        match fs::read_to_string(file_path) {
            Ok(input) => {
                match parser::parse_program(&input) {
                    Ok(user_items) => {
                        println!("Successfully parsed {} user item(s)!", user_items.len());
                        for (i, item) in user_items.iter().enumerate() {
                            match item {
                                TopLevelItem::Category(_) => {
                                    println!("  {}. Category definition", i + 1)
                                }
                                TopLevelItem::PipeExpr(_) => {
                                    println!("  {}. Pipe expression", i + 1)
                                }
                                TopLevelItem::AppExpr(_) => {
                                    println!("  {}. Application expression", i + 1)
                                }
                                TopLevelItem::CompositionExpr(_) => {
                                    println!("  {}. Composition expression", i + 1)
                                }
                                TopLevelItem::Pipeline(_) => {
                                    println!("  {}. Pipeline definition", i + 1)
                                }
                                TopLevelItem::Export(_) => {
                                    println!("  {}. Export directive", i + 1)
                                }
                            }
                        }
                        // TODO: Combine user_items with prelude_definitions for further processing
                    }
                    Err(e) => println!("Parse error in {}: {:?}", file_path, e),
                }
            }
            Err(e) => println!("Error reading file {}: {}", file_path, e),
        }
    } else if prelude_definitions.is_some() {
        println!("\nPrelude loaded successfully. No user file provided.");
        println!("Usage: borf [file_path]");
    }
}
