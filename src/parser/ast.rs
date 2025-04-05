use super::SourceLocation;
use rustc_hash::FxHashMap;
use serde::{ser::SerializeStruct, Deserialize, Serialize};
use smallvec::{smallvec, SmallVec};
use std::collections::HashMap;
use std::hash::Hash;

/// Type alias for small vectors that typically have few elements
pub type SmallVec8<T> = SmallVec<[T; 8]>;

/// The AST node types for the Borf language

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Module {
    pub name: String,
    pub types: SmallVec8<String>,
    pub operations: SmallVec8<String>,
    pub functions: SmallVec8<String>,
    pub declarations: Vec<Declaration>,
    // Source location ignored for equality comparison
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<SourceLocation>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Declaration {
    Type(
        String,
        TypeExpr,
        #[serde(skip_serializing_if = "Option::is_none")] Option<SourceLocation>,
    ),
    Operation(
        String,
        TypeExpr,
        #[serde(skip_serializing_if = "Option::is_none")] Option<SourceLocation>,
    ),
    Function(
        String,
        TypeExpr,
        Box<Expr>,
        #[serde(skip_serializing_if = "Option::is_none")] Option<SourceLocation>,
    ),
    Dependency(
        String,
        String,
        bool,
        #[serde(skip_serializing_if = "Option::is_none")] Option<SourceLocation>,
    ), // (import, export, direct)
    Entity(
        String,
        TypeExpr,
        Option<Box<Expr>>,
        #[serde(skip_serializing_if = "Option::is_none")] Option<SourceLocation>,
    ),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TypeExpr {
    Name(
        String,
        #[serde(skip_serializing_if = "Option::is_none")] Option<SourceLocation>,
    ),
    Variable(
        String,
        #[serde(skip_serializing_if = "Option::is_none")] Option<SourceLocation>,
    ),
    Function(
        Box<TypeExpr>,
        Box<TypeExpr>,
        #[serde(skip_serializing_if = "Option::is_none")] Option<SourceLocation>,
    ),
    LinearFunction(
        Box<TypeExpr>,
        Box<TypeExpr>,
        #[serde(skip_serializing_if = "Option::is_none")] Option<SourceLocation>,
    ),
    Product(
        Box<TypeExpr>,
        Box<TypeExpr>,
        #[serde(skip_serializing_if = "Option::is_none")] Option<SourceLocation>,
    ),
    Sum(
        Box<TypeExpr>,
        Box<TypeExpr>,
        #[serde(skip_serializing_if = "Option::is_none")] Option<SourceLocation>,
    ),
    Map(
        Box<TypeExpr>,
        Box<TypeExpr>,
        #[serde(skip_serializing_if = "Option::is_none")] Option<SourceLocation>,
    ),
    List(
        Box<TypeExpr>,
        #[serde(skip_serializing_if = "Option::is_none")] Option<SourceLocation>,
    ),
    Set(
        Box<TypeExpr>,
        #[serde(skip_serializing_if = "Option::is_none")] Option<SourceLocation>,
    ),
    Option(
        Box<TypeExpr>,
        #[serde(skip_serializing_if = "Option::is_none")] Option<SourceLocation>,
    ),
    Linear(
        Box<TypeExpr>,
        #[serde(skip_serializing_if = "Option::is_none")] Option<SourceLocation>,
    ),
    Sequence(
        Box<TypeExpr>,
        #[serde(skip_serializing_if = "Option::is_none")] Option<SourceLocation>,
    ),
    Universal(#[serde(skip_serializing_if = "Option::is_none")] Option<SourceLocation>), // Any type
    Void(#[serde(skip_serializing_if = "Option::is_none")] Option<SourceLocation>), // Empty type
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expr {
    Literal(
        Literal,
        #[serde(skip_serializing_if = "Option::is_none")] Option<SourceLocation>,
    ),
    Variable(
        String,
        #[serde(skip_serializing_if = "Option::is_none")] Option<SourceLocation>,
    ),
    QualifiedName(
        SmallVec8<String>,
        #[serde(skip_serializing_if = "Option::is_none")] Option<SourceLocation>,
    ),
    Lambda(
        SmallVec8<Box<Pattern>>,
        Box<Expr>,
        #[serde(skip_serializing_if = "Option::is_none")] Option<SourceLocation>,
    ),
    Application(
        Box<Expr>,
        SmallVec8<Box<Expr>>,
        #[serde(skip_serializing_if = "Option::is_none")] Option<SourceLocation>,
    ),
    Let(
        Box<Pattern>,
        Box<Expr>,
        Box<Expr>,
        #[serde(skip_serializing_if = "Option::is_none")] Option<SourceLocation>,
    ),
    If(
        Box<Expr>,
        Box<Expr>,
        Box<Expr>,
        #[serde(skip_serializing_if = "Option::is_none")] Option<SourceLocation>,
    ),
    BinaryOp(
        String,
        Box<Expr>,
        Box<Expr>,
        #[serde(skip_serializing_if = "Option::is_none")] Option<SourceLocation>,
    ),
    UnaryOp(
        String,
        Box<Expr>,
        #[serde(skip_serializing_if = "Option::is_none")] Option<SourceLocation>,
    ),
    List(
        SmallVec8<Box<Expr>>,
        #[serde(skip_serializing_if = "Option::is_none")] Option<SourceLocation>,
    ),
    Set(
        SmallVec8<Box<Expr>>,
        #[serde(skip_serializing_if = "Option::is_none")] Option<SourceLocation>,
    ),
    Map(
        FxHashMap<String, Box<Expr>>,
        #[serde(skip_serializing_if = "Option::is_none")] Option<SourceLocation>,
    ),

    // Homoiconicity features
    Quote(
        Box<Expr>,
        #[serde(skip_serializing_if = "Option::is_none")] Option<SourceLocation>,
    ), // Represents a quoted expression '(expr)
    Unquote(
        Box<Expr>,
        #[serde(skip_serializing_if = "Option::is_none")] Option<SourceLocation>,
    ), // Represents an unquoted expression ~expr
    UnquoteSplice(
        Box<Expr>,
        #[serde(skip_serializing_if = "Option::is_none")] Option<SourceLocation>,
    ), // Represents an unquote-splice expression ~@expr
    Quasiquote(
        Box<Expr>,
        #[serde(skip_serializing_if = "Option::is_none")] Option<SourceLocation>,
    ), // Represents a quasiquoted expression `(expr)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Literal {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
}

// Box the recursive parts to avoid infinite size issues
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Pattern {
    Variable(
        String,
        #[serde(skip_serializing_if = "Option::is_none")] Option<SourceLocation>,
    ),
    Literal(
        Literal,
        #[serde(skip_serializing_if = "Option::is_none")] Option<SourceLocation>,
    ),
    Wildcard(#[serde(skip_serializing_if = "Option::is_none")] Option<SourceLocation>),
    List(
        SmallVec8<Box<Pattern>>,
        #[serde(skip_serializing_if = "Option::is_none")] Option<SourceLocation>,
    ),
    Set(
        SmallVec8<Box<Pattern>>,
        #[serde(skip_serializing_if = "Option::is_none")] Option<SourceLocation>,
    ),
    Map(
        FxHashMap<String, Box<Pattern>>,
        #[serde(skip_serializing_if = "Option::is_none")] Option<SourceLocation>,
    ),
    TypeAnnotated(
        Box<Pattern>,
        TypeExpr,
        #[serde(skip_serializing_if = "Option::is_none")] Option<SourceLocation>,
    ),
}

/// Implement PartialEq for SourceLocation
impl PartialEq for SourceLocation {
    fn eq(&self, _other: &Self) -> bool {
        // Always return true for SourceLocation to ignore it in equality comparisons
        true
    }
}

/// Creates a qualified identifier from parts
pub fn make_qualified_name(parts: Vec<String>) -> String {
    parts.join(".")
}

/// Creates a simple module with a name
pub fn make_module(name: &str) -> Module {
    Module {
        name: name.to_string(),
        types: SmallVec8::new(),
        operations: SmallVec8::new(),
        functions: SmallVec8::new(),
        declarations: vec![],
        location: None,
    }
}

/// Adds a type declaration to a module
pub fn add_type(module: &mut Module, name: &str) {
    module.types.push(name.to_string());
}

/// Adds an operation declaration to a module
pub fn add_operation(module: &mut Module, name: &str) {
    module.operations.push(name.to_string());
}

/// Adds a function declaration to a module
pub fn add_function(module: &mut Module, name: &str) {
    module.functions.push(name.to_string());
}

/// Adds a general declaration to a module
pub fn add_declaration(module: &mut Module, decl: Declaration) {
    module.declarations.push(decl);
}

/// Creates a quoted expression ('expr)
pub fn make_quote(expr: Expr, location: Option<SourceLocation>) -> Expr {
    Expr::Quote(Box::new(expr), location)
}

/// Creates an unquoted expression (~expr)
pub fn make_unquote(expr: Expr, location: Option<SourceLocation>) -> Expr {
    Expr::Unquote(Box::new(expr), location)
}

/// Creates an unquote-splice expression (~@expr)
pub fn make_unquote_splice(expr: Expr, location: Option<SourceLocation>) -> Expr {
    Expr::UnquoteSplice(Box::new(expr), location)
}

/// Creates a quasiquoted expression (`expr)
pub fn make_quasiquote(expr: Expr, location: Option<SourceLocation>) -> Expr {
    Expr::Quasiquote(Box::new(expr), location)
}

/// REPL Input type
#[derive(Debug, Clone, PartialEq)]
pub enum ReplInput {
    Expression(Expr),
    Declaration(Declaration),
}

// Implement Serialize and Deserialize for SourceLocation
// This is needed since SourceLocation is defined in mod.rs
impl Serialize for SourceLocation {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("SourceLocation", 5)?;
        state.serialize_field("start", &self.start)?;
        state.serialize_field("end", &self.end)?;
        state.serialize_field("line", &self.line)?;
        state.serialize_field("column", &self.column)?;
        state.serialize_field("source_name", &self.source_name)?;
        state.end()
    }
}

// Only implement the visitor pattern for Deserialize as a placeholder
// (this is a simplification; a real implementation would need more code)
impl<'de> Deserialize<'de> for SourceLocation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Simple implementation to make it compile
        #[derive(Deserialize)]
        struct SourceLocationDef {
            start: usize,
            end: usize,
            line: usize,
            column: usize,
            source_name: Option<String>,
        }

        let def = SourceLocationDef::deserialize(deserializer)?;
        Ok(SourceLocation {
            start: def.start,
            end: def.end,
            line: def.line,
            column: def.column,
            source_name: def.source_name,
        })
    }
}
