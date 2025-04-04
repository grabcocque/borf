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
mod tests;

// Re-export the key types and functions
pub use ast::*;
pub use category::parse_category_def;
pub use directives::{parse_export_directive, parse_import_directive};
pub use expressions::{
    parse_app_expr, parse_composition_expr, parse_pipe_expr, parse_pipeline_def,
};
pub use laws::parse_law;

use error::{convert_pest_error, make_span, BorfError, NamedSource, SourceSpan, SyntaxError};
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;

/// The pest parser struct generated from the grammar defined in `borf.pest`.
#[derive(Parser)]
#[grammar = "parser/borf.pest"]
pub struct BorfParser;

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
        let src = NamedSource::new("input.borf", input.to_string());
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
        let src = NamedSource::new("input.borf", input.to_string());
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
        match element.as_rule() {
            Rule::statement => {
                // Statement now contains one of our top-level items
                let inner_element = element.into_inner().next().unwrap();
                match inner_element.as_rule() {
                    Rule::category_statement => {
                        items.push(TopLevelItem::Category(parse_category_def(inner_element)?));
                    }
                    Rule::pipeline_statement => {
                        items.push(TopLevelItem::Pipeline(parse_pipeline_def(inner_element)?));
                    }
                    Rule::pipe_statement => {
                        items.push(TopLevelItem::PipeExpr(parse_pipe_expr(inner_element)?));
                    }
                    Rule::app_statement => {
                        items.push(TopLevelItem::AppExpr(parse_app_expr(inner_element)?));
                    }
                    Rule::composition_statement => {
                        items.push(TopLevelItem::CompositionExpr(parse_composition_expr(
                            inner_element,
                        )?));
                    }
                    Rule::export_statement => {
                        items.push(TopLevelItem::Export(parse_export_directive(inner_element)?));
                    }
                    Rule::import_statement => {
                        items.push(TopLevelItem::Import(parse_import_directive(inner_element)?));
                    }
                    _ => {
                        // Create error for unexpected inner rule
                        let rule_str = format!("{:?}", inner_element.as_rule());
                        let span = pair_to_span(&inner_element);
                        let src = get_named_source(input);

                        return Err(Box::new(BorfError::SyntaxError(SyntaxError::new(
                            &format!("Unexpected statement element: {}", rule_str),
                            src,
                            span,
                            "Only category, pipeline, pipe, application, composition, and export statements are allowed",
                            &format!("Unexpected {} here", rule_str)
                        ))));
                    }
                }
            }
            Rule::category_statement => {
                items.push(TopLevelItem::Category(parse_category_def(element)?));
            }
            Rule::pipeline_statement => {
                items.push(TopLevelItem::Pipeline(parse_pipeline_def(element)?));
            }
            Rule::pipe_statement => {
                items.push(TopLevelItem::PipeExpr(parse_pipe_expr(element)?));
            }
            Rule::app_statement => {
                items.push(TopLevelItem::AppExpr(parse_app_expr(element)?));
            }
            Rule::composition_statement => {
                items.push(TopLevelItem::CompositionExpr(parse_composition_expr(
                    element,
                )?));
            }
            Rule::export_statement => {
                items.push(TopLevelItem::Export(parse_export_directive(element)?));
            }
            Rule::import_statement => {
                items.push(TopLevelItem::Import(parse_import_directive(element)?));
            }
            Rule::EOI => (),
            _ => {
                // Create a better error with source location
                let rule_str = format!("{:?}", element.as_rule());
                let span = pair_to_span(&element);
                let src = get_named_source(input);

                return Err(Box::new(BorfError::SyntaxError(SyntaxError::new(
                    &format!("Unexpected top-level element: {}", rule_str),
                    src,
                    span,
                    "Only category, pipeline, pipe, application, composition, and export statements are allowed at the top level",
                    &format!("Unexpected {} here", rule_str)
                ))));
            }
        }
    }

    Ok(items)
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
    let start = pair.as_span().start();
    let end = pair.as_span().end();
    make_span(start, end)
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
        NamedSource::new(&name, input.to_string())
    } else {
        NamedSource::new("input.borf", input.to_string())
    }
}
