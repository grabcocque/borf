//! Borf - A metacircular evaluator for the Borf language
//!
//! This crate provides a parser and evaluator for the Borf language,
//! which is designed for structural subtyping, clear equivalence domains,
//! categorical structures, and transformation pipelines.

use crate::errors::{BorfError, ParseError};
use crate::parser::BorfParser;
use crate::parser::{ast, Rule};
use colored::*;
use miette;
use pest::Parser;
use std::fs;
use std::path::Path;

// Load the evaluator module from src/evaluator/mod.rs
pub mod evaluator;
// Load the parser module from src/parser/mod.rs (which includes pub mod ast)
pub mod parser;
// Load the error handling module
pub mod errors;
// Load the error reporting module
pub mod error_reporting;

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
    parser::parse_file(path).map_err(BorfError::from)
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
        // Use the Evaluation variant directly instead of ParseError
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

/// Process a directory of Borf prelude files
///
/// # Arguments
///
/// * `dir_path` - Path to the directory containing Borf prelude files
///
/// # Returns
///
/// An evaluator with all prelude files loaded
///
/// # Errors
///
/// Returns an error if any prelude file cannot be processed
pub fn process_prelude_directory<P: AsRef<Path>>(
    dir_path: P,
) -> Result<evaluator::Evaluator, BorfError> {
    let evaluator = evaluator::Evaluator::new();

    let dir_entries = std::fs::read_dir(dir_path).map_err(BorfError::Io)?;

    for entry in dir_entries {
        let entry = entry.map_err(BorfError::Io)?;
        let path = entry.path();

        if path.is_file() && path.extension().is_some_and(|ext| ext == "borf") {
            println!("Processing prelude file: {:?}", path);
            let file_content = fs::read_to_string(&path)?;
            // let source_name = path.to_string_lossy().to_string();

            let parsed_module =
                match parser::parse_string(&file_content, Some(path.to_string_lossy().to_string()))
                    .map_err(BorfError::from)
                {
                    Ok(m) => m,
                    Err(e) => {
                        // Return the specific parsing/conversion error
                        eprintln!(
                            "{}",
                            format!("Failed to parse prelude file {:?}", path).red()
                        );
                        return Err(e); // Propagate the BorfError
                    }
                };

            match evaluator.evaluate_module(&parsed_module) {
                Ok(_) => {}
                Err(e) => {
                    // TODO: Create a dedicated EvaluationError variant
                    eprintln!(
                        "{}",
                        format!("Failed to evaluate prelude file {:?}: {}", path, e).red()
                    );
                    return Err(BorfError::Parse(ParseError::Unexpected(format!(
                        "Failed to evaluate prelude file {:?}: {}",
                        path, e
                    ))));
                }
            }
        }
    }

    Ok(evaluator)
}

/// Parse a single line of REPL input into a `ReplInput` AST node
pub fn parse_repl_input(input: &str) -> Result<parser::ast::ReplInput, BorfError> {
    let source_name = Some("REPL".to_string());

    let mut parsed_pairs = parser::BorfParser::parse(Rule::repl_input, input)
        .map_err(|e| BorfError::from(ParseError::from_pest(e, input, source_name.clone())))?;

    let inner_pair = parsed_pairs
        .next() // Get the repl_input rule
        .and_then(|p| p.into_inner().next()) // Get the inner declaration or expr
        .ok_or_else(|| {
            // Create a more specific error
            let src = miette::NamedSource::new(
                source_name.unwrap_or_else(|| "REPL".to_string()),
                input.to_string(),
            );
            let span = (0, input.len().min(20)).into(); // Show just the beginning of input

            BorfError::Parse(ParseError::SyntaxError {
                message: "Invalid or empty REPL input".to_string(),
                src,
                span,
                location: "1:1".to_string(),
                help_message: "The REPL expects either a declaration or an expression.".to_string(),
                suggestion: Some(
                    "Try entering a simple expression like '42' or 'let x = 1'.".to_string(),
                ),
            })
        })?;

    match inner_pair.as_rule() {
        Rule::declaration => {
            // Use the existing declaration parsing logic from the parser module
            let declaration = parser::parse_declaration(inner_pair, &source_name, input)?;
            Ok(parser::ast::ReplInput::Declaration(declaration))
        }
        Rule::expr => {
            // Use the existing expression parsing logic from the parser module
            let expression = parser::parse_expr(inner_pair, &source_name)?;
            Ok(parser::ast::ReplInput::Expression(expression))
        }
        _ => {
            // Create a more specific error for unexpected rule
            let rule_text = format!("{:?}", inner_pair.as_rule());
            let span = parser::get_span_from_pair(&inner_pair);
            let location = parser::get_location_from_pair(&inner_pair);

            Err(BorfError::Parse(ParseError::UnexpectedToken {
                expected: "a declaration or expression".to_string(),
                found: rule_text,
                src: miette::NamedSource::new(
                    source_name.unwrap_or_else(|| "REPL".to_string()),
                    input.to_string(),
                ),
                span,
                location,
                help_message: "The REPL can only process declarations or expressions.".to_string(),
                suggestion: None,
            }))
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
    input: &str,
    source_name: Option<String>,
) -> Result<Vec<parser::ast::Expr>, BorfError> {
    parser::parse_string_to_exprs(input, source_name).map_err(BorfError::from)
}
