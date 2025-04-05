use crate::parser::Rule;
use miette::{Diagnostic, LabeledSpan, NamedSource, SourceSpan};
use pest::error::{Error as PestError, ErrorVariant, LineColLocation};
use std::fmt;
use std::io;
use thiserror::Error;

/// Errors that can occur during parsing and evaluation of Borf code
#[derive(Debug, Error, Diagnostic)]
pub enum BorfError {
    #[error(transparent)]
    #[diagnostic(code(borf::io_error))]
    Io(#[from] io::Error),

    #[error(transparent)]
    #[diagnostic(transparent)]
    Parse(#[from] ParseError),

    #[error("Evaluation error: {0}")]
    #[diagnostic(code(borf::evaluation_error))]
    Evaluation(String),
}

/// Specific errors that can occur during parsing
#[derive(Debug, Error, Diagnostic)]
pub enum ParseError {
    #[error("Syntax Error: {message} at {location}")]
    #[diagnostic(code(borf::parser::syntax_error))]
    SyntaxError {
        message: String,
        #[source_code]
        src: miette::NamedSource,
        #[label("here")]
        span: SourceSpan,
        location: String, // e.g., "line:col"
        #[help] // Use the field attribute instead of diagnostic attribute
        help_message: String,
        suggestion: Option<String>,
    },

    #[error("Empty Input: {help_message}")]
    #[diagnostic(code(borf::parser::empty_input))]
    EmptyInput {
        #[source_code]
        src: miette::NamedSource,
        #[help]
        help_message: String,
        suggestion: Option<String>,
    },

    #[error("Value Error: {message} at {location}")]
    #[diagnostic(code(borf::parser::value_error))]
    ValueError {
        message: String,
        #[source_code]
        src: miette::NamedSource,
        #[label("invalid value")]
        span: SourceSpan,
        location: String,
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
        src: miette::NamedSource,
        #[label("unexpected token")]
        span: SourceSpan,
        location: String,
        #[help] // Use the field attribute instead of diagnostic attribute
        help_message: String,
        suggestion: Option<String>,
    },

    #[error("Missing Token: expected {expected} at {location}")]
    #[diagnostic(code(borf::parser::missing_token))]
    MissingToken {
        expected: String,
        #[source_code]
        src: miette::NamedSource,
        #[label("token expected here")]
        span: SourceSpan,
        location: String,
        #[help] // Use the field attribute instead of diagnostic attribute
        help_message: String,
        suggestion: Option<String>,
    },

    #[error("Expected {expected} but none found at {location}")]
    #[diagnostic(code(borf::parser::missing_node))]
    MissingNode {
        expected: String,
        #[label("{expected} expected here")]
        span: SourceSpan,
        location: String,
    },

    #[error("Unexpected rule {found} at {location}, expected {expected}")]
    #[diagnostic(code(borf::parser::unexpected_rule))]
    UnexpectedRule {
        expected: String,
        found: String,
        #[label("didn't expect {found} here")]
        span: SourceSpan,
        location: String,
    },

    #[error("Invalid literal value '{value}' at {location}: {reason}")]
    #[diagnostic(code(borf::parser::invalid_literal))]
    InvalidLiteral {
        value: String,
        reason: String,
        #[label("invalid literal")]
        span: SourceSpan,
        location: String,
    },

    #[error(
        "Invalid map key type at {location}: expected string or identifier, found {found_type}"
    )]
    #[diagnostic(code(borf::parser::invalid_map_key))]
    InvalidMapKey {
        found_type: String,
        #[label("invalid key type")]
        span: SourceSpan,
        location: String,
        #[help]
        help_message: Option<String>,
    },

    #[error("Invalid map pattern key type at {location}: expected variable or string literal, found {found_type}")]
    #[diagnostic(code(borf::parser::invalid_map_pattern_key))]
    InvalidMapPatternKey {
        found_type: String,
        #[label("invalid key pattern type")]
        span: SourceSpan,
        location: String,
    },

    #[error(
        "Invalid type pattern structure at {location}: expected identifier, found {found_rule}"
    )]
    #[diagnostic(code(borf::parser::invalid_type_pattern))]
    InvalidTypePattern {
        found_rule: String,
        #[label("expected identifier here")]
        span: SourceSpan,
        location: String,
    },

    #[error("Multiple errors found")]
    #[diagnostic(code(borf::parser::multiple_errors))]
    MultipleErrors {
        #[related]
        errors: Vec<ParseError>,
    },

    #[error("I/O Error during parsing")]
    #[diagnostic(code(borf::parser::io_error))]
    Io(#[from] io::Error),

    // Keep a generic Unexpected variant for non-pest errors
    #[error("Unexpected Condition: {0}")]
    #[diagnostic(code(borf::parser::unexpected_condition))]
    Unexpected(String),
}

// Implement Clone manually since NamedSource doesn't implement Clone
impl Clone for ParseError {
    fn clone(&self) -> Self {
        match self {
            ParseError::SyntaxError {
                message,
                src,
                span,
                location,
                help_message,
                suggestion,
            } => ParseError::SyntaxError {
                message: message.clone(),
                src: miette::NamedSource::new(src.name().to_string(), src.read().to_string()),
                span: *span,
                location: location.clone(),
                help_message: help_message.clone(),
                suggestion: suggestion.clone(),
            },
            ParseError::EmptyInput {
                src,
                help_message,
                suggestion,
            } => ParseError::EmptyInput {
                src: miette::NamedSource::new(src.name().to_string(), src.read().to_string()),
                help_message: help_message.clone(),
                suggestion: suggestion.clone(),
            },
            ParseError::ValueError {
                message,
                src,
                span,
                location,
                help_message,
                suggestion,
            } => ParseError::ValueError {
                message: message.clone(),
                src: miette::NamedSource::new(src.name().to_string(), src.read().to_string()),
                span: *span,
                location: location.clone(),
                help_message: help_message.clone(),
                suggestion: suggestion.clone(),
            },
            ParseError::UnexpectedToken {
                expected,
                found,
                src,
                span,
                location,
                help_message,
                suggestion,
            } => ParseError::UnexpectedToken {
                expected: expected.clone(),
                found: found.clone(),
                src: miette::NamedSource::new(src.name().to_string(), src.read().to_string()),
                span: *span,
                location: location.clone(),
                help_message: help_message.clone(),
                suggestion: suggestion.clone(),
            },
            ParseError::MissingToken {
                expected,
                src,
                span,
                location,
                help_message,
                suggestion,
            } => ParseError::MissingToken {
                expected: expected.clone(),
                src: miette::NamedSource::new(src.name().to_string(), src.read().to_string()),
                span: *span,
                location: location.clone(),
                help_message: help_message.clone(),
                suggestion: suggestion.clone(),
            },
            ParseError::MissingNode {
                expected,
                span,
                location,
            } => ParseError::MissingNode {
                expected: expected.clone(),
                span: *span,
                location: location.clone(),
            },
            ParseError::UnexpectedRule {
                expected,
                found,
                span,
                location,
            } => ParseError::UnexpectedRule {
                expected: expected.clone(),
                found: found.clone(),
                span: *span,
                location: location.clone(),
            },
            ParseError::InvalidLiteral {
                value,
                reason,
                span,
                location,
            } => ParseError::InvalidLiteral {
                value: value.clone(),
                reason: reason.clone(),
                span: *span,
                location: location.clone(),
            },
            ParseError::InvalidMapKey {
                found_type,
                span,
                location,
                help_message,
            } => ParseError::InvalidMapKey {
                found_type: found_type.clone(),
                span: *span,
                location: location.clone(),
                help_message: help_message.clone(),
            },
            ParseError::InvalidMapPatternKey {
                found_type,
                span,
                location,
            } => ParseError::InvalidMapPatternKey {
                found_type: found_type.clone(),
                span: *span,
                location: location.clone(),
            },
            ParseError::InvalidTypePattern {
                found_rule,
                span,
                location,
            } => ParseError::InvalidTypePattern {
                found_rule: found_rule.clone(),
                span: *span,
                location: location.clone(),
            },
            ParseError::MultipleErrors { errors } => ParseError::MultipleErrors {
                errors: errors.clone(),
            },
            ParseError::Io(e) => ParseError::Io(*e),
            ParseError::Unexpected(s) => ParseError::Unexpected(s.clone()),
        }
    }
}

/// Generates a helpful suggestion based on the rule and the found token
fn suggest_fix(rule: Rule, found: &str) -> Option<String> {
    match (rule, found) {
        // Common typos and mistakes based on actual grammar rules
        (Rule::fn_decl, "fun") => Some("Did you mean 'fn'?".to_string()),
        (Rule::fn_decl, "func") => Some("Did you mean 'fn'?".to_string()),
        (Rule::fn_decl, "function") => Some("Did you mean 'fn'?".to_string()),
        (Rule::type_decl, "Type") => Some("Did you mean 'type'?".to_string()),
        (Rule::op_decl, "Op") => Some("Did you mean 'op'?".to_string()),
        (Rule::module_decl, _) if !found.starts_with('@') => Some(format!(
            "Module names must start with '@', like '@{}'?",
            found
        )),
        // Simple structural suggestions
        (Rule::parenthesized_expr, s) if s.ends_with('(') => {
            Some("Missing closing ')'?".to_string())
        }
        (Rule::module_body, s) if s.ends_with('{') => Some("Missing closing '}'?".to_string()),
        (Rule::list_literal | Rule::list_pattern, s) if s.ends_with('[') => {
            Some("Missing closing ']'?".to_string())
        }
        // TODO: Add more suggestions based on common errors
        _ => None,
    }
}

/// Generates a more contextual help message based on the error type and context
fn generate_help_message(rule: Option<Rule>, found: &str, expected: &str) -> String {
    match rule {
        Some(Rule::fn_decl) => format!(
            "Function declarations start with 'fn'. Found '{}' instead.",
            found
        ),
        Some(Rule::type_decl) => format!(
            "Type declarations start with 'type'. Found '{}' instead.",
            found
        ),
        Some(Rule::module_decl) => format!(
            "Module declarations start with '@'. Found '{}' instead.",
            found
        ),
        // Generic messages based on structure
        _ if found.contains('(') && !found.contains(')') => {
            "It looks like you're missing a closing parenthesis ')'.".to_string()
        }
        _ if found.contains('{') && !found.contains('}') => {
            "It looks like you're missing a closing brace '}'.".to_string()
        }
        _ if found.contains('[') && !found.contains(']') => {
            "It looks like you're missing a closing bracket ']'.".to_string()
        }
        // Default message
        _ if !expected.is_empty() => format!(
            "Expected {}. Check the syntax near this location.",
            expected
        ),
        _ => format!(
            "Something isn't right here. Expected {}, but found '{}'.",
            expected, found
        ),
    }
}

impl ParseError {
    // Helper to create ParseError from PestError
    pub fn from_pest(error: PestError<Rule>, input: &str, source_name: Option<String>) -> Self {
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
        let src = miette::NamedSource::new(source_name, input.to_string());

        match error.variant {
            ErrorVariant::ParsingError {
                positives,
                negatives,
            } => {
                // Format expected rules nicely
                let expected = positives
                    .iter()
                    .map(|r| format!("'{}'", friendly_rule_name(*r))) // Use friendly names
                    .collect::<Vec<_>>()
                    .join(" or ");

                // Determine what was actually found (often approximated)
                // Pest's `negatives` are often just the inverse of positives, not the actual token found.
                // Let's try to get the text at the span.
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
                        .map(|r| format!("'{}'", friendly_rule_name(*r)))
                        .collect::<Vec<_>>()
                        .join(", ")
                } else if found_text == "<end of input>" {
                    "end of input".to_string()
                } else {
                    "an unexpected token".to_string() // Fallback
                };

                // Generate suggestion based on the *first* expected rule and found text
                let suggestion = positives
                    .first()
                    .and_then(|rule| suggest_fix(*rule, found_text));

                // Generate help message
                let help_message =
                    generate_help_message(positives.first().copied(), found_text, &expected);

                // Decide between MissingToken and UnexpectedToken
                if positives.is_empty() && negatives.is_empty() {
                    // Should ideally not happen with Pest errors, but handle defensively
                    ParseError::SyntaxError {
                        message: "Unknown syntax error".to_string(),
                        src,
                        span,
                        location,
                        help_message,
                        suggestion,
                    }
                } else if positives.is_empty() {
                    // Only negatives means something unexpected was found where nothing specific was expected (rare)
                    ParseError::UnexpectedToken {
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
                    ParseError::MissingToken {
                        expected: expected.clone(),
                        src,
                        span: (input.len(), 0).into(), // Point to the very end
                        location: format!("{}:{}", line, col), // Use original line/col guess
                        help_message: format!("Expected {} before the end of the input.", expected),
                        suggestion,
                    }
                } else if span.len() == 0 {
                    // Zero-length span implies something expected is missing *before* the current point
                    ParseError::MissingToken {
                        expected,
                        src,
                        span, // Span points right *before* where token was expected
                        location,
                        help_message,
                        suggestion,
                    }
                } else {
                    // Non-zero span or negatives means an actual token was found that wasn't expected
                    ParseError::UnexpectedToken {
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
            ErrorVariant::CustomError { message } => ParseError::SyntaxError {
                message, // Use the custom message directly
                src,
                span, // Span provided by CustomError location
                location,
                help_message: "Check the syntax or logic around this location.".to_string(), // Generic help
                suggestion: None, // No suggestion for custom errors yet
            },
        }
    }
}

// Helper function to provide more user-friendly names for Pest rules
fn friendly_rule_name(rule: Rule) -> String {
    match rule {
        Rule::EOI => "end of input".to_string(),
        Rule::WHITESPACE => "whitespace".to_string(),
        Rule::COMMENT | Rule::MULTILINE_COMMENT => "a comment".to_string(),
        Rule::identifier => "an identifier (name)".to_string(),
        Rule::string => "a string literal (e.g., \"hello\")".to_string(), // Fixed escaping of quotes
        Rule::integer => "an integer number (e.g., 42)".to_string(),
        Rule::float => "a floating-point number (e.g., 3.14)".to_string(),
        Rule::boolean => "a boolean (true or false)".to_string(),
        Rule::literal => "a literal value (number, string, boolean)".to_string(),
        Rule::module_decl => "a module declaration (e.g., @Name: { ... })".to_string(),
        Rule::type_decl => "a type declaration (e.g., type: { T })".to_string(),
        Rule::op_decl => "an operation declaration (e.g., op: { f })".to_string(),
        Rule::fn_decl => "a function declaration (e.g., fn: { g })".to_string(),
        Rule::expr => "an expression".to_string(),
        Rule::lambda => "a lambda function (e.g., [x] x + 1)".to_string(),
        Rule::application => "a function application (e.g., (f x))".to_string(),
        Rule::let_expr => "a let binding (e.g., let x = 1 in x)".to_string(),
        Rule::if_expr => "an if expression (e.g., if c then t else e)".to_string(),
        Rule::op_add | Rule::op_sub | Rule::op_mul | Rule::op_div | Rule::op_eq => {
            "a binary operator".to_string()
        }
        Rule::parenthesized_expr => "a parenthesized expression (...)".to_string(),
        Rule::list_literal => "a list literal ([1, 2])".to_string(),
        Rule::map_literal => "a map literal ({key: value})".to_string(),
        Rule::set_literal => "a set literal ({1, 2})".to_string(),
        Rule::pattern => "a pattern".to_string(),
        // Add more rules as needed for clarity
        other => format!("'{:?}'", other), // Fallback to debug name
    }
}

/// Creates a nicely formatted error report from a BorfError
///
/// This is the recommended way to display errors to users.
pub fn create_report(error: BorfError, source_name: Option<String>) -> miette::Report {
    // If the error is a ParseError, enhance it with source name if not already present
    match error {
        BorfError::Parse(pe) => {
            // Attempt to add source name if missing. This requires pe to be mutable.
            // Cloning might be necessary if we can't get a mutable ref easily.
            // For now, we assume from_pest adds it correctly.
            miette::Report::new(BorfError::Parse(pe))
        }
        _ => miette::Report::new(error),
    }
}

/// Creates a nicely formatted error report directly from a ParseError
pub fn create_parse_report(error: ParseError, source_name: Option<String>) -> miette::Report {
    // Similar to create_report, ensure source name is included.
    // Cloning might be necessary.
    miette::Report::new(error)
}
