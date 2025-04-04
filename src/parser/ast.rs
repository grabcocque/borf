//! Abstract Syntax Tree definitions for the Borf language.
//!
//! This module contains the type definitions that make up the AST for Borf,
//! including top-level items, category elements, constraints, and expressions.

/// Represents infix operators.
#[derive(Debug, Clone, PartialEq)]
pub enum InfixOperator {
    // Composition
    Compose,      // .
    ComposeRight, // >>
    Pipe,         // |>
    // Arithmetic
    Multiply, // *
    Divide,   // /
    Add,      // +
    Subtract, // -
    // Set
    Union,     // $cup
    Intersect, // $cap
    Subset,    // $subseteq
    // Comparison
    Equal,         // = or == or === (Semantic distinction needed later)
    TypeEqual,     // $teq
    ValueEqual,    // $veq
    StructEqual,   // $seq
    CategoryEqual, // $ceq
    Subtype,       // <::
    GreaterThan,   // >
    LessThan,      // <
    GreaterEqual,  // >=
    LessEqual,     // <=
    In,            // $in
    Compatible,    // $omega
    // Logical
    And,     // $and
    Or,      // $or
    Implies, // =>
    Iff,     // $iff
    // Additional operators
    TransitiveClosure, // ->+
    Assign,            // = (Adding specific assignment operator)
    Colon,             // : (For type constraints etc. in expressions)
}

/// Represents prefix operators.
#[derive(Debug, Clone, PartialEq)]
pub enum PrefixOperator {
    Not,             // $not
    Optional,        // ? (Potentially for types/expressions)
    Linear,          // ! (Potentially for types/expressions)
    CardinalityOpen, // | (opening of cardinality)
}

/// Represents postfix operators.
#[derive(Debug, Clone, PartialEq)]
pub enum PostfixOperator {
    FieldAccess(String),             // .ident
    ChainedFieldAccess(Vec<String>), // .field1.field2... for multi-level access
    FunctionCall(Vec<Expression>),   // (args)
    Index(Box<Expression>),          // [expr]
    CardinalityClose,                // | (closing of cardinality)
}

/// Represents a top-level item in the Borf program.
///
/// Top-level items include category definitions, pipeline definitions,
/// pipe expressions, application expressions, composition expressions,
/// and export/import directives.
#[derive(Debug, Clone)]
pub enum TopLevelItem {
    /// A category definition with objects, mappings, laws, and other elements
    Module(ModuleDef), // Renamed from Category
    /// A primitive block definition
    Primitive(PrimitiveDef), // Added for primitive_declaration
    /// An export directive specifying which elements to export
    Export(ExportDirective),
    /// An import directive for including external modules
    Import(ImportDirective),
    /// A standalone expression statement
    ExpressionStatement(Expression),
    // Removed Pipeline, PipeExpr, AppExpr, CompositionExpr as they are handled by ExpressionStatement + Pratt parsing
}

/// Represents a module definition (formerly Category) in the Borf language.
/// Corresponds to the `@ ident : { ... }` grammar rule.
#[derive(Debug, Clone)]
pub struct ModuleDef {
    /// The name of the module
    pub name: String,
    // Base category removed, modules are standalone based on current grammar view
    /// Collection of elements contained in this module
    pub elements: Vec<ModuleElement>,
}

/// Represents elements within a module definition (formerly CategoryElement).
/// Corresponds to the `module_element` grammar rule.
#[derive(Debug, Clone)]
pub enum ModuleElement {
    /// A unified declaration for objects, types, functions, laws, values.
    Declaration(Declaration),
    // Removed specific decls like ObjectDecl, MappingDecl, LawDecl, etc.
    // Removed FunctionDef, StructureMapping
    // Removed ConstraintDecl
}

/// Represents an export directive (`export ident;`).
#[derive(Debug, Clone)]
pub struct ExportDirective {
    /// The identifiers to export
    pub identifiers: Vec<String>,
}

/// Represents an import directive (`import "path";`).
#[derive(Debug, Clone)]
pub struct ImportDirective {
    /// The path to the module to import
    pub path: String,
}

/// NEW: SetExpr enum to represent different forms of set expressions
#[derive(Debug, Clone, PartialEq)]
pub enum SetExpr {
    Literal(SetLiteral),
    Comprehension(Box<SetComprehension>), // Renamed from NestedComprehension
    Operation(Box<SetOperation>),
    Identifier(String), // Assuming sets can be referred to by name
    Empty,              // Representing {}
}

/// Represents a literal set defined by enumerating elements: `{elem1, elem2, ...}`
#[derive(Debug, Clone, PartialEq)]
pub struct SetLiteral {
    /// The elements of the set.
    /// Using Expression for now, as elements can be idents, ints, symbols, tuples according to grammar.
    pub elements: Vec<Expression>,
}

/// Represents an operation between two sets.
#[derive(Debug, Clone, PartialEq)]
pub struct SetOperation {
    /// Left-hand side set identifier.
    pub lhs: String,
    /// The set operator (e.g., $cup, $cap, $in).
    pub op: String,
    /// Right-hand side set identifier.
    pub rhs: String,
}

/// Represents a lambda expression (`\ params . body`).
#[derive(Debug, Clone, PartialEq)]
pub struct LambdaExpr {
    pub params: Vec<String>,
    pub body: Box<Expression>,
}

/// Represents an if-then-else expression.
#[derive(Debug, Clone, PartialEq)]
pub struct IfExpr {
    pub condition: Box<Expression>,
    pub then_branch: Box<Expression>,
    pub else_branch: Box<Expression>,
}

/// Represents a let-rec expression (`let rec name params = bound_expr in body`).
#[derive(Debug, Clone, PartialEq)]
pub struct LetRecExpr {
    pub name: String,
    pub params: Vec<String>,
    pub bound_expr: Box<Expression>,
    pub body: Box<Expression>,
}

/// Represents an expression in the Borf language.
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    AtomExpr(Atom), // Renamed variant slightly to avoid name clash if Atom type is also named Atom
    Lambda(LambdaExpr),
    If(IfExpr),
    LetRec(LetRecExpr),
    Quantifier(QuantifierExpr), // Added for quantifier expressions
    TypeCalculation(TypeCalculationExpr), // Added for |{...}|
    FunctionChainCall(FunctionChainExpr), // Added for nested function chains
    Record(Vec<(String, Expression)>), // Added record expression
    Tuple(Vec<Expression>),     // Moved Tuple from Atom to Expression

    // Variants for Pratt Parser output
    InfixOp {
        lhs: Box<Expression>,
        op: InfixOperator,
        rhs: Box<Expression>,
    },
    PrefixOp {
        op: PrefixOperator,
        operand: Box<Expression>,
    },
    PostfixOp {
        operand: Box<Expression>,
        op: PostfixOperator,
    },
    TernaryOp {
        // Specific structure for ternary a ? b : c
        condition: Box<Expression>,
        if_true: Box<Expression>,
        if_false: Box<Expression>, // Changed ConstraintExpr to Expression
    },
    QualifiedName {
        base: String,
        access: Vec<String>,
    },
    ModuleAccess {
        module_param: String,
        path: Vec<String>,
    },
    TypeRange {
        base_type: String,
        modifier: String, // '+', '-', '*', 'n', 'p'
    },
    Cardinality {
        expr: Box<Expression>,
    },
    EmptySet, // Added for empty set literals {}
    EmptyList, // Added for empty list literals []
              // StringLiteral, Type, Operator removed - represented via AtomExpr(Atom::...)
}

/// Represents a chain of function calls like `f(a)(b)`.
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionChainExpr {
    pub base: String,                // Base function name or qualified name
    pub calls: Vec<Vec<Expression>>, // Sequence of argument lists for each nested call
}

/// Represents quantifiers in Borf.
#[derive(Debug, Clone, PartialEq)]
pub enum Quantifier {
    ForAll,
    Exists,
    ExistsUnique, // New: $exists!
}

/// Represents a set comprehension structure.
/// Corresponds to grammar rules like `{ expr | clauses }`
#[derive(Debug, Clone, PartialEq)]
pub struct SetComprehension {
    // Renamed from NestedComprehension
    pub expr: Box<Expression>,
    pub clauses: Vec<ComprehensionClause>,
}

/// Represents clauses within a set comprehension.
#[derive(Debug, Clone, PartialEq)]
pub enum ComprehensionClause {
    Generator {
        var: String,
        domain: Box<Expression>, // Changed ConstraintExpr to Expression
    },
    Constraint(Box<Expression>),   // Changed ConstraintExpr to Expression
    Nested(Box<SetComprehension>), // Updated reference to SetComprehension
}

/// Represents a primitive block definition `@PrimitiveName : { ... }`.
#[derive(Debug, Clone)]
pub struct PrimitiveDef {
    pub name: String,
    pub elements: Vec<PrimitiveElement>, // Assuming elements are like category elements for now
}

/// Represents elements within a primitive block.
#[derive(Debug, Clone, PartialEq)]
pub enum PrimitiveElement {
    Declaration(Declaration), // Changed from Mapping(MappingDecl)
                              // Removed incorrectly added variants. Primitives likely only contain Declarations.
}

/// Represents a type expression in Borf.
#[derive(Debug, Clone, PartialEq)]
pub enum TypeExpr {
    /// A base type identifier (e.g., `T`, `Bool`, `MyType`).
    Base(String),
    /// A symbolic type (`Sym`).
    Sym,
    /// A product type (e.g., `A * B`).
    Product(Box<TypeExpr>, Box<TypeExpr>),
    /// An arrow (function) type (e.g., `A -> B`).
    Arrow(Box<TypeExpr>, Box<TypeExpr>),
    /// A set type (e.g., `{A}`).
    Set(Box<TypeExpr>),
    /// A list type (e.g., `[A]`).
    List(Box<TypeExpr>),
    /// An optional type (e.g., `?A`).
    Optional(Box<TypeExpr>),
    /// A linear type (e.g., `!A`).
    Linear(Box<TypeExpr>),
    /// A tuple type (e.g., `(A, B, C)`).
    Tuple(Vec<TypeExpr>),
    /// Placeholder for errors or unparsed types during development.
    /// TODO: Remove or refine error handling.
    Unknown(String),
    /// The special X type from the prelude
    X,
    /// A type sum (e.g., `A + B`).
    TypeSum {
        lhs: Box<TypeExpr>,
        rhs: Box<TypeExpr>,
    },
    /// A type range (e.g., `Z+`).
    TypeRange {
        base: String,
        modifier: String, // '+', '-', '*', 'n', 'p'
    },
    /// A qualified type (e.g., `Mod.SubMod.Type`).
    QualifiedType { base: String, access: Vec<String> },
    /// A module qualified type (e.g., s.O)
    ModuleQualifiedType { module: String, type_name: String },
    /// A type with parameters and constraints (e.g., (s,t,f,x)->y | s:Cat)
    TypeWithParameters {
        params: Vec<String>, // Changed from Vec<TypeExpr> to Vec<String> based on grammar param_list
        return_type: Box<TypeExpr>,
        constraint: Option<Box<Expression>>, // Changed ConstraintExpr to Expression
    },
    /// Union of types (e.g., B $cup P)
    TypeUnion {
        lhs: Box<TypeExpr>,
        rhs: Box<TypeExpr>,
    },
    /// Intersection of types (e.g., B $cap P)
    TypeIntersection {
        lhs: Box<TypeExpr>,
        rhs: Box<TypeExpr>,
    },
    /// A record type { field: Type, ... }
    Record(Vec<(String, TypeExpr)>), // Added Record Type
    /// The `Any` keyword type.
    Any, // Added Any type
    /// The `Pattern` keyword type.
    Pattern, // Added Pattern type
}

/// Represents atomic expressions.
#[derive(Debug, Clone, PartialEq)]
pub enum Atom {
    Identifier(String),
    DollarIdentifier(String), // Added for $identifiers
    Integer(i64),
    Boolean(bool), // Added Boolean
    Symbol(String),
    // Tuple removed here, moved to Expression::Tuple
    Set(Box<SetExpr>),      // Changed to use new SetExpr enum
    EmptySet,               // Empty set literal {}
    EmptyList,              // Empty list literal []
    Paren(Box<Expression>), // Parenthesized expression
    StringLiteral(String),  // Added back: For `"ident"` in grammar
    LawIdentifier(String),  // For law.name identifiers
    // Added Type variant for when types appear as atoms (e.g., `!T`, `[T]`)
    // This might need adjustment based on how Pratt parser handles type prefixes.
    Type(TypeExpr),
    QualifiedName { base: String, access: Vec<String> }, // Added for qualified names as atoms
    Operator(String), // Added for operators used as atoms (e.g., `<::`)
}

/// Represents a quantifier expression (forall, exists).
/// Corresponds to the `quantifier_expr` rule.
#[derive(Debug, Clone, PartialEq)]
pub struct QuantifierExpr {
    pub quantifier: Quantifier, // Changed from String to Quantifier enum
    pub vars: Vec<String>,
    // Optional domain expression (`$in domain`)
    pub domain: Option<Box<Expression>>,
    // Optional body/filter expression (`: filter` or `=> body`)
    pub body: Option<Box<Expression>>,
}

/// Represents a type calculation expression like `|{ set_expr }|`.
#[derive(Debug, Clone, PartialEq)]
pub struct TypeCalculationExpr {
    pub set: Box<SetExpr>, // Changed to use Box<SetExpr>
}

/// Represents a unified declaration corresponding to the `mapping_decl` grammar rule.
/// Covers type declarations, value/function definitions, and laws.
#[derive(Debug, Clone, PartialEq)]
pub struct Declaration {
    /// The name(s) being declared (can be multiple: `a, b: T;`).
    pub names: Vec<String>,
    /// Optional type annotation (`: TypeExpr`).
    pub type_annotation: Option<TypeExpr>,
    /// Optional definition (`= Expression`).
    pub definition: Option<Expression>,
    /// Optional constraint (`| Expression`).
    pub constraint: Option<Expression>,
}
