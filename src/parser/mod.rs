use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;

// Re-export error types
pub use crate::parser::error::{
    convert_pest_error, BorfError, CategoryParseError, CompositionParseError, MappingParseError,
    SourceCache, SourceSpan, SyntaxError, UnexpectedTokenError,
};

// Export AST module
pub mod ast;
pub mod error;
#[cfg(test)]
mod parse_tests;
pub mod pratt;
#[cfg(test)]
mod prelude_parse_tests;

// Import our AST structures to build
use ast::{
    Declaration, Identifier, ModuleDef, ModuleElement, PrimitiveDecl, PrimitiveElement,
    TopLevelItem,
};

// Generate the Parser struct from the .pest grammar file
#[derive(Parser)]
#[grammar = "parser/borf.pest"]
pub struct BorfParser;

// Main entry point for parsing a Borf program
pub fn parse_program(input: &str) -> Result<Vec<TopLevelItem>, BorfError> {
    let parse_result = BorfParser::parse(Rule::program, input);

    match parse_result {
        Ok(mut pairs) => {
            // Pairs should have a single program pair at the top
            let program_pair = pairs.next().unwrap();
            assert_eq!(program_pair.as_rule(), Rule::program);

            // Process each statement in the program
            let mut items = Vec::new();
            for statement_pair in program_pair.into_inner() {
                match statement_pair.as_rule() {
                    Rule::statement => {
                        if let Some(item) = parse_statement(statement_pair)? {
                            items.push(item);
                        }
                    }
                    Rule::EOI => break, // End of input reached
                    _ => {
                        return Err(BorfError::NotYetImplemented {
                            feature: format!("Parsing rule: {:?}", statement_pair.as_rule()),
                            src: None,
                            span: None,
                        });
                    }
                }
            }

            Ok(items)
        }
        Err(err) => {
            // Convert pest error to our error type
            Err(convert_pest_error(err, "input", input))
        }
    }
}

fn parse_statement(pair: Pair<Rule>) -> Result<Option<TopLevelItem>, BorfError> {
    // Get the first inner pair which should be one of the statement types
    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        Rule::module_declaration => {
            let module = parse_module_declaration(inner)?;
            Ok(Some(TopLevelItem::Module(module)))
        }
        Rule::primitive_declaration => {
            let primitive = parse_primitive_declaration(inner)?;
            Ok(Some(TopLevelItem::Primitive(primitive)))
        }
        Rule::export_statement => {
            let ident = parse_export_statement(inner)?;
            Ok(Some(TopLevelItem::Export(ident)))
        }
        Rule::import_statement => {
            let path = parse_import_statement(inner)?;
            Ok(Some(TopLevelItem::Import(path)))
        }
        Rule::expression_statement => {
            // For now, we'll ignore standalone expressions
            // Could be enhanced to capture these expressions for evaluation
            Ok(None)
        }
        Rule::COMMENT => Ok(None), // Skip comments
        _ => Err(BorfError::NotYetImplemented {
            feature: format!("Parsing statement type: {:?}", inner.as_rule()),
            src: None,
            span: None,
        }),
    }
}

fn parse_module_declaration(pair: Pair<Rule>) -> Result<ModuleDef, BorfError> {
    let mut pairs = pair.into_inner();
    let name = pairs.next().unwrap().as_str().to_string();

    // Check for type parameter (optional) - e.g., <T>
    let mut type_param = None;
    // Peek at the next token after the name
    if let Some(token_after_name) = pairs.peek() {
        if token_after_name.as_str() == "<" {
            // Consume the '<'
            let _open_angle = pairs.next().unwrap();
            // Next token should be the identifier
            let ident_pair = pairs.next().ok_or_else(|| {
                BorfError::ParserError(
                    "Expected identifier after '<' in module type parameter".to_string(),
                )
            })?;
            if ident_pair.as_rule() != Rule::ident {
                return Err(BorfError::ParserError(format!(
                    "Expected identifier after '<', found {:?}",
                    ident_pair.as_rule()
                )));
            }
            type_param = Some(ident_pair.as_str().to_string());
            // Next token should be '>'
            let close_angle = pairs.next().ok_or_else(|| {
                BorfError::ParserError("Expected '>' after type parameter identifier".to_string())
            })?;
            if close_angle.as_str() != ">" {
                return Err(BorfError::ParserError(format!(
                    "Expected '>' after type parameter identifier, found '{}'",
                    close_angle.as_str()
                )));
            }
            // The next token (consumed by the loop or checked below) should be ':'
        }
        // If it's not '<', the next token should just be ':'
    }

    // Expect the colon now
    // `pairs.next()` consumes the token after the name (if no type param) or after '>' (if type param)
    let colon_pair = pairs.next().ok_or_else(|| {
        BorfError::ParserError("Expected ':' after module name/type parameter".to_string())
    })?;
    if colon_pair.as_str() != ":" {
        return Err(BorfError::ParserError(format!(
            "Expected ':' after module name/type parameter, found '{}' ({:?})",
            colon_pair.as_str(),
            colon_pair.as_rule()
        )));
    }

    // Now parse the module elements inside {}
    let mut elements = Vec::new();
    // Loop through remaining pairs until RBRACE (implicitly)
    for elem_pair in pairs {
        // pairs iterator is now positioned after the colon
        match elem_pair.as_rule() {
            Rule::module_element => {
                // Recurse into the actual element
                let inner_element_pair = elem_pair.into_inner().next().unwrap();
                match inner_element_pair.as_rule() {
                    Rule::object_decl => {
                        let decl = parse_object_declaration(inner_element_pair)?;
                        elements.push(ModuleElement::Declaration(decl));
                    }
                    Rule::mapping_decl => {
                        let decl = parse_mapping_declaration(inner_element_pair)?;
                        elements.push(ModuleElement::Declaration(decl));
                    }
                    Rule::law_decl => {
                        let law_decl = parse_law_declaration(inner_element_pair)?;
                        elements.push(ModuleElement::Declaration(law_decl));
                    }
                    // Skip comments - they're handled by COMMENT
                    _ => {
                        return Err(BorfError::NotYetImplemented {
                            feature: format!(
                                "Parsing module element type: {:?}",
                                inner_element_pair.as_rule()
                            ),
                            src: None,
                            span: None,
                        });
                    }
                }
            }
            // Rule::separator => { /* Skip separators */ } // Removed: separator is silent
            _ => {
                // This might catch unexpected tokens if grammar allows them between separators
                return Err(BorfError::ParserError(format!(
                    "Unexpected rule inside module body: {:?}",
                    elem_pair.as_rule()
                )));
            }
        }
    }

    Ok(ModuleDef {
        name,
        type_param,
        elements,
    })
}

fn parse_object_declaration(pair: Pair<Rule>) -> Result<Declaration, BorfError> {
    // Ensure the pair is indeed an object_decl
    assert_eq!(pair.as_rule(), Rule::object_decl);

    let mut inner_pairs = pair.into_inner(); // Gets ident, optional type_expr, optional ';'

    // First inner pair MUST be the identifier
    let ident_pair = inner_pairs.next().ok_or_else(|| {
        BorfError::ParserError("Expected identifier in object declaration".to_string())
    })?;
    assert_eq!(ident_pair.as_rule(), Rule::ident);
    let name = ident_pair.as_str().to_string();

    // Check if the next pair is a type expression
    let mut type_constraint = None;
    if let Some(next_pair) = inner_pairs.peek() {
        if next_pair.as_rule() == Rule::type_expr {
            // Consume the type_expr pair
            let type_expr_pair = inner_pairs.next().unwrap();
            type_constraint = Some(pratt::parse_type_expression(type_expr_pair)?);
        }
        // Ignore any remaining pair (which would be the optional semicolon)
    }

    Ok(Declaration::ObjectDecl {
        name,
        type_constraint,
    })
}

fn parse_mapping_declaration(pair: Pair<Rule>) -> Result<Declaration, BorfError> {
    let mut pairs = pair.into_inner();

    // Parse names (can be multiple, comma-separated, or a law identifier)
    let name_pair = pairs.next().unwrap();
    let name = match name_pair.as_rule() {
        Rule::ident | Rule::dollar_ident => name_pair.as_str().to_string(),
        Rule::law_identifier => name_pair.as_str().to_string(),
        _ => {
            return Err(BorfError::NotYetImplemented {
                feature: format!("Parsing mapping name type: {:?}", name_pair.as_rule()),
                src: None,
                span: None,
            });
        }
    };

    // Parse optional type signature
    let mut type_constraint = None;
    let mut next_pair = pairs.next();

    if next_pair.is_some() && next_pair.as_ref().unwrap().as_rule() == Rule::type_expr {
        type_constraint = Some(pratt::parse_type_expression(next_pair.unwrap())?);
        next_pair = pairs.next();
    }

    // Parse optional definition
    let mut value = None;
    if next_pair.is_some() {
        let expr_pair = next_pair.unwrap();
        value = Some(pratt::parse_expression(expr_pair)?);
    }

    // Parse optional constraint
    let mut constraint = None;
    let constraint_pair = pairs.next();
    if constraint_pair.is_some() {
        constraint = Some(pratt::parse_expression(constraint_pair.unwrap())?);
    }

    Ok(Declaration::MappingDecl {
        name,
        type_constraint,
        value,
        constraint,
    })
}

fn parse_law_declaration(pair: Pair<Rule>) -> Result<Declaration, BorfError> {
    assert_eq!(pair.as_rule(), Rule::law_decl);
    let mut pairs = pair.into_inner();

    // First pair is law identifier
    let law_id_pair = pairs.next().unwrap();
    assert_eq!(law_id_pair.as_rule(), Rule::law_identifier);
    let law_name = law_id_pair.as_str().to_string();

    // Next pair is the body (quantifier_block, quantifier_simple, or expr_or_simple)
    let body_pair = pairs.next().unwrap();
    let law_body = match body_pair.as_rule() {
        Rule::quantifier_block => {
            let mut block_pairs = body_pair.into_inner();
            // Parse quantifier ($forall or $exists)
            let quantifier_token = block_pairs.next().unwrap().as_str();
            let quantifier = match quantifier_token {
                "$forall" => ast::Quantifier::Forall,
                "$exists" => ast::Quantifier::Exists,
                _ => {
                    return Err(BorfError::ParserError(format!(
                        "Unexpected quantifier token: {}",
                        quantifier_token
                    )))
                }
            };
            // Parse variable ident
            let variable = block_pairs.next().unwrap().as_str().to_string();
            // Skip '$in'
            let _ = block_pairs.next().unwrap();
            // Parse domain ident
            let domain = block_pairs.next().unwrap().as_str().to_string();
            // Skip '{'
            let _ = block_pairs.next().unwrap();

            // Parse module elements inside the block
            let mut elements = Vec::new();
            // The next pair should contain all the module elements
            if let Some(elements_container_pair) = block_pairs.next() {
                // Check if it's not the closing '}' immediately (empty block)
                if elements_container_pair.as_rule() == Rule::module_element {
                    // Loop through actual elements if the container isn't empty
                    for elem_pair in elements_container_pair.into_inner() {
                        match elem_pair.as_rule() {
                            Rule::object_decl => {
                                let decl = parse_object_declaration(elem_pair)?;
                                elements.push(ast::ModuleElement::Declaration(decl));
                            }
                            Rule::mapping_decl => {
                                let decl = parse_mapping_declaration(elem_pair)?;
                                elements.push(ast::ModuleElement::Declaration(decl));
                            }
                            Rule::law_decl => {
                                // Recursively parse nested laws if allowed by grammar (might need check)
                                let law_decl = parse_law_declaration(elem_pair)?;
                                elements.push(ast::ModuleElement::Declaration(law_decl));
                            }
                            // Skip comments - they're handled by COMMENT
                            _ => {
                                return Err(BorfError::ParserError(format!(
                                    "Unexpected rule inside quantifier_block: {:?}",
                                    elem_pair.as_rule()
                                )))
                            }
                        }
                    }
                }
                // If elements_container_pair rule is not module_element, it might be the '}' or something else unexpected.
                // If it was '}', block_pairs.next() would be None after this.
                // If it was something else, it's an error handled below.
            }

            // Expect '}' at the end
            if block_pairs.next().is_some() {
                // If there's still a token after the elements container, it's unexpected.
                return Err(BorfError::ParserError(
                    "Unexpected token after elements in quantifier block.".to_string(),
                ));
            }

            ast::LawBody::Block {
                quantifier,
                variable,
                domain,
                elements,
            }
        }
        Rule::quantifier_simple => {
            // These parse directly to an Expression
            let expr = pratt::parse_expression(body_pair)?;
            ast::LawBody::Expression(expr)
        }
        _ => {
            return Err(BorfError::ParserError(format!(
                "Unexpected rule type for law body: {:?}",
                body_pair.as_rule()
            )));
        }
    };

    Ok(Declaration::LawDecl {
        name: law_name,
        body: law_body, // Use the parsed LawBody
    })
}

fn parse_primitive_declaration(pair: Pair<Rule>) -> Result<PrimitiveDecl, BorfError> {
    let mut pairs = pair.into_inner();
    let name = pairs.next().unwrap().as_str().to_string();

    // Skip the colon
    let _ = pairs.next().unwrap(); // Should be COLON

    // Parse elements inside {}
    let mut elements = Vec::new();
    // The rest of the pairs are separator* (primitive_element separator)*
    for elem_pair in pairs {
        // Loop through remaining pairs
        match elem_pair.as_rule() {
            Rule::primitive_element => {
                // Recurse into the actual element (mapping_decl or comment_decl)
                let inner_element_pair = elem_pair.into_inner().next().unwrap();
                match inner_element_pair.as_rule() {
                    Rule::mapping_decl => {
                        let decl = parse_mapping_declaration(inner_element_pair)?;
                        elements.push(PrimitiveElement::Declaration(decl));
                    }
                    // Skip comments - they're handled by COMMENT
                    _ => {
                        return Err(BorfError::NotYetImplemented {
                            feature: format!(
                                "Parsing primitive element type: {:?}",
                                inner_element_pair.as_rule()
                            ),
                            src: None,
                            span: None,
                        });
                    }
                }
            }
            // Rule::separator => { /* Skip separators */ } // Removed: separator is silent
            _ => {
                // This shouldn't happen if grammar is correct
                return Err(BorfError::ParserError(format!(
                    "Unexpected rule inside primitive body: {:?}",
                    elem_pair.as_rule()
                )));
            }
        }
    }

    Ok(PrimitiveDecl { name, elements })
}

fn parse_export_statement(pair: Pair<Rule>) -> Result<String, BorfError> {
    let inner = pair.into_inner().next().unwrap();
    Ok(inner.as_str().to_string())
}

fn parse_import_statement(pair: Pair<Rule>) -> Result<String, BorfError> {
    let inner = pair.into_inner().next().unwrap();
    // Remove quotes from string literal
    let path = inner.as_str();
    let trimmed = path.trim_matches('"');
    Ok(trimmed.to_string())
}

// Type expression parsing
// fn parse_type_expr(pair: Pair<Rule>) -> Result<TypeExpr, BorfError> {
//     // Use our stub in pratt module
//     pratt::parse_type_expression(pair)
// }

// Expression parsing
// fn parse_expression(pair: Pair<Rule>) -> Result<Expression, BorfError> {
//     // Use our stub in pratt module
//     pratt::parse_expression(pair)
// }

// Utility functions
pub fn to_identifier(text: &str) -> Identifier {
    Identifier(text.to_string())
}
