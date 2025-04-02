use miette::{Diagnostic, Report};
use pest;
use std::collections::HashMap;
use thiserror::Error;

// Re-export these types for use in other modules
pub use miette::{NamedSource, SourceSpan};

// Forward declare the Rule enum from the parser module
pub use crate::parser::Rule;

/// Main source cache to keep track of source files for better error reporting
#[derive(Debug, Default, Clone)]
pub struct SourceCache {
    files: HashMap<String, NamedSource<String>>,
}

impl SourceCache {
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
        }
    }

    pub fn add_file(&mut self, name: &str, content: String) {
        self.files
            .insert(name.to_string(), NamedSource::new(name, content));
    }

    pub fn get_file(&self, name: &str) -> Option<&NamedSource<String>> {
        self.files.get(name)
    }
}

/// Severity levels for diagnostics
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
    Hint,
}

/// Meta information for error codes to enable documentation and tracking
#[derive(Debug, Clone)]
pub struct ErrorMeta {
    pub code: String,
    pub url: Option<String>,
    pub severity: Severity,
}

impl ErrorMeta {
    pub fn new(code: &str, severity: Severity) -> Self {
        let url = Some(format!("https://docs.borf-lang.org/errors/{}", code));
        Self {
            code: code.to_string(),
            url,
            severity,
        }
    }

    pub fn error(code: &str) -> Self {
        Self::new(code, Severity::Error)
    }

    pub fn warning(code: &str) -> Self {
        Self::new(code, Severity::Warning)
    }

    pub fn info(code: &str) -> Self {
        Self::new(code, Severity::Info)
    }

    pub fn hint(code: &str) -> Self {
        Self::new(code, Severity::Hint)
    }
}

/// The main error type for the Borf interaction calculus implementation.
#[derive(Error, Diagnostic, Debug)]
pub enum BorfError {
    #[error(transparent)]
    #[diagnostic(code(borf::io_error), help("An input/output operation failed."))]
    IoError(#[from] std::io::Error),

    // Parser errors with source spans
    #[error(transparent)]
    #[diagnostic(code(borf::parser::unexpected_token))]
    UnexpectedToken(#[from] UnexpectedTokenError),

    #[error(transparent)]
    #[diagnostic(code(borf::parser::malformed_category))]
    MalformedCategory(#[from] CategoryParseError),

    #[error(transparent)]
    #[diagnostic(code(borf::parser::mapping))]
    MappingError(#[from] MappingParseError),

    #[error(transparent)]
    #[diagnostic(code(borf::parser::composition))]
    CompositionError(#[from] CompositionParseError),

    #[error(transparent)]
    #[diagnostic(code(borf::parser::syntax))]
    SyntaxError(#[from] SyntaxError),

    // Semantic errors
    #[error(transparent)]
    #[diagnostic(code(borf::semantic::undefined_symbol))]
    UndefinedSymbol(#[from] UndefinedSymbolError),

    #[error(transparent)]
    #[diagnostic(code(borf::semantic::type_error))]
    TypeError(#[from] TypeError),

    #[error(transparent)]
    #[diagnostic(code(borf::semantic::inconsistent_composition))]
    InconsistentComposition(#[from] InconsistentCompositionError),

    #[error(transparent)]
    #[diagnostic(code(borf::semantic::category_structure))]
    CategoryStructureError(#[from] CategoryStructureError),

    // Runtime errors
    #[error(transparent)]
    #[diagnostic(code(borf::runtime::reduction_error))]
    ReductionError(#[from] ReductionError),

    #[error(transparent)]
    #[diagnostic(code(borf::runtime::termination_error))]
    TerminationError(#[from] TerminationError),

    // Legacy fallback for migration
    #[error("Parser error: {0}")]
    #[diagnostic(
        code(borf::parser),
        help("The input code could not be parsed correctly.")
    )]
    ParserError(String),

    #[error("Semantic error: {0}")]
    #[diagnostic(
        code(borf::semantic),
        help("A semantic rule of the language was violated.")
    )]
    SemanticError(String),

    #[error("Runtime error: {0}")]
    #[diagnostic(
        code(borf::runtime),
        help("An error occurred during the execution of the interaction net.")
    )]
    RuntimeError(String),
}

// --- Category Parsing Errors ---

/// Error when parsing a category definition
#[derive(Error, Debug, Diagnostic)]
#[error("Invalid category definition: {message}")]
#[diagnostic(
    code(borf::e0001),
    url("https://docs.borf-lang.org/errors/e0001"),
    help("{help}")
)]
pub struct CategoryParseError {
    pub message: String,
    #[source_code]
    pub src: NamedSource<String>,
    #[label("Invalid category definition")]
    pub span: SourceSpan,
    pub help: String,
    pub label: String,
    #[related]
    pub related: Vec<RelatedError>,
}

impl CategoryParseError {
    pub fn new(
        message: &str,
        src: NamedSource<String>,
        span: SourceSpan,
        help: &str,
        label: &str,
    ) -> Self {
        Self {
            message: message.to_string(),
            src,
            span,
            help: help.to_string(),
            label: label.to_string(),
            related: Vec::new(),
        }
    }

    pub fn with_related(mut self, related: Vec<RelatedError>) -> Self {
        self.related = related;
        self
    }
}

// --- Mapping Parse Errors ---

/// Error when parsing a mapping declaration
#[derive(Error, Debug, Diagnostic)]
#[error("Failed to parse mapping declaration: {message}")]
#[diagnostic(
    code(borf::e0002),
    url("https://docs.borf-lang.org/errors/e0002"),
    help("Check the mapping syntax: name:domain $to codomain")
)]
pub struct MappingParseError {
    pub message: String,
    #[source_code]
    pub src: NamedSource<String>,
    #[label("Error in mapping")]
    pub span: SourceSpan,
    pub label: String,
    #[related]
    pub related: Vec<RelatedError>,
}

impl MappingParseError {
    pub fn new(message: &str, src: NamedSource<String>, span: SourceSpan, label: &str) -> Self {
        Self {
            message: message.to_string(),
            src,
            span,
            label: label.to_string(),
            related: Vec::new(),
        }
    }

    pub fn with_related(mut self, related: Vec<RelatedError>) -> Self {
        self.related = related;
        self
    }
}

// --- Composition Parse Errors ---

/// Error when parsing composition expressions
#[derive(Error, Debug, Diagnostic)]
#[error("Failed to parse composition: {message}")]
#[diagnostic(
    code(borf::e0003),
    url("https://docs.borf-lang.org/errors/e0003"),
    help("Composition should be formatted as: result = f $comp g where g is applied first")
)]
pub struct CompositionParseError {
    pub message: String,
    #[source_code]
    pub src: NamedSource<String>,
    #[label("Invalid composition")]
    pub span: SourceSpan,
    pub label: String,
    #[related]
    pub related: Vec<RelatedError>,
}

impl CompositionParseError {
    pub fn new(message: &str, src: NamedSource<String>, span: SourceSpan, label: &str) -> Self {
        Self {
            message: message.to_string(),
            src,
            span,
            label: label.to_string(),
            related: Vec::new(),
        }
    }

    pub fn with_related(mut self, related: Vec<RelatedError>) -> Self {
        self.related = related;
        self
    }
}

// --- Syntax Errors ---

/// General syntax error for borf code
#[derive(Error, Debug, Diagnostic)]
#[error("Syntax error: {message}")]
#[diagnostic(
    code(borf::e0004),
    url("https://docs.borf-lang.org/errors/e0004"),
    help("{help}")
)]
pub struct SyntaxError {
    pub message: String,
    #[source_code]
    pub src: NamedSource<String>,
    #[label("Syntax error")]
    pub span: SourceSpan,
    pub help: String,
    pub label: String,
    #[related]
    pub related: Vec<RelatedError>,
}

impl SyntaxError {
    pub fn new(
        message: &str,
        src: NamedSource<String>,
        span: SourceSpan,
        help: &str,
        label: &str,
    ) -> Self {
        Self {
            message: message.to_string(),
            src,
            span,
            help: help.to_string(),
            label: label.to_string(),
            related: Vec::new(),
        }
    }

    pub fn with_related(mut self, related: Vec<RelatedError>) -> Self {
        self.related = related;
        self
    }

    pub fn with_expected_tokens(
        src: NamedSource<String>,
        span: SourceSpan,
        expected: Vec<&str>,
    ) -> Self {
        let expected_str = expected.join(", ");
        let help = format!("Expected one of: {}", expected_str);
        Self::new(
            "Unexpected token",
            src,
            span,
            &help,
            "Unexpected token here",
        )
    }
}

/// Unexpected token error, with detailed expected token information
#[derive(Error, Debug, Diagnostic)]
#[error("Unexpected token encountered")]
#[diagnostic(
    code(borf::e0005),
    url("https://docs.borf-lang.org/errors/e0005"),
    help("Expected: {expected}")
)]
pub struct UnexpectedTokenError {
    #[source_code]
    pub src: NamedSource<String>,
    #[label("Unexpected token")]
    pub span: SourceSpan,
    pub expected: String,
    #[related]
    pub related: Vec<RelatedError>,
}

impl UnexpectedTokenError {
    pub fn new(src: NamedSource<String>, span: SourceSpan, expected: &str) -> Self {
        Self {
            src,
            span,
            expected: expected.to_string(),
            related: Vec::new(),
        }
    }

    pub fn with_related(mut self, related: Vec<RelatedError>) -> Self {
        self.related = related;
        self
    }
}

// --- Semantic Errors ---

/// Error for undefined symbols
#[derive(Error, Debug, Diagnostic)]
#[error("Undefined symbol: '{symbol}'")]
#[diagnostic(
    code(borf::e0010),
    url("https://docs.borf-lang.org/errors/e0010"),
    help("Ensure '{symbol}' is defined before use")
)]
pub struct UndefinedSymbolError {
    pub symbol: String,
    #[source_code]
    pub src: NamedSource<String>,
    #[label("Symbol not defined")]
    pub span: SourceSpan,
    #[related]
    pub related: Vec<RelatedError>,
}

impl UndefinedSymbolError {
    pub fn new(symbol: &str, src: NamedSource<String>, span: SourceSpan) -> Self {
        Self {
            symbol: symbol.to_string(),
            src,
            span,
            related: Vec::new(),
        }
    }

    pub fn with_related(mut self, related: Vec<RelatedError>) -> Self {
        self.related = related;
        self
    }

    pub fn with_similar_symbols(mut self, similar: Vec<String>) -> Self {
        if !similar.is_empty() {
            let details = format!("Similar symbols: {}", similar.join(", "));
            let related = RelatedError::new("Did you mean one of these?", &details);
            self.related.push(related);
        }
        self
    }
}

/// Type errors in borf code
#[derive(Error, Debug, Diagnostic)]
#[error("Type error: {message}")]
#[diagnostic(
    code(borf::e0011),
    url("https://docs.borf-lang.org/errors/e0011"),
    help("{help}")
)]
pub struct TypeError {
    pub message: String,
    #[source_code]
    pub src: NamedSource<String>,
    #[label("Type mismatch")]
    pub span: SourceSpan,
    pub help: String,
    pub label: String,
    #[related]
    pub related: Vec<RelatedError>,
}

impl TypeError {
    pub fn new(
        message: &str,
        src: NamedSource<String>,
        span: SourceSpan,
        help: &str,
        label: &str,
    ) -> Self {
        Self {
            message: message.to_string(),
            src,
            span,
            help: help.to_string(),
            label: label.to_string(),
            related: Vec::new(),
        }
    }

    pub fn with_related(mut self, related: Vec<RelatedError>) -> Self {
        self.related = related;
        self
    }

    pub fn with_expected_type(
        actual: &str,
        expected: &str,
        src: NamedSource<String>,
        span: SourceSpan,
    ) -> Self {
        let message = format!("Expected type '{}' but found '{}'", expected, actual);
        let help = format!("Make sure the expression has type '{}'", expected);
        let label = format!("Has type '{}' but expected '{}'", actual, expected);
        Self::new(&message, src, span, &help, &label)
    }
}

/// Error for inconsistent compositions
#[derive(Error, Debug, Diagnostic)]
#[error("Inconsistent composition: {message}")]
#[diagnostic(
    code(borf::e0012),
    url("https://docs.borf-lang.org/errors/e0012"),
    help("Ensure the codomain of each function matches the domain of the next function")
)]
pub struct InconsistentCompositionError {
    pub message: String,
    #[source_code]
    pub src: NamedSource<String>,
    #[label("Inconsistent composition")]
    pub span: SourceSpan,
    pub label: String,
    #[related]
    pub related: Vec<RelatedError>,
}

impl InconsistentCompositionError {
    pub fn new(message: &str, src: NamedSource<String>, span: SourceSpan, label: &str) -> Self {
        Self {
            message: message.to_string(),
            src,
            span,
            label: label.to_string(),
            related: Vec::new(),
        }
    }

    pub fn with_related(mut self, related: Vec<RelatedError>) -> Self {
        self.related = related;
        self
    }

    pub fn type_mismatch(
        f_name: &str,
        g_name: &str,
        f_codomain: &str,
        g_domain: &str,
        src: NamedSource<String>,
        span: SourceSpan,
    ) -> Self {
        let message = format!(
            "Cannot compose '{}' with '{}': codomain '{}' doesn't match domain '{}'",
            f_name, g_name, f_codomain, g_domain
        );
        let label = "Composition fails here".to_string();

        let mut error = Self::new(&message, src, span, &label);

        let details = format!(
            "'{}'s codomain is '{}', but '{}'s domain is '{}'",
            f_name, f_codomain, g_name, g_domain
        );

        let related = RelatedError::new("Type mismatch in composition", &details);

        error.related.push(related);
        error
    }
}

/// Error for category structure problems
#[derive(Error, Debug, Diagnostic)]
#[error("Category structure error: {message}")]
#[diagnostic(
    code(borf::e0013),
    url("https://docs.borf-lang.org/errors/e0013"),
    help("{help}")
)]
pub struct CategoryStructureError {
    pub message: String,
    #[source_code]
    pub src: NamedSource<String>,
    #[label("Category structure error")]
    pub span: SourceSpan,
    pub help: String,
    pub label: String,
    #[related]
    pub related: Vec<RelatedError>,
}

impl CategoryStructureError {
    pub fn new(
        message: &str,
        src: NamedSource<String>,
        span: SourceSpan,
        help: &str,
        label: &str,
    ) -> Self {
        Self {
            message: message.to_string(),
            src,
            span,
            help: help.to_string(),
            label: label.to_string(),
            related: Vec::new(),
        }
    }

    pub fn with_related(mut self, related: Vec<RelatedError>) -> Self {
        self.related = related;
        self
    }

    pub fn missing_required_element(
        category_name: &str,
        missing: &str,
        src: NamedSource<String>,
        span: SourceSpan,
    ) -> Self {
        let message = format!(
            "Category '{}' is missing required element '{}'",
            category_name, missing
        );
        let help = format!("Add '{}' to the category definition", missing);
        let label = format!("Category defined here is missing '{}'", missing);
        Self::new(&message, src, span, &help, &label)
    }
}

// --- Runtime Errors ---

/// Errors during reduction
#[derive(Error, Debug, Diagnostic)]
#[error("Reduction error: {message}")]
#[diagnostic(
    code(borf::e0020),
    url("https://docs.borf-lang.org/errors/e0020"),
    help("{help}")
)]
pub struct ReductionError {
    pub message: String,
    #[source_code]
    pub src: Option<NamedSource<String>>,
    #[label("Error during reduction")]
    pub span: Option<SourceSpan>,
    pub help: String,
    pub label: String,
    #[related]
    pub related: Vec<RelatedError>,
}

impl ReductionError {
    pub fn new(message: &str, help: &str) -> Self {
        Self {
            message: message.to_string(),
            src: None,
            span: None,
            help: help.to_string(),
            label: String::new(),
            related: Vec::new(),
        }
    }

    pub fn with_location(
        mut self,
        src: NamedSource<String>,
        span: SourceSpan,
        label: &str,
    ) -> Self {
        self.src = Some(src);
        self.span = Some(span);
        self.label = label.to_string();
        self
    }

    pub fn with_related(mut self, related: Vec<RelatedError>) -> Self {
        self.related = related;
        self
    }
}

/// Termination errors
#[derive(Error, Debug, Diagnostic)]
#[error("Termination error: {message}")]
#[diagnostic(
    code(borf::e0021),
    url("https://docs.borf-lang.org/errors/e0021"),
    help("{help}")
)]
pub struct TerminationError {
    pub message: String,
    #[source_code]
    pub src: Option<NamedSource<String>>,
    #[label("Non-terminating execution")]
    pub span: Option<SourceSpan>,
    pub help: String,
    pub label: String,
    #[related]
    pub related: Vec<RelatedError>,
}

impl TerminationError {
    pub fn new(message: &str, help: &str) -> Self {
        Self {
            message: message.to_string(),
            src: None,
            span: None,
            help: help.to_string(),
            label: String::new(),
            related: Vec::new(),
        }
    }

    pub fn with_location(
        mut self,
        src: NamedSource<String>,
        span: SourceSpan,
        label: &str,
    ) -> Self {
        self.src = Some(src);
        self.span = Some(span);
        self.label = label.to_string();
        self
    }

    pub fn with_related(mut self, related: Vec<RelatedError>) -> Self {
        self.related = related;
        self
    }

    pub fn cycle_detected() -> Self {
        Self::new(
            "Reduction doesn't terminate due to a cycle",
            "The reduction process has entered a cycle and will not terminate",
        )
    }
}

// --- Related Errors Support ---

/// A related error that can be attached to the main diagnostic
#[derive(Error, Debug, Diagnostic)]
#[error("{title}")]
pub struct RelatedError {
    pub title: String,
    pub details: String,
}

impl RelatedError {
    pub fn new(title: &str, details: &str) -> Self {
        Self {
            title: title.to_string(),
            details: details.to_string(),
        }
    }
}

// --- Helper functions ---

pub fn get_help_message(error_type: &str) -> String {
    match error_type {
        "category_definition" => "Categories should be defined as @Category: { ... }".to_string(),
        "mapping_definition" => {
            "Mappings should be defined as name:domain $to codomain".to_string()
        }
        "composition" => "Composition is written as f $comp g".to_string(),
        "export" => "Export is written as @export { ... }".to_string(),
        _ => format!("Check the syntax for {}", error_type),
    }
}

// Function to format an error with the appropriate severity
pub fn format_error(error: BorfError) -> Report {
    miette::Report::new(error)
}

// Function to format a warning
pub fn format_warning<E: Diagnostic + Send + Sync + 'static>(warning: E) -> Report {
    miette::Report::new(warning)
}

// Utility to create SourceSpan from indexes
pub fn make_span(start: usize, end: usize) -> SourceSpan {
    SourceSpan::new(start.into(), end - start)
}

// Utility to create SourceSpan from line and column information
pub fn make_span_from_line_col(line: usize, column: usize, length: usize) -> SourceSpan {
    // This is a simplified conversion - a real implementation would need to count chars
    let start = (line * 80) + column; // Very rough estimation assuming 80 chars per line
    SourceSpan::new(start.into(), length)
}

// Utilities for handling Pest errors
pub fn convert_pest_error(
    error: pest::error::Error<Rule>,
    source_name: &str,
    source: &str,
) -> BorfError {
    let source_code = NamedSource::new(source_name, source.to_string());

    // Get the position information from the pest error
    let (_line_col, span) = match error.line_col {
        pest::error::LineColLocation::Pos((line, col)) => {
            // Approximate the position in the source
            let line_idx = line - 1; // Convert to 0-indexed
            let col_idx = col - 1; // Convert to 0-indexed

            // Convert line/col to a rough character index
            let pos = source
                .lines()
                .take(line_idx)
                .map(|l| l.len() + 1)
                .sum::<usize>()
                + col_idx;
            ((line, col), SourceSpan::new(pos.into(), 1_usize))
        }
        pest::error::LineColLocation::Span((start_line, start_col), (end_line, end_col)) => {
            // Convert to character positions
            let start_line_idx = start_line - 1;
            let start_col_idx = start_col - 1;
            let end_line_idx = end_line - 1;
            let end_col_idx = end_col - 1;

            let start_pos = source
                .lines()
                .take(start_line_idx)
                .map(|l| l.len() + 1)
                .sum::<usize>()
                + start_col_idx;
            let end_pos = source
                .lines()
                .take(end_line_idx)
                .map(|l| l.len() + 1)
                .sum::<usize>()
                + end_col_idx;

            (
                (start_line, start_col),
                SourceSpan::new(start_pos.into(), end_pos - start_pos),
            )
        }
    };

    // Convert expected tokens
    let expected_tokens = match error.variant {
        pest::error::ErrorVariant::ParsingError {
            positives,
            negatives: _,
        } => positives
            .iter()
            .map(|rule| format!("{:?}", rule))
            .collect::<Vec<_>>(),
        _ => vec!["valid token".to_string()],
    };

    let expected = expected_tokens.join(", ");

    // Create a new UnexpectedTokenError with properly typed values
    BorfError::UnexpectedToken(UnexpectedTokenError::new(source_code, span, &expected))
}

// Export conversions from old error types to new ones
pub fn upgrade_parser_error(
    message: &str,
    source_name: &str,
    source: &str,
    span: SourceSpan,
) -> BorfError {
    BorfError::SyntaxError(SyntaxError {
        message: message.to_string(),
        src: NamedSource::new(source_name, source.to_string()),
        span,
        help: get_help_message("syntax"),
        label: "Syntax error occurred here".to_string(),
        related: Vec::new(),
    })
}

// Utility function to convert string slice to owned string
pub fn to_owned_string(s: &str) -> String {
    s.to_string()
}
