//! Parsers for common expression types in the Borf language.

use super::ast::{Expression, SetExpr, SetLiteral};
use super::error::{BorfError, SyntaxError};
use crate::parser::{get_named_source, pair_to_span, Rule};
use pest::iterators::Pair;

// Helper function to create a syntax error
pub fn create_syntax_error(
    message: &str,
    pair: &Pair<Rule>,
    help: &str,
    label: &str,
) -> Box<BorfError> {
    let span = pair_to_span(pair);
    let src = get_named_source(pair.as_str()); // Or get source context if needed
    Box::new(BorfError::SyntaxError(SyntaxError::new(
        message, src, span, help, label,
    )))
}

/// Parses a set expression.
///
/// # Arguments
///
/// * `pair` - A pest Pair representing a set expression
///
/// # Returns
///
/// * `Result<SetExpr, Box<BorfError>>` - The parsed set expression or an error
pub fn parse_set_expr(pair: Pair<Rule>) -> Result<SetExpr, Box<BorfError>> {
    let _span = pair_to_span(&pair);
    let _src = get_named_source(pair.as_str());

    // For now, just return an empty set literal
    // This will need to be expanded to handle different kinds of set expressions
    Ok(SetExpr::Literal(SetLiteral {
        elements: Vec::new(),
    }))
}

/// Parses a tuple expression.
///
/// A tuple is of the form `(expr1, expr2, ...)`.
pub fn parse_tuple(pair: Pair<Rule>) -> Result<Expression, Box<BorfError>> {
    let _span = pair_to_span(&pair);
    let _src = get_named_source(pair.as_str());

    let inner = pair.into_inner();
    let mut elements = Vec::new();

    // Create a placeholder expression for each element
    for _expr_pair in inner {
        // Use a valid Expression variant as a placeholder
        elements.push(Expression::AtomExpr(super::ast::Atom::Identifier(
            "placeholder".to_string(),
        )));
    }

    Ok(Expression::Tuple(elements))
}

// TODO: Add tests for these parsers.
