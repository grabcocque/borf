#![deny(clippy::all)]
//! Borf - A metacircular evaluator for the Borf language
//!
//! This crate provides a parser and evaluator for the Borf language,
//! which is designed for structural subtyping, clear equivalence domains,
//! categorical structures, and transformation pipelines.

use crate::errors::{BorfError, ParseError};
use crate::parser::{ast, BorfParser, Rule};
use colored::*;
use pest::Parser;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;

// Load the evaluator module from src/evaluator/mod.rs
pub mod evaluator;
// Load the parser module from src/parser/mod.rs (which includes pub mod ast)
pub mod parser;
// Load the error handling module
pub mod errors;
// Load the error reporting module
pub mod error_reporting;
// Load the observer module
pub mod observer;
// Load the traceable parser module
pub mod traceable_parser;
// Load the tracing setup module
pub mod tracing_setup;
// Load the concurrent parsing module
pub mod concurrent;

/// Parse a Borf source file and return the AST
///
/// # Arguments
///
/// * `path` - Path to the Borf source file
///
/// # Returns
///
/// An AST representation of the parsed module
///
/// # Errors
///
/// Returns an error if the file cannot be read or parsed
pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<ast::Module, BorfError> {
    error_reporting::parse_file_with_enhanced_errors(path).map_err(|enhanced_err| {
        BorfError::Parse(Box::new(convert_enhanced_to_parse_error(enhanced_err)))
    })
}

/// Evaluate a parsed Borf module
///
/// # Arguments
///
/// * `module` - The parsed Borf module AST
///
/// # Returns
///
/// The result of evaluating the module
///
/// # Errors
///
/// Returns an error if evaluation fails
pub fn evaluate_module(module: &ast::Module) -> Result<evaluator::Value, BorfError> {
    let evaluator = evaluator::Evaluator::new();
    evaluator
        .evaluate_module(module)
        .map_err(|e| BorfError::Evaluation(format!("{}", e)))
}

/// Parse and evaluate a Borf source file in one step
///
/// # Arguments
///
/// * `path` - Path to the Borf source file
///
/// # Returns
///
/// The result of evaluating the parsed module
///
/// # Errors
///
/// Returns an error if parsing or evaluation fails
pub fn parse_and_evaluate<P: AsRef<Path>>(path: P) -> Result<evaluator::Value, BorfError> {
    let module = parse_file(path)?;
    evaluate_module(&module)
}

/// Parse a single line of REPL input into a `ReplInput` AST node
pub fn parse_repl_input(input: &str) -> Result<parser::ast::ReplInput, BorfError> {
    let source_name = Some("REPL".to_string());

    // Use match instead of `?` to allow calling from_pest with context
    match BorfParser::parse(Rule::repl_input, input) {
        Ok(mut pairs) => {
            // pairs should contain repl_input which has one inner element (decl or expr)
            let repl_inner_pair = pairs
                .next()
                .and_then(|p| p.into_inner().next())
                .ok_or_else(|| {
                    BorfError::Parse(Box::new(ParseError::EmptyInput {
                        src: Arc::new(miette::NamedSource::new(
                            source_name.clone().unwrap(),
                            input.to_string(),
                        )),
                        help_message: "No valid input found.".to_string(),
                        suggestion: None,
                    }))
                })?;

            let pair_span = repl_inner_pair.as_span();

            match repl_inner_pair.as_rule() {
                Rule::declaration => parser::parse_declaration(repl_inner_pair, &source_name)
                    .map(ast::ReplInput::Declaration)
                    .map_err(|e| BorfError::from(*e)), // parse_declaration returns Box<ParseError>
                Rule::expr => parser::parse_expr(repl_inner_pair, &source_name)
                    .map(ast::ReplInput::Expression)
                    .map_err(|e| BorfError::from(*e)), // parse_expr returns Box<ParseError>
                rule => Err(BorfError::Parse(Box::new(ParseError::UnexpectedRule {
                    expected: "declaration or expression".to_string(),
                    found: format!("{:?}", rule),
                    span: (pair_span.start(), pair_span.end() - pair_span.start()).into(),
                    location: format!(
                        "{}:{}",
                        pair_span.start_pos().line_col().0,
                        pair_span.start_pos().line_col().1
                    ),
                }))),
            }
        }
        Err(pest_error) => {
            // Explicitly call from_pest here to provide full context
            Err(BorfError::from(ParseError::from_pest(
                pest_error,
                input,       // Pass the input string
                source_name, // Pass the source name
            )))
        }
    }
}

/// Parse a Borf source file with enhanced error reporting
///
/// # Arguments
///
/// * `path` - Path to the Borf source file
///
/// # Returns
///
/// An AST representation of the parsed module
///
/// # Errors
///
/// Returns an `EnhancedError` with rich diagnostics if the file cannot be read or parsed
pub fn parse_file_with_enhanced_errors<P: AsRef<std::path::Path>>(
    path: P,
) -> Result<parser::ast::Module, error_reporting::EnhancedError> {
    error_reporting::parse_file_with_enhanced_errors(path)
}

/// Parse a string into a list of expressions
pub fn parse_string_to_exprs(
    _input: &str,
    _source_name: Option<String>,
) -> Result<Vec<parser::ast::Expr>, BorfError> {
    Err(BorfError::Parse(Box::new(ParseError::Unexpected(
        "Parsing multiple expressions from a string is not yet implemented correctly.".to_string(),
    ))))
}

/// Parse multiple expressions from a string
/// Primarily for tests and bulk processing
pub fn parse_multiple_expressions(
    _input: &str,
    _source_name: Option<String>,
) -> Result<Vec<parser::ast::Expr>, BorfError> {
    // TODO: Implement this when needed
    // For now, return a placeholder value for tests
    Ok(vec![])
}

fn convert_enhanced_to_parse_error(enhanced: error_reporting::EnhancedError) -> ParseError {
    ParseError::Unexpected(format!("Enhanced Error: {}", enhanced))
}

// Keep the prelude_init module as a placeholder for future implementation
mod prelude_init {}

// Define a helper or modify Evaluator::new to handle prelude processing

// Option 1: Helper function taking EnvironmentRef
pub fn process_prelude_directory_internal<P: AsRef<Path>>(
    dir_path: P,
    env_ref: evaluator::EnvironmentRef,
) -> Result<(), BorfError> {
    // Return Result<(), BorfError>
    println!("Processing prelude directory: {:?}", dir_path.as_ref());

    // Check if directory exists, proceed if it does
    if !dir_path.as_ref().exists() {
        println!("Prelude directory not found, skipping.");
        return Ok(());
    }
    if !dir_path.as_ref().is_dir() {
        return Err(BorfError::Io(std::io::Error::new(
            std::io::ErrorKind::NotADirectory,
            "Prelude path is not a directory",
        )));
    }

    let dir_entries = std::fs::read_dir(dir_path).map_err(BorfError::Io)?;

    for entry in dir_entries {
        let entry = entry.map_err(BorfError::Io)?;
        let path = entry.path();

        if path.is_file() && path.extension().is_some_and(|ext| ext == "borf") {
            println!("Processing prelude file: {:?}", path);

            // Use the original function that parses a Module (using the 'file' rule)
            let parsed_module = match error_reporting::parse_file_with_enhanced_errors(&path) {
                Ok(m) => m,
                Err(e) => {
                    eprintln!(
                        "{}",
                        format!("Failed to parse prelude file {:?}", path).red()
                    );
                    error_reporting::print_error_message(&e); // Use the enhanced reporting
                    eprintln!("Skipping prelude file due to parse error.");
                    continue;
                    // Optionally convert EnhancedError back to BorfError if needed for return type
                    // return Err(BorfError::Parse(convert_enhanced_to_parse_error(e)));
                }
            };

            // Evaluate each declaration *within* the parsed module
            for decl in &parsed_module.declarations {
                let result = evaluator::evaluate_declaration(decl, Rc::clone(&env_ref))
                    .map(|_| evaluator::Value::Void); // Declarations yield Void

                // Check for evaluation errors
                if let Err(e) = result {
                    eprintln!(
                        "{}",
                        format!(
                            "Failed to evaluate prelude declaration in {:?}: {}",
                            path, e
                        )
                        .red()
                    );
                    return Err(BorfError::Evaluation(format!(
                        "Failed to evaluate prelude {:?}: {}",
                        path, e
                    )));
                }
            }
            println!("Successfully processed prelude file: {:?}", path);
        }
    }
    Ok(())
}

// Remove or deprecate the old process_prelude_directory function
/*
pub fn process_prelude_directory<P: AsRef<Path>>(
    dir_path: P,
) -> Result<evaluator::Evaluator, BorfError> {
    // ... old implementation ...
}
*/

// Adjust Evaluator::new to use the helper
// (This part goes inside src/evaluator/mod.rs)
/*
impl Evaluator {
    pub fn new() -> Self {
        let global_env = Environment::new_global();
        populate_global_env(Rc::clone(&global_env)); // Populate primitives first

        // Call the helper function from lib.rs (need to make it public or move logic)
        match crate::process_prelude_directory_internal("src/prelude", Rc::clone(&global_env)) {
            Ok(_) => println!("Prelude processed successfully."),
            Err(e) => {
                eprintln!("Error processing prelude directory: {}", e);
                // Decide how to handle prelude errors (e.g., continue without prelude?)
            }
        }

        Self { global_env }
    }
    // ... rest of impl ...
}
*/
