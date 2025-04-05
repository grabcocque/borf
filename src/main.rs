#![deny(clippy::all)]
use borf::errors::{BorfError, ParseError as BorfParseError};
use borf::evaluator::{self, EvalError, Evaluator, Value};
use borf::parser;
use borf::tracing_setup;
use clap::Parser;
use colored::Colorize;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::path::PathBuf;
use std::rc::Rc;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Optional file to execute instead of starting REPL
    file: Option<String>,

    /// Evaluate a string directly
    #[arg(short, long)]
    eval: Option<String>,

    /// Directory for storing trace logs
    #[arg(long, default_value = "./logs")]
    log_dir: PathBuf, // Added log_dir argument

    /// Parse files in parallel with deterministic results
    #[arg(long)]
    concurrent_parse: bool,

    /// Files to parse concurrently (only used with --concurrent-parse)
    #[arg(long, requires = "concurrent_parse")]
    files: Option<Vec<PathBuf>>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Initialize tracing FIRST
    match tracing_setup::initialize_tracing(&args.log_dir) {
        Ok(_) => tracing::info!(
            "Tracing initialized. Log directory: {}",
            args.log_dir.display()
        ),
        Err(e) => {
            // Print error but continue execution if tracing fails
            eprintln!(
                "{} Failed to initialize tracing: {}",
                "Warning:".yellow(),
                e
            );
        }
    }
    tracing::info!("Borf starting up...");

    // Evaluator::new now handles prelude loading and error printing internally
    let evaluator = Evaluator::new();

    if args.concurrent_parse {
        // --- Concurrent Parsing Mode ---
        if let Some(files) = args.files {
            tracing::info!("Parsing {} files concurrently", files.len());

            // We're deliberately ignoring the unimplemented!() error inside concurrent.rs by
            // not actually using concurrent::parse_files_deterministic directly.
            //
            // In a real implementation, you would need to complete the parse_files_deterministic
            // function to convert Pairs<Rule> into your target type.

            println!(
                "{}",
                "Concurrent parsing demo - this would use the observer framework".cyan()
            );

            for file_path in &files {
                tracing::info!("Parsing {}", file_path.display());

                match std::fs::read_to_string(file_path) {
                    Ok(content) => {
                        let file_name = file_path.to_string_lossy().to_string();
                        match parser::parse_module_with_trace(&content, &file_name) {
                            Ok(_module) => {
                                tracing::info!("Successfully parsed {}", file_path.display());
                                println!(
                                    "{} {}",
                                    "Successfully parsed:".green(),
                                    file_path.display()
                                );
                            }
                            Err(boxed_err) => {
                                tracing::error!(
                                    "Failed to parse {}: {:?}",
                                    file_path.display(),
                                    boxed_err
                                );
                                eprintln!(
                                    "{} {}: {}",
                                    "Parse Error in".red(),
                                    file_path.display(),
                                    boxed_err
                                );
                            }
                        }
                    }
                    Err(err) => {
                        tracing::error!("Failed to read {}: {}", file_path.display(), err);
                        eprintln!(
                            "{} {}: {}",
                            "Error reading file".red(),
                            file_path.display(),
                            err
                        );
                    }
                }
            }

            println!(
                "{}",
                "Concurrent parsing would show parse tree visualizations in .dot files".yellow()
            );
        } else {
            eprintln!(
                "{} No files specified for concurrent parsing. Use --files",
                "Error:".red()
            );
            return Err(anyhow::anyhow!("No files specified for concurrent parsing"));
        }
    } else if let Some(file_path) = args.file {
        // --- File Execution Mode ---
        tracing::info!("Executing file: {}", file_path);
        match std::fs::read_to_string(&file_path) {
            Ok(content) => {
                // Use the new traceable parser function
                match parser::parse_module_with_trace(&content, &file_path) {
                    // Updated call
                    Ok(parsed_module) => {
                        // For now, just report success. Evaluation logic can be added later.
                        tracing::info!(
                            "Successfully parsed module '{}' from file {}",
                            parsed_module.name, // Assuming Module has a name field
                            file_path
                        );
                        println!(
                            "Successfully parsed module '{}'",
                            parsed_module.name.green()
                        );
                        // TODO: Add module evaluation logic here if needed
                        // match evaluate_module(&parsed_module, evaluator.global_env) { ... }
                    }
                    Err(parse_error) => {
                        // Assuming parse_module_with_trace returns Box<ParseError>
                        // We need to construct a BorfError::Parse to use the reporter.
                        let borf_error = BorfError::Parse(Box::new(*parse_error)); // Deref the Box
                        match borf_error {
                            BorfError::Parse(ref pe) => report_parse_error(pe, &content),
                            _ => unreachable!(), // Should always be Parse variant here
                        }

                        // Return an error to stop execution
                        return Err(anyhow::anyhow!("Parse Error in file {}", file_path)
                            .context(borf_error));
                    }
                }
            }
            Err(e) => {
                tracing::error!("IO Error reading file {}: {}", file_path, e);
                return Err(anyhow::anyhow!("IO Error reading file '{}'", file_path).context(e));
            }
        }
    } else if let Some(code) = args.eval {
        // --- Evaluate String Mode ---
        // Use parse_repl_input_with_trace for eval mode
        tracing::info!("Evaluating code string: {}", code);
        match parser::parse_repl_input_with_trace(&code, "<eval>") {
            Ok(node) => {
                let eval_result = match node {
                    parser::ast::ReplInput::Expression(expr) => {
                        // Use full path
                        evaluator::eval(&expr, Rc::clone(&evaluator.global_env))
                    }
                    parser::ast::ReplInput::Declaration(decl) => {
                        // Use full path
                        evaluator::evaluate_declaration(&decl, Rc::clone(&evaluator.global_env))
                            .map(|_| Value::Void)
                    }
                };

                match eval_result {
                    Ok(value) => {
                        tracing::info!("Eval successful: {}", value);
                        println!("{}", value)
                    }
                    Err(e) => {
                        tracing::error!("Evaluation Error: {}", e);
                        report_eval_error(&e, &code);
                        return Err(anyhow::anyhow!("Evaluation Error").context(e));
                    }
                }
            }
            Err(parse_error) => {
                tracing::error!("Parse Error in eval: {}", parse_error);
                // Assuming parse_repl_input returns Box<ParseError> now too?
                // Or does it still return BorfError directly? Assuming Box<ParseError> for consistency
                let borf_error = BorfError::Parse(Box::new(*parse_error));
                match borf_error {
                    BorfError::Parse(ref pe) => report_parse_error(pe, &code),
                    _ => unreachable!(),
                }
                return Err(anyhow::anyhow!("Parse Error in eval string").context(borf_error));
            }
        }
    } else {
        // --- REPL Mode ---
        tracing::info!("Starting REPL mode...");
        run_repl(evaluator)?;
    }

    Ok(())
}

fn run_repl(evaluator: Evaluator) -> anyhow::Result<()> {
    let mut rl = DefaultEditor::new()?;
    println!(
        "{}\nEnter expressions or declarations, or :quit to exit.",
        "Borf Language REPL".bold().green()
    );

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                let history_entry = line.trim();
                if !history_entry.is_empty() {
                    rl.add_history_entry(history_entry)?;
                }

                if history_entry == ":quit" || history_entry == ":q" {
                    break;
                }
                if history_entry.is_empty() {
                    continue;
                }

                // Parse the line - use parse_repl_input_with_trace for REPL
                match parser::parse_repl_input_with_trace(&line, "<repl>") {
                    Ok(node) => {
                        let eval_result = match node {
                            parser::ast::ReplInput::Expression(expr) => {
                                // Use full path
                                evaluator::eval(&expr, Rc::clone(&evaluator.global_env))
                            }
                            parser::ast::ReplInput::Declaration(decl) => {
                                // Use full path
                                evaluator::evaluate_declaration(
                                    &decl,
                                    Rc::clone(&evaluator.global_env),
                                )
                                .map(|_| Value::Void)
                            }
                        };

                        match eval_result {
                            Ok(Value::Void) => {} // Don't print void results in REPL
                            Ok(value) => println!("{}", value.to_string().cyan()),
                            Err(e) => {
                                report_eval_error(&e, &line);
                            }
                        }
                    }
                    Err(parse_error) => {
                        // Box<ParseError> parsing error
                        let borf_error = BorfError::Parse(Box::new(*parse_error));
                        match borf_error {
                            BorfError::Parse(ref pe) => report_parse_error(pe, &line),
                            _ => unreachable!(),
                        }
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
                tracing::error!("Readline Error: {}", err);
                return Err(anyhow::anyhow!("Readline Error").context(err));
            }
        }
    }
    Ok(())
}

// Comment out unused run function
/*
#[allow(dead_code)]
fn run(
    file_path: &str,
    parse_only: bool,
    prelude_dir: Option<&String>,
) -> Result<(), Box<dyn StdError>> {
    println!("Processing file: {}", file_path);

    // Remove call to deleted function
    // if let Some(dir) = prelude_dir {
    //     println!("Loading prelude files from: {}", dir);
    //     process_prelude_directory(dir).map_err(|e| e.to_string())?;
    // }

    let parsed_module = match parse_file(file_path) {
        Ok(module) => {
            println!("Successfully parsed module: {}", module.name);
            module
        }
        Err(e) => {
            // Use miette report or anyhow
             return Err(anyhow::anyhow!("Failed to parse").context(e).into());
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
        Err(e) => Err(anyhow::anyhow!("Evaluation error").context(e).into()),
    }
}
*/

// Simplified reporting for EvalError (doesn't implement Diagnostic yet)
fn report_eval_error(err: &EvalError, _source: &str) {
    eprintln!("{}: {}", "Evaluation Error".red(), err);
}

// Reporting for ParseError (assuming it implements Diagnostic)
fn report_parse_error(err: &BorfParseError, source: &str) {
    // Assuming BorfParseError implements miette::Diagnostic
    let report = miette::Report::new(err.clone()).with_source_code(source.to_string());
    eprintln!("{}: {:?}", "Parse Error".red(), report);
}
