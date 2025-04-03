//! Parsers for category definitions and related elements in the Borf language.
//!
//! This module provides functions for parsing category definitions, object declarations,
//! mapping declarations, structure mappings, and function definitions.

use super::ast::{
    CategoryDef, CategoryElement, DomainType, ExpressionType, FunctionDef, MappingDecl,
    MappingType, ObjectDecl, StructureMapping,
};
use crate::error::{BorfError, SyntaxError};
use crate::parser::laws::parse_law;
use crate::parser::{get_named_source, pair_to_span, Rule};
use pest::iterators::Pair;

/// Parses a category definition from a pest pair.
///
/// Category definitions include a name, optional base category, and a collection of elements.
///
/// # Arguments
///
/// * `pair` - A pest Pair representing a category definition
///
/// # Returns
///
/// * `Result<CategoryDef, Box<BorfError>>` - The parsed category definition or an error
pub fn parse_category_def(pair: Pair<Rule>) -> Result<CategoryDef, Box<BorfError>> {
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();

    // Handle optional base category (in angle brackets)
    let next_token = inner.peek().unwrap().as_rule();
    let (base_category, elements_pair) = if next_token == Rule::ident {
        // This could be a base category in angle brackets syntax
        let base = inner.next().unwrap().as_str().to_string();
        (Some(base), inner.next().unwrap())
    } else {
        // No base category
        (None, inner.next().unwrap())
    };

    // Parse category elements
    let mut elements = Vec::new();
    for element in elements_pair.into_inner() {
        match element.as_rule() {
            Rule::object_decl => {
                elements.push(CategoryElement::ObjectDecl(parse_object_decl(element)?))
            }
            Rule::mapping_decl => {
                elements.push(CategoryElement::MappingDecl(parse_mapping_decl(element)?))
            }
            Rule::law_decl => elements.push(CategoryElement::LawDecl(parse_law(element)?)),
            Rule::structure_mapping_decl => elements.push(CategoryElement::StructureMapping(
                parse_structure_mapping(element)?,
            )),
            Rule::function_def_decl => {
                elements.push(CategoryElement::FunctionDef(parse_function_def(element)?))
            }
            _ => {
                // Create better error with source location
                let rule_str = format!("{:?}", element.as_rule());
                let span = pair_to_span(&element);
                let src = get_named_source(element.as_str());

                return Err(Box::new(BorfError::SyntaxError(SyntaxError::new(
                    &format!("Unexpected category element: {}", rule_str),
                    src,
                    span,
                    "Only object declarations, mapping declarations, laws, structure mappings, and function definitions are allowed in a category",
                    &format!("Unexpected {} here", rule_str)
                ))));
            }
        }
    }

    Ok(CategoryDef {
        name,
        base_category,
        elements,
    })
}

/// Parses an object declaration from a pest pair.
///
/// Object declarations define one or more objects within a category.
///
/// # Arguments
///
/// * `pair` - A pest Pair representing an object declaration
///
/// # Returns
///
/// * `Result<ObjectDecl, Box<BorfError>>` - The parsed object declaration or an error
pub fn parse_object_decl(pair: Pair<Rule>) -> Result<ObjectDecl, Box<BorfError>> {
    let pair_clone = pair.clone(); // Clone the pair before consuming it
    let inner = pair.into_inner();
    let names: Vec<String> = inner.map(|p| p.as_str().to_string()).collect();

    if names.is_empty() {
        let span = pair_to_span(&pair_clone);
        let src = get_named_source(pair_clone.as_str());
        return Err(Box::new(BorfError::SyntaxError(SyntaxError::new(
            "Object declaration with no objects",
            src,
            span,
            "Object declarations must contain at least one object name",
            "No objects declared",
        ))));
    }

    Ok(ObjectDecl { names })
}

/// Parses a mapping declaration from a pest pair.
///
/// Mapping declarations define relationships between objects in a category.
///
/// # Arguments
///
/// * `pair` - A pest Pair representing a mapping declaration
///
/// # Returns
///
/// * `Result<MappingDecl, Box<BorfError>>` - The parsed mapping declaration or an error
pub fn parse_mapping_decl(pair: Pair<Rule>) -> Result<MappingDecl, Box<BorfError>> {
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();
    let domain_pair = inner.next().unwrap();
    let domain = domain_pair.as_str().to_string();
    let domain_type = if domain_pair.as_rule() == Rule::set_literal {
        DomainType::SetComprehension
    } else {
        DomainType::Simple
    };

    let mapping_type_pair = inner.next().unwrap();
    let mapping_type = match mapping_type_pair.as_str() {
        "$to" => MappingType::To,
        "$subseteq" => MappingType::Subseteq,
        "<->" => MappingType::Bidirectional,
        "*" => MappingType::Times,
        "$teq" => MappingType::TypeEquiv,
        "$veq" => MappingType::ValueEquiv,
        "$seq" => MappingType::StructEquiv,
        "$ceq" => MappingType::CatEquiv,
        "$omega" => MappingType::Compatibility,
        _ => {
            let span = pair_to_span(&mapping_type_pair);
            let src = get_named_source(mapping_type_pair.as_str());
            return Err(Box::new(BorfError::SyntaxError(SyntaxError::new(
                &format!("Unknown mapping type: {}", mapping_type_pair.as_str()),
                src,
                span,
                "Mapping types must be one of: $to, $subseteq, <->, *, $teq, $veq, $seq, $ceq, $omega",
                "Invalid mapping type"
            ))));
        }
    };

    let codomain = inner.next().unwrap().as_str().to_string();

    Ok(MappingDecl {
        name,
        domain,
        domain_type,
        mapping_type,
        codomain,
    })
}

/// Parses a structure mapping from a pest pair.
///
/// Structure mappings assign expressions to names within a category.
///
/// # Arguments
///
/// * `pair` - A pest Pair representing a structure mapping
///
/// # Returns
///
/// * `Result<StructureMapping, Box<BorfError>>` - The parsed structure mapping or an error
pub fn parse_structure_mapping(pair: Pair<Rule>) -> Result<StructureMapping, Box<BorfError>> {
    let mut inner = pair.into_inner();
    let lhs = inner.next().unwrap().as_str().to_string();
    let rhs_pair = inner.next().unwrap();

    let rhs = match rhs_pair.as_rule() {
        Rule::expr => {
            // Examine inner elements of expr
            let inner_expr = rhs_pair.into_inner().next().unwrap();

            match inner_expr.as_rule() {
                Rule::term => {
                    // Check if it's a simple term
                    let term_inner = inner_expr.into_inner().next().unwrap();
                    match term_inner.as_rule() {
                        Rule::ident => ExpressionType::Simple(term_inner.as_str().to_string()),
                        Rule::int => ExpressionType::Simple(term_inner.as_str().to_string()),
                        Rule::symbol_literal => {
                            let term_inner_clone = term_inner.clone(); // Clone before using into_inner
                            let symbol_name = term_inner
                                .into_inner()
                                .next()
                                .unwrap_or(term_inner_clone)
                                .as_str()
                                .to_string();
                            let symbol_name = symbol_name.trim_start_matches(':'); // Remove the leading colon
                            ExpressionType::Symbol(symbol_name.to_string())
                        }
                        _ => ExpressionType::Composite(term_inner.as_str().to_string()),
                    }
                }
                _ => ExpressionType::Composite(inner_expr.as_str().to_string()),
            }
        }
        Rule::match_expr => {
            let mut match_parts = rhs_pair.into_inner();
            let scrutinee = match_parts.next().unwrap().as_str().to_string();
            let mut cases = Vec::new();

            // Process match cases
            while let Some(pattern) = match_parts.next() {
                // ident -> expr pattern
                let arrow = "->";
                let result = match_parts
                    .next()
                    .unwrap_or(pattern.clone())
                    .as_str()
                    .to_string();
                cases.push((pattern.as_str().to_string(), arrow.to_string(), result));
            }

            ExpressionType::Match(scrutinee, cases)
        }
        Rule::symbol_literal => {
            let rhs_pair_clone = rhs_pair.clone(); // Clone before consuming
            let symbol_name = rhs_pair
                .into_inner()
                .next()
                .unwrap_or(rhs_pair_clone)
                .as_str()
                .to_string();
            let symbol_name = symbol_name.trim_start_matches(':'); // Remove the leading colon
            ExpressionType::Symbol(symbol_name.to_string())
        }
        _ => ExpressionType::Composite(rhs_pair.as_str().to_string()),
    };

    Ok(StructureMapping { lhs, rhs })
}

/// Parses a function definition from a pest pair.
///
/// Function definitions specify the name, parameters, and body of a function.
///
/// # Arguments
///
/// * `pair` - A pest Pair representing a function definition
///
/// # Returns
///
/// * `Result<FunctionDef, Box<BorfError>>` - The parsed function definition or an error
pub fn parse_function_def(pair: Pair<Rule>) -> Result<FunctionDef, Box<BorfError>> {
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();

    // Collect parameters
    let mut params = Vec::new();
    for next_pair in inner {
        if next_pair.as_rule() == Rule::ident {
            params.push(next_pair.as_str().to_string());
        } else {
            // This should be the expr after the "="
            // Get the rest as a string for the body
            let body = next_pair.as_str().to_string();
            return Ok(FunctionDef { name, params, body });
        }
    }

    // If we get here, we didn't find a body
    Ok(FunctionDef {
        name,
        params,
        body: String::new(),
    })
}
