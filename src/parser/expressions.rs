//! Parsers for expressions in the Borf language.
//!
//! This module provides functions for parsing various expression types in Borf.

use super::ast::{Expression, FunctionChainExpr, IfExpr, LambdaExpr, LetRecExpr};
use super::error::BorfError;
use crate::parser::{build_expr_ast, get_named_source, pair_to_span, Rule};
use pest::iterators::Pair;

/// Parses a function chain expression (e.g., f(a)(b)(c)).
/// Represents nested function calls.
///
/// # Arguments
///
/// * `pair` - A pest Pair representing a function chain expression
///
/// # Returns
///
/// * `Result<Expression, Box<BorfError>>` - The parsed expression or an error
pub fn parse_function_chain_expr(pair: Pair<Rule>) -> Result<Expression, Box<BorfError>> {
    let _span = pair_to_span(&pair);
    let _src = get_named_source(pair.as_str());

    let mut inner = pair.into_inner();
    let base_pair = inner.next().unwrap();

    // Get the base identifier as a string (simplification - in a real parser we would handle more complex bases)
    let base = base_pair.as_str().to_string();

    // Process the chain calls
    let mut calls = Vec::new();
    for args_pair in inner {
        if args_pair.as_rule() == Rule::function_args {
            let mut arg_list = Vec::new();
            for arg_pair in args_pair.into_inner() {
                if arg_pair.as_rule() == Rule::expression {
                    arg_list.push(build_expr_ast(arg_pair.into_inner())?);
                }
            }
            calls.push(arg_list);
        }
    }

    Ok(Expression::FunctionChainCall(FunctionChainExpr {
        base, // Now it's a String as expected
        calls,
    }))
}

/// Parses a lambda expression (e.g., \x.expr or \x,y,z.expr).
///
/// # Arguments
///
/// * `pair` - A pest Pair representing a lambda expression
///
/// # Returns
///
/// * `Result<Expression, Box<BorfError>>` - The parsed expression or an error
pub fn parse_lambda_expr(pair: Pair<Rule>) -> Result<Expression, Box<BorfError>> {
    let _span = pair_to_span(&pair);
    let _src = get_named_source(pair.as_str());

    let mut inner = pair.into_inner();
    let params_pair = inner.next().unwrap();

    // Extract parameter names
    let mut params = Vec::new();
    for param in params_pair.into_inner() {
        if param.as_rule() == Rule::ident {
            params.push(param.as_str().to_string());
        }
    }

    // Parse the body expression
    let body_pair = inner.next().unwrap();
    let body = build_expr_ast(body_pair.into_inner())?;

    Ok(Expression::Lambda(LambdaExpr {
        params,
        body: Box::new(body),
    }))
}

/// Parses an if-then-else expression (e.g., if cond then expr1 else expr2).
///
/// # Arguments
///
/// * `pair` - A pest Pair representing an if-then-else expression
///
/// # Returns
///
/// * `Result<Expression, Box<BorfError>>` - The parsed expression or an error
pub fn parse_if_expr(pair: Pair<Rule>) -> Result<Expression, Box<BorfError>> {
    let _span = pair_to_span(&pair);
    let _src = get_named_source(pair.as_str());

    let mut inner = pair.into_inner();
    let condition_pair = inner.next().unwrap();
    let condition = build_expr_ast(condition_pair.into_inner())?;

    let then_pair = inner.next().unwrap();
    let then_branch = build_expr_ast(then_pair.into_inner())?;

    let else_pair = inner.next().unwrap();
    let else_branch = build_expr_ast(else_pair.into_inner())?;

    Ok(Expression::If(IfExpr {
        condition: Box::new(condition),
        then_branch: Box::new(then_branch),
        else_branch: Box::new(else_branch),
    }))
}

/// Parses a let-rec expression (e.g., let rec name params = bound_expr in body).
///
/// # Arguments
///
/// * `pair` - A pest Pair representing a let-rec expression
///
/// # Returns
///
/// * `Result<Expression, Box<BorfError>>` - The parsed expression or an error
pub fn parse_let_rec_expr(pair: Pair<Rule>) -> Result<Expression, Box<BorfError>> {
    let _span = pair_to_span(&pair);
    let _src = get_named_source(pair.as_str());

    let mut inner = pair.into_inner();
    let name_pair = inner.next().unwrap();
    let name = name_pair.as_str().to_string();

    // Extract parameter names
    let params_pair = inner.next().unwrap();
    let mut params = Vec::new();
    for param in params_pair.into_inner() {
        if param.as_rule() == Rule::ident {
            params.push(param.as_str().to_string());
        }
    }

    // Parse the bound expression and body
    let bound_expr_pair = inner.next().unwrap();
    let bound_expr = build_expr_ast(bound_expr_pair.into_inner())?;

    let body_pair = inner.next().unwrap();
    let body = build_expr_ast(body_pair.into_inner())?;

    Ok(Expression::LetRec(LetRecExpr {
        name,
        params,
        bound_expr: Box::new(bound_expr),
        body: Box::new(body),
    }))
}

/// Parses a conditional/ternary expression (e.g., a ? b : c).
///
/// # Arguments
///
/// * `pair` - A pest Pair representing a conditional expression
///
/// # Returns
///
/// * `Result<Expression, Box<BorfError>>` - The parsed expression or an error
pub fn parse_conditional_expr_inline(pair: Pair<Rule>) -> Result<Expression, Box<BorfError>> {
    let _span = pair_to_span(&pair);
    let _src = get_named_source(pair.as_str());

    let mut inner = pair.into_inner();
    let condition_pair = inner.next().unwrap();
    let condition = build_expr_ast(condition_pair.into_inner())?;

    let if_true_pair = inner.next().unwrap();
    let if_true = build_expr_ast(if_true_pair.into_inner())?;

    let if_false_pair = inner.next().unwrap();
    let if_false = build_expr_ast(if_false_pair.into_inner())?;

    Ok(Expression::TernaryOp {
        condition: Box::new(condition),
        if_true: Box::new(if_true),
        if_false: Box::new(if_false),
    })
}
