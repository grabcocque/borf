use colored::*;
use miette::{Diagnostic, LabeledSpan, NamedSource, Report, SourceSpan};
use pest::error::{Error as PestError, ErrorVariant, LineColLocation};
use std::path::PathBuf;
use thiserror::Error;

use crate::parser::ast;

/// An enhanced error type for parser errors with rich context and suggestions
#[derive(Debug, Error, Diagnostic)]
pub enum EnhancedError {
    #[error("Syntax Error: {message} at {location}")]
    #[diagnostic(code(borf::parser::syntax_error))]
    SyntaxError {
        message: String,
        #[source_code]
        src: NamedSource<String>,
        #[label("here")]
        span: SourceSpan,
        location: String, // e.g., "line:col"
        #[help]
        help_message: String,
        suggestion: Option<String>,
    },

    #[error("Unexpected Token: expected {expected}, found {found} at {location}")]
    #[diagnostic(code(borf::parser::unexpected_token))]
    UnexpectedToken {
        expected: String,
        found: String,
        #[source_code]
        src: NamedSource<String>,
        #[label("unexpected token")]
        span: SourceSpan,
        location: String,
        #[help]
        help_message: String,
        suggestion: Option<String>,
    },

    #[error("Missing Token: expected {expected} at {location}")]
    #[diagnostic(code(borf::parser::missing_token))]
    MissingToken {
        expected: String,
        #[source_code]
        src: NamedSource<String>,
        #[label("token expected here")]
        span: SourceSpan,
        location: String,
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

    #[error("I/O Error: {0}")]
    #[diagnostic(code(borf::io_error))]
    Io(#[from] std::io::Error),

    #[error("Unexpected Condition: {0}")]
    #[diagnostic(code(borf::parser::unexpected_condition))]
    Unexpected(String),
}

/// Convert a Pest error into an EnhancedError with rich context
pub fn enhance_pest_error(
    error: PestError<crate::parser::Rule>,
    input: &str,
    source_name: Option<String>,
) -> EnhancedError {
    let (line, col) = match error.line_col {
        LineColLocation::Pos((line, col)) => (line, col),
        LineColLocation::Span((line, col), _) => (line, col),
    };
    let location = format!("{}:{}", line, col);

    let span: SourceSpan = match error.location {
        pest::error::InputLocation::Pos(pos) => (pos, 0).into(), // Zero-length span for position
        pest::error::InputLocation::Span((start, end)) => (start, end - start).into(),
    };

    // Use provided source name or fallback
    let source_name = source_name.unwrap_or_else(|| "<unknown>".to_string());
    let src = NamedSource::new(source_name, input.to_string());

    match error.variant {
        ErrorVariant::ParsingError {
            positives,
            negatives,
        } => {
            // Format expected rules nicely
            let expected = positives
                .iter()
                .map(|r| format!("'{:?}'", r)) // Use rule names directly
                .collect::<Vec<_>>()
                .join(" or ");

            // Extract the text at the error position
            let found_text = input
                .get(span.offset()..span.offset() + span.len())
                .unwrap_or("<end of input>") // Handle case where span is at EOF
                .trim();

            // Determine found description (either text or negative rules if no text)
            let found_desc = if !found_text.is_empty() && span.len() > 0 {
                format!("'{}'", found_text)
            } else if !negatives.is_empty() {
                negatives
                    .iter()
                    .map(|r| format!("'{:?}'", r))
                    .collect::<Vec<_>>()
                    .join(", ")
            } else if found_text == "<end of input>" {
                "end of input".to_string()
            } else {
                "an unexpected token".to_string() // Fallback
            };

            // Generate suggestion based on the error
            let suggestion = suggest_fix(&positives, found_text);

            // Generate help message
            let help_message = generate_help_message(&positives, found_text, &expected);

            // Decide between MissingToken and UnexpectedToken
            if positives.is_empty() && negatives.is_empty() {
                // Should ideally not happen with Pest errors, but handle defensively
                EnhancedError::SyntaxError {
                    message: "Unknown syntax error".to_string(),
                    src,
                    span,
                    location,
                    help_message,
                    suggestion,
                }
            } else if positives.is_empty() {
                // Only negatives means something unexpected was found where nothing specific was expected (rare)
                EnhancedError::UnexpectedToken {
                    expected: "something else".to_string(), // Generic
                    found: found_desc,
                    src,
                    span,
                    location,
                    help_message,
                    suggestion,
                }
            } else if span.len() == 0 && found_text == "<end of input>" {
                // Zero-length span at EOF usually means something is missing
                EnhancedError::MissingToken {
                    expected,
                    src,
                    span: (input.len(), 0).into(), // Point to the very end
                    location: format!("{}:{}", line, col), // Use original line/col guess
                    help_message: format!("Expected {} before the end of the input.", expected),
                    suggestion,
                }
            } else if span.len() == 0 {
                // Zero-length span implies something expected is missing *before* the current point
                EnhancedError::MissingToken {
                    expected,
                    src,
                    span, // Span points right *before* where token was expected
                    location,
                    help_message,
                    suggestion,
                }
            } else {
                // Non-zero span or negatives means an actual token was found that wasn't expected
                EnhancedError::UnexpectedToken {
                    expected,
                    found: found_desc,
                    src,
                    span,
                    location,
                    help_message,
                    suggestion,
                }
            }
        }
        ErrorVariant::CustomError { message } => EnhancedError::SyntaxError {
            message, // Use the custom message directly
            src,
            span, // Span provided by CustomError location
            location,
            help_message: "Check the syntax or logic around this location.".to_string(), // Generic help
            suggestion: None, // No suggestion for custom errors yet
        },
    }
}

/// Generate a helpful suggestion based on the error context
fn suggest_fix(rules: &[crate::parser::Rule], found: &str) -> Option<String> {
    if rules.is_empty() {
        return None;
    }

    // Helper function to check if a rule is within a list
    let rule_is = |rule: crate::parser::Rule, names: &[&str]| -> bool {
        let rule_name = format!("{:?}", rule).to_lowercase();
        names.iter().any(|&name| rule_name.contains(name))
    };

    let rule = rules[0]; // Take the first rule as primary

    // Common typos and mistakes by rule type
    if rule_is(rule, &["fn_decl", "function"]) {
        if found.contains("fun") || found.contains("func") {
            return Some("Did you mean 'fn'?".to_string());
        }
    } else if rule_is(rule, &["type_decl", "type"]) {
        if found == "Type" {
            return Some("Did you mean 'type'?".to_string());
        }
    } else if rule_is(rule, &["op_decl", "operation"]) {
        if found == "Op" {
            return Some("Did you mean 'op'?".to_string());
        }
    } else if rule_is(rule, &["module_decl", "module"]) && !found.starts_with('@') {
        return Some(format!(
            "Module names must start with '@', like '@{}'?",
            found
        ));
    }

    // Simple structural suggestions
    if rule_is(rule, &["parenthesized_expr"]) && found.ends_with('(') {
        return Some("Missing closing ')'?".to_string());
    } else if rule_is(rule, &["module_body"]) && found.ends_with('{') {
        return Some("Missing closing '}'?".to_string());
    } else if (rule_is(rule, &["list_literal"]) || rule_is(rule, &["list_pattern"]))
        && found.ends_with('[')
    {
        return Some("Missing closing ']'?".to_string());
    }

    // No specific suggestion found
    None
}

/// Generate a helpful message based on the error context
fn generate_help_message(rules: &[crate::parser::Rule], found: &str, expected: &str) -> String {
    if rules.is_empty() {
        if !found.is_empty() {
            // Generic structure-based hints
            if found.contains('(') && !found.contains(')') {
                return "It looks like you're missing a closing parenthesis ')'.".to_string();
            } else if found.contains('{') && !found.contains('}') {
                return "It looks like you're missing a closing brace '}'.".to_string();
            } else if found.contains('[') && !found.contains(']') {
                return "It looks like you're missing a closing bracket ']'.".to_string();
            }
        }

        // Default message
        if !expected.is_empty() {
            return format!(
                "Expected {}. Check the syntax near this location.",
                expected
            );
        } else {
            return format!(
                "Something isn't right here. Expected {}, but found '{}'.",
                expected, found
            );
        }
    }

    // Helper function to check if a rule is within a list
    let rule_is = |rule: crate::parser::Rule, names: &[&str]| -> bool {
        let rule_name = format!("{:?}", rule).to_lowercase();
        names.iter().any(|&name| rule_name.contains(name))
    };

    let rule = rules[0]; // Take the first rule as primary

    // Specific messages based on rule type
    if rule_is(rule, &["fn_decl", "function"]) {
        return format!(
            "Function declarations start with 'fn'. Found '{}' instead.",
            found
        );
    } else if rule_is(rule, &["type_decl", "type"]) {
        return format!(
            "Type declarations start with 'type'. Found '{}' instead.",
            found
        );
    } else if rule_is(rule, &["module_decl", "module"]) {
        return format!(
            "Module declarations start with '@'. Found '{}' instead.",
            found
        );
    }

    // Default message with expected and found
    format!(
        "Expected {}. Check the syntax near this location.",
        expected
    )
}

/// Create a nicely formatted report for an EnhancedError
pub fn create_enhanced_report(error: EnhancedError) -> miette::Report {
    miette::Report::new(error)
}

/// Helper function to print a colored error message to the console
pub fn print_error_message(error: &EnhancedError) {
    match error {
        EnhancedError::SyntaxError {
            message,
            location,
            help_message,
            suggestion,
            ..
        } => {
            eprintln!("{}: {}", "Syntax Error".bright_red().bold(), message);
            eprintln!("{}: {}", "Location".yellow(), location);
            eprintln!("{}: {}", "Help".cyan(), help_message);
            if let Some(suggestion) = suggestion {
                eprintln!("{}: {}", "Suggestion".green(), suggestion);
            }
        }
        EnhancedError::UnexpectedToken {
            expected,
            found,
            location,
            help_message,
            suggestion,
            ..
        } => {
            eprintln!(
                "{}: Expected {}, found {}",
                "Unexpected Token".bright_red().bold(),
                expected,
                found
            );
            eprintln!("{}: {}", "Location".yellow(), location);
            eprintln!("{}: {}", "Help".cyan(), help_message);
            if let Some(suggestion) = suggestion {
                eprintln!("{}: {}", "Suggestion".green(), suggestion);
            }
        }
        EnhancedError::MissingToken {
            expected,
            location,
            help_message,
            suggestion,
            ..
        } => {
            eprintln!(
                "{}: Expected {}",
                "Missing Token".bright_red().bold(),
                expected
            );
            eprintln!("{}: {}", "Location".yellow(), location);
            eprintln!("{}: {}", "Help".cyan(), help_message);
            if let Some(suggestion) = suggestion {
                eprintln!("{}: {}", "Suggestion".green(), suggestion);
            }
        }
        EnhancedError::MultipleErrors { errors } => {
            eprintln!(
                "{}: Found {} errors",
                "Multiple Errors".bright_red().bold(),
                errors.len()
            );
            for (i, error) in errors.iter().enumerate() {
                eprintln!("{}:", format!("Error #{}", i + 1).yellow().bold());
                print_error_message(error);
                eprintln!(); // Add a blank line between errors
            }
        }
        EnhancedError::Io(e) => {
            eprintln!("{}: {}", "I/O Error".bright_red().bold(), e);
        }
        EnhancedError::Unexpected(msg) => {
            eprintln!("{}: {}", "Unexpected Error".bright_red().bold(), msg);
        }
    }
}

/// Parse a file and return enhanced error reporting if it fails
pub fn parse_file_with_enhanced_errors(
    path: impl AsRef<std::path::Path>,
) -> Result<ast::Module, EnhancedError> {
    let path_string = path.as_ref().to_string_lossy().to_string();
    let input = std::fs::read_to_string(&path).map_err(EnhancedError::Io)?;

    parse_string_with_enhanced_errors(&input, Some(path_string))
}

/// Parse a string and return enhanced error reporting if it fails
pub fn parse_string_with_enhanced_errors(
    input: &str,
    source_name: Option<String>,
) -> Result<ast::Module, EnhancedError> {
    match crate::parser::parse_string(input) {
        Ok(module) => Ok(module),
        Err(err) => match err {
            crate::errors::ParseError::Pest(e) => Err(enhance_pest_error(e, input, source_name)),
            // Handle other error types
            _ => Err(EnhancedError::Unexpected(format!("{}", err))),
        },
    }
}

/// Try to parse using existing parser, but provide enhanced errors if it fails
pub fn try_parse_with_recovery(
    input: &str,
    source_name: Option<String>,
) -> Result<(ast::Module, Vec<EnhancedError>), EnhancedError> {
    let result = parse_string_with_enhanced_errors(input, source_name.clone());

    match result {
        Ok(module) => Ok((module, vec![])),
        Err(error) => {
            // In a real implementation, we would try to recover and continue parsing
            // For now, we'll just return the single error
            Err(error)
        }
    }
}
