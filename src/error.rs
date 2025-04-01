use miette::Diagnostic;
use thiserror::Error;

/// The main error type for the Borf interaction calculus implementation.
#[derive(Error, Diagnostic, Debug)]
pub enum BorfError {
    #[error(transparent)]
    #[diagnostic(code(borf::io_error), help("An input/output operation failed."))]
    IoError(#[from] std::io::Error),

    // Placeholder for general parser errors - refine later
    #[error("Parser error: {0}")]
    #[diagnostic(
        code(borf::parser),
        help("The input code could not be parsed correctly.")
    )]
    ParserError(String),

    // Example of integrating a specific, detailed parser error
    #[error(transparent)]
    #[diagnostic(code(borf::parser::unexpected_token))]
    UnexpectedToken(#[from] UnexpectedTokenError),

    // Placeholder for general semantic errors - refine later
    #[error("Semantic error: {0}")]
    #[diagnostic(
        code(borf::semantic),
        help("A semantic rule of the language was violated.")
    )]
    SemanticError(String),

    // Placeholder for general runtime errors - refine later
    #[error("Runtime error: {0}")]
    #[diagnostic(
        code(borf::runtime),
        help("An error occurred during the execution of the interaction net.")
    )]
    RuntimeError(String),
    // --- Add more specific error variants below as needed ---
}

// --- Specific Error Structs ---

/// Represents an error where an unexpected token was encountered during parsing.
#[derive(Error, Diagnostic, Debug)]
#[error("Unexpected token encountered")]
#[diagnostic(help("Expected one of: {expected}"))]
pub struct UnexpectedTokenError {
    /// A snippet of the source code where the error occurred.
    /// Use `miette::NamedSource` when reading from files for better context.
    #[source_code]
    pub src: String, // Or miette::NamedSource

    /// The specific location (span) of the unexpected token.
    #[label("This token is unexpected")]
    pub span: miette::SourceSpan,

    /// A description of what tokens were expected instead.
    pub expected: String,
}

// Add other specific error structs like `UndefinedAgentError`, `TypeError`, etc. here.
// Remember to add corresponding variants to `BorfError` and potentially `#[from]` implementations.
// Example:
// #[derive(Error, Diagnostic, Debug)]
// #[error("Undefined agent: {agent_name}")]
// #[diagnostic(code(borf::semantic::undefined_agent), help("Ensure '{agent_name}' is defined before use."))]
// pub struct UndefinedAgentError {
//     pub agent_name: String,
//     #[source_code]
//     pub src: String, // Or miette::NamedSource
//     #[label("'{agent_name}' used here is not defined")]
//     pub span: miette::SourceSpan,
// }
// Then in BorfError:
// #[error(transparent)]
// #[diagnostic(code(borf::semantic::undefined_agent))]
// UndefinedAgent(#[from] UndefinedAgentError),
