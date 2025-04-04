//! Parsers for category definitions and related elements in the Borf language.
//!
//! This module provides functions for parsing category definitions, object declarations,
//! mapping declarations, structure mappings, and function definitions.

use super::ast::{
    CategoryDef, CategoryElement, DomainType, ExpressionType, FunctionDef, MappingDecl,
    MappingType, ObjectDecl, StructureMapping,
};
use super::error::{BorfError, SyntaxError};
use super::Rule;
use crate::parser::laws::{parse_constraint_expr, parse_law};
use crate::parser::{common_expr::parse_expression, get_named_source, pair_to_span};
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
            Rule::constraint_decl => {
                let constraint_pair = element.into_inner().next().unwrap();
                elements.push(CategoryElement::ConstraintDecl(parse_constraint_expr(
                    constraint_pair,
                )?))
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
                    "Only object declarations, mapping declarations, laws, structure mappings, function definitions, and constraint declarations are allowed in a category",
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
    let names_pair = inner.next().unwrap();
    let names: Vec<String> = names_pair
        .into_inner()
        .map(|p| p.as_str().to_string())
        .collect();

    let domain_pair = inner.next().unwrap();
    let domain = domain_pair.as_str().to_string();
    let domain_type = if domain_pair.as_rule() == Rule::set_literal {
        DomainType::SetComprehension
    } else {
        DomainType::Simple
    };

    let mapping_type_pair = inner.next().unwrap();
    let mapping_type = match mapping_type_pair.as_str() {
        "->" => MappingType::To,
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
                "Mapping types must be one of: ->, $subseteq, <->, *, $teq, $veq, $seq, $ceq, $omega",
                "Invalid mapping type"
            ))));
        }
    };

    let codomain = inner.next().unwrap().as_str().to_string();

    Ok(MappingDecl {
        names,
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
    let rhs_pair = inner.next().unwrap(); // This is the 'expression' rule

    // Use the new expression parser
    let rhs = parse_expression(rhs_pair)?;

    // Convert from Expression to ExpressionType
    let rhs_expr_type = ExpressionType::Composite(format!("{:?}", rhs));

    Ok(StructureMapping {
        lhs,
        rhs: rhs_expr_type,
    })
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
    // Grammar: function_def_decl = { ident ~ ":" ~ domain ~ "->" ~ codomain ~ "=" ~ expression ~ ";" }
    let pair_clone = pair.clone(); // Clone for error reporting
    let mut inner = pair.into_inner();

    let name = inner
        .next()
        .expect("FunctionDef expected name")
        .as_str()
        .to_string();

    // Iterate through the remaining parts, looking for domain, codomain, and expression
    let mut _domain_pair: Option<Pair<Rule>> = None;
    let mut _codomain_pair: Option<Pair<Rule>> = None;
    let mut body_pair: Option<Pair<Rule>> = None;

    for current_pair in inner {
        match current_pair.as_rule() {
            Rule::domain => _domain_pair = Some(current_pair),
            Rule::codomain => _codomain_pair = Some(current_pair),
            Rule::expression => body_pair = Some(current_pair),
            _ => {} // Ignore literals like ':', '->', '=', ';' and the ident rule for name
        }
    }

    // Check if we found the body expression
    let body = match body_pair {
        Some(bp) => parse_expression(bp)?,
        None => {
            return Err(crate::parser::common_expr::create_syntax_error(
                "Missing expression body in function definition",
                &pair_clone, // Use the clone for error location
                "Function definitions must have a body after '='.",
                "Expected function body",
            ));
        }
    };

    // TODO: The FunctionDef AST currently doesn't store domain/codomain.
    // let _domain_str = domain_pair.map(|p| p.as_str().to_string());
    // let _codomain_str = codomain_pair.map(|p| p.as_str().to_string());

    Ok(FunctionDef {
        name,
        params: vec![], // Parameters are not parsed by function_def_decl in grammar
        body,
    })
}
