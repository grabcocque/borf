//! Parsers for category definitions and related elements in the Borf language.
//!
//! This module provides functions for parsing category definitions, object declarations,
//! mapping declarations, structure mappings, and function definitions.

use super::ast::{
    CategoryDef, CategoryElement, Constraint, DomainType, ExpressionType, FunctionDef, MappingDecl,
    MappingType, ObjectDecl, StructureMapping,
};
use super::error::{BorfError, SyntaxError};
use crate::parser::ast::Constraint;
use crate::parser::error::{CategoryParseError, Constraint};
use crate::parser::laws::{parse_constraint_expr, parse_law};
use crate::parser::Rule;
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
    let span = pair_to_span(&pair);
    let src = get_named_source(pair.as_str());
    let mut inner_pairs = pair.into_inner();

    // Extract category name
    let name_pair = inner_pairs.next().unwrap(); // Expecting ident
    let name = name_pair.as_str().to_string();

    // Optional base category
    let base_category = if inner_pairs
        .peek()
        .map_or(false, |p| p.as_rule() == Rule::ident)
    {
        Some(inner_pairs.next().unwrap().as_str().to_string())
    } else {
        None
    };

    // Parse category elements
    let mut elements = Vec::new();
    // Variable no longer needed for this simplified loop
    // let mut codomain_pair: Option<Pair<Rule>> = None;

    for element in inner_pairs {
        match element.as_rule() {
            Rule::object_decl => {
                let names = element
                    .into_inner()
                    .map(|ident_pair| ident_pair.as_str().to_string())
                    .collect();
                elements.push(CategoryElement::ObjectDecl(ObjectDecl { names }));
            }
            Rule::mapping_decl => {
                // TODO: Implement mapping_decl parsing based on its new unified structure
                // Needs to handle optional type, optional value, optional constraint
                elements.push(CategoryElement::MappingDecl(parse_mapping_decl(element)?));
            }
            Rule::comment_decl => { /* Skip comments */ }
            // Removed old/specific rules:
            // Rule::law_decl => ...
            // Rule::structure_mapping_decl => ...
            // Rule::function_def_decl => ...
            // Rule::constraint_decl => ...
            _ => {
                let element_span = pair_to_span(&element);
                let element_src = get_named_source(element.as_str());
                return Err(Box::new(BorfError::MalformedCategory(
                    CategoryParseError::new(
                        &format!("Unexpected element in category definition: {:?}", element.as_rule()),
                        element_src,
                        element_span,
                        "Expected object declaration, mapping, law, function, or structure mapping.",
                        "Unexpected element"
                    )
                )));
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
    let span = pair_to_span(&pair);
    let src = get_named_source(pair.as_str());
    Err(Box::new(BorfError::NotYetImplemented {
        feature: "parse_mapping_decl".to_string(),
        src: Some(src),
        span: Some(span),
    }))
    // Placeholder implementation
    // let mut inner = pair.into_inner();
    // let name_part = inner.next().unwrap();
    // let name = name_part.as_str().to_string();
    // let mut type_expr: Option<TypeExpr> = None;
    // let mut value_expr: Option<Expression> = None;
    // let mut constraint_expr: Option<Expression> = None;
    // // ... logic to parse optional parts based on peek/next ...
    // Ok(MappingDecl { /* fields based on parsed parts */ })
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
    // Grammar: function_def_decl = { ident ~ ":" ~ type_expr ~ "->" ~ type_expr ~ "=" ~ expression ~ ";" }
    let pair_clone = pair.clone(); // Clone for error reporting
    let mut inner = pair.into_inner();

    let name = inner
        .next()
        .expect("FunctionDef expected name")
        .as_str()
        .to_string();

    // Iterate through the remaining parts, looking for domain, codomain, and expression
    let mut domain_pair: Option<Pair<Rule>> = None;
    let mut codomain_pair: Option<Pair<Rule>> = None;
    let mut body_pair: Option<Pair<Rule>> = None;

    for current_pair in inner {
        match current_pair.as_rule() {
            Rule::type_expr => {
                if domain_pair.is_none() {
                    domain_pair = Some(current_pair);
                } else {
                    codomain_pair = Some(current_pair);
                }
            }
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

// --- Constraint Parsing (Placeholder - Needs to use Pratt Parser) ---

/// Placeholder function for parsing a constraint expression.
/// TODO: This should eventually use the build_expr_ast Pratt parser function.
pub fn parse_constraint(pair: Pair<Rule>) -> Result<Constraint, Box<BorfError>> {
    let span = pair_to_span(&pair);
    let src = get_named_source(pair.as_str());
    Err(Box::new(BorfError::NotYetImplemented {
        feature: "parse_constraint".to_string(),
        src: Some(src),
        span: Some(span),
    }))
}
