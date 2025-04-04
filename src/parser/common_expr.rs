//! Parsers for common expression types in the Borf language.

use super::ast::{
    Atom, CompositionExprRhs, Expression, IfExpr, LambdaExpr, LetRecExpr, SetExpr, SetLiteral,
};
use super::error::{BorfError, SyntaxError};
use crate::parser::{get_named_source, pair_to_span, Rule};
use pest::iterators::Pair;

// Helper function to create a syntax error
pub fn create_syntax_error(
    message: &str,
    pair: &Pair<Rule>,
    help: &str,
    context: &str,
) -> Box<BorfError> {
    let span = pair_to_span(pair);
    let src = get_named_source(pair.as_str()); // Or get source context if needed
    Box::new(BorfError::SyntaxError(SyntaxError::new(
        message, src, span, help, context,
    )))
}

/// Parses a Borf expression based on the grammar rule.
pub fn parse_expression(pair: Pair<Rule>) -> Result<Expression, Box<BorfError>> {
    // Ensure the pair is actually an expression or one of its direct variants
    match pair.as_rule() {
        Rule::expression => {
            // Descend into the actual expression variant
            parse_expression(pair.into_inner().next().unwrap())
        }
        Rule::atom => Ok(Expression::AtomExpr(parse_atom(pair)?)),
        Rule::lambda => Ok(Expression::Lambda(parse_lambda(pair)?)),
        Rule::if_expr => Ok(Expression::If(parse_if_expr(pair)?)),
        Rule::let_rec => Ok(Expression::LetRec(parse_let_rec(pair)?)),
        Rule::composition => Ok(Expression::Composition(parse_composition_rhs(pair)?)),
        Rule::set_expr => {
            // Handle set expressions
            // For now, just wrap it in an Atom representation
            Ok(Expression::AtomExpr(Atom::Set(Box::new(SetExpr::Literal(
                SetLiteral {
                    elements: vec![Expression::AtomExpr(Atom::Identifier(
                        pair.as_str().to_string(),
                    ))],
                },
            )))))
        }
        // Handle cases where an atom rule might be passed directly
        Rule::function_app
        | Rule::module_access
        | Rule::ident
        | Rule::int
        | Rule::symbol_literal
        | Rule::tuple => Ok(Expression::AtomExpr(parse_atom(pair)?)),
        _ => Err(create_syntax_error(
            &format!("Unexpected rule in parse_expression: {:?}", pair.as_rule()),
            &pair,
            "Expected an expression variant (atom, lambda, if, let_rec, composition).",
            "Invalid expression",
        )),
    }
}

/// Parses an atomic expression component.
fn parse_atom(pair: Pair<Rule>) -> Result<Atom, Box<BorfError>> {
    match pair.as_rule() {
        Rule::atom => {
            // Descend into the actual atom variant
            let inner_pair = pair.into_inner().next().unwrap();
            // Check if it's a parenthesized expression directly
            if inner_pair.as_str().starts_with('(') && inner_pair.as_rule() == Rule::expression {
                // Grammar atom -> "(" ~ expression ~ ")"
                // The inner_pair here IS the expression inside the parentheses
                Ok(Atom::Paren(Box::new(parse_expression(inner_pair)?)))
            } else {
                // Otherwise, parse the specific atom type
                parse_atom(inner_pair)
            }
        }
        Rule::function_app => {
            let mut inner = pair.into_inner();
            let func = inner.next().unwrap().as_str().to_string();
            let args_pair = inner.next().unwrap(); // function_args rule
            let mut args = Vec::new();
            for arg_pair in args_pair.into_inner() {
                if arg_pair.as_rule() == Rule::expression {
                    args.push(parse_expression(arg_pair)?);
                } else {
                    // Handle cases where it might be a simpler rule like ident directly?
                    // For now, assume function_args contains expression rules
                    return Err(create_syntax_error(
                        &format!("Unexpected rule in function args: {:?}", arg_pair.as_rule()),
                        &arg_pair,
                        "Function arguments should be expressions.",
                        "Invalid function argument",
                    ));
                }
            }
            Ok(Atom::FunctionApp { func, args })
        }
        Rule::module_access => {
            let path: Vec<String> = pair.into_inner().map(|p| p.as_str().to_string()).collect();
            Ok(Atom::ModuleAccess { path })
        }
        Rule::ident => Ok(Atom::Identifier(pair.as_str().to_string())),
        Rule::int => pair
            .as_str()
            .parse::<i64>()
            .map(Atom::Integer)
            .map_err(|e| {
                create_syntax_error(
                    &format!("Invalid integer literal: {}", e),
                    &pair,
                    "Ensure the number is a valid 64-bit integer.",
                    "Invalid integer",
                )
            }),
        Rule::symbol_literal => {
            let symbol_name = pair.as_str().trim_start_matches(':').to_string();
            Ok(Atom::Symbol(symbol_name))
        }
        Rule::tuple => {
            // Assuming Rule::tuple from grammar: "(" ~ expression ~ "," ~ expression ~ ")"
            let mut inner = pair.into_inner();
            let first = parse_expression(inner.next().unwrap())?;
            let second = parse_expression(inner.next().unwrap())?;
            Ok(Atom::Tuple(Box::new(first), Box::new(second)))
        }
        Rule::string_literal => {
            // Grammar: """ ~ (!""" ~ ANY)* ~ """
            let literal = pair.as_str();
            // Remove leading/trailing quotes
            let content = literal
                .strip_prefix('"')
                .unwrap_or(literal)
                .strip_suffix('"')
                .unwrap_or(literal)
                .to_string();
            Ok(Atom::StringLiteral(content))
        }
        _ => Err(create_syntax_error(
            &format!("Unexpected rule in parse_atom: {:?}", pair.as_rule()),
            &pair,
            "Expected an atomic expression component.",
            "Invalid atom",
        )),
    }
}

/// Parses a lambda expression.
fn parse_lambda(pair: Pair<Rule>) -> Result<LambdaExpr, Box<BorfError>> {
    let mut inner = pair.into_inner();
    let params_pair = inner.next().unwrap(); // lambda_params rule
    let mut params = Vec::new();

    // Handle both single ident and parenthesized param list
    match params_pair.as_rule() {
        Rule::lambda_params => {
            for param_ident in params_pair.into_inner() {
                if param_ident.as_rule() == Rule::ident {
                    params.push(param_ident.as_str().to_string());
                }
                // Ignore commas, parentheses if they are distinct rules within lambda_params
            }
        }
        Rule::ident => {
            // Case where lambda_params directly matches a single ident
            params.push(params_pair.as_str().to_string());
        }
        _ => {
            return Err(create_syntax_error(
                &format!(
                    "Unexpected rule for lambda parameters: {:?}",
                    params_pair.as_rule()
                ),
                &params_pair,
                "Lambda parameters should be identifiers or a list of identifiers.",
                "Invalid lambda parameters",
            ))
        }
    }

    let body_pair = inner.next().unwrap(); // Should be the expression rule
    let body = parse_expression(body_pair)?;

    Ok(LambdaExpr {
        params,
        body: Box::new(body),
    })
}

/// Parses an if-then-else expression.
fn parse_if_expr(pair: Pair<Rule>) -> Result<IfExpr, Box<BorfError>> {
    // Grammar: "if" ~ expression ~ compare_op? ~ expression ~ "then" ~ expression ~ "else" ~ expression
    // AST: condition, then_branch, else_branch. Condition comparison needs handling.
    // For simplicity, let's initially parse the first expression as the whole condition.
    // Refinement needed if compare_op + second expression are essential for the AST structure.

    let mut inner = pair.into_inner();

    let condition_expr = inner.next().unwrap(); // First expression
    let condition = parse_expression(condition_expr)?;

    // Peek to check for compare_op and potentially the second expression for comparison
    // let next_rule = inner.peek().map(|p| p.as_rule());
    // let condition_op = None;
    // let condition_rhs = None;
    // if next_rule == Some(Rule::compare_op) {
    //     condition_op = Some(inner.next().unwrap().as_str().to_string());
    //     let rhs_expr = inner.next().unwrap(); // Second expression for comparison
    //     condition_rhs = Some(parse_expression(rhs_expr)?);
    // }

    let then_expr = inner.next().unwrap(); // Expression after "then"
    let then_branch = parse_expression(then_expr)?;

    let else_expr = inner.next().unwrap(); // Expression after "else"
    let else_branch = parse_expression(else_expr)?;

    Ok(IfExpr {
        condition: Box::new(condition),
        // condition_op,
        // condition_rhs: condition_rhs.map(Box::new),
        then_branch: Box::new(then_branch),
        else_branch: Box::new(else_branch),
    })
}

/// Parses a let-rec expression.
fn parse_let_rec(pair: Pair<Rule>) -> Result<LetRecExpr, Box<BorfError>> {
    // Grammar: "let" ~ "rec" ~ ident ~ lambda_params ~ "=" ~ expression ~ "in" ~ expression
    let mut inner = pair.into_inner();

    let name = inner.next().unwrap().as_str().to_string();

    // Parse the parameters
    let mut params = Vec::new();
    while let Some(param) = inner.next() {
        if param.as_rule() == Rule::ident {
            params.push(param.as_str().to_string());
        } else {
            // Found something other than an identifier, must be the "=" or an expression
            // We'll leave it to the next part to handle
            break;
        }
    }

    // The next item should be the equals sign or expression after equals
    // Then the body expression of the let-rec function
    let bound_expr = inner.next().unwrap(); // Should be the expression rule
    let bound = parse_expression(bound_expr)?;

    // Look for "in" (handled by grammar)
    // Then the expression after the "in" keyword
    let body_expr = inner.next().unwrap(); // Expression after "in"
    let body = parse_expression(body_expr)?;

    Ok(LetRecExpr {
        name,
        params,
        bound_expr: Box::new(bound),
        body: Box::new(body),
    })
}

/// Parses the right-hand side of a composition expression (used within the main 'expression' rule).
fn parse_composition_rhs(pair: Pair<Rule>) -> Result<CompositionExprRhs, Box<BorfError>> {
    // Grammar: ident ~ comp_op ~ ident
    let mut inner = pair.into_inner();
    let left = inner.next().unwrap().as_str().to_string();
    let op = inner.next().unwrap().as_str().to_string(); // comp_op rule
    let right = inner.next().unwrap().as_str().to_string();

    Ok(CompositionExprRhs { left, op, right })
}

// TODO: Add tests for these parsers.
