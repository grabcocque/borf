#![allow(dead_code)]
use colored::*;
use miette::Diagnostic;
use std::sync::Arc;
use thiserror::Error;

use crate::errors::{self, ParseError};
use crate::parser;
use crate::parser::{ast, BorfParser, Rule};
use pest::Parser;

/// Custom errors with enhanced diagnostics for better error reporting
#[derive(Debug, Error, Clone, Diagnostic)]
pub enum EnhancedError {
    #[error(transparent)]
    #[diagnostic(transparent)]
    ParseError(Box<ParseError>),

    #[error("I/O Error: {0}")]
    #[diagnostic(code(borf::parser::io_error))]
    Io(String), // Can't contain io::Error directly as it's not Clone

    #[error("Unexpected Error: {0}")]
    #[diagnostic(code(borf::parser::unexpected))]
    Unexpected(String),

    #[error("Empty Input: {help_message}")]
    #[diagnostic(code(borf::parser::empty_input))]
    EmptyInput {
        #[source_code]
        src: Arc<miette::NamedSource<String>>,
        #[help]
        help_message: String,
        suggestion: Option<String>,
    },

    #[error("Multiple errors found")]
    #[diagnostic(code(borf::parser::multiple_errors))]
    MultipleErrors {
        #[related]
        errors: Vec<EnhancedError>,
    },
}

impl From<ParseError> for EnhancedError {
    fn from(err: ParseError) -> Self {
        EnhancedError::ParseError(Box::new(err))
    }
}

impl EnhancedError {
    /// Create an enhanced error from a pest error with source information
    pub fn from_pest_with_source(
        pest_error: pest::error::Error<Rule>,
        source: &str,
        source_name: Option<String>,
    ) -> Self {
        // Convert to ParseError first, then to EnhancedError
        let parse_error = errors::ParseError::from_pest(pest_error, source, source_name);
        EnhancedError::from(parse_error)
    }
}

pub fn create_enhanced_report(error: EnhancedError) -> miette::Report {
    miette::Report::new(error)
}

pub fn print_error_message(error: &EnhancedError) {
    eprintln!("{}", "Error:".red().bold());

    match error {
        EnhancedError::MultipleErrors { errors } => {
            eprintln!("  Multiple errors occurred:");
            for (i, e) in errors.iter().enumerate() {
                eprintln!("  Error {}:", i + 1);
                print_nested_error(e, "    ");
            }
        }
        _ => {
            eprintln!("  {}", error);
            if let Some(help) = get_help_message(error) {
                eprintln!("  Help: {}", help.cyan());
            }
            if let Some(suggestion) = get_suggestion(error) {
                eprintln!("  Suggestion: {}", suggestion.yellow());
            }
        }
    }
}

fn print_nested_error(error: &EnhancedError, indent: &str) {
    eprintln!("{}{}", indent, error);
    if let Some(help) = get_help_message(error) {
        eprintln!("{}> Help: {}", indent, help.cyan());
    }
    if let Some(suggestion) = get_suggestion(error) {
        eprintln!("{}> Suggestion: {}", indent, suggestion.yellow());
    }
    if let EnhancedError::MultipleErrors { errors } = error {
        let nested_indent = format!("{}  ", indent);
        for e in errors {
            print_nested_error(e, &nested_indent);
        }
    }
}

// Function to extract help message from an error
fn get_help_message(error: &EnhancedError) -> Option<&str> {
    match error {
        EnhancedError::ParseError(pe) => {
            match &**pe {
                ParseError::SyntaxError { help_message, .. } => Some(help_message),
                ParseError::EmptyInput { help_message, .. } => Some(help_message),
                ParseError::ValueError { help_message, .. } => Some(help_message),
                ParseError::UnexpectedToken { help_message, .. } => Some(help_message),
                ParseError::MissingToken { help_message, .. } => Some(help_message),
                // Other variants as needed
                _ => None,
            }
        }
        EnhancedError::EmptyInput { help_message, .. } => Some(help_message),
        // Simple string messages don't have help
        EnhancedError::Io(_) | EnhancedError::Unexpected(_) => None,
        // For multiple errors, return the first help message
        EnhancedError::MultipleErrors { errors } => errors.iter().find_map(get_help_message),
    }
}

// Function to extract suggestion from an error
fn get_suggestion(error: &EnhancedError) -> Option<&str> {
    match error {
        EnhancedError::ParseError(pe) => {
            match &**pe {
                ParseError::SyntaxError { suggestion, .. } => suggestion.as_deref(),
                ParseError::EmptyInput { suggestion, .. } => suggestion.as_deref(),
                ParseError::ValueError { suggestion, .. } => suggestion.as_deref(),
                ParseError::UnexpectedToken { suggestion, .. } => suggestion.as_deref(),
                ParseError::MissingToken { suggestion, .. } => suggestion.as_deref(),
                // Other variants as needed
                _ => None,
            }
        }
        EnhancedError::EmptyInput { suggestion, .. } => suggestion.as_deref(),
        // Simple string messages don't have suggestions
        EnhancedError::Io(_) | EnhancedError::Unexpected(_) => None,
        // For multiple errors, return the first suggestion
        EnhancedError::MultipleErrors { errors } => errors.iter().find_map(get_suggestion),
    }
}

/// Parses a file, converting parser errors to EnhancedError.
pub fn parse_file_with_enhanced_errors<P: AsRef<std::path::Path>>(
    path: P,
) -> Result<ast::Module, EnhancedError> {
    let path_buf = path.as_ref().to_path_buf();
    let file_name = path_buf
        .file_name()
        .and_then(|os_str| os_str.to_str())
        .unwrap_or("<unknown>")
        .to_string();
    let source_name = Some(file_name.clone());

    // Read the file
    let file_content = match std::fs::read_to_string(&path) {
        Ok(content) => content,
        Err(io_err) => {
            return Err(EnhancedError::Io(io_err.to_string()));
        }
    };

    parse_string_to_module_with_enhanced_errors(&file_content, source_name)
}

/// Parses a string, converting parser errors to EnhancedError.
pub fn parse_string_to_module_with_enhanced_errors(
    input: &str,
    source_name: Option<String>,
) -> Result<ast::Module, EnhancedError> {
    // Try to parse the input as a module
    match BorfParser::parse(Rule::file, input) {
        Ok(mut pairs) => {
            // Get the first (and only) successful parse result
            let module_pair = pairs.next().unwrap();
            // Extract the module declaration from inside the file rule
            let module_decl_pair = module_pair.into_inner().next().unwrap();

            // Convert to AST
            parser::parse_module(module_decl_pair, &source_name)
                .map_err(|e| EnhancedError::from(*e))
        }
        Err(err) => {
            // Add source information to the error
            let error_with_source =
                EnhancedError::from_pest_with_source(err, input, source_name.clone());
            Err(error_with_source)
        }
    }
}

/// Attempts to parse, potentially recovering from errors.
#[allow(dead_code)]
fn handle_parse_error() {
    // Empty implementation for now
}

/// Convert a pest error to an enhanced error for better reporting
#[allow(dead_code)]
fn create_enhanced_error(
    error: pest::error::Error<Rule>,
    input: &str,
    source_name: Option<String>,
) -> EnhancedError {
    // First convert to a ParseError
    let parse_error = ParseError::from_pest(error, input, source_name);

    // Now wrap in our EnhancedError
    EnhancedError::from(parse_error)
}

// Function to convert a ParseError to EnhancedError
pub fn convert_parse_error_to_enhanced(
    error: ParseError,
    _additional_context: Option<&str>,
) -> EnhancedError {
    // Simply use the From implementation
    EnhancedError::from(error)
}

// Function to extract diagnostics from an EnhancedError
pub fn get_diagnostics_from_error(error: &EnhancedError) -> Vec<String> {
    match error {
        EnhancedError::ParseError(parse_error) => {
            // Extract diagnostics from the parse error
            vec![format!("Parse error: {}", parse_error)]
        }
        EnhancedError::EmptyInput { help_message, .. } => {
            vec![format!("Empty input: {}", help_message)]
        }
        EnhancedError::Io(message) => {
            vec![format!("IO error: {}", message)]
        }
        EnhancedError::Unexpected(message) => {
            vec![format!("Unexpected error: {}", message)]
        }
        EnhancedError::MultipleErrors { errors } => {
            // Collect diagnostics from each error
            errors.iter().flat_map(get_diagnostics_from_error).collect()
        }
    }
}
