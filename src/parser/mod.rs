pub mod ast;
use crate::errors::ParseError;
use crate::observer::ParserObserver;
use crate::traceable_parser::TraceableParser;
use ast::*;
use lru::LruCache;
use miette;
use once_cell::sync::Lazy;
use pest::error::Error as PestError;
use pest::iterators::{Pair, Pairs};
use pest::pratt_parser::{Assoc, Op, PrattParser};
use pest::Parser;
use pest_derive::Parser;
use rustc_hash::FxHashMap;
use std::cell::RefCell;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::num::NonZeroUsize;
use std::sync::Arc;

// Suppress warnings at the module level
#[allow(dead_code, clippy::result_large_err)]
// Grammar definition
#[derive(Parser)]
#[grammar = "parser/grammar.pest"]
pub struct BorfParser;

// Type alias for source location tracking in errors
#[allow(dead_code)]
type Location = String;

/// Location information for diagnostics
#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub start: usize,                // start position
    pub end: usize,                  // end position
    pub line: usize,                 // line number (1-indexed)
    pub column: usize,               // column number (1-indexed)
    pub source_name: Option<String>, // Source file name or identifier
}

impl SourceLocation {
    /// Create a location from a pest span
    pub fn from_pair(pair: &Pair<Rule>, source_name: Option<String>) -> Self {
        let span = pair.as_span();
        let (line, col) = span.start_pos().line_col();
        Self {
            start: span.start(),
            end: span.end(),
            line,
            column: col,
            source_name,
        }
    }

    pub fn to_miette_span(&self) -> miette::SourceSpan {
        (self.start, self.end - self.start).into()
    }
}

// Helper functions for AST construction
#[allow(dead_code)]
fn with_loc<T>(
    node: T,
    pair: &Pair<Rule>,
    source_name: &Option<String>,
) -> (T, Option<SourceLocation>) {
    (
        node,
        Some(SourceLocation::from_pair(pair, source_name.clone())),
    )
}

fn loc(pair: &Pair<Rule>, source_name: &Option<String>) -> Option<SourceLocation> {
    Some(SourceLocation::from_pair(pair, source_name.clone()))
}

// Type conversion utilities
#[allow(dead_code)]
fn to_box<T>(v: T) -> Box<T> {
    Box::new(v)
}

#[allow(dead_code)]
fn to_smallvec8<T>(vec: Vec<T>) -> SmallVec8<T> {
    SmallVec8::from_iter(vec)
}

#[allow(dead_code)]
fn to_boxed_smallvec8<T>(vec: Vec<T>) -> SmallVec8<Box<T>> {
    SmallVec8::from_iter(vec.into_iter().map(Box::new))
}

#[allow(dead_code)]
fn to_fxhashmap<K: Hash + Eq, V>(map: HashMap<K, V>) -> FxHashMap<K, V> {
    let mut result = FxHashMap::default();
    result.extend(map);
    result
}

#[allow(dead_code)]
fn to_boxed_fxhashmap<K: Hash + Eq, V>(map: HashMap<K, Box<V>>) -> FxHashMap<K, Box<V>> {
    let mut fxmap = FxHashMap::default();
    for (k, v) in map {
        fxmap.insert(k, v);
    }
    fxmap
}

// More efficient parse cache key for memoization
#[derive(Eq, PartialEq)]
#[allow(dead_code)]
struct ParseCacheKey(Rule, usize, usize, u64); // rule, start, end, source_name_hash

impl Hash for ParseCacheKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Using discriminant value for Rule enum for better hashing performance
        std::mem::discriminant(&self.0).hash(state);
        self.1.hash(state);
        self.2.hash(state);
        self.3.hash(state);
    }
}

// ThreadLocal LRU cache for memoizing expression parsing results
thread_local! {
    static EXPR_CACHE: RefCell<LruCache<ParseCacheKey, Expr>> =
        RefCell::new(LruCache::new(NonZeroUsize::new(1000).unwrap()));
}

// Restore original DiagnosticsCollector
#[derive(Debug, Clone)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone)]
pub struct CodeSuggestion {
    pub description: String,
    pub replacement: String,
    pub span: miette::SourceSpan,
}

#[derive(Debug, Clone)]
pub struct ParseDiagnostic {
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub span: miette::SourceSpan,
    pub suggestions: Vec<CodeSuggestion>,
}

pub struct DiagnosticsCollector {
    diagnostics: Vec<ParseDiagnostic>,
}

impl Default for DiagnosticsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl DiagnosticsCollector {
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
        }
    }

    pub fn add_diagnostic(&mut self, diagnostic: ParseDiagnostic) {
        self.diagnostics.push(diagnostic);
    }

    pub fn suggest_fix(&mut self, message: &str, span: miette::SourceSpan, fix: &str) {
        self.add_diagnostic(ParseDiagnostic {
            severity: DiagnosticSeverity::Error,
            message: format!("Try: {}", fix),
            span,
            suggestions: vec![CodeSuggestion {
                description: message.to_string(),
                replacement: fix.to_string(),
                span,
            }],
        });
    }

    pub fn suggest_missing_semicolon(&mut self, span: miette::SourceSpan) {
        self.suggest_fix("Add missing semicolon", span, ";");
    }

    pub fn suggest_missing_closing_paren(&mut self, span: miette::SourceSpan) {
        self.suggest_fix("Add missing closing parenthesis", span, ")");
    }

    pub fn suggest_missing_closing_bracket(&mut self, span: miette::SourceSpan) {
        self.suggest_fix("Add missing closing bracket", span, "]");
    }

    pub fn suggest_missing_closing_brace(&mut self, span: miette::SourceSpan) {
        self.suggest_fix("Add missing closing brace", span, "}");
    }

    pub fn suggest_variable_correction(
        &mut self,
        span: miette::SourceSpan,
        _typo: &str, // Keep original signature for now
        correction: &str,
    ) {
        self.suggest_fix(&format!("Did you mean '{}'?", correction), span, correction);
    }

    pub fn suggest_missing_operator(&mut self, span: miette::SourceSpan, likely_op: &str) {
        self.suggest_fix(
            &format!("Add missing operator '{}'", likely_op),
            span,
            likely_op,
        );
    }

    pub fn suggest_unwrapping(&mut self, span: miette::SourceSpan, value: &str, wrapper: &str) {
        let unwrapped = match wrapper {
            "Option" => format!("{}.unwrap()", value),
            "Result" => format!("{}.unwrap()", value),
            "Some" => format!("{}.unwrap()", value),
            "Ok" => format!("{}.unwrap()", value),
            _ => value.to_string(),
        };
        self.suggest_fix(&format!("Unwrap the {} value", wrapper), span, &unwrapped);
    }

    pub fn get_diagnostics(&self) -> &[ParseDiagnostic] {
        &self.diagnostics
    }
}

// Restore original ParserContext
pub struct ParserContext<'a> {
    pub input: &'a str,
    pub source_name: Option<String>,
    pub error_recovery: bool,
    pub diagnostics: DiagnosticsCollector,
    pub config: ParserConfig,
}

pub struct ParserConfig {
    pub max_errors: usize,
    pub collect_suggestions: bool,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            max_errors: 10,
            collect_suggestions: true,
        }
    }
}

impl<'a> ParserContext<'a> {
    pub fn new(input: &'a str, source_name: Option<String>) -> Self {
        Self {
            input,
            source_name,
            error_recovery: false,
            diagnostics: DiagnosticsCollector::new(),
            config: ParserConfig::default(),
        }
    }

    pub fn with_recovery(mut self) -> Self {
        self.error_recovery = true;
        self
    }

    pub fn with_config(mut self, config: ParserConfig) -> Self {
        self.config = config;
        self
    }

    // Restore original record_error
    pub fn record_error(&mut self, error: &ParseError, pair: &Pair<Rule>) {
        if self.diagnostics.diagnostics.len() >= self.config.max_errors
            && self.config.max_errors > 0
        {
            return;
        }
        // Basic diagnostic conversion, might need refinement based on original code
        let span = get_span_from_pair(pair);
        let diagnostic = ParseDiagnostic {
            severity: DiagnosticSeverity::Error,
            message: error.to_string(),
            span,
            suggestions: vec![], // Placeholder, add suggestion logic if present before
        };
        self.diagnostics.add_diagnostic(diagnostic);
    }

    // Restore original suggest_fix
    pub fn suggest_fix(&mut self, message: &str, span: miette::SourceSpan, replacement: &str) {
        if self.config.collect_suggestions {
            self.diagnostics.suggest_fix(message, span, replacement);
        }
    }
}

// ... Restore original ExprVisitor, ExprTransformer if they existed ...
// (Assuming they were removed in the previous erroneous edit)

// Restore original Pratt parser setup
static PRATT_PARSER: Lazy<PrattParser<Rule>> = Lazy::new(|| {
    PrattParser::new()
        .op(Op::infix(Rule::op_or, Assoc::Left))
        .op(Op::infix(Rule::op_and, Assoc::Left))
        .op(Op::infix(Rule::op_eq, Assoc::Left))
        .op(Op::infix(Rule::op_neq, Assoc::Left))
        .op(Op::infix(Rule::op_lt, Assoc::Left))
        .op(Op::infix(Rule::op_lte, Assoc::Left))
        .op(Op::infix(Rule::op_gt, Assoc::Left))
        .op(Op::infix(Rule::op_gte, Assoc::Left))
        .op(Op::infix(Rule::op_pipe, Assoc::Left)) // Check original associativity
        .op(Op::infix(Rule::op_add, Assoc::Left))
        .op(Op::infix(Rule::op_sub, Assoc::Left))
        .op(Op::infix(Rule::op_mul, Assoc::Left))
        .op(Op::infix(Rule::op_div, Assoc::Left))
        .op(Op::prefix(Rule::op_not))
        .op(Op::prefix(Rule::op_neg))
        .op(Op::prefix(Rule::op_unquote))
    // Add other operators from original if any (e.g., type annotation op_colon?)
});

// Restore original parse_expr_with_context
// This likely took `context: &mut ParserContext`
fn _parse_expr_with_context(
    pair: Pair<Rule>,
    context: &mut ParserContext,
) -> Result<Expr, Box<ParseError>> {
    // Clone the source_name for use in the function
    let source_name = context.source_name.clone();

    // Check cache first (requires immutable borrow logic, maybe separate?)
    let key = {
        let span = pair.as_span();
        let mut hasher = DefaultHasher::new();
        source_name.hash(&mut hasher);
        ParseCacheKey(pair.as_rule(), span.start(), span.end(), hasher.finish())
    };

    if let Some(cached_expr) = EXPR_CACHE.with(|cache| cache.borrow().peek(&key).cloned()) {
        return Ok(cached_expr);
    }

    // Forward to the non-context version
    let result = parse_expr(pair.clone(), &source_name);

    // Cache result if successful
    if let Ok(expr) = &result {
        EXPR_CACHE.with(|cache| {
            cache.borrow_mut().put(key, expr.clone());
        });
    }

    result
}

// Restore original is_complex_expr
fn _is_complex_expr(pair: &Pair<Rule>) -> bool {
    match pair.as_rule() {
        Rule::expr | Rule::application | Rule::lambda | Rule::let_expr | Rule::ternary => true,
        Rule::parenthesized_expr => (pair.as_span().end() - pair.as_span().start()) > 20, // Arbitrary threshold
        _ => false,
    }
}

// Restore original parse_pratt_expr_with_context
// This is where the borrow checker errors likely occurred.
// Keep original structure for now to isolate the pattern fix.
fn _parse_pratt_expr_with_context(
    pairs: Pairs<Rule>,
    context: &mut ParserContext,
) -> Result<Expr, Box<ParseError>> {
    // Clone context.source_name for use within the function
    let source_name = context.source_name.clone();

    // First attempt to parse without recording errors
    let result = parse_pratt_expr(pairs, &source_name);

    // If parsing failed and we're in error recovery mode,
    // we could create a placeholder expression
    if let Err(ref _e) = result {
        // Here we would record the error, but we'd need the pair that caused it
        // Since we don't have it, this is a simplification
        if context.error_recovery {
            return Ok(Expr::Variable("__recovery_placeholder__".to_string(), None));
        }
    }

    result
}

// Restore original parse_primary_expr_with_context
fn _parse_primary_expr_with_context(
    pair: Pair<Rule>,
    context: &mut ParserContext,
) -> Result<Expr, Box<ParseError>> {
    // Forward to the non-context version
    let source_name = context.source_name.clone();
    let result = parse_primary_expr(pair, &source_name);

    // If we're in error recovery mode and got an error, return a placeholder
    if let Err(ref _e) = result {
        if context.error_recovery {
            return Ok(Expr::Variable("__recovery_placeholder__".to_string(), None));
        }
    }

    result
}

// ... Restore original parse_literal, parse_lambda, etc. if changed ...
// Assuming they took &Option<String> and not context before.

pub fn parse_literal(
    pair: Pair<Rule>,
    source_name: &Option<String>,
) -> Result<Expr, Box<ParseError>> {
    let location = loc(&pair, source_name);
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::primitive_literal => {
            let lit = parse_primitive_literal_to_lit(inner)?;
            Ok(Expr::Literal(lit, location))
        }
        Rule::collection_literal => parse_collection_literal(inner, source_name),
        _ => unreachable!("Unexpected literal type"),
    }
}

fn parse_primitive_literal_to_lit(pair: Pair<Rule>) -> Result<Literal, Box<ParseError>> {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::integer => inner
            .as_str()
            .parse::<i64>()
            .map(Literal::Integer)
            .map_err(|_| {
                Box::new(ParseError::Unexpected(format!(
                    "Invalid integer: {}",
                    inner.as_str()
                )))
            }),
        Rule::float => inner
            .as_str()
            .parse::<f64>()
            .map(Literal::Float)
            .map_err(|_| {
                Box::new(ParseError::Unexpected(format!(
                    "Invalid float: {}",
                    inner.as_str()
                )))
            }),
        Rule::string => {
            let s = inner.as_str();
            let without_quotes = &s[1..s.len() - 1]; // Remove quotes
            Ok(Literal::String(without_quotes.to_string()))
        }
        Rule::boolean => match inner.as_str() {
            "true" => Ok(Literal::Boolean(true)),
            "false" => Ok(Literal::Boolean(false)),
            _ => Err(Box::new(ParseError::SyntaxError {
                message: format!("Invalid boolean: {}", inner.as_str()),
                src: Arc::new(miette::NamedSource::new(
                    "<unknown>",
                    inner.as_str().to_string(),
                )),
                span: (0, inner.as_str().len()).into(),
                location: String::new(),
                help_message: "Boolean must be 'true' or 'false'".to_string(),
                suggestion: None,
            })),
        },
        _ => Err(Box::new(ParseError::SyntaxError {
            message: format!("Unknown primitive literal type: {:?}", inner.as_rule()),
            src: Arc::new(miette::NamedSource::new(
                "<unknown>",
                inner.as_str().to_string(),
            )),
            span: (0, inner.as_str().len()).into(),
            location: String::new(),
            help_message: "Expected integer, float, string, or boolean".to_string(),
            suggestion: None,
        })),
    }
}

// ... (other parsing functions like parse_collection_literal, parse_lambda etc)

// Parse Pattern - Fix borrow error
fn parse_pattern(
    pair: Pair<Rule>,
    source_name: &Option<String>,
) -> Result<Pattern, Box<ParseError>> {
    let pair_for_location = pair.clone(); // Clone for location before consumption
    let location = loc(&pair_for_location, source_name);
    // Clone the pair *again* before calling into_inner() to fix the move error.
    let inner_pair = pair.clone().into_inner().next().unwrap();
    let inner_pair_clone = inner_pair.clone(); // Clone inner for potential reuse/error

    match inner_pair.as_rule() {
        Rule::literal => {
            // Pass the cloned inner_pair_clone here
            let lit_expr = parse_literal(inner_pair_clone, source_name)?;
            if let Expr::Literal(lit, _) = lit_expr {
                Ok(Pattern::Literal(lit, location))
            } else {
                // This case indicates an internal error if parse_literal works correctly
                unreachable!("parse_literal did not return Expr::Literal")
            }
        }
        Rule::identifier => {
            let name = inner_pair_clone.as_str();
            if name == "_" {
                Ok(Pattern::Wildcard(location))
            } else {
                Ok(Pattern::Variable(name.to_string(), location))
            }
        }
        Rule::wildcard => Ok(Pattern::Wildcard(location)),
        Rule::list_pattern => {
            // Consume inner_pair for its inner elements
            let patterns = inner_pair
                .into_inner()
                .map(|p| parse_pattern(p, source_name).map(Box::new))
                .collect::<Result<SmallVec8<_>, _>>()?;
            Ok(Pattern::List(patterns, location))
        }
        Rule::set_pattern => {
            // Consume inner_pair for its inner elements
            let patterns = inner_pair
                .into_inner()
                .map(|p| parse_pattern(p, source_name).map(Box::new))
                .collect::<Result<SmallVec8<_>, _>>()?;
            Ok(Pattern::Set(patterns, location))
        }
        Rule::map_pattern => {
            let mut map_patterns = FxHashMap::default();
            // Consume inner_pair for its inner elements (map_pattern_entry)
            for entry_pair in inner_pair.into_inner() {
                let mut entry_inner = entry_pair.into_inner();
                let key_pair = entry_inner.next().unwrap();
                let value_pair = entry_inner.next().unwrap();

                // Key must be variable or string literal pattern
                let key_pattern = parse_pattern(key_pair.clone(), source_name)?;
                let key = match key_pattern {
                    Pattern::Variable(name, _) => name,
                    Pattern::Literal(Literal::String(s), _) => s,
                    _ => {
                        return Err(Box::new(ParseError::InvalidMapPatternKey {
                            found_type: format!("{:?}", key_pattern),
                            span: get_span_from_pair(&key_pair), // Use original key_pair span
                            location: get_location_from_pair(&key_pair),
                        }));
                    }
                };

                let value_pattern = parse_pattern(value_pair, source_name)?;
                map_patterns.insert(key, Box::new(value_pattern));
            }
            Ok(Pattern::Map(map_patterns, location))
        }
        Rule::type_annotation_pattern => {
            // Consume inner_pair for its inner elements
            let mut inner_pairs = inner_pair.into_inner();
            let pattern_pair = inner_pairs.next().unwrap();
            let type_pair = inner_pairs.next().unwrap();
            let inner_pattern = parse_pattern(pattern_pair, source_name)?;
            let type_expr = parse_type_expr(type_pair)?; // Assuming parse_type_expr exists
            Ok(Pattern::TypeAnnotated(
                Box::new(inner_pattern),
                type_expr,
                location,
            ))
        }
        _ => Err(Box::new(ParseError::SyntaxError {
            message: format!("Unexpected pattern type: {:?}", inner_pair_clone.as_rule()), // Use clone for rule
            src: Arc::new(miette::NamedSource::new(
                source_name
                    .clone()
                    .unwrap_or_else(|| "<unknown>".to_string()),
                "".to_string(),
            )),
            span: get_span_from_pair(&inner_pair_clone), // Use clone for span
            location: get_location_from_pair(&inner_pair_clone),
            help_message: "Expected a valid pattern (literal, variable, _, list, etc.)."
                .to_string(),
            suggestion: None,
        })),
    }
}

// ... (parse_type_expr, etc.)
// Assume parse_module exists and handles Rule::module

pub fn parse_module(
    pair: Pair<Rule>,
    source_name: &Option<String>,
) -> Result<Module, Box<ParseError>> {
    // Create a context for error collection
    let mut context = ParserContext::new(pair.as_str(), source_name.clone());
    context.error_recovery = true;

    match pair.as_rule() {
        Rule::file => {
            // Find the module declaration
            let module_decl = pair.into_inner().find(|p| p.as_rule() == Rule::module_decl);

            if let Some(module_decl) = module_decl {
                // Get the module name
                let mut module_iter = module_decl.into_inner();
                let module_name = module_iter
                    .next()
                    .map(|id| id.as_str().to_string())
                    .unwrap_or_else(|| "unknown".to_string());

                // Get the module body
                let module_body = module_iter.next().unwrap_or_else(|| {
                    panic!("Could not find module body for module {}", module_name)
                });

                // Process all declarations in the module body, but also check for illegal commas
                let mut declarations = Vec::new();
                let mut has_errors = false;

                // Process all pairs in the module body
                for p in module_body.into_inner() {
                    match p.as_rule() {
                        Rule::declaration => match parse_declaration(p.clone(), source_name) {
                            Ok(decl) => declarations.push(decl),
                            Err(e) => {
                                context.record_error(&e, &p);
                                has_errors = true;
                            }
                        },
                        Rule::CATCH_ERROR => {
                            // Handle illegal comma
                            handle_illegal_comma(&p, &mut context);
                            has_errors = true;
                        }
                        _ => {} // Skip comments and whitespace
                    }
                }

                // If there were parsing errors and we're not in recovery mode, return an error
                if has_errors && !context.error_recovery {
                    // Collect all errors into a single error message
                    let error_messages: Vec<String> = context
                        .diagnostics
                        .get_diagnostics()
                        .iter()
                        .map(|d| d.message.clone())
                        .collect();

                    return Err(Box::new(ParseError::Other(format!(
                        "Failed to parse module {}. Errors: {}",
                        module_name,
                        error_messages.join(", ")
                    ))));
                }

                // Return the module with all valid declarations
                Ok(Module {
                    name: module_name,
                    declarations,
                    source: source_name.clone(),
                })
            } else {
                Err(Box::new(ParseError::MissingNode(
                    "module declaration".to_string(),
                )))
            }
        }
        _ => Err(Box::new(ParseError::UnexpectedToken(
            "file".to_string(),
            format!("{:?}", pair.as_rule()),
        ))),
    }
}

// Assumes parse_declaration exists
pub fn parse_declaration(
    pair: Pair<Rule>,
    source_name: &Option<String>,
) -> Result<Declaration, Box<ParseError>> {
    let location = loc(&pair, source_name);
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::type_decl => {
            let mut items = inner.into_inner();
            let name = items.next().unwrap().as_str().to_string();
            let type_expr = parse_type_expr(items.next().unwrap())?;
            Ok(Declaration::Type(name, type_expr, location))
        }
        Rule::op_decl => {
            let mut items = inner.into_inner();
            let name = items.next().unwrap().as_str().to_string();
            let type_expr = parse_type_expr(items.next().unwrap())?;
            Ok(Declaration::Operation(name, type_expr, location))
        }
        Rule::fn_decl => {
            // Assuming fn_decl handles both signature and optional body
            let mut items = inner.into_inner();
            let name = items.next().unwrap().as_str().to_string();
            let signature_pair = items.next().unwrap();
            let signature = parse_type_expr(signature_pair)?;
            // Check for optional body
            let body_expr = if let Some(body_pair) = items.next() {
                Some(Box::new(parse_expr(body_pair, source_name)?))
            } else {
                None // No body provided, just a declaration
            };
            // Need to adjust Declaration enum - current Function variant requires body.
            // Maybe use Entity for functions without body?
            // For now, let's assume body is required by Declaration::Function
            if let Some(body) = body_expr {
                Ok(Declaration::Function(name, signature, body, location))
            } else {
                // Create an Entity declaration if no body is present?
                // This deviates from the current AST structure.
                // Returning error for now, AST needs adjustment for body-less fn decls.
                Err(Box::new(ParseError::SyntaxError {
                     message: format!("Function declaration '{}' missing implementation body.", name),
                     src: Arc::new(miette::NamedSource::new(
                        source_name.clone().unwrap_or_else(|| "<unknown>".to_string()),
                        String::new(),
                     )),
                     span: location.clone().map_or_else(|| (0,0).into(), |l| l.to_miette_span()),
                     location: location.map_or("".to_string(), |l| format!("{}:{}", l.line, l.column)),
                     help_message: "Function declarations currently require a body ('=' followed by expression).".to_string(),
                     suggestion: None,
                 }))
            }
        }
        Rule::dep_decl => {
            let mut items = inner.into_inner();
            let import = items.next().unwrap().as_str().to_string();
            let export_or_from = items.next().map(|p| p.as_str().to_string());
            let direct = export_or_from.is_none(); // Rough check
            let export = export_or_from.unwrap_or_default(); // Simplify
            Ok(Declaration::Dependency(import, export, direct, location))
        }
        Rule::entity_decl => {
            // Assuming an entity rule exists
            let mut items = inner.into_inner();
            let name = items.next().unwrap().as_str().to_string();
            let type_expr = parse_type_expr(items.next().unwrap())?;
            let value_expr = items
                .next()
                .map(|p| parse_expr(p, source_name).map(Box::new))
                .transpose()?;
            Ok(Declaration::Entity(name, type_expr, value_expr, location))
        }
        _ => Err(Box::new(ParseError::UnexpectedRule {
            expected: "declaration (type, op, fn, dep, entity)".to_string(),
            found: format!("{:?}", inner.as_rule()),
            span: get_span_from_pair(&inner),
            location: get_location_from_pair(&inner),
        })),
    }
}

// ... other helper functions ...

pub fn get_span_from_pair(pair: &Pair<Rule>) -> miette::SourceSpan {
    let span = pair.as_span();
    (span.start(), span.end() - span.start()).into()
}

pub fn get_location_from_pair(pair: &Pair<Rule>) -> String {
    let (line, col) = pair.as_span().start_pos().line_col();
    format!("{}:{}", line, col)
}

// Keep standalone parse_expr if needed by REPL or other parts
// This one doesn't use the context or cache
pub fn parse_expr(pair: Pair<Rule>, source_name: &Option<String>) -> Result<Expr, Box<ParseError>> {
    let pairs = pair.into_inner();
    parse_pratt_expr(pairs, source_name)
}

// Standalone Pratt parser logic (without context/cache)
fn parse_pratt_expr(
    pairs: Pairs<Rule>,
    source_name: &Option<String>,
) -> Result<Expr, Box<ParseError>> {
    let cloned_source_name = source_name.clone(); // Clone for closures
    let result = PRATT_PARSER
        .map_primary(|primary| parse_primary_expr(primary, &cloned_source_name))
        .map_prefix(|op, rhs_result| match rhs_result {
            Ok(rhs) => Ok(Expr::UnaryOp(
                op.as_str().to_string(),
                Box::new(rhs),
                loc(&op, &cloned_source_name),
            )),
            Err(e) => Err(e),
        })
        .map_infix(
            |lhs_result, op, rhs_result| match (lhs_result, rhs_result) {
                (Ok(lhs), Ok(rhs)) => Ok(Expr::BinaryOp(
                    op.as_str().to_string(),
                    Box::new(lhs),
                    Box::new(rhs),
                    loc(&op, &cloned_source_name),
                )),
                (Err(e), _) => Err(e),
                (_, Err(e)) => Err(e),
            },
        )
        .parse(pairs);

    result
}

/// Helper to parse the content of a `primary` rule
fn parse_primary_expr(
    pair: Pair<Rule>,
    source_name: &Option<String>,
) -> Result<Expr, Box<ParseError>> {
    let pair_clone = pair.clone();
    let inner = pair.into_inner().next().ok_or_else(|| {
        Box::new(ParseError::MissingToken {
            expected: "expression inside primary".to_string(),
            src: Arc::new(miette::NamedSource::new(
                source_name
                    .clone()
                    .unwrap_or_else(|| "<unknown>".to_string()),
                "".to_string(),
            )),
            span: get_span_from_pair(&pair_clone),
            location: get_location_from_pair(&pair_clone),
            help_message: "Primary expression is empty".to_string(),
            suggestion: None,
        })
    })?;

    match inner.as_rule() {
        Rule::literal => parse_literal(inner, source_name),
        Rule::identifier => Ok(Expr::Variable(
            inner.as_str().to_string(),
            loc(&inner, source_name),
        )),
        Rule::lambda => parse_lambda(inner, source_name),
        Rule::application => parse_application(inner, source_name),
        Rule::let_expr => parse_let_expr(inner, source_name),
        Rule::ternary => parse_ternary(inner, source_name),
        Rule::parenthesized_expr => {
            let inner_clone = inner.clone();
            let expr_pair = inner.into_inner().next().ok_or_else(|| {
                Box::new(ParseError::MissingToken {
                    expected: "expression inside parentheses".to_string(),
                    src: Arc::new(miette::NamedSource::new(
                        source_name
                            .clone()
                            .unwrap_or_else(|| "<unknown>".to_string()),
                        "".to_string(),
                    )),
                    span: get_span_from_pair(&inner_clone),
                    location: get_location_from_pair(&inner_clone),
                    help_message: "Empty parentheses not allowed".to_string(),
                    suggestion: None,
                })
            })?;
            parse_expr(expr_pair, source_name)
        }
        Rule::quoting_expr => parse_quoting_expr(inner, source_name),
        _ => Err(Box::new(ParseError::SyntaxError {
            message: format!("Unexpected expression type: {:?}", inner.as_rule()),
            src: Arc::new(miette::NamedSource::new(
                source_name
                    .clone()
                    .unwrap_or_else(|| "<unknown>".to_string()),
                "".to_string(),
            )),
            span: get_span_from_pair(&inner),
            location: get_location_from_pair(&inner),
            help_message: "Expected a valid expression".to_string(),
            suggestion: None,
        })),
    }
}

// Add these functions before parse_primary_expr_with_context

fn parse_lambda(pair: Pair<Rule>, source_name: &Option<String>) -> Result<Expr, Box<ParseError>> {
    // Stub implementation
    Ok(Expr::Literal(
        Literal::String("lambda stub".to_string()),
        loc(&pair, source_name),
    ))
}

fn parse_application(
    pair: Pair<Rule>,
    source_name: &Option<String>,
) -> Result<Expr, Box<ParseError>> {
    // Stub implementation
    Ok(Expr::Literal(
        Literal::String("application stub".to_string()),
        loc(&pair, source_name),
    ))
}

fn parse_let_expr(pair: Pair<Rule>, source_name: &Option<String>) -> Result<Expr, Box<ParseError>> {
    let location = loc(&pair, source_name);
    let mut inner = pair.into_inner();

    // Get the binding pair (contains pattern and expr)
    let binding_pair = inner.next().ok_or_else(|| {
        Box::new(ParseError::MissingNode {
            expected: "binding (pattern = expr)".to_string(),
            span: location
                .clone()
                .map_or_else(|| (0, 0).into(), |l| l.to_miette_span()),
            location: "let expression".to_string(),
        })
    })?;

    // Parse the binding
    let mut binding_inner = binding_pair.into_inner();

    // Get and parse the pattern
    let pattern_pair = binding_inner.next().ok_or_else(|| {
        Box::new(ParseError::MissingNode {
            expected: "pattern".to_string(),
            span: location
                .clone()
                .map_or_else(|| (0, 0).into(), |l| l.to_miette_span()),
            location: "binding".to_string(),
        })
    })?;
    let pattern = parse_pattern(pattern_pair, source_name)?;

    // Get and parse the bound expression
    let expr_pair = binding_inner.next().ok_or_else(|| {
        Box::new(ParseError::MissingNode {
            expected: "expression".to_string(),
            span: location
                .clone()
                .map_or_else(|| (0, 0).into(), |l| l.to_miette_span()),
            location: "binding".to_string(),
        })
    })?;
    let bound_expr = parse_expr(expr_pair, source_name)?;

    // Check if there's an optional "in expr" part
    // If not provided, use a default expression (typically Variable::Void)
    let body_expr = if let Some(in_expr_pair) = inner.next() {
        // Process the "in expr" part
        let body_pair = in_expr_pair.into_inner().next().ok_or_else(|| {
            Box::new(ParseError::MissingNode {
                expected: "expression after 'in'".to_string(),
                span: location
                    .clone()
                    .map_or_else(|| (0, 0).into(), |l| l.to_miette_span()),
                location: "let expression".to_string(),
            })
        })?;
        parse_expr(body_pair, source_name)?
    } else {
        // If no "in expr" is provided in REPL mode, use a void expression as body
        Expr::Literal(Literal::String("void".to_string()), location.clone())
    };

    // Create the Let expression with the pattern, bound expression, and body expression
    Ok(Expr::Let(
        Box::new(pattern),
        Box::new(bound_expr),
        Box::new(body_expr),
        location,
    ))
}

fn parse_ternary(pair: Pair<Rule>, source_name: &Option<String>) -> Result<Expr, Box<ParseError>> {
    let location = loc(&pair, source_name);
    let mut inner = pair.into_inner();

    // With the new syntax, the order is: true_expr, condition, false_expr
    // Get the true_expr (comes first now)
    let true_expr_pair = inner.next().ok_or_else(|| {
        Box::new(ParseError::MissingNode {
            expected: "true branch expression".to_string(),
            span: location
                .clone()
                .map_or_else(|| (0, 0).into(), |l| l.to_miette_span()),
            location: "ternary expression".to_string(),
        })
    })?;
    let true_expr = parse_expr(true_expr_pair, source_name)?;

    // Get the condition (comes second, after 'iff' keyword)
    let condition_pair = inner.next().ok_or_else(|| {
        Box::new(ParseError::MissingNode {
            expected: "condition expression".to_string(),
            span: location
                .clone()
                .map_or_else(|| (0, 0).into(), |l| l.to_miette_span()),
            location: "ternary expression".to_string(),
        })
    })?;
    let condition = parse_expr(condition_pair, source_name)?;

    // Get the false_expr (comes third, after 'or_else' keyword)
    let false_expr_pair = inner.next().ok_or_else(|| {
        Box::new(ParseError::MissingNode {
            expected: "false branch expression".to_string(),
            span: location
                .clone()
                .map_or_else(|| (0, 0).into(), |l| l.to_miette_span()),
            location: "ternary expression".to_string(),
        })
    })?;
    let false_expr = parse_expr(false_expr_pair, source_name)?;

    // The Expr::If AST node still takes arguments in the order: condition, then_expr, else_expr
    Ok(Expr::If(
        Box::new(condition),
        Box::new(true_expr),
        Box::new(false_expr),
        location,
    ))
}

fn parse_quoting_expr(
    pair: Pair<Rule>,
    source_name: &Option<String>,
) -> Result<Expr, Box<ParseError>> {
    // Stub implementation
    Ok(Expr::Literal(
        Literal::String("quoting stub".to_string()),
        loc(&pair, source_name),
    ))
}

fn parse_collection_literal(
    pair: Pair<Rule>,
    source_name: &Option<String>,
) -> Result<Expr, Box<ParseError>> {
    // Stub implementation
    Ok(Expr::Literal(
        Literal::String("collection stub".to_string()),
        loc(&pair, source_name),
    ))
}

fn parse_type_expr(_pair: Pair<Rule>) -> Result<TypeExpr, Box<ParseError>> {
    // This is a stub implementation for now
    Ok(TypeExpr::Name("stub".to_string(), None))
}

/// Parses a module from input string with tracing enabled.
pub fn parse_module_with_trace(input: &str, filename: &str) -> Result<Module, Box<ParseError>> {
    let source_name = Some(filename.to_string());
    let observer = ParserObserver::new(filename, input);
    let parser = TraceableParser::with_observer(BorfParser, observer.clone());

    // traceable_parse now returns Result<_, Box<PestError<R>>>
    let result = match parser.traceable_parse(Rule::file, input) {
        Ok(mut pairs) => {
            // `Rule::file` contains optional comments/ws then `Rule::module_decl`
            // Find the module_decl pair within the file pair
            let file_pair = pairs.next().ok_or_else(|| {
                Box::new(ParseError::MissingNode {
                    expected: "top-level file rule".to_string(),
                    span: (0, 0).into(),
                    location: filename.to_string(),
                })
            })?;

            let module_decl_pair = file_pair
                .into_inner() // Get children of file rule
                .find(|p| p.as_rule() == Rule::module_decl) // Find the module_decl
                .ok_or_else(|| {
                    Box::new(ParseError::MissingNode {
                        expected: "module declaration (@Name: { ... })".to_string(),
                        span: (0, input.len()).into(), // Span the whole input if not found
                        location: filename.to_string(),
                    })
                })?;

            // Call the original parsing logic with the extracted module_decl pair
            parse_module(module_decl_pair, &source_name)
        }
        Err(boxed_pest_error) => {
            // Convert the boxed pest error to Box<ParseError>
            // Assuming ParseError::from_pest takes PestError<R>, not Box<PestError<R>>
            // We need to unbox it first.
            let pest_error: PestError<Rule> = *boxed_pest_error;
            Err(Box::new(ParseError::from_pest(
                pest_error,
                input,
                source_name,
            )))
        }
    };

    // Log the observer report regardless of success or failure
    tracing::debug!(
        // parse_id = observer.parse_id, // Observer doesn't expose parse_id publicly
        source = filename,
        "Parse report generated:\n{}",
        observer.generate_report()
    );

    // Optionally export parse tree visualization
    if let Some(dot_graph) = observer.export_parse_tree_dot() {
        let dot_path = format!("{}.dot", filename);
        if let Err(e) = std::fs::write(&dot_path, dot_graph) {
            tracing::warn!("Failed to write DOT file: {}", e);
        } else {
            tracing::info!("Wrote parse tree visualization to {}", dot_path);
        }
    }

    result
}

// ... (Keep existing parse_repl_input if needed, potentially wrap it similarly)

pub fn parse_repl_input(input: &str, source_name: Option<String>) -> Result<Expr, Box<ParseError>> {
    // TODO: Integrate tracing here as well if desired, similar to parse_module_with_trace
    // Use match instead of `?`
    match BorfParser::parse(Rule::repl_input, input) {
        Ok(pairs) => {
            // This version assumes repl_input yields an expression directly
            let first_pair = pairs.peek().ok_or_else(|| {
                Box::new(ParseError::MissingNode {
                    expected: "expression or declaration".to_string(),
                    span: (0, 0).into(), // No span available here
                    location: source_name.clone().unwrap_or("REPL".to_string()),
                })
            })?;
            // Assuming the first pair IS the expression, not the wrapper
            // This logic might differ from the lib.rs version
            parse_expr(first_pair.clone(), &source_name)
        }
        Err(pest_error) => {
            // Explicitly call from_pest here to provide full context
            Err(Box::new(ParseError::from_pest(
                pest_error,
                input,       // Pass the input string
                source_name, // Pass the source name
            )))
        }
    }
}

/// Parses REPL input with tracing enabled
pub fn parse_repl_input_with_trace(
    input: &str,
    source_name_str: &str,
) -> Result<ast::ReplInput, Box<ParseError>> {
    let source_name = Some(source_name_str.to_string());
    let observer = crate::observer::ParserObserver::new(source_name_str, input);
    let parser =
        crate::traceable_parser::TraceableParser::with_observer(BorfParser, observer.clone());

    // traceable_parse now returns Result<_, Box<PestError<R>>>
    let result = match parser.traceable_parse(Rule::repl_input, input) {
        Ok(mut pairs) => {
            // pairs should contain repl_input which has one inner element (decl or expr)
            let repl_pair = pairs.next().ok_or_else(|| {
                Box::new(ParseError::MissingNode {
                    expected: "repl_input rule".to_string(),
                    span: (0, 0).into(),
                    location: source_name_str.to_string(),
                })
            })?;

            let repl_inner_pair = repl_pair.into_inner().next().ok_or_else(|| {
                Box::new(ParseError::MissingNode {
                    expected: "declaration or expression".to_string(),
                    span: (0, input.len()).into(),
                    location: source_name_str.to_string(),
                })
            })?;

            let pair_span = repl_inner_pair.as_span();

            match repl_inner_pair.as_rule() {
                Rule::declaration => parse_declaration(repl_inner_pair, &source_name)
                    .map(ast::ReplInput::Declaration),
                Rule::expr => {
                    parse_expr(repl_inner_pair, &source_name).map(ast::ReplInput::Expression)
                }
                rule => Err(Box::new(ParseError::UnexpectedRule {
                    expected: "declaration or expression".to_string(),
                    found: format!("{:?}", rule),
                    span: (pair_span.start(), pair_span.end() - pair_span.start()).into(),
                    location: format!(
                        "{}:{}",
                        pair_span.start_pos().line_col().0,
                        pair_span.start_pos().line_col().1
                    ),
                })),
            }
        }
        Err(boxed_pest_error) => {
            // Convert the boxed pest error to Box<ParseError>
            let pest_error: PestError<Rule> = *boxed_pest_error;
            Err(Box::new(ParseError::from_pest(
                pest_error,
                input,
                source_name,
            )))
        }
    };

    // Log the observer report regardless of success or failure
    tracing::debug!(
        source = source_name_str,
        "Parse report generated:\n{}",
        observer.generate_report()
    );

    // Optionally export parse tree visualization
    if let Some(dot_graph) = observer.export_parse_tree_dot() {
        let dot_path = format!("{}.dot", source_name_str);
        if let Err(e) = std::fs::write(&dot_path, dot_graph) {
            tracing::warn!("Failed to write DOT file: {}", e);
        } else {
            tracing::info!("Wrote parse tree visualization to {}", dot_path);
        }
    }

    result
}

// Add the custom handler for illegal commas
pub fn handle_illegal_comma(pair: &Pair<Rule>, context: &mut ParserContext) {
    let span = get_span_from_pair(pair);
    let start_pos = pair.as_span().start_pos();
    let line_col = start_pos.line_col();
    let location = format!("{}:{}", line_col.0, line_col.1);
    let src = Arc::new(miette::NamedSource::new(
        context
            .source_name
            .clone()
            .unwrap_or_else(|| "<unknown>".to_string()),
        context.input.to_string(),
    ));

    // Create a specific legacy comma error
    let error = ParseError::LegacyComma {
        src,
        span,
        location,
        help_message: "Commas are only valid as separators in collection literals and pattern matching. This appears to be legacy syntax that needs to be updated.".to_string(),
    };

    // Add the error to the diagnostics
    context.diagnostics.add_diagnostic(ParseDiagnostic {
        severity: DiagnosticSeverity::Error,
        message: format!(
            "Legacy Syntax Error: Illegal comma at line {}:{}",
            line_col.0, line_col.1
        ),
        span,
        suggestions: vec![CodeSuggestion {
            description: "Remove the comma and use proper Borf syntax".to_string(),
            replacement: "".to_string(), // Default is to remove the comma
            span,
        }],
    });
}
