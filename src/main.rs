// Use the library crate
use borf::parser::{self, BorfParser, Rule, TopLevelItem};
use pest::Parser;
use std::env;
use std::fs;
use std::path::Path;

const PRELUDE_PATH: &str = "src/prelude/mod.borf";

// Simple function to test direct parse
fn test_direct_parse(input: &str, rule: Rule) -> bool {
    println!("Testing direct parse of rule {:?}", rule);
    println!("Input: {}", input);
    let result = BorfParser::parse(rule, input);
    match result {
        Ok(_) => {
            println!("Success!");
            true
        }
        Err(e) => {
            println!("Failed: {:?}", e);
            false
        }
    }
}

fn test_category_parsing() {
    println!("\n--- Testing Category Parsing ---");

    // Test just the object declaration first
    println!("Testing object_decl parsing:");
    let test_obj = "A";
    test_direct_parse(test_obj, Rule::object_decl);

    // Test mapping declaration
    println!("Testing mapping_decl parsing:");
    let test_mapping = "f: A $to B";
    test_direct_parse(test_mapping, Rule::mapping_decl);

    // Test declaration with semicolon
    println!("Testing declaration parsing:");
    let test_decl = "A;";
    test_direct_parse(test_decl, Rule::object_decl);

    // Test category content (now as part of a minimal statement)
    println!("Testing category content parsing:");
    let test_content = "@Test: { A; B; f: A $to B; }";
    test_direct_parse(test_content, Rule::category_statement);

    // Test very simple category
    println!("Testing minimal category parsing:");
    let test_category = "@Simple: {}";
    test_direct_parse(test_category, Rule::category_statement);

    // Now test a category with some content
    println!("Testing simple category statement parsing:");
    let simple_category = "@Category: { A; B; f: A $to B; }";
    if test_direct_parse(simple_category, Rule::category_statement) {
        println!("Simple category parses correctly!");
    } else {
        println!("Simple category fails to parse!");
    }

    // Test in a full program context
    println!("Testing simple category in a program context:");
    if test_direct_parse(simple_category, Rule::statement) {
        println!("Simple category statement parses correctly!");
    } else {
        println!("Simple category statement fails to parse as a statement!");
    }

    if test_direct_parse(simple_category, Rule::program) {
        println!("Simple category program parses correctly!");
    } else {
        println!("Simple category program fails to parse!");
    }
}

fn load_and_parse_prelude() -> Result<Vec<TopLevelItem>, String> {
    println!("Loading prelude from: {}", PRELUDE_PATH);
    let file_path = Path::new(PRELUDE_PATH);

    match fs::read_to_string(file_path) {
        Ok(prelude_content) => {
            println!("Parsing prelude...");

            // Print the first few lines for debugging
            let first_few_lines: Vec<&str> = prelude_content.lines().take(30).collect();
            println!("First 30 lines of prelude:");
            for (i, line) in first_few_lines.iter().enumerate() {
                println!("[{:2}] {}", i + 1, line);
            }

            // Try to parse the file directly with the BorfParser
            let parse_result = BorfParser::parse(Rule::program, &prelude_content);

            match parse_result {
                Ok(mut pairs) => {
                    let program = pairs.next().unwrap();
                    println!("Successfully parsed with PEG grammar!");
                    println!("Got program: {:?}", program.as_rule());

                    // Continue with the normal parsing
                    parser::parse_program(&prelude_content)
                        .map_err(|e| format!("Prelude parsing failed: {:?}", e))
                }
                Err(e) => {
                    println!("PEG grammar parse failed: {:?}", e);
                    Err(format!("PEG grammar parse failed: {:?}", e))
                }
            }
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

    // First run our dedicated parsing tests
    test_category_parsing();

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
