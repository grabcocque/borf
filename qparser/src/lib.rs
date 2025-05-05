// Library root for Q parser crate

pub mod ast;
pub mod parser;

/// Parse a Q expression from the input string.
/// Returns the AST on success, or a parse error.
/// Parse a Q expression from the input string.
/// Returns the AST on success, or a stringified error.
use crate::ast::Expr;

/// Parse a Q expression from the input string.
/// Returns the AST on success, or a stringified error.
/// Parse a Q expression from the input string.
/// Returns the AST on success, or a stringified list of parse errors.
pub fn parse(input: &str) -> Result<Expr, String> {
    parser::parse_expr(input)
}

/// Parse and evaluate a Q expression.
/// Allocates AST in a bump arena that is dropped immediately.
/// Returns formatted result or error.
pub fn eval_str(input: &str) -> Result<String, String> {
    let expr = parse(input)?;
    match expr.eval() {
        Ok(res) => Ok(res.to_string()),
        Err(err) => Err(err),
    }
}

#[cfg(test)]
mod tests;
