pub mod ast;
use ast::*;
use lru::LruCache;
use miette;
use miette::{SourceOffset, SourceSpan};
use once_cell::sync::Lazy;
use pest::error::LineColLocation;
use pest::iterators::{Pair, Pairs};
use pest::pratt_parser::{Assoc, Op, PrattParser};
use pest::Parser;
use pest_derive::Parser;
use rustc_hash::FxHashMap;
use smallvec::SmallVec;
use std::cell::RefCell;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fs;
use std::hash::{Hash, Hasher};
use std::num::NonZeroUsize;
use std::path::Path;

use crate::errors::ParseError;

// Type alias for source location tracking in errors
type Location = String;

// Add SourceLocation for tracking position in source
#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
    pub source_name: Option<String>,
}

impl SourceLocation {
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
fn to_box<T>(v: T) -> Box<T> {
    Box::new(v)
}
fn to_smallvec8<T>(vec: Vec<T>) -> SmallVec8<T> {
    SmallVec8::from_iter(vec)
}
fn to_boxed_smallvec8<T>(vec: Vec<T>) -> SmallVec8<Box<T>> {
    SmallVec8::from_iter(vec.into_iter().map(Box::new))
}
fn to_fxhashmap<K: Hash + Eq, V>(map: HashMap<K, V>) -> FxHashMap<K, V> {
    let mut result = FxHashMap::default();
    result.extend(map);
    result
}
fn to_boxed_fxhashmap<K: Hash + Eq, V>(map: HashMap<K, Box<V>>) -> FxHashMap<K, Box<V>> {
    let mut fxmap = FxHashMap::default();
    for (k, v) in map {
        fxmap.insert(k, v);
    }
    fxmap
}

// More efficient parse cache key for memoization
#[derive(Eq, PartialEq)]
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

// Diagnostics collection
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

impl DiagnosticsCollector {
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
        }
    }

    pub fn add_diagnostic(&mut self, diagnostic: ParseDiagnostic) {
        self.diagnostics.push(diagnostic);
    }

    /// Add a suggestion to fix a parsing issue
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

    /// Suggest fixing a missing semicolon issue
    pub fn suggest_missing_semicolon(&mut self, span: miette::SourceSpan) {
        self.suggest_fix("Add missing semicolon", span, ";");
    }

    /// Suggest fixing a missing parenthesis issue
    pub fn suggest_missing_closing_paren(&mut self, span: miette::SourceSpan) {
        self.suggest_fix("Add missing closing parenthesis", span, ")");
    }

    /// Suggest fixing a missing bracket issue
    pub fn suggest_missing_closing_bracket(&mut self, span: miette::SourceSpan) {
        self.suggest_fix("Add missing closing bracket", span, "]");
    }

    /// Suggest fixing a missing brace issue
    pub fn suggest_missing_closing_brace(&mut self, span: miette::SourceSpan) {
        self.suggest_fix("Add missing closing brace", span, "}");
    }

    /// Suggest a variable name correction based on edit distance to known variables
    pub fn suggest_variable_correction(
        &mut self,
        span: miette::SourceSpan,
        typo: &str,
        correction: &str,
    ) {
        self.suggest_fix(&format!("Did you mean '{}'?", correction), span, correction);
    }

    /// Suggest adding a missing operator
    pub fn suggest_missing_operator(&mut self, span: miette::SourceSpan, likely_op: &str) {
        self.suggest_fix(
            &format!("Add missing operator '{}'", likely_op),
            span,
            likely_op,
        );
    }

    /// Suggest unwrapping a value to fix type mismatch
    pub fn suggest_unwrapping(&mut self, span: miette::SourceSpan, value: &str, wrapper: &str) {
        let unwrapped = match wrapper {
            "Option" => format!("{}.unwrap()", value),
            "Result" => format!("{}.unwrap()", value),
            "Some" => format!("{}.unwrap()", value),
            "Ok" => format!("{}.unwrap()", value),
            _ => format!("{}", value),
        };

        self.suggest_fix(&format!("Unwrap the {} value", wrapper), span, &unwrapped);
    }

    pub fn get_diagnostics(&self) -> &[ParseDiagnostic] {
        &self.diagnostics
    }
}

/// Enhanced parser context with more configuration options
pub struct ParserContext<'a> {
    pub input: &'a str,
    pub source_name: Option<String>,
    pub error_recovery: bool,
    pub diagnostics: DiagnosticsCollector,
    pub config: ParserConfig,
}

/// Configuration options for the parser
#[derive(Debug, Clone)]
pub struct ParserConfig {
    pub max_errors: usize, // Maximum number of errors to collect before stopping
    pub collect_suggestions: bool, // Whether to collect code suggestions
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

    pub fn record_error(&mut self, error: &ParseError, pair: &Pair<Rule>) {
        if self.diagnostics.diagnostics.len() >= self.config.max_errors {
            return;
        }

        let span = get_span_from_pair(pair);
        match error {
            ParseError::SyntaxError { message, .. } => {
                self.diagnostics.add_diagnostic(ParseDiagnostic {
                    severity: DiagnosticSeverity::Error,
                    message: message.clone(),
                    span,
                    suggestions: vec![],
                });
            }
            // Handle other error types...
            _ => {
                self.diagnostics.add_diagnostic(ParseDiagnostic {
                    severity: DiagnosticSeverity::Error,
                    message: format!("{}", error),
                    span,
                    suggestions: vec![],
                });
            }
        }
    }

    pub fn suggest_fix(&mut self, message: &str, span: miette::SourceSpan, replacement: &str) {
        if !self.config.collect_suggestions {
            return;
        }

        self.diagnostics.suggest_fix(message, span, replacement);
    }
}

// Enhanced AST Visitor implementation with generic transformation capability
pub trait ExprVisitor {
    /// Visit an expression
    fn visit_literal(&mut self, lit: &Literal, loc: &Option<SourceLocation>);
    fn visit_variable(&mut self, name: &str, loc: &Option<SourceLocation>);
    fn visit_qualified_name(&mut self, parts: &SmallVec8<String>, loc: &Option<SourceLocation>);
    fn visit_lambda(
        &mut self,
        params: &SmallVec8<Box<Pattern>>,
        body: &Expr,
        loc: &Option<SourceLocation>,
    );
    fn visit_application(
        &mut self,
        func: &Expr,
        args: &SmallVec8<Box<Expr>>,
        loc: &Option<SourceLocation>,
    );
    fn visit_let(
        &mut self,
        pattern: &Pattern,
        value: &Expr,
        body: &Expr,
        loc: &Option<SourceLocation>,
    );
    fn visit_if(
        &mut self,
        cond: &Expr,
        then: &Expr,
        else_expr: &Expr,
        loc: &Option<SourceLocation>,
    );
    fn visit_binary_op(&mut self, op: &str, lhs: &Expr, rhs: &Expr, loc: &Option<SourceLocation>);
    fn visit_unary_op(&mut self, op: &str, expr: &Expr, loc: &Option<SourceLocation>);
    fn visit_list(&mut self, elements: &SmallVec8<Box<Expr>>, loc: &Option<SourceLocation>);
    fn visit_set(&mut self, elements: &SmallVec8<Box<Expr>>, loc: &Option<SourceLocation>);
    fn visit_map(&mut self, entries: &FxHashMap<String, Box<Expr>>, loc: &Option<SourceLocation>);
    fn visit_quote(&mut self, expr: &Expr, loc: &Option<SourceLocation>);
    fn visit_unquote(&mut self, expr: &Expr, loc: &Option<SourceLocation>);
    fn visit_unquote_splice(&mut self, expr: &Expr, loc: &Option<SourceLocation>);
    fn visit_quasiquote(&mut self, expr: &Expr, loc: &Option<SourceLocation>);
}

// Extension to Expr for visitor pattern
impl Expr {
    /// Accept a visitor to traverse this expression
    pub fn accept<V: ExprVisitor>(&self, visitor: &mut V) {
        match self {
            Expr::Literal(lit, loc) => visitor.visit_literal(lit, loc),
            Expr::Variable(name, loc) => visitor.visit_variable(name, loc),
            Expr::QualifiedName(parts, loc) => visitor.visit_qualified_name(parts, loc),
            Expr::Lambda(params, body, loc) => visitor.visit_lambda(params, body, loc),
            Expr::Application(func, args, loc) => visitor.visit_application(func, args, loc),
            Expr::Let(pattern, value, body, loc) => visitor.visit_let(pattern, value, body, loc),
            Expr::If(cond, then, else_expr, loc) => visitor.visit_if(cond, then, else_expr, loc),
            Expr::BinaryOp(op, lhs, rhs, loc) => visitor.visit_binary_op(op, lhs, rhs, loc),
            Expr::UnaryOp(op, expr, loc) => visitor.visit_unary_op(op, expr, loc),
            Expr::List(elements, loc) => visitor.visit_list(elements, loc),
            Expr::Set(elements, loc) => visitor.visit_set(elements, loc),
            Expr::Map(entries, loc) => visitor.visit_map(entries, loc),
            Expr::Quote(expr, loc) => visitor.visit_quote(expr, loc),
            Expr::Unquote(expr, loc) => visitor.visit_unquote(expr, loc),
            Expr::UnquoteSplice(expr, loc) => visitor.visit_unquote_splice(expr, loc),
            Expr::Quasiquote(expr, loc) => visitor.visit_quasiquote(expr, loc),
        }
    }

    /// Transform an expression using a visitor
    pub fn transform<T: ExprTransformer>(&self, transformer: &mut T) -> Expr {
        transformer.transform_expr(self)
    }
}

/// Transformer trait for AST manipulation
pub trait ExprTransformer {
    /// Transform an expression
    fn transform_expr(&mut self, expr: &Expr) -> Expr {
        match expr {
            Expr::Literal(lit, loc) => self.transform_literal(lit, loc),
            Expr::Variable(name, loc) => self.transform_variable(name, loc),
            Expr::QualifiedName(parts, loc) => self.transform_qualified_name(parts, loc),
            Expr::Lambda(params, body, loc) => self.transform_lambda(params, body, loc),
            Expr::Application(func, args, loc) => self.transform_application(func, args, loc),
            Expr::Let(pattern, value, body, loc) => self.transform_let(pattern, value, body, loc),
            Expr::If(cond, then, else_expr, loc) => self.transform_if(cond, then, else_expr, loc),
            Expr::BinaryOp(op, lhs, rhs, loc) => self.transform_binary_op(op, lhs, rhs, loc),
            Expr::UnaryOp(op, expr, loc) => self.transform_unary_op(op, expr, loc),
            Expr::List(elements, loc) => self.transform_list(elements, loc),
            Expr::Set(elements, loc) => self.transform_set(elements, loc),
            Expr::Map(entries, loc) => self.transform_map(entries, loc),
            Expr::Quote(expr, loc) => self.transform_quote(expr, loc),
            Expr::Unquote(expr, loc) => self.transform_unquote(expr, loc),
            Expr::UnquoteSplice(expr, loc) => self.transform_unquote_splice(expr, loc),
            Expr::Quasiquote(expr, loc) => self.transform_quasiquote(expr, loc),
        }
    }

    fn transform_literal(&mut self, lit: &Literal, loc: &Option<SourceLocation>) -> Expr {
        Expr::Literal(lit.clone(), loc.clone())
    }

    fn transform_variable(&mut self, name: &str, loc: &Option<SourceLocation>) -> Expr {
        Expr::Variable(name.to_string(), loc.clone())
    }

    fn transform_qualified_name(
        &mut self,
        parts: &SmallVec8<String>,
        loc: &Option<SourceLocation>,
    ) -> Expr {
        Expr::QualifiedName(parts.clone(), loc.clone())
    }

    fn transform_lambda(
        &mut self,
        params: &SmallVec8<Box<Pattern>>,
        body: &Expr,
        loc: &Option<SourceLocation>,
    ) -> Expr {
        Expr::Lambda(
            params.clone(),
            Box::new(self.transform_expr(body)),
            loc.clone(),
        )
    }

    fn transform_application(
        &mut self,
        func: &Expr,
        args: &SmallVec8<Box<Expr>>,
        loc: &Option<SourceLocation>,
    ) -> Expr {
        let new_func = Box::new(self.transform_expr(func));
        let new_args = args
            .iter()
            .map(|arg| Box::new(self.transform_expr(arg)))
            .collect();
        Expr::Application(new_func, new_args, loc.clone())
    }

    fn transform_let(
        &mut self,
        pattern: &Pattern,
        value: &Expr,
        body: &Expr,
        loc: &Option<SourceLocation>,
    ) -> Expr {
        let new_value = Box::new(self.transform_expr(value));
        let new_body = Box::new(self.transform_expr(body));
        Expr::Let(Box::new(pattern.clone()), new_value, new_body, loc.clone())
    }

    fn transform_if(
        &mut self,
        cond: &Expr,
        then: &Expr,
        else_expr: &Expr,
        loc: &Option<SourceLocation>,
    ) -> Expr {
        let new_cond = Box::new(self.transform_expr(cond));
        let new_then = Box::new(self.transform_expr(then));
        let new_else = Box::new(self.transform_expr(else_expr));
        Expr::If(new_cond, new_then, new_else, loc.clone())
    }

    fn transform_binary_op(
        &mut self,
        op: &str,
        lhs: &Expr,
        rhs: &Expr,
        loc: &Option<SourceLocation>,
    ) -> Expr {
        let new_lhs = Box::new(self.transform_expr(lhs));
        let new_rhs = Box::new(self.transform_expr(rhs));
        Expr::BinaryOp(op.to_string(), new_lhs, new_rhs, loc.clone())
    }

    fn transform_unary_op(&mut self, op: &str, expr: &Expr, loc: &Option<SourceLocation>) -> Expr {
        let new_expr = Box::new(self.transform_expr(expr));
        Expr::UnaryOp(op.to_string(), new_expr, loc.clone())
    }

    fn transform_list(
        &mut self,
        elements: &SmallVec8<Box<Expr>>,
        loc: &Option<SourceLocation>,
    ) -> Expr {
        let new_elements = elements
            .iter()
            .map(|e| Box::new(self.transform_expr(e)))
            .collect();
        Expr::List(new_elements, loc.clone())
    }

    fn transform_set(
        &mut self,
        elements: &SmallVec8<Box<Expr>>,
        loc: &Option<SourceLocation>,
    ) -> Expr {
        let new_elements = elements
            .iter()
            .map(|e| Box::new(self.transform_expr(e)))
            .collect();
        Expr::Set(new_elements, loc.clone())
    }

    fn transform_map(
        &mut self,
        entries: &FxHashMap<String, Box<Expr>>,
        loc: &Option<SourceLocation>,
    ) -> Expr {
        let mut new_entries = FxHashMap::default();
        for (k, v) in entries {
            new_entries.insert(k.clone(), Box::new(self.transform_expr(v)));
        }
        Expr::Map(new_entries, loc.clone())
    }

    fn transform_quote(&mut self, expr: &Expr, loc: &Option<SourceLocation>) -> Expr {
        Expr::Quote(Box::new(self.transform_expr(expr)), loc.clone())
    }

    fn transform_unquote(&mut self, expr: &Expr, loc: &Option<SourceLocation>) -> Expr {
        Expr::Unquote(Box::new(self.transform_expr(expr)), loc.clone())
    }

    fn transform_unquote_splice(&mut self, expr: &Expr, loc: &Option<SourceLocation>) -> Expr {
        Expr::UnquoteSplice(Box::new(self.transform_expr(expr)), loc.clone())
    }

    fn transform_quasiquote(&mut self, expr: &Expr, loc: &Option<SourceLocation>) -> Expr {
        Expr::Quasiquote(Box::new(self.transform_expr(expr)), loc.clone())
    }
}

// Enhanced Pratt parser builder with context
fn create_pratt_parser<'a>(_context: &'a ParserContext) -> PrattParser<Rule> {
    PrattParser::new()
        // Precedence: Higher levels bind tighter.
        // Start with lowest precedence (level 0)
        // Level 0: Type Annotation (:)
        .op(Op::infix(Rule::op_colon, Assoc::Left))
        // Level 1: Logical OR (or)
        .op(Op::infix(Rule::op_or, Assoc::Left))
        // Level 2: Logical AND (and)
        .op(Op::infix(Rule::op_and, Assoc::Left))
        // Level 3: Comparison (==, !=, <, >, <=, >=)
        .op(Op::infix(Rule::op_eq, Assoc::Left)
            | Op::infix(Rule::op_neq, Assoc::Left)
            | Op::infix(Rule::op_lt, Assoc::Left)
            | Op::infix(Rule::op_gt, Assoc::Left)
            | Op::infix(Rule::op_lte, Assoc::Left)
            | Op::infix(Rule::op_gte, Assoc::Left))
        // Level 4: Additive (+, -)
        .op(Op::infix(Rule::op_add, Assoc::Left) | Op::infix(Rule::op_sub, Assoc::Left))
        // Level 5: Multiplicative (*, /)
        .op(Op::infix(Rule::op_mul, Assoc::Left) | Op::infix(Rule::op_div, Assoc::Left))
        // Level 6: Unary Prefix (!, -, ~)
        .op(Op::prefix(Rule::op_not) | Op::prefix(Rule::op_neg) | Op::prefix(Rule::op_unquote))
        // Level 7: Pipe (|>)
        .op(Op::infix(Rule::op_pipe, Assoc::Right))
}

/// Parse expression with enhanced error handling
fn parse_expr_with_context(
    pair: Pair<Rule>,
    context: &mut ParserContext,
) -> Result<Expr, ParseError> {
    match pair.as_rule() {
        Rule::expr => {
            // We have an expression, delegate to the Pratt parser
            let pairs = pair.into_inner();
            parse_pratt_expr_with_context(pairs, context)
        }
        _ => {
            // Attempt to handle directly
            parse_primary_expr_with_context(pair, context)
        }
    }
}

// Helper to determine if an expression is complex enough to warrant memoization
fn is_complex_expr(pair: &Pair<Rule>) -> bool {
    match pair.as_rule() {
        Rule::lambda | Rule::application | Rule::let_expr | Rule::if_expr => true,
        Rule::expr => {
            // Check inner expressions
            for inner in pair.clone().into_inner() {
                if is_complex_expr(&inner) {
                    return true;
                }
            }
            false
        }
        _ => false,
    }
}

/// Pratt parser logic with context
fn parse_pratt_expr_with_context(
    pairs: Pairs<Rule>,
    context: &mut ParserContext,
) -> Result<Expr, ParseError> {
    let pratt_parser = create_pratt_parser(context);

    let result = pratt_parser
        .map_primary(|primary| match primary.as_rule() {
            Rule::primary => parse_primary_expr_with_context(primary.clone(), context),
            Rule::identifier => Ok(Expr::Variable(
                primary.as_str().to_string(),
                loc(&primary, &context.source_name),
            )),
            // ... other rules
            _ => {
                if context.error_recovery {
                    // Record error but return a placeholder for recovery
                    let diag = ParseDiagnostic {
                        severity: DiagnosticSeverity::Error,
                        message: format!("Unexpected token: {:?}", primary.as_rule()),
                        span: get_span_from_pair(&primary),
                        suggestions: vec![],
                    };
                    context.diagnostics.add_diagnostic(diag);
                    Ok(Expr::Variable(
                        "_error_".to_string(),
                        loc(&primary, &context.source_name),
                    ))
                } else {
                    Err(ParseError::UnexpectedToken {
                        expected: "expression".to_string(),
                        found: format!("{:?}", primary.as_rule()),
                        src: miette::NamedSource::new(
                            context
                                .source_name
                                .clone()
                                .unwrap_or_else(|| "<unknown>".to_string()),
                            context.input.to_string(),
                        ),
                        span: get_span_from_pair(&primary),
                        location: get_location_from_pair(&primary),
                        help_message: "Expected a valid expression here".to_string(),
                        suggestion: None,
                    })
                }
            }
        })
        .map_prefix(|op, rhs| {
            let op_str = op.as_str();
            match rhs {
                Ok(rhs_expr) => Ok(Expr::UnaryOp(
                    op_str.to_string(),
                    Box::new(rhs_expr),
                    loc(&op, &context.source_name),
                )),
                Err(e) if context.error_recovery => {
                    // Use placeholder for right side
                    let placeholder =
                        Expr::Variable("_error_".to_string(), loc(&op, &context.source_name));
                    let diag = ParseDiagnostic {
                        severity: DiagnosticSeverity::Error,
                        message: format!("Invalid expression after {}", op_str),
                        span: get_span_from_pair(&op),
                        suggestions: vec![],
                    };
                    context.diagnostics.add_diagnostic(diag);
                    Ok(Expr::UnaryOp(
                        op_str.to_string(),
                        Box::new(placeholder),
                        loc(&op, &context.source_name),
                    ))
                }
                Err(e) => Err(e),
            }
        })
        .map_infix(|lhs, op, rhs| {
            let op_str = op.as_str();

            // Handle binary operators
            match (lhs, rhs) {
                (Ok(lhs_expr), Ok(rhs_expr)) => {
                    // Both sides are good
                    Ok(Expr::BinaryOp(
                        op_str.to_string(),
                        Box::new(lhs_expr),
                        Box::new(rhs_expr),
                        loc(&op, &context.source_name),
                    ))
                }
                (lhs_result, rhs_result) if context.error_recovery => {
                    // At least one side has an error and we're in recovery mode
                    let loc_op = loc(&op, &context.source_name);
                    let placeholder = Expr::Variable("_error_".to_string(), loc_op.clone());

                    // We'll create appropriate diagnostics and placeholders
                    match (lhs_result, rhs_result) {
                        (Err(_), Err(_)) => {
                            // Both sides failed
                            let diag = ParseDiagnostic {
                                severity: DiagnosticSeverity::Error,
                                message: format!("Invalid expressions around operator {}", op_str),
                                span: get_span_from_pair(&op),
                                suggestions: vec![],
                            };
                            context.diagnostics.add_diagnostic(diag);
                            Ok(placeholder)
                        }
                        (Err(_), Ok(rhs_expr)) => {
                            // Left side failed
                            let diag = ParseDiagnostic {
                                severity: DiagnosticSeverity::Error,
                                message: format!("Invalid expression before operator {}", op_str),
                                span: get_span_from_pair(&op),
                                suggestions: vec![],
                            };
                            context.diagnostics.add_diagnostic(diag);

                            Ok(Expr::BinaryOp(
                                op_str.to_string(),
                                Box::new(placeholder),
                                Box::new(rhs_expr),
                                loc_op,
                            ))
                        }
                        (Ok(lhs_expr), Err(_)) => {
                            // Right side failed
                            let diag = ParseDiagnostic {
                                severity: DiagnosticSeverity::Error,
                                message: format!("Invalid expression after operator {}", op_str),
                                span: get_span_from_pair(&op),
                                suggestions: vec![],
                            };
                            context.diagnostics.add_diagnostic(diag);

                            Ok(Expr::BinaryOp(
                                op_str.to_string(),
                                Box::new(lhs_expr),
                                Box::new(placeholder),
                                loc_op,
                            ))
                        }
                        _ => unreachable!(), // This case is already covered by the first pattern
                    }
                }
                (Err(e), _) => Err(e),
                (_, Err(e)) => Err(e),
            }
        })
        .parse(pairs);

    match result {
        Ok(expr) => Ok(expr),
        Err(err) => {
            if context.error_recovery {
                // Parse failed but we're in recovery mode, return a placeholder
                let err_str = err.to_string();
                let diag = ParseDiagnostic {
                    severity: DiagnosticSeverity::Error,
                    message: format!("Parse error: {}", err_str),
                    span: miette::SourceSpan::new(
                        miette::SourceOffset::from(0),
                        miette::SourceOffset::from(0),
                    ), // Placeholder span
                    suggestions: vec![],
                };
                context.diagnostics.add_diagnostic(diag);
                Ok(Expr::Variable("_error_".to_string(), None))
            } else {
                Err(ParseError::SyntaxError {
                    message: format!("Failed to parse expression: {}", err),
                    src: miette::NamedSource::new(
                        context
                            .source_name
                            .clone()
                            .unwrap_or_else(|| "<unknown>".to_string()),
                        context.input.to_string(),
                    ),
                    span: miette::SourceSpan::new(
                        miette::SourceOffset::from(0),
                        miette::SourceOffset::from(0),
                    ), // Better span needed
                    location: "<unknown>".to_string(),
                    help_message: "Failed to parse expression".to_string(),
                    suggestion: None,
                })
            }
        }
    }
}

fn parse_primary_expr_with_context(
    pair: Pair<Rule>,
    context: &mut ParserContext,
) -> Result<Expr, ParseError> {
    let pair_clone = pair.clone();
    let result = match pair.as_rule() {
        // Handle different expression types...
        // ... existing code
        _ => {
            if context.error_recovery {
                // Record error but return a placeholder for recovery
                let diag = ParseDiagnostic {
                    severity: DiagnosticSeverity::Error,
                    message: format!("Unexpected expression type: {:?}", pair_clone.as_rule()),
                    span: get_span_from_pair(&pair_clone),
                    suggestions: vec![],
                };
                context.diagnostics.add_diagnostic(diag);
                Ok(Expr::Variable(
                    "_error_".to_string(),
                    loc(&pair_clone, &context.source_name),
                ))
            } else {
                Err(ParseError::UnexpectedToken {
                    expected: "primary expression".to_string(),
                    found: format!("{:?}", pair_clone.as_rule()),
                    src: miette::NamedSource::new(
                        context
                            .source_name
                            .clone()
                            .unwrap_or_else(|| "<unknown>".to_string()),
                        context.input.to_string(),
                    ),
                    span: get_span_from_pair(&pair_clone),
                    location: get_location_from_pair(&pair_clone),
                    help_message: "Expected a valid expression here".to_string(),
                    suggestion: None,
                })
            }
        }
    };

    result
}

/// Parse a literal value
pub fn parse_literal(pair: Pair<Rule>, source_name: &Option<String>) -> Result<Expr, ParseError> {
    let pair_clone = pair.clone();
    let inner = pair
        .into_inner()
        .next()
        .ok_or_else(|| ParseError::MissingToken {
            expected: "literal value".to_string(),
            src: miette::NamedSource::new(
                source_name
                    .clone()
                    .unwrap_or_else(|| "<unknown>".to_string()),
                "".to_string(),
            ),
            span: get_span_from_pair(&pair_clone),
            location: get_location_from_pair(&pair_clone),
            help_message: "Expected a literal value inside the literal rule".to_string(),
            suggestion: None,
        })?;

    let location = loc(&pair_clone, source_name);

    match inner.as_rule() {
        Rule::primitive_literal => {
            let lit = parse_primitive_literal_to_lit(inner)?;
            Ok(Expr::Literal(lit, location))
        }
        Rule::collection_literal => parse_collection_literal(inner, source_name),
        _ => Err(ParseError::SyntaxError {
            message: format!("Invalid literal type: {:?}", inner.as_rule()),
            src: miette::NamedSource::new(
                source_name
                    .clone()
                    .unwrap_or_else(|| "<unknown>".to_string()),
                "".to_string(),
            ),
            span: get_span_from_pair(&inner),
            location: get_location_from_pair(&inner),
            help_message: "Expected a primitive or collection literal".to_string(),
            suggestion: None,
        }),
    }
}

// Renamed to clarify it returns Literal, not Expr
fn parse_primitive_literal_to_lit(pair: Pair<Rule>) -> Result<Literal, ParseError> {
    let pair_clone = pair.clone();
    let inner = pair
        .into_inner()
        .next()
        .ok_or_else(|| ParseError::MissingToken {
            expected: "primitive literal value".to_string(),
            src: miette::NamedSource::new("<unknown>", "".to_string()),
            span: get_span_from_pair(&pair_clone),
            location: get_location_from_pair(&pair_clone),
            help_message: "Expected a primitive literal like number, string, boolean".to_string(),
            suggestion: None,
        })?;

    match inner.as_rule() {
        Rule::integer => {
            // Parse integer literals
            match inner.as_str().parse::<i64>() {
                Ok(value) => Ok(Literal::Integer(value)),
                Err(_) => Err(ParseError::InvalidLiteral {
                    value: inner.as_str().to_string(),
                    reason: "integer value out of range".to_string(),
                    span: get_span_from_pair(&inner),
                    location: get_location_from_pair(&inner),
                }),
            }
        }
        Rule::float => {
            // Parse float literals
            match inner.as_str().parse::<f64>() {
                Ok(value) => Ok(Literal::Float(value)),
                Err(_) => Err(ParseError::InvalidLiteral {
                    value: inner.as_str().to_string(),
                    reason: "invalid float value".to_string(),
                    span: get_span_from_pair(&inner),
                    location: get_location_from_pair(&inner),
                }),
            }
        }
        Rule::string => {
            // Parse string literals (removing quotes)
            let text = inner.as_str();
            // Strip quotes (first and last character)
            let content = &text[1..text.len() - 1];
            Ok(Literal::String(content.to_string()))
        }
        Rule::boolean => {
            // Parse boolean literals
            let text = inner.as_str();
            match text {
                "true" => Ok(Literal::Boolean(true)),
                "false" => Ok(Literal::Boolean(false)),
                _ => Err(ParseError::InvalidLiteral {
                    value: text.to_string(),
                    reason: "expected 'true' or 'false'".to_string(),
                    span: get_span_from_pair(&inner),
                    location: get_location_from_pair(&inner),
                }),
            }
        }
        _ => Err(ParseError::SyntaxError {
            message: format!("Invalid primitive literal: {:?}", inner.as_rule()),
            src: miette::NamedSource::new("<unknown>", "".to_string()),
            span: get_span_from_pair(&inner),
            location: get_location_from_pair(&inner),
            help_message: "Expected a number, string, or boolean".to_string(),
            suggestion: None,
        }),
    }
}

// Returns Expr directly (Expr::List, Expr::Map, Expr::Set)
fn parse_collection_literal(
    pair: Pair<Rule>,
    source_name: &Option<String>,
) -> Result<Expr, ParseError> {
    let pair_clone = pair.clone();
    let inner = pair
        .into_inner()
        .next()
        .ok_or_else(|| ParseError::MissingToken {
            expected: "collection literal content".to_string(),
            src: miette::NamedSource::new(
                source_name
                    .clone()
                    .unwrap_or_else(|| "<unknown>".to_string()),
                "".to_string(),
            ),
            span: get_span_from_pair(&pair_clone),
            location: get_location_from_pair(&pair_clone),
            help_message: "Expected list, map, or set literal".to_string(),
            suggestion: None,
        })?;

    match inner.as_rule() {
        Rule::list_literal => parse_list_literal(inner, source_name),
        Rule::map_literal => parse_map_literal(inner, source_name),
        Rule::set_literal => parse_set_literal(inner, source_name),
        _ => Err(ParseError::SyntaxError {
            message: format!("Invalid collection literal: {:?}", inner.as_rule()),
            src: miette::NamedSource::new(
                source_name
                    .clone()
                    .unwrap_or_else(|| "<unknown>".to_string()),
                "".to_string(),
            ),
            span: get_span_from_pair(&inner),
            location: get_location_from_pair(&inner),
            help_message: "Expected list, map, or set literal syntax".to_string(),
            suggestion: None,
        }),
    }
}

// Returns Expr::List
fn parse_list_literal(pair: Pair<Rule>, source_name: &Option<String>) -> Result<Expr, ParseError> {
    let location = loc(&pair, source_name);
    let mut elements = Vec::new();

    for element_pair in pair.into_inner() {
        elements.push(Box::new(parse_expr(element_pair, source_name)?));
    }

    Ok(Expr::List(to_smallvec8(elements), location))
}

// Returns Expr::Map
fn parse_map_literal(pair: Pair<Rule>, source_name: &Option<String>) -> Result<Expr, ParseError> {
    let location = loc(&pair, source_name);
    let mut map = HashMap::new();

    for entry_pair in pair.into_inner() {
        let (key, value) = parse_map_entry(entry_pair, source_name)?;
        map.insert(key, Box::new(value));
    }

    Ok(Expr::Map(to_fxhashmap(map), location))
}

// Returns (String, Expr) for map entries
fn parse_map_entry(
    pair: Pair<Rule>,
    source_name: &Option<String>,
) -> Result<(String, Expr), ParseError> {
    let pair_clone = pair.clone();
    let mut inner = pair.into_inner();
    let key_pair = inner.next().ok_or_else(|| ParseError::MissingToken {
        expected: "map key".to_string(),
        src: miette::NamedSource::new(
            source_name
                .clone()
                .unwrap_or_else(|| "<unknown>".to_string()),
            "".to_string(),
        ),
        span: get_span_from_pair(&pair_clone),
        location: get_location_from_pair(&pair_clone),
        help_message: "Map entries must have a key".to_string(),
        suggestion: None,
    })?;

    let value_pair = inner.next().ok_or_else(|| ParseError::MissingToken {
        expected: "map value".to_string(),
        src: miette::NamedSource::new(
            source_name
                .clone()
                .unwrap_or_else(|| "<unknown>".to_string()),
            "".to_string(),
        ),
        span: get_span_from_pair(&pair_clone),
        location: get_location_from_pair(&pair_clone),
        help_message: "Map entries must have a value".to_string(),
        suggestion: None,
    })?;

    // Key must be a string literal or identifier
    let key = match key_pair.as_rule() {
        Rule::string => {
            let text = key_pair.as_str();
            // Strip quotes (first and last character)
            text[1..text.len() - 1].to_string()
        }
        Rule::identifier => key_pair.as_str().to_string(),
        _ => {
            let key_pair_clone = key_pair.clone();
            let key_expr = parse_expr(key_pair, source_name)?;
            return Err(ParseError::InvalidMapKey {
                found_type: format!("{:?}", key_expr),
                span: get_span_from_pair(&key_pair_clone),
                location: get_location_from_pair(&key_pair_clone),
                help_message: Some(
                    "Only string literals or identifiers can be used as map keys".to_string(),
                ),
            });
        }
    };

    let value = parse_expr(value_pair, source_name)?;
    Ok((key, value))
}

// Returns Expr::Set
fn parse_set_literal(pair: Pair<Rule>, source_name: &Option<String>) -> Result<Expr, ParseError> {
    let location = loc(&pair, source_name);
    let mut elements = Vec::new();

    for element_pair in pair.into_inner() {
        elements.push(Box::new(parse_expr(element_pair, source_name)?));
    }

    Ok(Expr::Set(to_smallvec8(elements), location))
}

/// Parse a lambda expression
fn parse_lambda(pair: Pair<Rule>, source_name: &Option<String>) -> Result<Expr, ParseError> {
    let pair_clone = pair.clone();
    let location = loc(&pair_clone, source_name);
    let mut inner_pairs = pair.into_inner();

    let params_pair = inner_pairs.next().ok_or_else(|| ParseError::MissingToken {
        expected: "lambda parameters".to_string(),
        src: miette::NamedSource::new(
            source_name
                .clone()
                .unwrap_or_else(|| "<unknown>".to_string()),
            "".to_string(),
        ),
        span: get_span_from_pair(&pair_clone),
        location: get_location_from_pair(&pair_clone),
        help_message: "Lambda expression must have parameters (can be empty)".to_string(),
        suggestion: None,
    })?;

    // Parse lambda parameters (patterns)
    let patterns = parse_params(params_pair, source_name)?;
    let params: SmallVec8<Box<Pattern>> = patterns.into_iter().map(Box::new).collect();

    let body_pair = inner_pairs.next().ok_or_else(|| ParseError::MissingToken {
        expected: "lambda body".to_string(),
        src: miette::NamedSource::new(
            source_name
                .clone()
                .unwrap_or_else(|| "<unknown>".to_string()),
            "".to_string(),
        ),
        span: get_span_from_pair(&pair_clone),
        location: get_location_from_pair(&pair_clone),
        help_message: "Lambda must have a body expression".to_string(),
        suggestion: None,
    })?;

    let body = parse_expr(body_pair, source_name)?;

    Ok(Expr::Lambda(params, Box::new(body), location))
}

/// Parse function parameters
fn parse_params(
    pair: Pair<Rule>,
    source_name: &Option<String>,
) -> Result<Vec<Pattern>, ParseError> {
    let mut params = Vec::new();
    for pattern_pair in pair.into_inner() {
        params.push(parse_pattern(pattern_pair, source_name)?);
    }
    Ok(params)
}

/// Parse a function application
fn parse_application(pair: Pair<Rule>, source_name: &Option<String>) -> Result<Expr, ParseError> {
    let pair_clone = pair.clone();
    let location = loc(&pair_clone, source_name);
    let mut inner = pair.into_inner();

    let function_pair = inner.next().ok_or_else(|| ParseError::MissingToken {
        expected: "function expression".to_string(),
        src: miette::NamedSource::new(
            source_name
                .clone()
                .unwrap_or_else(|| "<unknown>".to_string()),
            "".to_string(),
        ),
        span: get_span_from_pair(&pair_clone),
        location: get_location_from_pair(&pair_clone),
        help_message: "Function application must have a function".to_string(),
        suggestion: None,
    })?;

    let function = parse_expr(function_pair, source_name)?;

    let mut args = vec![];
    for arg_pair in inner {
        let arg = parse_expr(arg_pair, source_name)?;
        args.push(Box::new(arg));
    }

    Ok(Expr::Application(Box::new(function), args.into(), location))
}

/// Parse a let expression
fn parse_let_expr(pair: Pair<Rule>, source_name: &Option<String>) -> Result<Expr, ParseError> {
    let pair_clone = pair.clone();
    let location = loc(&pair_clone, source_name);
    let mut inner = pair.into_inner();

    let binding_pair = inner.next().ok_or_else(|| ParseError::MissingToken {
        expected: "binding expression".to_string(),
        src: miette::NamedSource::new(
            source_name
                .clone()
                .unwrap_or_else(|| "<unknown>".to_string()),
            "".to_string(),
        ),
        span: get_span_from_pair(&pair_clone),
        location: get_location_from_pair(&pair_clone),
        help_message: "Let expression must have a binding".to_string(),
        suggestion: None,
    })?;

    let (pattern, value) = parse_binding(binding_pair, source_name)?;

    // "in" part is optional
    let body = if let Some(body_pair) = inner.next() {
        parse_expr(body_pair, source_name)?
    } else {
        // If there's no body, use a unit literal as a placeholder
        Expr::Literal(Literal::String("".to_string()), None)
    };

    Ok(Expr::Let(
        Box::new(pattern),
        Box::new(value),
        Box::new(body),
        location,
    ))
}

/// Parse a binding (pattern = expr)
fn parse_binding(
    pair: Pair<Rule>,
    source_name: &Option<String>,
) -> Result<(Pattern, Expr), ParseError> {
    let pair_clone = pair.clone();
    let mut inner = pair.into_inner();
    let pattern_pair = inner.next().ok_or_else(|| ParseError::MissingToken {
        expected: "pattern".to_string(),
        src: miette::NamedSource::new(
            source_name
                .clone()
                .unwrap_or_else(|| "<unknown>".to_string()),
            "".to_string(),
        ),
        span: get_span_from_pair(&pair_clone),
        location: get_location_from_pair(&pair_clone),
        help_message: "Binding requires a pattern on the left side".to_string(),
        suggestion: None,
    })?;

    let value_pair = inner.next().ok_or_else(|| ParseError::MissingToken {
        expected: "binding value".to_string(),
        src: miette::NamedSource::new(
            source_name
                .clone()
                .unwrap_or_else(|| "<unknown>".to_string()),
            "".to_string(),
        ),
        span: get_span_from_pair(&pair_clone),
        location: get_location_from_pair(&pair_clone),
        help_message: "Binding needs a value expression on the right side".to_string(),
        suggestion: None,
    })?;

    let pattern = parse_pattern(pattern_pair, source_name)?;
    let value = parse_expr(value_pair, source_name)?;

    Ok((pattern, value))
}

/// Parse an if expression
fn parse_if_expr(pair: Pair<Rule>, source_name: &Option<String>) -> Result<Expr, ParseError> {
    let pair_clone = pair.clone();
    let location = loc(&pair_clone, source_name);
    let mut inner = pair.into_inner();

    let condition_pair = inner.next().ok_or_else(|| ParseError::MissingToken {
        expected: "condition expression".to_string(),
        src: miette::NamedSource::new(
            source_name
                .clone()
                .unwrap_or_else(|| "<unknown>".to_string()),
            "".to_string(),
        ),
        span: get_span_from_pair(&pair_clone),
        location: get_location_from_pair(&pair_clone),
        help_message: "If expressions need a condition".to_string(),
        suggestion: None,
    })?;

    let then_pair = inner.next().ok_or_else(|| ParseError::MissingToken {
        expected: "then branch".to_string(),
        src: miette::NamedSource::new(
            source_name
                .clone()
                .unwrap_or_else(|| "<unknown>".to_string()),
            "".to_string(),
        ),
        span: get_span_from_pair(&pair_clone),
        location: get_location_from_pair(&pair_clone),
        help_message: "If expressions need a 'then' branch".to_string(),
        suggestion: None,
    })?;

    let else_pair = inner.next().ok_or_else(|| ParseError::MissingToken {
        expected: "else branch".to_string(),
        src: miette::NamedSource::new(
            source_name
                .clone()
                .unwrap_or_else(|| "<unknown>".to_string()),
            "".to_string(),
        ),
        span: get_span_from_pair(&pair_clone),
        location: get_location_from_pair(&pair_clone),
        help_message: "If expressions need an 'else' branch".to_string(),
        suggestion: None,
    })?;

    let condition = parse_expr(condition_pair, source_name)?;
    let then_branch = parse_expr(then_pair, source_name)?;
    let else_branch = parse_expr(else_pair, source_name)?;

    Ok(Expr::If(
        Box::new(condition),
        Box::new(then_branch),
        Box::new(else_branch),
        location,
    ))
}

/// Parse quoting expressions (quote, unquote, unquote-splice, quasiquote)
fn parse_quoting_expr(pair: Pair<Rule>, source_name: &Option<String>) -> Result<Expr, ParseError> {
    let pair_clone = pair.clone();
    let inner = pair
        .into_inner()
        .next()
        .ok_or_else(|| ParseError::MissingToken {
            expected: "quoting expression".to_string(),
            src: miette::NamedSource::new(
                source_name
                    .clone()
                    .unwrap_or_else(|| "<unknown>".to_string()),
                "".to_string(),
            ),
            span: get_span_from_pair(&pair_clone),
            location: get_location_from_pair(&pair_clone),
            help_message: "Expected a quoting expression".to_string(),
            suggestion: None,
        })?;

    let location = loc(&inner, source_name);

    match inner.as_rule() {
        Rule::quote_expr => {
            // Handle 'expr
            let inner_pair_clone = inner.clone();
            let expr_pair = inner
                .into_inner()
                .next()
                .ok_or_else(|| ParseError::MissingToken {
                    expected: "quoted expression".to_string(),
                    src: miette::NamedSource::new(
                        source_name
                            .clone()
                            .unwrap_or_else(|| "<unknown>".to_string()),
                        "".to_string(),
                    ),
                    span: get_span_from_pair(&inner_pair_clone),
                    location: get_location_from_pair(&inner_pair_clone),
                    help_message: "Quote requires an expression to quote".to_string(),
                    suggestion: None,
                })?;
            let expr = parse_expr(expr_pair, source_name)?;
            Ok(Expr::Quote(Box::new(expr), location))
        }
        Rule::unquote_expr => {
            // Handle ~expr
            let inner_pair_clone = inner.clone();
            let expr_pair = inner
                .into_inner()
                .next()
                .ok_or_else(|| ParseError::MissingToken {
                    expected: "unquoted expression".to_string(),
                    src: miette::NamedSource::new(
                        source_name
                            .clone()
                            .unwrap_or_else(|| "<unknown>".to_string()),
                        "".to_string(),
                    ),
                    span: get_span_from_pair(&inner_pair_clone),
                    location: get_location_from_pair(&inner_pair_clone),
                    help_message: "Unquote requires an expression to unquote".to_string(),
                    suggestion: None,
                })?;
            let expr = parse_expr(expr_pair, source_name)?;
            Ok(Expr::Unquote(Box::new(expr), location))
        }
        Rule::unquote_splice_expr => {
            // Handle ~@expr
            let inner_pair_clone = inner.clone();
            let expr_pair = inner
                .into_inner()
                .next()
                .ok_or_else(|| ParseError::MissingToken {
                    expected: "unquote-splice expression".to_string(),
                    src: miette::NamedSource::new(
                        source_name
                            .clone()
                            .unwrap_or_else(|| "<unknown>".to_string()),
                        "".to_string(),
                    ),
                    span: get_span_from_pair(&inner_pair_clone),
                    location: get_location_from_pair(&inner_pair_clone),
                    help_message: "Unquote-splice requires an expression to splice".to_string(),
                    suggestion: None,
                })?;
            let expr = parse_expr(expr_pair, source_name)?;
            Ok(Expr::UnquoteSplice(Box::new(expr), location))
        }
        Rule::quasiquote_expr => {
            // Handle `expr
            let inner_pair_clone = inner.clone();
            let expr_pair = inner
                .into_inner()
                .next()
                .ok_or_else(|| ParseError::MissingToken {
                    expected: "quasiquoted expression".to_string(),
                    src: miette::NamedSource::new(
                        source_name
                            .clone()
                            .unwrap_or_else(|| "<unknown>".to_string()),
                        "".to_string(),
                    ),
                    span: get_span_from_pair(&inner_pair_clone),
                    location: get_location_from_pair(&inner_pair_clone),
                    help_message: "Quasiquote requires an expression".to_string(),
                    suggestion: None,
                })?;
            let expr = parse_expr(expr_pair, source_name)?;
            Ok(Expr::Quasiquote(Box::new(expr), location))
        }
        _ => Err(ParseError::SyntaxError {
            message: format!("Unexpected quoting rule: {:?}", inner.as_rule()),
            src: miette::NamedSource::new(
                source_name
                    .clone()
                    .unwrap_or_else(|| "<unknown>".to_string()),
                "".to_string(),
            ),
            span: get_span_from_pair(&inner),
            location: get_location_from_pair(&inner),
            help_message: "Expected a quote, unquote, unquote-splice, or quasiquote".to_string(),
            suggestion: None,
        }),
    }
}

/// Parse a pattern (used in bindings and function parameters)
fn parse_pattern(pair: Pair<Rule>, source_name: &Option<String>) -> Result<Pattern, ParseError> {
    match pair.as_rule() {
        Rule::pattern => {
            // Pattern is a wrapper, get the inner pattern
            let pair_clone = pair.clone();
            let inner = pair
                .into_inner()
                .next()
                .ok_or_else(|| ParseError::MissingToken {
                    expected: "pattern expression".to_string(),
                    src: miette::NamedSource::new(
                        source_name
                            .clone()
                            .unwrap_or_else(|| "<unknown>".to_string()),
                        "".to_string(),
                    ),
                    span: get_span_from_pair(&pair_clone),
                    location: get_location_from_pair(&pair_clone),
                    help_message: "Expected a pattern expression".to_string(),
                    suggestion: None,
                })?;

            // Parse the inner pattern
            parse_pattern(inner, source_name)
        }
        Rule::identifier => {
            // Variable pattern
            let location = loc(&pair, source_name);
            let ident = pair.as_str();

            // Skip "_" as a special wildcard pattern
            if ident == "_" {
                Ok(Pattern::Wildcard(location))
            } else {
                Ok(Pattern::Variable(ident.to_string(), location))
            }
        }
        Rule::list_pattern => {
            // List pattern [pat1, pat2, ...]
            let location = loc(&pair, source_name);
            let mut patterns = vec![];

            for pair_item in pair.into_inner() {
                let pattern = parse_pattern(pair_item, source_name)?;
                patterns.push(Box::new(pattern));
            }

            Ok(Pattern::List(patterns.into(), location))
        }
        Rule::map_pattern => {
            // Map pattern {key1: pat1, key2: pat2, ...}
            let location = loc(&pair, source_name);
            let mut entries = FxHashMap::default();

            for pair_item in pair.into_inner() {
                if pair_item.as_rule() == Rule::map_pattern_entry {
                    // For each entry, extract key and pattern
                    let pair_item_clone = pair_item.clone();
                    let mut kv_pair = pair_item.into_inner();
                    let key_pair = kv_pair.next().ok_or_else(|| ParseError::MissingToken {
                        expected: "pattern map key".to_string(),
                        src: miette::NamedSource::new(
                            source_name
                                .clone()
                                .unwrap_or_else(|| "<unknown>".to_string()),
                            "".to_string(),
                        ),
                        span: get_span_from_pair(&pair_item_clone),
                        location: get_location_from_pair(&pair_item_clone),
                        help_message: "Map pattern entries must have a key".to_string(),
                        suggestion: None,
                    })?;

                    let value_pair = kv_pair.next().ok_or_else(|| ParseError::MissingToken {
                        expected: "pattern map value".to_string(),
                        src: miette::NamedSource::new(
                            source_name
                                .clone()
                                .unwrap_or_else(|| "<unknown>".to_string()),
                            "".to_string(),
                        ),
                        span: get_span_from_pair(&pair_item_clone),
                        location: get_location_from_pair(&pair_item_clone),
                        help_message: "Map pattern entries must have a value".to_string(),
                        suggestion: None,
                    })?;

                    // Key must be a simple string or identifier
                    let key = match key_pair.as_rule() {
                        Rule::string => {
                            let text = key_pair.as_str();
                            text[1..text.len() - 1].to_string() // Remove quotes
                        }
                        Rule::identifier => key_pair.as_str().to_string(),
                        _ => {
                            return Err(ParseError::InvalidMapPatternKey {
                                found_type: format!("{:?}", key_pair.as_rule()),
                                span: get_span_from_pair(&key_pair),
                                location: get_location_from_pair(&key_pair),
                            });
                        }
                    };

                    let value_pattern = parse_pattern(value_pair, source_name)?;
                    entries.insert(key, Box::new(value_pattern));
                }
            }

            Ok(Pattern::Map(entries, location))
        }
        Rule::set_pattern => {
            // Set pattern {pat1, pat2, ...}
            let location = loc(&pair, source_name);
            let mut patterns = vec![];

            for pair_item in pair.into_inner() {
                let pattern = parse_pattern(pair_item, source_name)?;
                patterns.push(Box::new(pattern));
            }

            Ok(Pattern::Set(patterns.into(), location))
        }
        Rule::literal => {
            // Literal pattern (exact match)
            let pair_clone = pair.clone();
            let inner = pair
                .into_inner()
                .next()
                .ok_or_else(|| ParseError::MissingToken {
                    expected: "literal value".to_string(),
                    src: miette::NamedSource::new(
                        source_name
                            .clone()
                            .unwrap_or_else(|| "<unknown>".to_string()),
                        "".to_string(),
                    ),
                    span: get_span_from_pair(&pair_clone),
                    location: get_location_from_pair(&pair_clone),
                    help_message: "Expected a literal value".to_string(),
                    suggestion: None,
                })?;

            let location = loc(&pair, source_name);
            match inner.as_rule() {
                Rule::primitive_literal => {
                    let lit = parse_primitive_literal_to_lit(inner)?;
                    Ok(Pattern::Literal(lit, location))
                }
                _ => Err(ParseError::SyntaxError {
                    message: format!("Invalid literal pattern: {:?}", inner.as_rule()),
                    src: miette::NamedSource::new(
                        source_name
                            .clone()
                            .unwrap_or_else(|| "<unknown>".to_string()),
                        "".to_string(),
                    ),
                    span: get_span_from_pair(&inner),
                    location: get_location_from_pair(&inner),
                    help_message: "Expected a primitive literal".to_string(),
                    suggestion: None,
                }),
            }
        }
        _ => Err(ParseError::SyntaxError {
            message: format!("Invalid pattern rule: {:?}", pair.as_rule()),
            src: miette::NamedSource::new(
                source_name
                    .clone()
                    .unwrap_or_else(|| "<unknown>".to_string()),
                "".to_string(),
            ),
            span: get_span_from_pair(&pair),
            location: get_location_from_pair(&pair),
            help_message: "Expected a valid pattern".to_string(),
            suggestion: None,
        }),
    }
}

/// Parse a type expression using the restructured grammar
pub fn parse_type_expr(pair: Pair<Rule>) -> Result<TypeExpr, ParseError> {
    let mut pairs = pair.clone().into_inner();

    // First element must be a type_primary
    let first_pair = pairs.next().ok_or_else(|| ParseError::MissingToken {
        expected: "type expression".to_string(),
        src: miette::NamedSource::new("<unknown>", "".to_string()),
        span: get_span_from_pair(&pair),
        location: get_location_from_pair(&pair),
        help_message: "Expected a type expression".to_string(),
        suggestion: None,
    })?;

    let mut result = parse_type_primary(first_pair)?;

    while let Some(op_pair) = pairs.next() {
        let op_inner =
            op_pair
                .clone()
                .into_inner()
                .next()
                .ok_or_else(|| ParseError::MissingToken {
                    expected: "type operator".to_string(),
                    src: miette::NamedSource::new("<unknown>", "".to_string()),
                    span: get_span_from_pair(&op_pair),
                    location: get_location_from_pair(&op_pair),
                    help_message: "Expected a type operator".to_string(),
                    suggestion: None,
                })?;

        let rhs_pair = pairs.next().ok_or_else(|| ParseError::MissingToken {
            expected: "right-hand side of type expression".to_string(),
            src: miette::NamedSource::new("<unknown>", "".to_string()),
            span: get_span_from_pair(&op_pair),
            location: get_location_from_pair(&op_pair),
            help_message: "Expected a type expression after the operator".to_string(),
            suggestion: None,
        })?;

        let rhs = parse_type_primary(rhs_pair)?;

        result = match op_inner.as_rule() {
            Rule::op_arrow => TypeExpr::Function(Box::new(result), Box::new(rhs), None),
            Rule::op_linear_arrow => {
                TypeExpr::LinearFunction(Box::new(result), Box::new(rhs), None)
            }
            Rule::op_prod => TypeExpr::Product(Box::new(result), Box::new(rhs), None),
            Rule::op_sum => TypeExpr::Sum(Box::new(result), Box::new(rhs), None),
            _ => {
                return Err(ParseError::UnexpectedRule {
                    expected: "type operator".to_string(),
                    found: format!("{:?}", op_inner.as_rule()),
                    span: get_span_from_pair(&op_inner),
                    location: get_location_from_pair(&op_inner),
                });
            }
        };
    }

    Ok(result)
}

/// Parse the primary components of a type expression
fn parse_type_primary(pair: Pair<Rule>) -> Result<TypeExpr, ParseError> {
    match pair.as_rule() {
        Rule::type_identifier => {
            // Handle simple type names
            let name = pair.as_str().to_string();
            Ok(TypeExpr::Name(name, None))
        }
        Rule::identifier => {
            let name = pair.as_str().to_string();
            Ok(TypeExpr::Variable(name, None))
        }
        Rule::parenthesized_expr => {
            let inner =
                pair.clone()
                    .into_inner()
                    .next()
                    .ok_or_else(|| ParseError::MissingToken {
                        expected: "type expression".to_string(),
                        src: miette::NamedSource::new("<unknown>", "".to_string()),
                        span: get_span_from_pair(&pair),
                        location: get_location_from_pair(&pair),
                        help_message: "Expected a type expression here".to_string(),
                        suggestion: None,
                    })?;
            parse_type_expr(inner)
        }
        Rule::list_type => {
            let inner =
                pair.clone()
                    .into_inner()
                    .next()
                    .ok_or_else(|| ParseError::MissingToken {
                        expected: "element type".to_string(),
                        src: miette::NamedSource::new("<unknown>", "".to_string()),
                        span: get_span_from_pair(&pair),
                        location: get_location_from_pair(&pair),
                        help_message: "Expected a type expression here".to_string(),
                        suggestion: None,
                    })?;
            let inner_type = parse_type_expr(inner)?;
            Ok(TypeExpr::List(Box::new(inner_type), None))
        }
        Rule::set_type => {
            let inner =
                pair.clone()
                    .into_inner()
                    .next()
                    .ok_or_else(|| ParseError::MissingToken {
                        expected: "element type".to_string(),
                        src: miette::NamedSource::new("<unknown>", "".to_string()),
                        span: get_span_from_pair(&pair),
                        location: get_location_from_pair(&pair),
                        help_message: "Expected a type expression here".to_string(),
                        suggestion: None,
                    })?;
            let inner_type = parse_type_expr(inner)?;
            Ok(TypeExpr::Set(Box::new(inner_type), None))
        }
        Rule::option_type => {
            let inner =
                pair.clone()
                    .into_inner()
                    .next()
                    .ok_or_else(|| ParseError::MissingToken {
                        expected: "inner type".to_string(),
                        src: miette::NamedSource::new("<unknown>", "".to_string()),
                        span: get_span_from_pair(&pair),
                        location: get_location_from_pair(&pair),
                        help_message: "Expected a type expression here".to_string(),
                        suggestion: None,
                    })?;
            let inner_type = parse_type_expr(inner)?;
            Ok(TypeExpr::Option(Box::new(inner_type), None))
        }
        _ => Err(ParseError::UnexpectedRule {
            expected: "type expression".to_string(),
            found: format!("{:?}", pair.as_rule()),
            span: get_span_from_pair(&pair),
            location: get_location_from_pair(&pair),
        }),
    }
}

/// Add serialization and prettification utilities to the Module struct
impl Module {
    /// Convert the module AST to a pretty-printed JSON string
    pub fn to_pretty_json(&self) -> String {
        match serde_json::to_string_pretty(self) {
            Ok(json) => json,
            Err(e) => format!("Error serializing to JSON: {}", e),
        }
    }

    /// Generate a DOT graph representation of the AST for visualization
    pub fn to_dot_graph(&self) -> String {
        // Start the DOT graph
        let mut dot = String::from("digraph AST {\n");
        dot.push_str("  // Graph styling\n");
        dot.push_str("  graph [fontname=\"Arial\", rankdir=LR];\n");
        dot.push_str(
            "  node [fontname=\"Arial\", shape=box, style=filled, fillcolor=lightblue];\n",
        );
        dot.push_str("  edge [fontname=\"Arial\"];\n\n");

        // Add module node
        dot.push_str(&format!(
            "  module [label=\"Module: {}\", fillcolor=lightgreen];\n",
            self.name
        ));

        // Add all declarations
        for (i, decl) in self.declarations.iter().enumerate() {
            let decl_id = format!("decl_{}", i);
            match decl {
                Declaration::Type(name, _, _) => {
                    dot.push_str(&format!("  {} [label=\"Type: {}\"];\n", decl_id, name));
                    dot.push_str(&format!("  module -> {};\n", decl_id));
                }
                Declaration::Operation(name, _, _) => {
                    dot.push_str(&format!("  {} [label=\"Operation: {}\"];\n", decl_id, name));
                    dot.push_str(&format!("  module -> {};\n", decl_id));
                }
                Declaration::Function(name, _, body, _) => {
                    dot.push_str(&format!("  {} [label=\"Function: {}\"];\n", decl_id, name));
                    dot.push_str(&format!("  module -> {};\n", decl_id));

                    // Add function body
                    let body_id = format!("body_{}", i);
                    self.add_expr_to_dot(&mut dot, body, &body_id, &decl_id);
                }
                Declaration::Dependency(name, target, direct, _) => {
                    dot.push_str(&format!(
                        "  {} [label=\"Dependency: {} -> {}{}\"];\n",
                        decl_id,
                        name,
                        target,
                        if *direct { " (direct)" } else { "" }
                    ));
                    dot.push_str(&format!("  module -> {};\n", decl_id));
                }
                Declaration::Entity(name, _, expr_opt, _) => {
                    dot.push_str(&format!("  {} [label=\"Entity: {}\"];\n", decl_id, name));
                    dot.push_str(&format!("  module -> {};\n", decl_id));

                    // Add entity expression if present
                    if let Some(expr) = expr_opt {
                        let expr_id = format!("entity_{}", i);
                        self.add_expr_to_dot(&mut dot, expr, &expr_id, &decl_id);
                    }
                }
            }
        }

        dot.push_str("}\n");
        dot
    }

    /// Helper function to add an expression to the DOT graph
    fn add_expr_to_dot(&self, dot: &mut String, expr: &Expr, node_id: &str, parent_id: &str) {
        match expr {
            Expr::Literal(lit, _) => {
                let lit_str = match lit {
                    Literal::Integer(i) => format!("Int: {}", i),
                    Literal::Float(f) => format!("Float: {}", f),
                    Literal::String(s) => format!("String: \"{}\"", s.replace("\"", "\\\"")),
                    Literal::Boolean(b) => format!("Bool: {}", b),
                };
                dot.push_str(&format!(
                    "  {} [label=\"{}\", fillcolor=lightyellow];\n",
                    node_id, lit_str
                ));
                dot.push_str(&format!("  {} -> {};\n", parent_id, node_id));
            }
            Expr::Variable(name, _) => {
                dot.push_str(&format!(
                    "  {} [label=\"Var: {}\", fillcolor=lightpink];\n",
                    node_id, name
                ));
                dot.push_str(&format!("  {} -> {};\n", parent_id, node_id));
            }
            Expr::BinaryOp(op, lhs, rhs, _) => {
                dot.push_str(&format!("  {} [label=\"BinOp: {}\"];\n", node_id, op));
                dot.push_str(&format!("  {} -> {};\n", parent_id, node_id));

                let lhs_id = format!("{}_lhs", node_id);
                let rhs_id = format!("{}_rhs", node_id);

                self.add_expr_to_dot(dot, lhs, &lhs_id, node_id);
                self.add_expr_to_dot(dot, rhs, &rhs_id, node_id);
            }
            // Other expression types would be handled similarly...
            _ => {
                dot.push_str(&format!("  {} [label=\"{:?}\"];\n", node_id, expr));
                dot.push_str(&format!("  {} -> {};\n", parent_id, node_id));
            }
        }
    }
}

// Pratt parser instance with operator precedence
// Using Lazy for one-time initialization
static PRATT_PARSER: Lazy<PrattParser<Rule>> =
    Lazy::new(|| create_pratt_parser(&ParserContext::new("", None)));

// Re-export Rule and BorfParser for use in lib.rs
pub use pest::iterators::Pair as PestPair;
pub use pest::Parser as PestParser;

// Define the parser using Pest's derive macro
#[derive(Parser)]
#[grammar = "src/parser/grammar.pest"]
pub struct BorfParser;

// Make helper functions public
pub fn get_span_from_pair(pair: &Pair<Rule>) -> miette::SourceSpan {
    let start = pair.as_span().start();
    let end = pair.as_span().end();
    (start, end - start).into()
}

pub fn get_location_from_pair(pair: &Pair<Rule>) -> String {
    let (line, col) = pair.as_span().start_pos().line_col();
    format!("{}:{}", line, col)
}

/// Convert a pest error to a miette Span
fn pest_error_to_span(error: &pest::error::Error<Rule>) -> SourceSpan {
    match error.location {
        pest::error::InputLocation::Pos(pos) => {
            let pos = pos as usize;
            SourceSpan::new(SourceOffset::from(pos), SourceOffset::from(0))
        }
        pest::error::InputLocation::Span((start, end)) => {
            let start = start as usize;
            let end = end as usize;
            SourceSpan::new(SourceOffset::from(start), SourceOffset::from(end - start))
        }
    }
}

/// Parse an expression
pub fn parse_expr(pair: Pair<Rule>, source_name: &Option<String>) -> Result<Expr, ParseError> {
    // Use context-aware version for enhanced functionality when appropriate
    let input = pair.as_str();
    let mut context = ParserContext::new(input, source_name.clone());

    // Check if we should collect suggestions (this could be a configuration setting)
    if true {
        // Always use enhanced parser for now
        context = context.with_recovery();
        parse_expr_with_context(pair, &mut context)
    } else {
        // Use original Pratt parser logic for backward compatibility
        parse_pratt_expr(pair.into_inner(), source_name)
    }
}

/// Original Pratt parser logic for backward compatibility
fn parse_pratt_expr(pairs: Pairs<Rule>, source_name: &Option<String>) -> Result<Expr, ParseError> {
    PRATT_PARSER
        .map_primary(|primary| match primary.as_rule() {
            Rule::primary => parse_primary_expr(primary, source_name),
            Rule::identifier => Ok(Expr::Variable(
                primary.as_str().to_string(),
                loc(&primary, source_name),
            )),
            Rule::literal => parse_literal(primary, source_name),
            _ => Err(ParseError::SyntaxError {
                message: format!("Expected primary rule, found {:?}", primary.as_rule()),
                src: miette::NamedSource::new(
                    source_name
                        .clone()
                        .unwrap_or_else(|| "<unknown>".to_string()),
                    "".to_string(),
                ),
                span: get_span_from_pair(&primary),
                location: get_location_from_pair(&primary),
                help_message: "Expected a primary expression".to_string(),
                suggestion: None,
            }),
        })
        .map_infix(|lhs, op, rhs| {
            match (lhs, rhs) {
                (Ok(lhs_expr), Ok(rhs_expr)) => {
                    // Both expressions parsed successfully
                    Ok(Expr::BinaryOp(
                        op.as_str().to_string(),
                        Box::new(lhs_expr),
                        Box::new(rhs_expr),
                        loc(&op, source_name),
                    ))
                }
                (Err(e), _) => {
                    // Left-hand side failed to parse
                    Err(e)
                }
                (_, Err(e)) => {
                    // Right-hand side failed to parse
                    Err(e)
                }
            }
        })
        .map_prefix(|op, rhs| {
            match rhs {
                Ok(rhs_expr) => {
                    // Expression parsed successfully
                    Ok(Expr::UnaryOp(
                        op.as_str().to_string(),
                        Box::new(rhs_expr),
                        loc(&op, source_name),
                    ))
                }
                Err(e) => {
                    // Expression failed to parse
                    Err(e)
                }
            }
        })
        .parse(pairs)
}

/// Helper to parse the content of a `primary` rule
fn parse_primary_expr(pair: Pair<Rule>, source_name: &Option<String>) -> Result<Expr, ParseError> {
    let pair_clone = pair.clone();
    let inner = pair
        .into_inner()
        .next()
        .ok_or_else(|| ParseError::MissingToken {
            expected: "expression inside primary".to_string(),
            src: miette::NamedSource::new(
                source_name
                    .clone()
                    .unwrap_or_else(|| "<unknown>".to_string()),
                "".to_string(),
            ),
            span: get_span_from_pair(&pair_clone),
            location: get_location_from_pair(&pair_clone),
            help_message: "Primary expression is empty".to_string(),
            suggestion: None,
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
        Rule::if_expr => parse_if_expr(inner, source_name),
        Rule::parenthesized_expr => {
            let inner_clone = inner.clone();
            let expr_pair = inner
                .into_inner()
                .next()
                .ok_or_else(|| ParseError::MissingToken {
                    expected: "expression inside parentheses".to_string(),
                    src: miette::NamedSource::new(
                        source_name
                            .clone()
                            .unwrap_or_else(|| "<unknown>".to_string()),
                        "".to_string(),
                    ),
                    span: get_span_from_pair(&inner_clone),
                    location: get_location_from_pair(&inner_clone),
                    help_message: "Empty parentheses not allowed".to_string(),
                    suggestion: None,
                })?;
            parse_expr(expr_pair, source_name)
        }
        Rule::quoting_expr => parse_quoting_expr(inner, source_name),
        _ => Err(ParseError::SyntaxError {
            message: format!("Unexpected expression type: {:?}", inner.as_rule()),
            src: miette::NamedSource::new(
                source_name
                    .clone()
                    .unwrap_or_else(|| "<unknown>".to_string()),
                "".to_string(),
            ),
            span: get_span_from_pair(&inner),
            location: get_location_from_pair(&inner),
            help_message: "Expected a valid expression".to_string(),
            suggestion: None,
        }),
    }
}
