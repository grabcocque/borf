use crate::parser::Rule;
use miette::{Diagnostic, SourceSpan};
use pest::error::{Error as PestError, ErrorVariant, LineColLocation};
use std::io;
use std::sync::Arc;
use thiserror::Error;

/// Errors that can occur during parsing and evaluation of Borf code
#[derive(Debug, Error, Diagnostic)]
pub enum BorfError {
    #[error(transparent)]
    #[diagnostic(code(borf::io_error))]
    Io(#[from] io::Error),

    #[error(transparent)]
    #[diagnostic(transparent)]
    Parse(Box<ParseError>),

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
        src: Arc<miette::NamedSource<String>>,
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
        src: Arc<miette::NamedSource<String>>,
        #[help]
        help_message: String,
        suggestion: Option<String>,
    },

    #[error("Value Error: {message} at {location}")]
    #[diagnostic(code(borf::parser::value_error))]
    ValueError {
        message: String,
        #[source_code]
        src: Arc<miette::NamedSource<String>>,
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
        src: Arc<miette::NamedSource<String>>,
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
        src: Arc<miette::NamedSource<String>>,
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
                src: Arc::clone(src),
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
                src: Arc::clone(src),
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
                src: Arc::clone(src),
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
                src: Arc::clone(src),
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
                src: Arc::clone(src),
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
            ParseError::Io(e) => {
                // io::Error is not Clone. Return a placeholder Unexpected error,
                // including the original error's string representation.
                ParseError::Unexpected(format!("Cloned I/O error: {}", e))
            }
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
    // Helper function to convert pest errors into custom ParseError
    pub fn from_pest(error: PestError<Rule>, input: &str, source_name: Option<String>) -> Self {
        // Use NamedSource to hold the input for better error reporting
        let src_name = source_name.unwrap_or_else(|| "<unknown>".to_string());
        let src = Arc::new(miette::NamedSource::new(src_name, input.to_string())); // Create the Arc here

        let (line, col) = match error.line_col {
            LineColLocation::Pos((l, c)) => (l, c),
            LineColLocation::Span((l, c), _) => (l, c),
        };
        let location = format!("{}:{}", line, col);
        let span: SourceSpan = match error.location {
            pest::error::InputLocation::Pos(p) => (p, 0).into(),
            pest::error::InputLocation::Span((s, e)) => (s, e - s).into(),
        };

        match error.variant {
            ErrorVariant::ParsingError {
                positives, // Expected rules
                negatives, // Unexpected rules
            } => {
                // Determine if it's a MissingToken or UnexpectedToken case
                if !negatives.is_empty() {
                    // Found an unexpected rule
                    let found_rule = negatives[0];
                    let found_desc = friendly_rule_name(found_rule);
                    let expected = positives
                        .iter()
                        .map(|r| friendly_rule_name(*r))
                        .collect::<Vec<_>>()
                        .join(" or ");
                    let help_message =
                        generate_help_message(Some(found_rule), &found_desc, &expected);
                    let suggestion = suggest_fix(found_rule, &found_desc);

                    ParseError::UnexpectedToken {
                        expected,
                        found: found_desc,
                        src: Arc::clone(&src), // Clone the Arc
                        span,
                        location,
                        help_message,
                        suggestion,
                    }
                } else {
                    // Expected something, but found end of input or incorrect structure
                    let expected = positives
                        .iter()
                        .map(|r| friendly_rule_name(*r))
                        .collect::<Vec<_>>()
                        .join(" or ");
                    let help_message = generate_help_message(None, "end of input", &expected);
                    // Decide if it's MissingToken or MissingNode based on context? Hard to tell.
                    // Default to MissingToken as it's common
                    ParseError::MissingToken {
                        expected: expected.clone(),
                        src: Arc::clone(&src),                 // Clone the Arc
                        span: (input.len(), 0).into(),         // Point to the very end
                        location: format!("{}:{}", line, col), // Use original line/col guess
                        help_message,
                        suggestion: None, // No specific suggestion usually for EOF
                    }
                }
            }
            ErrorVariant::CustomError { message } => ParseError::SyntaxError {
                message,               // Use the custom message directly
                src: Arc::clone(&src), // Clone the Arc
                span,                  // Span provided by CustomError location
                location,
                help_message: "Check the syntax near this location.".to_string(), // Generic help
                suggestion: None,
            },
        }
    }

    // Function to check if the error can be recovered from
    pub fn is_recoverable(&self) -> bool {
        // Define which errors might be recoverable in an interactive session
        match self {
            // Likely recoverable
            ParseError::UnexpectedToken { .. } => true,
            ParseError::MissingToken { .. } => true,
            ParseError::SyntaxError { .. } => true,
            // Usually not recoverable without fixing input
            ParseError::EmptyInput { .. } => false,
            ParseError::ValueError { .. } => false,
            ParseError::MissingNode { .. } => false,
            ParseError::UnexpectedRule { .. } => false,
            ParseError::InvalidLiteral { .. } => false,
            ParseError::InvalidMapKey { .. } => false,
            ParseError::InvalidMapPatternKey { .. } => false,
            ParseError::InvalidTypePattern { .. } => false,
            // Aggregate/System errors
            ParseError::MultipleErrors { .. } => false, // Or maybe recoverable if sub-errors are?
            ParseError::Io(_) => false,
            ParseError::Unexpected(_) => false, // Assume unexpected conditions are fatal
        }
    }
}

// Helper function to provide more user-friendly names for Pest rules
pub fn friendly_rule_name(rule: Rule) -> String {
    match rule {
        Rule::expr => "an expression".to_string(),
        Rule::literal => "a literal (number, string, etc.)".to_string(),
        Rule::identifier => "an identifier".to_string(),
        Rule::lambda => "a lambda expression (e.g., [x -> x + 1])".to_string(),
        Rule::application => "a function application (e.g., (f x))".to_string(),
        Rule::let_expr => "a let expression (e.g., let x = 10 in x * 2)".to_string(),
        Rule::ternary => "a ternary expression (e.g., x iff c or_else y)".to_string(),
        Rule::parenthesized_expr => "a parenthesized expression (e.g., (1 + 2))".to_string(),
        Rule::quoting_expr => "a quoting expression (e.g., 'expr, ~expr)".to_string(),
        Rule::quote_expr => "a quote expression (e.g., 'expr)".to_string(),
        Rule::unquote_expr => "an unquote expression (e.g., ~expr)".to_string(),
        Rule::unquote_splice_expr => "an unquote-splice expression (e.g., ~@expr)".to_string(),
        Rule::quasiquote_expr => "a quasiquote expression (e.g., `expr)".to_string(),
        Rule::primitive_literal => "a primitive literal (number, string, boolean)".to_string(),
        Rule::collection_literal => "a collection literal (list, map, set)".to_string(),
        Rule::integer => "an integer".to_string(),
        Rule::float => "a floating-point number".to_string(),
        Rule::string => "a string".to_string(),
        Rule::boolean => "a boolean (true or false)".to_string(),
        Rule::list_literal => "a list literal (e.g., [1, 2, 3])".to_string(),
        Rule::map_literal => "a map literal (e.g., {x: 1, y: 2})".to_string(),
        Rule::set_literal => "a set literal (e.g., {1, 2, 3})".to_string(),
        Rule::map_entry => "a map entry (e.g., key: value)".to_string(),
        Rule::pattern => "a pattern (variable, literal, or data structure)".to_string(),
        Rule::binding => "a binding (e.g., x = 10)".to_string(),
        Rule::wildcard => "a wildcard pattern (_)".to_string(),
        Rule::type_annotation_pattern => "a type annotation pattern (e.g., x: Int)".to_string(),
        Rule::list_pattern => "a list pattern (e.g., [x, y, z])".to_string(),
        Rule::map_pattern => "a map pattern (e.g., {x, y})".to_string(),
        Rule::set_pattern => "a set pattern (e.g., {x, y})".to_string(),
        Rule::map_pattern_entry => "a map pattern entry (e.g., key: value)".to_string(),
        Rule::type_expr => "a type expression (e.g., Int, String, [Int])".to_string(),
        Rule::type_primary => "a type primary (e.g., Int, [Int])".to_string(),
        Rule::type_infix_op => "a type infix operator (e.g., ->, +, *)".to_string(),
        Rule::type_function_op => "a function type operator (->)".to_string(),
        Rule::type_product_op => "a product type operator (*)".to_string(),
        Rule::type_sum_op => "a sum type operator (+)".to_string(),
        Rule::type_map_op => "a map type operator (:->)".to_string(),
        Rule::type_identifier => "a type identifier (e.g., Int, String)".to_string(),
        Rule::list_type => "a list type (e.g., [Int])".to_string(),
        Rule::set_type => "a set type (e.g., {String})".to_string(),
        Rule::option_type => "an option type (e.g., ?Int)".to_string(),
        // Other rules...
        _ => format!("a {:?}", rule),
    }
}

/// Creates a nicely formatted error report from a BorfError
///
/// This is the recommended way to display errors to users.
pub fn create_report(error: BorfError, _source_name: Option<String>) -> miette::Report {
    match error {
        BorfError::Parse(err) => create_parse_report(*err, None),
        BorfError::Evaluation(err) => miette::Report::new(BorfError::Evaluation(err)),
        BorfError::Io(err) => miette::Report::new(BorfError::Io(err)),
    }
}

/// Creates a nicely formatted error report directly from a ParseError
pub fn create_parse_report(error: ParseError, _source_name: Option<String>) -> miette::Report {
    // Similar to create_report, ensure source name is included.
    // Cloning might be necessary.
    miette::Report::new(error)
}

// Implement From for BorfError to handle boxing ParseError
impl From<ParseError> for BorfError {
    fn from(error: ParseError) -> Self {
        BorfError::Parse(Box::new(error))
    }
}
