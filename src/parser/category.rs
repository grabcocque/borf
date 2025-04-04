//! Parsers for category definitions and related elements in the Borf language.
//!
//! This module provides functions for parsing category definitions, object declarations,
//! mapping declarations, structure mappings, and function definitions.

use super::ast::{Declaration, ModuleDef, ModuleElement, TopLevelItem, TypeExpr};
use super::error::{BorfError, SyntaxError};
use crate::parser::{build_expr_ast, get_named_source, pair_to_span, Rule};
use pest::iterators::Pair;

/// Parses a module definition from a pest pair.
///
/// A module definition includes a name and a collection of elements such as
/// objects, mappings, and laws.
///
/// # Arguments
///
/// * `pair` - A pest Pair representing a module definition
///
/// # Returns
///
/// * `Result<TopLevelItem, Box<BorfError>>` - The parsed module definition or an error
pub fn parse_module_def(pair: Pair<Rule>) -> Result<TopLevelItem, Box<BorfError>> {
    let _span = pair_to_span(&pair);
    let _src = get_named_source(pair.as_str());

    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();

    // Parse module elements
    let mut elements = Vec::new();
    for element_pair in inner {
        match element_pair.as_rule() {
            Rule::module_element => {
                match parse_module_element(element_pair) {
                    Ok(Some(elem)) => elements.push(elem),
                    Ok(None) => { /* Skip empty elements */ }
                    Err(e) => return Err(e),
                }
            }
            _ => {
                // Skip unexpected rules
                continue;
            }
        }
    }

    Ok(TopLevelItem::Module(ModuleDef { name, elements }))
}

/// Parses a module element from a pest pair.
///
/// Elements include declarations, mapped types, structure mappings, etc.
///
/// # Arguments
///
/// * `pair` - A pest Pair representing a module element
///
/// # Returns
///
/// * `Result<Option<ModuleElement>, Box<BorfError>>` - The parsed element, None for skipped elements, or an error
fn parse_module_element(pair: Pair<Rule>) -> Result<Option<ModuleElement>, Box<BorfError>> {
    let _span = pair_to_span(&pair);
    let _src = get_named_source(pair.as_str());
    let pair_clone = pair.clone(); // Clone before it's moved

    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::object_decl => {
            // Convert to unified Declaration
            let mut declarations = Vec::new();

            for decl_pair in inner.into_inner() {
                if decl_pair.as_rule() == Rule::ident {
                    // Simple declaration
                    declarations.push(ModuleElement::Declaration(Declaration {
                        names: vec![decl_pair.as_str().to_string()],
                        type_annotation: None,
                        definition: None,
                        constraint: None,
                    }));
                }
                // Handle other cases as needed
            }

            if declarations.is_empty() {
                Ok(None)
            } else {
                // Return the first declaration for now, but ideally handle multiple
                Ok(Some(declarations[0].clone()))
            }
        }
        Rule::mapping_decl => {
            // Parse into the new unified Declaration
            let decl = parse_declaration(inner)?;
            Ok(Some(ModuleElement::Declaration(decl)))
        }
        Rule::comment_decl => {
            // Comments are ignored in the AST
            Ok(None)
        }
        _ => {
            let error_span = pair_to_span(&pair_clone);
            let error_src = get_named_source(pair_clone.as_str());
            Err(Box::new(BorfError::SyntaxError(SyntaxError::new(
                &format!("Unexpected module element rule: {:?}", inner.as_rule()),
                error_src,
                error_span,
                "Expected object_decl, mapping_decl, or comment_decl",
                "Unexpected module element",
            ))))
        }
    }
}

/// Parses a declaration.
///
/// Declarations include type declarations, function declarations, and variable declarations.
fn parse_declaration(pair: Pair<Rule>) -> Result<Declaration, Box<BorfError>> {
    let _span = pair_to_span(&pair);
    let _src = get_named_source(pair.as_str());

    let mut names = Vec::new();
    let mut type_annotation = None;
    let mut definition = None;
    let mut constraint = None;

    // Process the declaration parts
    let inner = pair.into_inner();

    // First get identifiers
    for id_pair in inner {
        match id_pair.as_rule() {
            Rule::ident | Rule::dollar_ident => {
                names.push(id_pair.as_str().to_string());
            }
            Rule::type_expr => {
                // Type annotation
                // Simplified - would need to properly parse type expression
                type_annotation = Some(TypeExpr::Base(id_pair.as_str().to_string()));
            }
            Rule::expression => {
                // Definition
                definition = Some(build_expr_ast(id_pair.into_inner())?);
            }
            Rule::constraint_expr => {
                // Constraint
                constraint = Some(build_expr_ast(id_pair.into_inner())?);
            }
            _ => {
                // Skip commas and other separators
                continue;
            }
        }
    }

    Ok(Declaration {
        names,
        type_annotation,
        definition,
        constraint,
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
/// * `Result<Declaration, Box<BorfError>>` - The parsed declaration or an error
pub fn parse_object_decl(pair: Pair<Rule>) -> Result<Declaration, Box<BorfError>> {
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

    Ok(Declaration {
        names,
        type_annotation: None,
        definition: None,
        constraint: None,
    })
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
/// * `Result<Declaration, Box<BorfError>>` - The parsed declaration or an error
pub fn parse_mapping_decl(pair: Pair<Rule>) -> Result<Declaration, Box<BorfError>> {
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
/// * `Result<Declaration, Box<BorfError>>` - The parsed declaration or an error
pub fn parse_structure_mapping(pair: Pair<Rule>) -> Result<Declaration, Box<BorfError>> {
    let mut inner = pair.into_inner();
    let lhs = inner.next().unwrap().as_str().to_string();
    let rhs_pair = inner.next().unwrap(); // This is the 'expression' rule

    // Use the new expression parser
    let rhs = build_expr_ast(rhs_pair.into_inner())?;

    // Convert to a declaration
    Ok(Declaration {
        names: vec![lhs],
        type_annotation: None,
        definition: Some(rhs),
        constraint: None,
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
/// * `Result<Declaration, Box<BorfError>>` - The parsed declaration or an error
pub fn parse_function_def(pair: Pair<Rule>) -> Result<Declaration, Box<BorfError>> {
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
    let mut _codomain_pair: Option<Pair<Rule>> = None;
    let mut body_pair: Option<Pair<Rule>> = None;

    for current_pair in inner {
        match current_pair.as_rule() {
            Rule::type_expr => {
                if domain_pair.is_none() {
                    domain_pair = Some(current_pair);
                } else {
                    _codomain_pair = Some(current_pair);
                }
            }
            Rule::expression => body_pair = Some(current_pair),
            _ => {} // Ignore literals like ':', '->', '=', ';' and the ident rule for name
        }
    }

    // Check if we found the body expression
    let body = match body_pair {
        Some(bp) => build_expr_ast(bp.into_inner())?,
        None => {
            return Err(crate::parser::common_expr::create_syntax_error(
                "Missing expression body in function definition",
                &pair_clone, // Use the clone for error location
                "Function definitions must have a body after '='.",
                "Expected function body",
            ));
        }
    };

    // For now, simplify by ignoring domain/codomain - in a real implementation,
    // these would be used to set up a proper type_annotation field
    Ok(Declaration {
        names: vec![name],
        type_annotation: None, // Should use domain/codomain
        definition: Some(body),
        constraint: None,
    })
}

// --- Constraint Parsing (Placeholder - Needs to use Pratt Parser) ---

/// Placeholder function for parsing a constraint expression.
/// TODO: This should eventually use the build_expr_ast Pratt parser function.
pub fn parse_constraint(pair: Pair<Rule>) -> Result<Declaration, Box<BorfError>> {
    let span = pair_to_span(&pair);
    let src = get_named_source(pair.as_str());
    Err(Box::new(BorfError::NotYetImplemented {
        feature: "parse_constraint".to_string(),
        src: Some(src),
        span: Some(span),
    }))
}
