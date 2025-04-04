//! Parsers for laws and constraints in the Borf language.
//!
//! This module provides functions for parsing laws (composition, forall, exists),
//! constraints, and related expressions.

use super::ast::{Constraint, ConstraintExpr, Law, LawDefinition, SetCondition, SetExpr};
use super::error::{BorfError, SyntaxError};
use super::{get_named_source, pair_to_span};
use crate::parser::Rule;
use pest::iterators::Pair;

/// Parses a named law declaration from a pest pair.
///
/// Expects the input pair to be a `law_decl` rule, which must contain a `named_law` rule.
///
/// # Arguments
///
/// * `pair` - A pest Pair representing a `law_decl` rule
///
/// # Returns
///
/// * `Result<Law, Box<BorfError>>` - The parsed named law or an error
pub fn parse_law(pair: Pair<Rule>) -> Result<Law, Box<BorfError>> {
    // law_decl should contain exactly one named_law
    let pair_clone = pair.clone(); // Clone for potential error reporting
    let named_law_pair = pair.into_inner().next().ok_or_else(|| {
        let span = pair_to_span(&pair_clone); // Use the clone here
        let src = get_named_source(pair_clone.as_str()); // Use the clone here
        Box::new(BorfError::SyntaxError(SyntaxError::new(
            "Empty law declaration",
            src,
            span,
            "Law declarations cannot be empty.",
            "Empty law",
        )))
    })?;

    if named_law_pair.as_rule() != Rule::named_law {
        let span = pair_to_span(&named_law_pair);
        let src = get_named_source(named_law_pair.as_str());
        return Err(Box::new(BorfError::SyntaxError(SyntaxError::new(
            &format!(
                "Expected a named_law within law_decl, found {:?}",
                named_law_pair.as_rule()
            ),
            src,
            span,
            "Internal error: law_decl grammar rule should enforce containing only named_law.",
            "Invalid law structure",
        ))));
    }

    // Parse the parts of named_law: "law." ~ ident ~ "=" ~ (definition) ~ ";"
    let mut named_parts = named_law_pair.into_inner();
    let name_pair = named_parts.next().unwrap(); // Should be the ident after "law."
    let name = name_pair.as_str().to_string();

    let definition_pair = named_parts.next().unwrap(); // The expression after "="

    let definition = match definition_pair.as_rule() {
        Rule::forall_expr => parse_forall_expr_definition(definition_pair)?,
        Rule::exists_expr => parse_exists_expr_definition(definition_pair)?,
        Rule::composition_law_expr => parse_composition_law_definition(definition_pair)?,
        Rule::constraint_expr => {
            // Parse the constraint expression and wrap it in LawDefinition::Constraint
            let constraint = parse_constraint_expr(definition_pair)?;
            LawDefinition::Constraint(constraint)
        }
        _ => {
            // This case should technically be unreachable if the grammar is correct
            let span = pair_to_span(&definition_pair);
            let src = get_named_source(definition_pair.as_str());
            return Err(Box::new(BorfError::SyntaxError(SyntaxError::new(
                &format!(
                    "Unexpected definition type {:?} for named law '{}'",
                    definition_pair.as_rule(),
                    name
                ),
                src,
                span,
                "Internal Error: Grammar mismatch in named_law rule.",
                "Invalid named law content",
            ))));
        }
    };

    Ok(Law { name, definition })
}

/// Parses the definition of a universal quantification (forall) expression.
///
/// # Arguments
///
/// * `pair` - A pest Pair representing a forall_expr rule
///
/// # Returns
///
/// * `Result<LawDefinition, Box<BorfError>>` - The parsed forall definition or an error
fn parse_forall_expr_definition(pair: Pair<Rule>) -> Result<LawDefinition, Box<BorfError>> {
    let inner_pairs = pair.into_inner().collect::<Vec<_>>();
    let mut vars = Vec::new();
    let mut domain = String::new();
    let mut constraint = None;

    // Process based on the forall_expr pattern: "$forall" ~ ident ~ ("," ~ ident)* ~ "$in" ~ ident ~ ":" ~ constraint_expr
    if !inner_pairs.is_empty() && inner_pairs[0].as_rule() == Rule::ident {
        vars.push(inner_pairs[0].as_str().to_string());
        let mut i = 1;
        while i < inner_pairs.len() && inner_pairs[i].as_rule() == Rule::ident {
            vars.push(inner_pairs[i].as_str().to_string());
            i += 1;
        }
        if i < inner_pairs.len() {
            domain = inner_pairs[i].as_str().to_string();
            i += 1;
        }
        if i < inner_pairs.len() && inner_pairs[i].as_rule() == Rule::constraint_expr {
            constraint = Some(parse_constraint_expr(inner_pairs[i].clone())?);
        }
    }

    let final_constraint = constraint.unwrap_or_else(|| Constraint::Equality {
        lhs: Box::new(ConstraintExpr::Identifier("true".to_string())),
        rhs: Box::new(ConstraintExpr::Identifier("true".to_string())),
    });

    Ok(LawDefinition::ForAll {
        vars,
        domain,
        constraint: final_constraint,
    })
}

/// Parses the definition of an existential quantification (exists) expression.
///
/// # Arguments
///
/// * `pair` - A pest Pair representing an exists_expr rule
///
/// # Returns
///
/// * `Result<LawDefinition, Box<BorfError>>` - The parsed exists definition or an error
fn parse_exists_expr_definition(pair: Pair<Rule>) -> Result<LawDefinition, Box<BorfError>> {
    let inner_pairs = pair.into_inner().collect::<Vec<_>>();
    let mut vars = Vec::new();
    let mut domain = String::new();
    let mut constraint = None;

    // Process based on the exists_expr pattern similar to forall
    if !inner_pairs.is_empty() && inner_pairs[0].as_rule() == Rule::ident {
        vars.push(inner_pairs[0].as_str().to_string());
        let mut i = 1;
        while i < inner_pairs.len() && inner_pairs[i].as_rule() == Rule::ident {
            vars.push(inner_pairs[i].as_str().to_string());
            i += 1;
        }
        if i < inner_pairs.len() {
            domain = inner_pairs[i].as_str().to_string();
            i += 1;
        }
        if i < inner_pairs.len() && inner_pairs[i].as_rule() == Rule::constraint_expr {
            constraint = Some(parse_constraint_expr(inner_pairs[i].clone())?);
        }
    }

    let final_constraint = constraint.unwrap_or_else(|| Constraint::Equality {
        lhs: Box::new(ConstraintExpr::Identifier("true".to_string())),
        rhs: Box::new(ConstraintExpr::Identifier("true".to_string())),
    });

    Ok(LawDefinition::Exists {
        vars,
        domain,
        constraint: final_constraint,
    })
}

/// Parses the definition of a composition law expression.
///
/// # Arguments
///
/// * `pair` - A pest Pair representing a composition_law_expr rule
///
/// # Returns
///
/// * `Result<LawDefinition, Box<BorfError>>` - The parsed composition definition or an error
fn parse_composition_law_definition(pair: Pair<Rule>) -> Result<LawDefinition, Box<BorfError>> {
    // composition_law_expr = { ident ~ "$comp" ~ ident ~ "===" ~ ident }
    let mut parts = pair.into_inner();
    let lhs = parts.next().unwrap().as_str().to_string();
    // Skip the $comp part (pest handles it as a literal in the rule)
    let middle = parts.next().unwrap().as_str().to_string();
    // Skip the === part (pest handles it)
    let rhs = parts.next().unwrap().as_str().to_string();
    Ok(LawDefinition::Composition {
        lhs,
        op: "$comp".to_string(), // Assuming $comp is the intended op here
        middle,
        rhs,
    })
}

/// Parses a constraint expression from a pest pair.
///
/// Constraints express relationships between terms, such as equality,
/// logical combinations, and various forms of equivalence.
///
/// # Arguments
///
/// * `pair` - A pest Pair representing a constraint expression
///
/// # Returns
///
/// * `Result<Constraint, Box<BorfError>>` - The parsed constraint or an error
pub fn parse_constraint_expr(pair: Pair<Rule>) -> Result<Constraint, Box<BorfError>> {
    let pair_clone = pair.clone(); // Clone for error reporting

    // The structure of constraint_expr is: primary_constraint_term ~ (constraint_op ~ primary_constraint_term)*
    let inner_pairs: Vec<_> = pair.into_inner().collect();
    if inner_pairs.is_empty() {
        let span = pair_to_span(&pair_clone);
        let src = get_named_source(pair_clone.as_str());
        return Err(Box::new(BorfError::SyntaxError(SyntaxError::new(
            "Empty constraint expression",
            src,
            span,
            "Constraint expressions must contain at least one term",
            "Empty constraint",
        ))));
    }

    // Get the first term
    let first_term = parse_primary_constraint_term(inner_pairs[0].clone())?;

    // If there's an operator and a second term, create the appropriate constraint type
    if inner_pairs.len() >= 3 && inner_pairs[1].as_rule() == Rule::constraint_op {
        let op = inner_pairs[1].as_str();
        let second_term = parse_primary_constraint_term(inner_pairs[2].clone())?;

        match op {
            "=" | "==" | "===" => Ok(Constraint::Equality {
                lhs: Box::new(first_term),
                rhs: Box::new(second_term),
            }),
            "$and" => Ok(Constraint::LogicalAnd {
                lhs: Box::new(first_term),
                rhs: Box::new(second_term),
            }),
            "=>" => Ok(Constraint::Implies {
                lhs: Box::new(first_term),
                rhs: Box::new(second_term),
            }),
            ">" => Ok(Constraint::GreaterThan {
                lhs: Box::new(first_term),
                rhs: Box::new(second_term),
            }),
            ">=" => Ok(Constraint::GreaterThanEqual {
                lhs: Box::new(first_term),
                rhs: Box::new(second_term),
            }),
            "<" => Ok(Constraint::LessThan {
                lhs: Box::new(first_term),
                rhs: Box::new(second_term),
            }),
            "<=" => Ok(Constraint::LessThanEqual {
                lhs: Box::new(first_term),
                rhs: Box::new(second_term),
            }),
            "$teq" => Ok(Constraint::TypeEquiv {
                lhs: Box::new(first_term),
                rhs: Box::new(second_term),
            }),
            "$veq" => Ok(Constraint::ValueEquiv {
                lhs: Box::new(first_term),
                rhs: Box::new(second_term),
            }),
            "$seq" => Ok(Constraint::StructuralEquiv {
                lhs: Box::new(first_term),
                rhs: Box::new(second_term),
            }),
            "$ceq" => Ok(Constraint::CategoricalEquiv {
                lhs: Box::new(first_term),
                rhs: Box::new(second_term),
            }),
            "$omega" => Ok(Constraint::Compatibility {
                lhs: Box::new(first_term),
                rhs: Box::new(second_term),
            }),
            _ => {
                let span = pair_to_span(&inner_pairs[1]);
                let src = get_named_source(inner_pairs[1].as_str());
                Err(Box::new(BorfError::SyntaxError(SyntaxError::new(
                    &format!("Unknown constraint operator: {}", op),
                    src,
                    span,
                    "Constraint operators must be one of: =, $and, =>, >, >=, <, <=, $teq, $veq, $seq, $ceq, $omega",
                    "Unknown operator",
                ))))
            }
        }
    } else {
        // If there's just a single term, return a default equality constraint
        Ok(Constraint::Equality {
            lhs: Box::new(first_term),
            rhs: Box::new(ConstraintExpr::Integer(0)),
        })
    }
}

/// Parses a primary constraint term from a pest pair.
///
/// Primary constraint terms can be integers, identifiers, set expressions,
/// function applications, or symbol literals.
///
/// # Arguments
///
/// * `pair` - A pest Pair representing a primary constraint term
///
/// # Returns
///
/// * `Result<ConstraintExpr, Box<BorfError>>` - The parsed constraint expression or an error
fn parse_primary_constraint_term(pair: Pair<Rule>) -> Result<ConstraintExpr, Box<BorfError>> {
    let rule = pair.as_rule();
    match rule {
        Rule::int => {
            let value = pair.as_str().parse::<i64>().map_err(|e| {
                let span = pair_to_span(&pair);
                let src = get_named_source(pair.as_str());
                Box::new(BorfError::SyntaxError(SyntaxError::new(
                    &format!("Invalid integer literal: {}", e),
                    src,
                    span,
                    "Integer literals must be valid 64-bit integers",
                    "Invalid integer",
                )))
            })?;
            Ok(ConstraintExpr::Integer(value))
        }
        Rule::ident => Ok(ConstraintExpr::Identifier(pair.as_str().to_string())),
        Rule::symbol_literal => {
            let symbol_text = pair.as_str();
            let symbol_name = symbol_text.trim_start_matches(':'); // Remove the leading colon
            Ok(ConstraintExpr::Symbol(symbol_name.to_string()))
        }
        Rule::set_expr => parse_set_expr(pair.clone()),
        Rule::function_app => {
            let pair_clone = pair.clone(); // Clone for fallback
            let mut inner = pair.into_inner();
            let func = inner
                .next()
                .unwrap_or_else(|| pair_clone.clone())
                .as_str()
                .to_string();
            let arg = if let Some(arg_pair) = inner.next() {
                arg_pair.as_str().to_string()
            } else {
                "".to_string()
            };
            Ok(ConstraintExpr::FunctionApp { func, arg })
        }
        _ => {
            // Likely a nested expression or a rule we don't directly handle
            let inner_pairs: Vec<_> = pair.clone().into_inner().collect();
            if !inner_pairs.is_empty() {
                // Recursively process the first inner pair
                parse_primary_constraint_term(inner_pairs[0].clone())
            } else {
                // Default to identifier with the term's text
                Ok(ConstraintExpr::Identifier(pair.as_str().to_string()))
            }
        }
    }
}

/// Parses a set expression from a pest pair.
///
/// Set expressions can be set comprehensions or Cartesian products.
///
/// # Arguments
///
/// * `pair` - A pest Pair representing a set expression
///
/// # Returns
///
/// * `Result<ConstraintExpr, Box<BorfError>>` - The parsed set expression or an error
fn parse_set_expr(pair: Pair<Rule>) -> Result<ConstraintExpr, Box<BorfError>> {
    let inner_pairs: Vec<_> = pair.clone().into_inner().collect();
    if inner_pairs.is_empty() {
        let span = pair_to_span(&pair);
        let src = get_named_source(pair.as_str());
        return Err(Box::new(BorfError::SyntaxError(SyntaxError::new(
            "Empty set expression",
            src,
            span,
            "Set expressions must contain elements",
            "Empty set expression",
        ))));
    }

    let first_inner = &inner_pairs[0];
    let rule = first_inner.as_rule();

    // Check if it's a set comprehension or a Cartesian product
    if rule == Rule::set_element {
        // Set comprehension: "{" ~ set_element ~ ("|" ~ set_condition)? ~ "}"
        let elements: Vec<String> = first_inner
            .clone()
            .into_inner()
            .map(|p| p.as_str().to_string())
            .collect();

        // Check for optional condition
        let condition = if inner_pairs.len() > 1 && inner_pairs[1].as_str().starts_with("|") {
            let mut cond_pairs = inner_pairs[1].clone().into_inner();
            let func1 = cond_pairs
                .next()
                .unwrap_or(inner_pairs[1].clone())
                .as_str()
                .to_string();
            let arg1 = if let Some(arg) = cond_pairs.next() {
                arg.as_str().to_string()
            } else {
                "".to_string()
            };

            // Check for additional parts of the condition
            let (func2, arg2) = if cond_pairs.next().is_some() {
                // Skip "$and"
                let f2 = cond_pairs.next().map(|p| p.as_str().to_string());
                let a2 = cond_pairs.next().map(|p| p.as_str().to_string());
                (f2, a2)
            } else {
                (None, None)
            };

            Some(SetCondition {
                func1,
                arg1,
                func2,
                arg2,
            })
        } else {
            None
        };

        Ok(ConstraintExpr::SetExpr(SetExpr::Comprehension {
            elements,
            condition,
        }))
    } else if pair.as_str().contains('*') || pair.as_str().contains('×') {
        // Cartesian product case: ident ~ ("*" | "×") ~ ident
        let text = pair.as_str();
        let parts: Vec<&str> = if text.contains('*') {
            text.split('*').collect()
        } else {
            text.split('×').collect()
        };

        if parts.len() >= 2 {
            let lhs = parts[0].trim().to_string();
            let rhs = parts[1].trim().to_string();
            Ok(ConstraintExpr::SetExpr(SetExpr::CartesianProduct {
                lhs,
                rhs,
            }))
        } else {
            // Default to a simple identifier
            Ok(ConstraintExpr::Identifier(text.to_string()))
        }
    } else {
        // Handle as a general expression
        Ok(ConstraintExpr::Identifier(pair.as_str().to_string()))
    }
}
