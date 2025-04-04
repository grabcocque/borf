//! Parser module for the Borf programming language.
//!
//! This module organizes the parsing functionality into submodules for better maintainability.

pub mod ast;
pub mod category;
pub mod common_expr;
pub mod directives;
pub mod error;
pub mod expressions;
pub mod laws;

#[cfg(test)]
mod pratt_tests;
#[cfg(test)]
mod tests;

// Re-export the key types and functions
pub use ast::*;
pub use category::parse_category_def;
pub use directives::{parse_export_directive, parse_import_directive};
pub use laws::parse_law;

// Explicit imports (remove duplicates)
// Remove shadowed AST imports, keep only specific error types
// use crate::parser::ast::{Atom, Expression, InfixOperator, PostfixOperator, PrefixOperator, SetExpr, TypeExpr, QuantifierExpr, TypeCalculationExpr, LambdaExpr, IfExpr, LetRecExpr};
use crate::parser::error::{make_span, BorfError, NamedSource, SourceSpan, SyntaxError};

use pest::iterators::{Pair, Pairs};
use pest::pratt_parser::{Assoc, Op, PrattParser};
use pest::Parser;
use pest_derive::Parser;

/// The pest parser struct generated from the grammar defined in `borf.pest`.
#[derive(Parser)]
#[grammar = "parser/borf.pest"]
pub struct BorfParser;

// Re-export the error and utility functions from the error module
pub use error::{convert_pest_error, format_error}; // Keep these re-exports

// --- Pratt Parser Setup ---

// Lazily initialize the Pratt parser instances
lazy_static::lazy_static! {
    static ref EXPR_PRATT_PARSER: PrattParser<Rule> = {
        use Assoc::*;
        use Rule::*;

        PrattParser::new()
            // Precedence levels (highest to lowest)
            .op(Op::postfix(postfix_call_op) | Op::postfix(postfix_index_op) | Op::postfix(postfix_access_op))
            .op(Op::prefix(op_prefix_not))
            .op(Op::infix(op_multiplicative, Left))
            .op(Op::infix(op_additive, Left))
            .op(Op::infix(op_composition, Right))
            .op(Op::infix(op_set, Left))
            .op(Op::infix(op_comparison, Left))
            .op(Op::infix(op_logical_and, Left))
            .op(Op::infix(op_logical_or, Left))
            .op(Op::infix(op_implication, Right))
            .op(Op::infix(op_iff, Right))
            // Ternary needs custom handling in map_infix or a dedicated Pratt op
            .op(Op::infix(op_ternary_q, Right) | Op::infix(op_ternary_c, Right)) // Keep for now, handle in map_infix
    };

    static ref TYPE_PRATT_PARSER: PrattParser<Rule> = {
        use Assoc::*;
        use Rule::*;

        PrattParser::new()
            // Prefix operators for types
            .op(Op::prefix(optional_type) | Op::prefix(linear_type)) // Rules '?' and '!' directly? No, use rules wrapping them.
            // Need specific rules for prefix type ops if not optional_type/linear_type
            // .op(Op::prefix(type_op_optional) | Op::prefix(type_op_linear))
            .op(Op::infix(type_op_product, Left))
            .op(Op::infix(type_op_arrow, Right))
    };
}

// --- AST Building Functions using Pratt Parser ---

/// Parses a flattened sequence of terms and operators into an Expression AST node.
fn build_expr_ast(pairs: Pairs<Rule>) -> Result<Expression, Box<BorfError>> {
    // Removed clone and passing to map_infix
    EXPR_PRATT_PARSER
        .map_primary(|primary| map_primary_expr(primary))
        .map_prefix(|op, rhs| map_prefix_expr(op, rhs?))
        // Removed pairs_clone_for_err argument
        .map_infix(|lhs, op, rhs| map_infix_expr(lhs?, op, rhs?))
        .map_postfix(|lhs, op| map_postfix_expr(lhs?, op))
        .parse(pairs)
}

/// Helper for map_primary in expression Pratt parser.
fn map_primary_expr(primary: Pair<Rule>) -> Result<Expression, Box<BorfError>> {
    let span = pair_to_span(&primary);
    let src = get_named_source(primary.as_str());
    match primary.as_rule() {
        Rule::atom => {
            let inner = primary.into_inner().next().unwrap();
            match inner.as_rule() {
                Rule::ident => Ok(Expression::AtomExpr(Atom::Identifier(
                    inner.as_str().to_string(),
                ))),
                Rule::dollar_ident => Ok(Expression::AtomExpr(Atom::Identifier(
                    inner.as_str().to_string(),
                ))),
                Rule::qualified_name => {
                    // Handle qualified_name (parse out each part)
                    let parts: Vec<String> =
                        inner.into_inner().map(|p| p.as_str().to_string()).collect();
                    if parts.is_empty() {
                        return Err(Box::new(BorfError::SyntaxError(SyntaxError::new(
                            "Empty qualified name",
                            src,
                            span,
                            "Expected at least one identifier in qualified name.",
                            "Empty qualified name",
                        ))));
                    }
                    let base = parts[0].clone();
                    let access = parts[1..].to_vec();
                    Ok(Expression::QualifiedName { base, access })
                }
                Rule::law_identifier => {
                    // Get the name part of law.name
                    let name = inner.into_inner().next().unwrap().as_str().to_string();
                    Ok(Expression::LawIdentifier { name })
                }
                Rule::int => inner
                    .as_str()
                    .parse::<i64>()
                    .map(|n| Expression::AtomExpr(Atom::Integer(n)))
                    .map_err(|_| {
                        Box::new(BorfError::SyntaxError(SyntaxError::new(
                            "Invalid integer literal",
                            src,
                            span,
                            "Expected a valid 64-bit integer.",
                            "Invalid integer",
                        )))
                    }),
                Rule::boolean_literal => Ok(Expression::AtomExpr(Atom::Boolean(
                    inner.as_str() == "true",
                ))),
                Rule::string_literal => {
                    let inner_str = inner.as_str();
                    let content = inner_str
                        .get(1..inner_str.len() - 1)
                        .unwrap_or("")
                        .to_string();
                    Ok(Expression::AtomExpr(Atom::StringLiteral(content)))
                }
                Rule::symbol_literal => {
                    let symbol_name = inner.as_str().strip_prefix(":").unwrap_or("").to_string();
                    Ok(Expression::AtomExpr(Atom::Symbol(symbol_name)))
                }
                Rule::empty_set => Ok(Expression::EmptySet),
                Rule::expression => build_expr_ast(inner.into_inner()),
                _ => {
                    let rule_str = format!("{:?}", inner.as_rule());
                    Err(Box::new(BorfError::SyntaxError(SyntaxError::new(
                        &format!("Unexpected atom content rule: {}", rule_str),
                        src,
                        span,
                        "Expected identifier, literal, or parenthesized expression.",
                        "Unexpected content",
                    ))))
                }
            }
        }
        Rule::lambda => parse_lambda_expr(primary),
        Rule::if_expr => parse_if_expr(primary),
        Rule::conditional_expr_inline => parse_conditional_expr_inline(primary),
        Rule::let_rec => parse_let_rec_expr(primary),
        Rule::set_expr => Ok(Expression::AtomExpr(Atom::Set(Box::new(parse_set_expr(
            primary,
        )?)))),
        Rule::tuple_expr => parse_tuple_expr(primary),
        Rule::quantifier_expr => parse_quantifier_expr(primary),
        Rule::type_calculation_expr => parse_type_calculation_expr(primary),
        _ => {
            let rule_str = format!("{:?}", primary.as_rule());
            Err(Box::new(BorfError::SyntaxError(SyntaxError::new(
                &format!("Unexpected primary expression rule: {}", rule_str),
                src,
                span,
                "Expected an atom, lambda, if, let, set, tuple, quantifier, or type calculation.",
                "Unexpected primary",
            ))))
        }
    }
}

/// Helper for map_prefix in expression Pratt parser.
fn map_prefix_expr(op: Pair<Rule>, rhs: Expression) -> Result<Expression, Box<BorfError>> {
    let span = pair_to_span(&op);
    let src = get_named_source(op.as_str());
    match op.as_rule() {
        Rule::op_prefix_not => Ok(Expression::PrefixOp {
            op: PrefixOperator::Not,
            operand: Box::new(rhs),
        }),
        _ => {
            let rule_str = format!("{:?}", op.as_rule());
            Err(Box::new(BorfError::SyntaxError(SyntaxError::new(
                &format!("Unexpected prefix operator rule: {}", rule_str),
                src,
                span,
                "Expected known prefix operator like '$not'.",
                "Unexpected prefix op",
            ))))
        }
    }
}

/// Helper for map_infix in expression Pratt parser.
fn map_infix_expr(
    lhs: Expression,
    op: Pair<Rule>,
    rhs: Expression,
) -> Result<Expression, Box<BorfError>> {
    let op_span = pair_to_span(&op);
    let op_src = get_named_source(op.as_str());

    let operator = match op.as_rule() {
        Rule::op_composition => match op.as_str() {
            "." => InfixOperator::Compose,
            ">>" => InfixOperator::ComposeRight,
            "|>" => InfixOperator::Pipe,
            _ => {
                return Err(Box::new(BorfError::SyntaxError(SyntaxError::new(
                    "Unknown composition op",
                    op_src,
                    op_span,
                    "Expected '.', '>>', or '|>'.",
                    "Unknown op",
                ))))
            }
        },
        Rule::op_multiplicative => match op.as_str() {
            "*" => InfixOperator::Multiply,
            "/" => InfixOperator::Divide,
            _ => {
                return Err(Box::new(BorfError::SyntaxError(SyntaxError::new(
                    "Unknown multiplicative op",
                    op_src,
                    op_span,
                    "Expected '*' or '/'.",
                    "Unknown op",
                ))))
            }
        },
        Rule::op_additive => match op.as_str() {
            "+" => InfixOperator::Add,
            "-" => InfixOperator::Subtract,
            _ => {
                return Err(Box::new(BorfError::SyntaxError(SyntaxError::new(
                    "Unknown additive op",
                    op_src,
                    op_span,
                    "Expected '+' or '-'.",
                    "Unknown op",
                ))))
            }
        },
        Rule::op_set => match op.as_str() {
            "$cup" => InfixOperator::Union,
            "$cap" => InfixOperator::Intersect,
            "$subseteq" => InfixOperator::Subset,
            _ => {
                return Err(Box::new(BorfError::SyntaxError(SyntaxError::new(
                    "Unknown set op",
                    op_src,
                    op_span,
                    "Expected '$cup', '$cap', or '$subseteq'.",
                    "Unknown op",
                ))))
            }
        },
        Rule::op_comparison => match op.as_str() {
            "=" | "==" | "===" => InfixOperator::Equal,
            "$teq" => InfixOperator::TypeEqual,
            "$veq" => InfixOperator::ValueEqual,
            "$seq" => InfixOperator::StructEqual,
            "$ceq" => InfixOperator::CategoryEqual,
            "<::" => InfixOperator::Subtype,
            ">" => InfixOperator::GreaterThan,
            "<" => InfixOperator::LessThan,
            ">=" => InfixOperator::GreaterEqual,
            "<=" => InfixOperator::LessEqual,
            "$in" => InfixOperator::In,
            "$omega" => InfixOperator::Compatible,
            _ => {
                return Err(Box::new(BorfError::SyntaxError(SyntaxError::new(
                    "Unknown comparison op",
                    op_src,
                    op_span,
                    "Expected known comparison operator.",
                    "Unknown op",
                ))))
            }
        },
        Rule::op_logical_and => InfixOperator::And,
        Rule::op_logical_or => InfixOperator::Or,
        Rule::op_implication => InfixOperator::Implies,
        Rule::op_iff => InfixOperator::Iff,
        Rule::op_ternary_q => {
            // TODO: Implement proper ternary handling
            return Err(Box::new(BorfError::NotYetImplemented {
                feature: "Ternary operator (?)".to_string(),
                src: Some(op_src),
                span: Some(op_span),
            }));
        }
        Rule::op_ternary_c => {
            // This indicates a dangling ':' without a preceding '?'
            return Err(Box::new(BorfError::SyntaxError(SyntaxError::new(
                "Unexpected ternary operator (:)",
                op_src,
                op_span,
                "Expected '?' before ':' for ternary expression.",
                "Unexpected colon",
            ))));
        }
        _ => {
            // Removed unused rule_str
            // let rule_str = format!("{:?}", op.as_rule());
            return Err(Box::new(BorfError::SyntaxError(SyntaxError::new(
                &format!("Unexpected infix operator rule: {:?}", op.as_rule()), // Use op.as_rule() directly
                op_src,
                op_span,
                "Expected a known infix operator.",
                "Unexpected infix op",
            ))));
        }
    };

    Ok(Expression::InfixOp {
        lhs: Box::new(lhs),
        op: operator,
        rhs: Box::new(rhs),
    })
}

/// Helper for map_postfix in expression Pratt parser.
fn map_postfix_expr(lhs: Expression, op: Pair<Rule>) -> Result<Expression, Box<BorfError>> {
    let op_span = pair_to_span(&op);
    let op_src = get_named_source(op.as_str());
    match op.as_rule() {
        // NOTE: This implementation assumes the *next* pairs in the Pratt sequence
        // contain the arguments (function_args, index_args, access_args).
        // This might require adjusting the grammar or using custom Pratt logic.
        Rule::postfix_call_op => {
            // Placeholder - needs to actually parse function_args from subsequent pairs
            Ok(Expression::PostfixOp {
                operand: Box::new(lhs),
                op: PostfixOperator::FunctionCall(vec![]), // Placeholder!
            })
        }
        Rule::postfix_index_op => {
            // Placeholder - needs to parse index_args from subsequent pairs
            Ok(Expression::PostfixOp {
                operand: Box::new(lhs),
                op: PostfixOperator::Index(Box::new(Expression::AtomExpr(Atom::Identifier(
                    "PLACEHOLDER_INDEX".to_string(),
                )))), // Placeholder!
            })
        }
        Rule::postfix_access_op => {
            // Placeholder - needs to parse access_args (ident) from subsequent pairs
            Ok(Expression::PostfixOp {
                operand: Box::new(lhs),
                op: PostfixOperator::FieldAccess("PLACEHOLDER_FIELD".to_string()), // Placeholder!
            })
        }
        _ => {
            // Removed unused rule_str
            // let rule_str = format!("{:?}", op.as_rule());
            Err(Box::new(BorfError::SyntaxError(SyntaxError::new(
                &format!("Unexpected postfix operator rule: {:?}", op.as_rule()), // Use op.as_rule() directly
                op_src,
                op_span,
                "Expected known postfix operator rule ('(', '[', or '.').",
                "Unexpected postfix op",
            ))))
        }
    }
}

/// Parses a flattened sequence of terms and operators into a TypeExpr AST node.
fn build_type_ast(pairs: Pairs<Rule>) -> Result<TypeExpr, Box<BorfError>> {
    TYPE_PRATT_PARSER
        .map_primary(|primary| map_primary_type(primary))
        .map_prefix(|op, rhs| map_prefix_type(op, rhs?))
        .map_infix(|lhs, op, rhs| map_infix_type(lhs?, op, rhs?))
        .parse(pairs)
}

/// Helper for map_primary in type Pratt parser.
fn map_primary_type(primary: Pair<Rule>) -> Result<TypeExpr, Box<BorfError>> {
    match primary.as_rule() {
        Rule::ident => Ok(TypeExpr::Base(primary.as_str().to_string())),
        Rule::dollar_ident => Ok(TypeExpr::Base(primary.as_str().to_string())), // Treat as base type
        Rule::Sym => Ok(TypeExpr::Sym),
        Rule::set_type => {
            let inner_type_pair = primary.into_inner().next().unwrap(); // Should be type_expr
            build_type_ast(inner_type_pair.into_inner()).map(|t| TypeExpr::Set(Box::new(t)))
        }
        Rule::list_type => {
            let inner_type_pair = primary.into_inner().next().unwrap(); // Should be type_expr
            build_type_ast(inner_type_pair.into_inner()).map(|t| TypeExpr::List(Box::new(t)))
        }
        Rule::tuple_type => {
            let types = primary
                .into_inner() // Pairs of type_expr
                .map(|pair| build_type_ast(pair.into_inner()))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(TypeExpr::Tuple(types))
        }
        Rule::type_expr => build_type_ast(primary.into_inner()), // Parenthesized type expression
        // optional_type and linear_type are handled as prefix ops now
        // Rule::optional_type => { ... }
        // Rule::linear_type => { ... }
        _ => {
            // Removed unused rule_str
            // let rule_str = format!("{:?}", primary.as_rule());
            let span = pair_to_span(&primary);
            let src = get_named_source(primary.as_str());
            Err(Box::new(BorfError::SyntaxError(SyntaxError::new(
                &format!("Unexpected primary type rule: {:?}", primary.as_rule()), // Use primary.as_rule() directly
                src,
                span,
                "Expected identifier, Sym, set, list, tuple, or parenthesized type.",
                "Unexpected type primary",
            ))))
        }
    }
}

/// Helper for map_prefix in type Pratt parser.
fn map_prefix_type(op: Pair<Rule>, rhs: TypeExpr) -> Result<TypeExpr, Box<BorfError>> {
    let span = pair_to_span(&op);
    let src = get_named_source(op.as_str());
    match op.as_rule() {
        Rule::optional_type => Ok(TypeExpr::Optional(Box::new(rhs))),
        Rule::linear_type => Ok(TypeExpr::Linear(Box::new(rhs))),
        _ => {
            // Removed unused rule_str
            // let rule_str = format!("{:?}", op.as_rule());
            Err(Box::new(BorfError::SyntaxError(SyntaxError::new(
                &format!("Unexpected prefix type operator: {:?}", op.as_rule()), // Use op.as_rule() directly
                src,
                span,
                "Expected '?' or '!'.",
                "Unexpected type prefix",
            ))))
        }
    }
}

/// Helper for map_infix in type Pratt parser.
fn map_infix_type(
    lhs: TypeExpr,
    op: Pair<Rule>,
    rhs: TypeExpr,
) -> Result<TypeExpr, Box<BorfError>> {
    let span = pair_to_span(&op);
    let src = get_named_source(op.as_str());
    match op.as_rule() {
        Rule::type_op_product => Ok(TypeExpr::Product(Box::new(lhs), Box::new(rhs))),
        Rule::type_op_arrow => Ok(TypeExpr::Arrow(Box::new(lhs), Box::new(rhs))),
        _ => {
            // Removed unused rule_str
            // let rule_str = format!("{:?}", op.as_rule());
            Err(Box::new(BorfError::SyntaxError(SyntaxError::new(
                &format!("Unexpected infix type operator: {:?}", op.as_rule()), // Use op.as_rule() directly
                src,
                span,
                "Expected '*' or '->'.",
                "Unexpected type infix",
            ))))
        }
    }
}

// --- Placeholder/Helper functions for parsing terms (To be implemented or refined) ---

fn parse_lambda_expr(pair: Pair<Rule>) -> Result<Expression, Box<BorfError>> {
    let span = pair_to_span(&pair);
    let src = get_named_source(pair.as_str());
    // Basic parsing - Refine later
    let mut inner = pair.into_inner();
    let params_pair = inner.next().ok_or_else(|| {
        Box::new(BorfError::SyntaxError(SyntaxError::new(
            "Missing lambda params",
            src.clone(),
            span,
            "Expected parameters after \\.",
            "Missing params",
        )))
    })?;
    let body_pair = inner.next().ok_or_else(|| {
        Box::new(BorfError::SyntaxError(SyntaxError::new(
            "Missing lambda body",
            src.clone(),
            span,
            "Expected body after parameters.",
            "Missing body",
        )))
    })?;

    let params = params_pair
        .into_inner()
        .map(|p| p.as_str().to_string())
        .collect();
    let body = build_expr_ast(body_pair.into_inner())?;
    Ok(Expression::Lambda(LambdaExpr {
        params,
        body: Box::new(body),
    }))
    // Err(Box::new(BorfError::NotYetImplemented { feature: "parse_lambda_expr".to_string(), src: Some(src), span: Some(span) }))
}

fn parse_if_expr(pair: Pair<Rule>) -> Result<Expression, Box<BorfError>> {
    let span = pair_to_span(&pair);
    let src = get_named_source(pair.as_str());
    Err(Box::new(BorfError::NotYetImplemented {
        feature: "parse_if_expr".to_string(),
        src: Some(src),
        span: Some(span),
    }))
}

fn parse_let_rec_expr(pair: Pair<Rule>) -> Result<Expression, Box<BorfError>> {
    let span = pair_to_span(&pair);
    let src = get_named_source(pair.as_str());
    Err(Box::new(BorfError::NotYetImplemented {
        feature: "parse_let_rec_expr".to_string(),
        src: Some(src),
        span: Some(span),
    }))
}

fn parse_set_expr(pair: Pair<Rule>) -> Result<SetExpr, Box<BorfError>> {
    let span = pair_to_span(&pair);
    let src = get_named_source(pair.as_str());
    Err(Box::new(BorfError::NotYetImplemented {
        feature: "parse_set_expr".to_string(),
        src: Some(src),
        span: Some(span),
    }))
}

fn parse_tuple_expr(pair: Pair<Rule>) -> Result<Expression, Box<BorfError>> {
    let span = pair_to_span(&pair);
    let src = get_named_source(pair.as_str());
    Err(Box::new(BorfError::NotYetImplemented {
        feature: "parse_tuple_expr".to_string(),
        src: Some(src),
        span: Some(span),
    }))
}

fn parse_quantifier_expr(pair: Pair<Rule>) -> Result<Expression, Box<BorfError>> {
    let span = pair_to_span(&pair);
    let src = get_named_source(pair.as_str());
    Err(Box::new(BorfError::NotYetImplemented {
        feature: "parse_quantifier_expr".to_string(),
        src: Some(src),
        span: Some(span),
    }))
}

fn parse_type_calculation_expr(pair: Pair<Rule>) -> Result<Expression, Box<BorfError>> {
    let span = pair_to_span(&pair);
    let src = get_named_source(pair.as_str());
    Err(Box::new(BorfError::NotYetImplemented {
        feature: "parse_type_calculation_expr".to_string(),
        src: Some(src),
        span: Some(span),
    }))
}

// Add this function below parse_if_expr
fn parse_conditional_expr_inline(pair: Pair<Rule>) -> Result<Expression, Box<BorfError>> {
    let span = pair_to_span(&pair);
    let src = get_named_source(pair.as_str());

    let mut inner = pair.into_inner();

    let condition_pair = inner.next().ok_or_else(|| {
        Box::new(BorfError::SyntaxError(SyntaxError::new(
            "Missing condition in ternary expression",
            src.clone(),
            span,
            "Expected condition before '?'.",
            "Missing condition",
        )))
    })?;

    let if_true_pair = inner.next().ok_or_else(|| {
        Box::new(BorfError::SyntaxError(SyntaxError::new(
            "Missing 'true' branch in ternary expression",
            src.clone(),
            span,
            "Expected expression after '?'.",
            "Missing 'true' branch",
        )))
    })?;

    let if_false_pair = inner.next().ok_or_else(|| {
        Box::new(BorfError::SyntaxError(SyntaxError::new(
            "Missing 'false' branch in ternary expression",
            src.clone(),
            span,
            "Expected expression after ':'.",
            "Missing 'false' branch",
        )))
    })?;

    let condition = build_expr_ast(condition_pair.into_inner())?;
    let if_true = build_expr_ast(if_true_pair.into_inner())?;
    let if_false = build_expr_ast(if_false_pair.into_inner())?;

    Ok(Expression::TernaryOp {
        condition: Box::new(condition),
        if_true: Box::new(if_true),
        if_false: Box::new(if_false),
    })
}

// --- Main Parsing Logic ---

// Storage for tracking file contents to improve error reporting
thread_local! {
    static CURRENT_SOURCE: std::cell::RefCell<Option<(String, String)>> = const { std::cell::RefCell::new(None) };
}

/// Sets the current source file name and content for better error reporting.
///
/// This function should be called before parsing a file to ensure that error
/// messages can reference the original source code.
///
/// # Arguments
///
/// * `name` - The name or path of the source file
/// * `content` - The content of the source file
pub fn set_current_source(name: &str, content: String) {
    CURRENT_SOURCE.with(|cell| {
        *cell.borrow_mut() = Some((name.to_string(), content));
    });
}

/// Gets the current source file name and content.
///
/// # Returns
///
/// * `Option<(String, String)>` - A tuple containing the name and content of the current source file, if set
pub fn get_current_source() -> Option<(String, String)> {
    CURRENT_SOURCE.with(|cell| cell.borrow().clone())
}

/// Parses the entire Borf program input into a vector of top-level items.
///
/// This is the main entry point for parsing Borf code. It takes a string of Borf code
/// and returns a Result containing either a vector of successfully parsed top-level items
/// or an error describing what went wrong during parsing.
///
/// # Arguments
///
/// * `input` - A string slice containing the Borf program to parse
///
/// # Returns
///
/// * `Result<Vec<TopLevelItem>, Box<BorfError>>` - The parsing result, either:
///   - `Ok(Vec<TopLevelItem>)` - The successfully parsed top-level items
///   - `Err(Box<BorfError>)` - An error explaining what went wrong during parsing
pub fn parse_program(input: &str) -> Result<Vec<TopLevelItem>, Box<BorfError>> {
    // Print input for debugging (only during tests)
    #[cfg(test)]
    eprintln!(
        "=== Input to parse_program ===\n{}\n===========================",
        input
    );

    // Store the input for error reporting
    set_current_source("input.borf", input.to_string());

    let mut parsed = BorfParser::parse(Rule::program, input)
        .map_err(|e| Box::new(convert_pest_error(e, "input.borf", input)))?;

    let program_pair = parsed.next().ok_or_else(|| {
        let src = get_named_source(input); // Use helper
        let span = make_span(0, 1); // Point to start of file
        Box::new(BorfError::SyntaxError(SyntaxError::new(
            "No 'program' rule found",
            src,
            span,
            "Ensure the input contains valid Borf code",
            "Expected program here",
        )))
    })?;

    if program_pair.as_rule() != Rule::program {
        let src = get_named_source(input); // Use helper
        let span = make_span(0, 1); // Point to start of file
        return Err(Box::new(BorfError::SyntaxError(SyntaxError::new(
            &format!(
                "Expected 'program' rule, found {:?}",
                program_pair.as_rule()
            ),
            src,
            span,
            "The parser expected a full program",
            "Expected program here",
        ))));
    }

    let mut items = Vec::new();

    for element in program_pair.into_inner() {
        let item_result = match element.as_rule() {
            Rule::statement => {
                // Get the actual statement rule inside
                let inner_element = element.into_inner().next().unwrap();
                parse_statement(inner_element)
            }
            Rule::EOI => Ok(None), // Ignore End Of Input marker
            _ => {
                let rule_str = format!("{:?}", element.as_rule());
                let span = pair_to_span(&element);
                let src = get_named_source(input);

                Err(Box::new(BorfError::SyntaxError(SyntaxError::new(
                    &format!("Unexpected top-level element: {}", rule_str),
                    src,
                    span,
                    "Only module, primitive, export, import, or expression statements are allowed",
                    &format!("Unexpected {} here", rule_str),
                ))))
            }
        };

        if let Some(item) = item_result? {
            items.push(item);
        }
    }

    Ok(items)
}

/// Parses a single statement pair into a TopLevelItem.
fn parse_statement(pair: Pair<Rule>) -> Result<Option<TopLevelItem>, Box<BorfError>> {
    match pair.as_rule() {
        Rule::module_declaration => {
            // TODO: Implement parse_module_declaration
            Ok(None) // Placeholder
                     // Ok(Some(TopLevelItem::Module(parse_module_declaration(pair)?)))
        }
        Rule::primitive_declaration => {
            // TODO: Implement parse_primitive_declaration
            Ok(None) // Placeholder
                     // Ok(Some(TopLevelItem::Primitive(parse_primitive_declaration(pair)?)))
        }
        Rule::export_statement => Ok(Some(TopLevelItem::Export(parse_export_directive(pair)?))),
        Rule::import_statement => Ok(Some(TopLevelItem::Import(parse_import_directive(pair)?))),
        Rule::expression_statement => {
            let expression_pair = pair
                .into_inner()
                .find(|p| p.as_rule() == Rule::expression)
                .unwrap();
            let expr = build_expr_ast(expression_pair.into_inner())?;
            Ok(Some(TopLevelItem::ExpressionStatement(expr)))
        }
        // Rule::comment_decl => Ok(None), // Skip comments
        _ => {
            let rule_str = format!("{:?}", pair.as_rule());
            let span = pair_to_span(&pair);
            let src = get_named_source(pair.as_str()); // Get source for specific pair

            Err(Box::new(BorfError::SyntaxError(SyntaxError::new(
                &format!("Unexpected statement type: {}", rule_str),
                src,
                span,
                "Expected module, primitive, export, import, or expression statement",
                &format!("Unexpected {} here", rule_str),
            ))))
        }
    }
}

/// Creates a span from a Pair for error reporting
///
/// # Arguments
///
/// * `pair` - A reference to a Pair from the pest parser
///
/// # Returns
///
/// * `SourceSpan` - A source span representing the span of the pair in the input
pub fn pair_to_span(pair: &Pair<Rule>) -> SourceSpan {
    let span = pair.as_span();
    error::make_span(span.start(), span.end() - span.start())
}

/// Gets a named source from the current input for error reporting
///
/// # Arguments
///
/// * `input` - The input string being parsed
///
/// # Returns
///
/// * `NamedSource<String>` - A named source for the input
pub fn get_named_source(input: &str) -> NamedSource<String> {
    if let Some((name, _)) = get_current_source() {
        NamedSource::new(name, input.to_string())
    } else {
        NamedSource::new("unknown.borf", input.to_string())
    }
}
