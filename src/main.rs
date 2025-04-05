use borf::errors::BorfError;
use borf::parser::ast::{Declaration, Expr, ReplInput};
use borf::{evaluate_module, parse_file, parse_repl_input, process_prelude_directory};
use miette::Report;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::env;
use std::error::Error;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Borf v0.0.1 REPL");
    println!("Enter Borf code (declarations or expressions), or :q to quit.");

    // Determine prelude directory path
    let mut current_dir = env::current_dir()?;
    current_dir.push("src");
    current_dir.push("prelude");
    let prelude_dir_path = current_dir.to_str().ok_or("Invalid prelude path")?;

    // Load prelude
    println!("Loading prelude from: {}...", prelude_dir_path);
    let evaluator = match process_prelude_directory(prelude_dir_path) {
        Ok(eval) => {
            println!("Prelude loaded successfully.");
            eval
        }
        Err(e) => {
            let report = Report::new(e);
            eprintln!("Fatal Error loading prelude:\n{:?}", report);
            return Err(Box::new(miette::MietteError::Report(report)));
        }
    };

    let mut rl = DefaultEditor::new()?;

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                let _ = rl.add_history_entry(line.as_str());
                let trimmed_line = line.trim();
                if trimmed_line == ":quit" || trimmed_line == ":q" {
                    break;
                }
                if trimmed_line.is_empty() {
                    continue;
                }

                match parse_repl_input(&line) {
                    Ok(repl_input) => match repl_input {
                        ReplInput::Expression(expr) => {
                            println!("Parsed Expression (eval not implemented): {:?}", expr);
                        }
                        ReplInput::Declaration(decl) => {
                            println!(
                                "Parsed Declaration (processing not implemented): {:?}",
                                decl
                            );
                        }
                    },
                    Err(e) => {
                        let report = Report::new(e);
                        eprintln!("{:?}", report);
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                eprintln!("REPL Error: {:?}", err);
                break;
            }
        }
    }

    Ok(())
}

#[allow(dead_code)] // Allow dead code for now, might be used later for file mode
fn run(
    file_path: &str,
    parse_only: bool,
    prelude_dir: Option<&String>,
) -> Result<(), Box<dyn Error>> {
    println!("Processing file: {}", file_path);

    if let Some(dir) = prelude_dir {
        println!("Loading prelude files from: {}", dir);
        process_prelude_directory(dir).map_err(|e| e.to_string())?;
    }

    let parsed_module = match parse_file(file_path) {
        Ok(module) => {
            println!("Successfully parsed module: {}", module.name);
            module
        }
        Err(e) => {
            return Err(format!("Failed to parse: {}", e).into());
        }
    };

    if parse_only {
        println!("Parsed module:\n{:?}", parsed_module);
        return Ok(());
    }

    println!("Evaluating module...");
    match evaluate_module(&parsed_module) {
        Ok(_result) => {
            println!("Evaluation successful.");
            println!("Module: {}", parsed_module.name);
            println!("Module declarations: {}", parsed_module.declarations.len());
            Ok(())
        }
        Err(e) => Err(format!("Evaluation error: {}", e).into()),
    }
}
