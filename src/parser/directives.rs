//! Parsers for directives in the Borf language.
//!
//! This module provides functions for parsing import and export directives.

use super::ast::{ExportDirective, ImportDirective};
use super::error::{BorfError, SyntaxError};
use super::Rule;
use crate::parser::{get_named_source, pair_to_span};
use pest::iterators::Pair;

/// Parses an export directive from a pest pair.
///
/// Export directives specify which identifiers are exported from a module.
///
/// # Arguments
///
/// * `pair` - A pest Pair representing an export directive
///
/// # Returns
///
/// * `Result<ExportDirective, Box<BorfError>>` - The parsed export directive or an error
pub fn parse_export_directive(pair: Pair<Rule>) -> Result<ExportDirective, Box<BorfError>> {
    let span = pair_to_span(&pair);
    let src = get_named_source(pair.as_str());

    // Create a vector to store all identifier names
    let mut identifiers = Vec::new();

    // Process all inner pairs
    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            // Handle the export id1, id2, id3 format
            Rule::comma_separated_ids => {
                for item in inner_pair.into_inner() {
                    if item.as_rule() == Rule::ident || item.as_rule() == Rule::transform_ident {
                        identifiers.push(item.as_str().to_string());
                    }
                }
            }
            // Direct identifiers (top level)
            Rule::ident | Rule::transform_ident => {
                identifiers.push(inner_pair.as_str().to_string());
            }
            _ => {} // Ignore other rules
        }
    }

    if identifiers.is_empty() {
        return Err(Box::new(BorfError::SyntaxError(SyntaxError::new(
            "Export directive with no identifiers",
            src,
            span,
            "Export directives must specify at least one identifier to export",
            "No identifiers to export",
        ))));
    }

    Ok(ExportDirective { identifiers })
}

/// Parses an import directive from a pest pair.
///
/// Import directives specify which modules are imported.
///
/// # Arguments
///
/// * `pair` - A pest Pair representing an import directive
///
/// # Returns
///
/// * `Result<ImportDirective, Box<BorfError>>` - The parsed import directive or an error
pub fn parse_import_directive(pair: Pair<Rule>) -> Result<ImportDirective, Box<BorfError>> {
    let pair_clone = pair.clone(); // Clone for error handling
    let path_pair = pair.into_inner().next().unwrap();
    let path_with_quotes = path_pair.as_str().to_string();

    // Remove the surrounding quotes if present
    let path = if path_with_quotes.starts_with('"') && path_with_quotes.ends_with('"') {
        path_with_quotes[1..path_with_quotes.len() - 1].to_string()
    } else {
        path_with_quotes
    };

    if path.is_empty() {
        let span = pair_to_span(&pair_clone);
        let src = get_named_source(pair_clone.as_str());
        return Err(Box::new(BorfError::SyntaxError(SyntaxError::new(
            "Import directive with empty path",
            src,
            span,
            "Import directives must specify a non-empty path to import",
            "Empty import path",
        ))));
    }

    Ok(ImportDirective { path })
}
