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
    Category(CategoryDef),
    /// A primitive block definition
    Primitive(PrimitiveDef), // Added for primitive_declaration
    /// A pipeline definition with input type, output type, and transformation steps
    Pipeline(PipelineDef),
    /// A pipe expression representing a sequence of transformations
    PipeExpr(PipeExpr),
    /// An application expression applying a function to an argument
    AppExpr(AppExpr),
    /// A composition expression combining multiple functions
    CompositionExpr(CompositionExpr),
    /// An export directive specifying which elements to export
    Export(ExportDirective),
    /// An import directive for including external modules
    Import(ImportDirective),
    /// A standalone expression statement
    ExpressionStatement(Expression),
}

/// Represents a category definition in the Borf language.
///
/// A category is a fundamental structure in Borf that encapsulates
/// objects, mappings between them, and laws that govern their behavior.
/// Categories can optionally derive from a base category.
#[derive(Debug, Clone)]
pub struct CategoryDef {
    /// The name of the category
    pub name: String,
    /// Optional base category that this category derives from
    pub base_category: Option<String>,
    /// Collection of elements contained in this category
    pub elements: Vec<CategoryElement>,
}

/// Represents elements within a category definition.
///
/// Categories can contain various elements: objects, mappings,
/// laws, structure mappings, and function definitions.
#[derive(Debug, Clone)]
pub enum CategoryElement {
    /// A declaration of one or more objects
    ObjectDecl(ObjectDecl),
    /// A mapping between objects or sets
    MappingDecl(MappingDecl),
    /// A law defining constraints or properties
    LawDecl(Law),
    /// A structural mapping assigning a value to a name
    StructureMapping(StructureMapping),
    /// A function definition with parameters and body
    FunctionDef(FunctionDef),
    /// A constraint declared directly within a category
    ConstraintDecl(Constraint),
}

/// Represents an object declaration within a category.
///
/// Objects are the basic elements in a category. They can represent types,
/// entities, or any element that can be related through mappings.
#[derive(Debug, Clone)]
pub struct ObjectDecl {
    /// List of object names declared
    pub names: Vec<String>,
}

/// Represents a mapping declaration between objects or sets.
///
/// Mappings define relationships between objects in a category,
/// specifying domains, codomains, and the type of mapping.
#[derive(Debug, Clone)]
pub struct MappingDecl {
    /// Name(s) of the mapping (can be multiple comma-separated)
    pub names: Vec<String>,
    /// Domain (source) of the mapping
    pub domain: String,
    /// Type of the domain (simple or set comprehension)
    pub domain_type: DomainType,
    /// Type of mapping relationship
    pub mapping_type: MappingType,
    /// Codomain (target) of the mapping
    pub codomain: String, // Can be an identifier or a set literal string
}

/// Specifies the type of domain in a mapping declaration.
#[derive(Debug, Clone, PartialEq)]
pub enum DomainType {
    /// A simple domain consisting of a single identifier
    Simple,
    /// A set comprehension defining a domain through conditions
    SetComprehension, // A set comprehension like {f $in Hom, g $in Hom | cod(f) = dom(g)}
}

/// Specifies the type of relationship in a mapping.
#[derive(Debug, Clone, PartialEq)]
pub enum MappingType {
    /// Standard mapping relation (->)
    To,
    /// Subset relation ($subseteq)
    Subseteq,
    /// Bidirectional mapping (<->)
    Bidirectional,
    /// Product relation (*)
    Times,
    /// Type equivalence ($teq)
    TypeEquiv,
    /// Value equality ($veq)
    ValueEquiv,
    /// Structural equivalence ($seq)
    StructEquiv,
    /// Categorical equivalence ($ceq)
    CatEquiv,
    /// Compatibility relation ($omega)
    Compatibility,
    /// Standard function arrow (->)
    FunctionArrow,
}

/// Represents a named law within a category.
///
/// Laws must have a name (e.g., law.refl) and a definition.
#[derive(Debug, Clone)]
pub struct Law {
    /// The name of the law (e.g., "refl")
    pub name: String,
    /// The definition of the law (forall, exists, composition)
    pub definition: LawDefinition,
}

/// Represents the definition of a law within a category.
///
/// Law definitions specify the actual logical content of a law,
/// separate from its name.
#[derive(Debug, Clone)]
pub enum LawDefinition {
    /// A composition law specifying how functions compose
    Composition {
        /// Left-hand side of the composition
        lhs: String,
        /// Composition operator (typically "$comp")
        op: String,
        /// Middle term in the composition
        middle: String,
        /// Right-hand side of the composition
        rhs: String, // Now using === instead of .equiv
    },
    /// Universal quantification (∀) expressing a property that holds for all elements
    ForAll {
        /// Variables bound by the quantifier
        vars: Vec<String>,
        /// Domain of the quantification
        domain: String,
        /// Constraint that must be satisfied
        constraint: Constraint,
    },
    /// Existential quantification (∃) expressing that at least one element exists
    Exists {
        /// Variables bound by the quantifier
        vars: Vec<String>,
        /// Domain of the quantification
        domain: String,
        /// Constraint that must be satisfied
        constraint: Constraint,
    },
    /// A law defined directly by a constraint expression
    Constraint(Constraint),
}

/// Represents a constraint or condition in a law.
///
/// Constraints express relationships between terms, such as equality,
/// logical combinations, inequalities, and various forms of equivalence.
#[derive(Debug, Clone)]
pub enum Constraint {
    /// Equality constraint (a = b)
    Equality {
        /// Left-hand side of the equality
        lhs: Box<ConstraintExpr>,
        /// Right-hand side of the equality
        rhs: Box<ConstraintExpr>,
    },
    /// Logical AND constraint (a $and b)
    LogicalAnd {
        /// Left-hand side of the logical AND
        lhs: Box<ConstraintExpr>,
        /// Right-hand side of the logical AND
        rhs: Box<ConstraintExpr>,
    },
    /// Greater-than-or-equal constraint (a >= b)
    GreaterThanEqual {
        /// Left-hand side of the comparison
        lhs: Box<ConstraintExpr>,
        /// Right-hand side of the comparison
        rhs: Box<ConstraintExpr>,
    },
    /// Greater-than constraint (a > b)
    GreaterThan {
        /// Left-hand side of the comparison
        lhs: Box<ConstraintExpr>,
        /// Right-hand side of the comparison
        rhs: Box<ConstraintExpr>,
    },
    /// Less-than-or-equal constraint (a <= b)
    LessThanEqual {
        /// Left-hand side of the comparison
        lhs: Box<ConstraintExpr>,
        /// Right-hand side of the comparison
        rhs: Box<ConstraintExpr>,
    },
    /// Less-than constraint (a < b)
    LessThan {
        /// Left-hand side of the comparison
        lhs: Box<ConstraintExpr>,
        /// Right-hand side of the comparison
        rhs: Box<ConstraintExpr>,
    },
    /// Logical implication (a => b)
    Implies {
        /// Premise (condition)
        lhs: Box<ConstraintExpr>,
        /// Conclusion
        rhs: Box<ConstraintExpr>,
    },
    /// Type equivalence (a $teq b)
    TypeEquiv {
        /// Left-hand side of the type equivalence
        lhs: Box<ConstraintExpr>,
        /// Right-hand side of the type equivalence
        rhs: Box<ConstraintExpr>,
    },
    /// Value equivalence (a $veq b)
    ValueEquiv {
        /// Left-hand side of the value equivalence
        lhs: Box<ConstraintExpr>,
        /// Right-hand side of the value equivalence
        rhs: Box<ConstraintExpr>,
    },
    /// Structural equivalence (a $seq b)
    StructuralEquiv {
        /// Left-hand side of the structural equivalence
        lhs: Box<ConstraintExpr>,
        /// Right-hand side of the structural equivalence
        rhs: Box<ConstraintExpr>,
    },
    /// Categorical equivalence (a $ceq b)
    CategoricalEquiv {
        /// Left-hand side of the categorical equivalence
        lhs: Box<ConstraintExpr>,
        /// Right-hand side of the categorical equivalence
        rhs: Box<ConstraintExpr>,
    },
    /// Compatibility relation (a $omega b)
    Compatibility {
        /// Left-hand side of the compatibility relation
        lhs: Box<ConstraintExpr>,
        /// Right-hand side of the compatibility relation
        rhs: Box<ConstraintExpr>,
    },
}

/// Represents an expression used in a constraint.
///
/// Constraint expressions can be integers, identifiers, set expressions,
/// function applications, or symbol literals.
#[derive(Debug, Clone)]
pub enum ConstraintExpr {
    /// An integer literal
    Integer(i64),
    /// An identifier (variable or constant name)
    Identifier(String),
    /// A set expression (comprehension or Cartesian product)
    SetExpr(SetExpr),
    /// A function application (func(arg))
    FunctionApp {
        /// The function name
        func: String,
        /// The argument to the function
        arg: String,
    },
    /// A symbol literal (like :Type)
    Symbol(String),
}

/// Represents a set expression in a constraint.
///
/// Set expressions can be either set comprehensions or Cartesian products.
#[derive(Debug, Clone, PartialEq)]
pub enum SetExpr {
    /// A set comprehension {elements | condition}
    Comprehension {
        /// Elements in the set
        elements: Vec<String>,
        /// Optional condition for the set comprehension
        condition: Option<SetCondition>,
    },
    /// A Cartesian product of two sets (A × B)
    CartesianProduct {
        /// Left-hand set in the product
        lhs: String,
        /// Right-hand set in the product
        rhs: String,
    },
    /// A literal set defined by enumerating elements: `{elem1, elem2, ...}`
    Literal(SetLiteral),
    /// A set defined by comprehension: `{ var $in domain | condition }`
    NewComprehension(SetComprehension),
    /// A set defined by an operation on other sets: `set1 $op set2`
    Operation(SetOperation),
}

/// Represents a condition in a set comprehension.
///
/// Set conditions specify constraints on the elements in a set comprehension.
#[derive(Debug, Clone, PartialEq)]
pub struct SetCondition {
    /// First function in the condition
    pub func1: String,
    /// Argument to the first function
    pub arg1: String,
    /// Optional second function in a compound condition
    pub func2: Option<String>,
    /// Optional argument to the second function
    pub arg2: Option<String>,
}

/// Represents a pipe expression in the Borf language.
///
/// Pipe expressions represent a sequence of transformations applied to a starting value.
#[derive(Debug, Clone)]
pub struct PipeExpr {
    /// The starting identifier or value
    pub start: String,
    /// The sequence of transformation steps to apply
    pub steps: Vec<String>,
}

/// Represents an application expression in the Borf language.
///
/// Application expressions represent applying a function to an argument.
#[derive(Debug, Clone)]
pub struct AppExpr {
    /// The function being applied
    pub func: String,
    /// The argument to which the function is applied
    pub arg: Box<AppExprArg>,
}

/// Represents an argument in an application expression.
///
/// Arguments can be either simple identifiers or nested application expressions.
#[derive(Debug, Clone)]
pub enum AppExprArg {
    /// A simple identifier argument
    Identifier(String),
    /// A nested application expression as an argument
    AppExpr(Box<AppExpr>),
}

/// Represents a composition expression in the Borf language.
///
/// Composition expressions represent combining multiple functions and applying
/// the resulting composed function to an argument.
#[derive(Debug, Clone)]
pub struct CompositionExpr {
    /// The identifier to which the composition result is assigned
    pub result: String,
    /// The sequence of functions to compose (applied right-to-left)
    pub functions: Vec<String>,
    /// The argument to which the composed function is applied
    pub arg: String,
}

/// Represents a pipeline definition in the Borf language.
///
/// Pipeline definitions specify a sequence of transformation steps
/// from an input type to an output type.
#[derive(Debug, Clone)]
pub struct PipelineDef {
    /// The name of the pipeline
    pub name: String,
    /// Optional type parameter for generic pipelines
    pub type_param: Option<String>,
    /// The input type that the pipeline accepts
    pub input_type: String,
    /// The output type that the pipeline produces
    pub output_type: String,
    /// The sequence of transformation steps in the pipeline
    pub steps: Vec<String>,
}

/// Represents an export directive in the Borf language.
///
/// Export directives specify which identifiers are exported from a module.
#[derive(Debug, Clone)]
pub struct ExportDirective {
    /// The identifiers to export
    pub identifiers: Vec<String>,
}

/// Represents an import directive in the Borf language.
///
/// Import directives specify which modules are imported.
#[derive(Debug, Clone)]
pub struct ImportDirective {
    /// The path to the module to import
    pub path: String,
}

/// Represents a structure mapping in the Borf language.
///
/// Structure mappings assign expressions to names.
#[derive(Debug, Clone)]
pub struct StructureMapping {
    /// The name to which the value is assigned
    pub lhs: String,
    /// The expression or value being assigned
    pub rhs: ExpressionType,
}

/// Represents different types of expressions in the Borf language.
///
/// Expressions can be simple identifiers, function applications,
/// set comprehensions, disjoint unions, match expressions, or composite expressions.
#[derive(Debug, Clone)]
pub enum ExpressionType {
    /// A simple identifier or literal
    Simple(String),
    /// A function application with arguments
    FunctionApp(String, Vec<String>),
    /// A set comprehension expression
    SetComprehension(String),
    /// A disjoint union (A + B)
    DisjointUnion(String, String),
    /// A match expression with cases
    Match(String, Vec<(String, String, String)>),
    /// A complex expression that can't be fully parsed yet
    Composite(String),
    /// A symbol literal
    Symbol(String),
}

/// Represents a function definition in the Borf language.
///
/// Function definitions specify the name, parameters, and body of a function.
#[derive(Debug, Clone)]
pub struct FunctionDef {
    /// The name of the function
    pub name: String,
    /// The parameters of the function
    pub params: Vec<String>,
    /// The body expression of the function
    pub body: Expression,
}

// --- Set Expression Related Types ---

/// Represents a literal set enumeration.
#[derive(Debug, Clone, PartialEq)]
pub struct SetLiteral {
    /// The elements of the set.
    /// Using Expression for now, as elements can be idents, ints, symbols, tuples according to grammar.
    pub elements: Vec<Expression>,
}

/// Represents a set comprehension.
#[derive(Debug, Clone, PartialEq)]
pub struct SetComprehension {
    /// The variable being bound.
    pub variable: String,
    /// The domain set the variable comes from.
    pub domain: String,
    /// The optional condition filtering the elements.
    /// Using Expression for generality, though grammar uses constraint_expr.
    /// Needs refinement based on actual constraint parsing.
    pub condition: Option<Box<Expression>>,
}

/// Represents a binary operation on sets.
#[derive(Debug, Clone, PartialEq)]
pub struct SetOperation {
    /// Left-hand side set identifier.
    pub lhs: String,
    /// The set operator (e.g., $cup, $cap, $in).
    pub op: String,
    /// Right-hand side set identifier.
    pub rhs: String,
}

// --- Expression AST Nodes (Lambda, If, LetRec, Composition) ---

#[derive(Debug, Clone, PartialEq)]
pub struct LambdaExpr {
    pub params: Vec<String>,
    pub body: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfExpr {
    pub condition: Box<Expression>,
    pub then_branch: Box<Expression>,
    pub else_branch: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LetRecExpr {
    pub name: String,
    pub params: Vec<String>,
    pub bound_expr: Box<Expression>,
    pub body: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct CompositionExprRhs {
    // Renamed to avoid conflict with TopLevelItem::CompositionExpr
    pub left: String, // Assuming simple identifiers for now based on grammar: ident comp_op ident
    pub op: String,
    pub right: String,
}

/// Represents the different forms an expression can take based on the grammar.
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    AtomExpr(Atom), // Renamed variant slightly to avoid name clash if Atom type is also named Atom
    Lambda(LambdaExpr),
    If(IfExpr),
    LetRec(LetRecExpr),
    Quantifier(QuantifierExpr), // Added for quantifier expressions
    TypeCalculation(TypeCalculationExpr), // Added for |{...}|
    FunctionChainCall(FunctionChainExpr), // Added for nested function chains

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
        if_false: Box<Expression>,
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
    LawIdentifier {
        name: String, // For law.name identifiers
    },
    EmptySet, // Added for empty set literals {}
}

/// Represents function chain expression (nested function calls)
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionChainExpr {
    pub base: String,                // Base function name or qualified name
    pub calls: Vec<Vec<Expression>>, // Sequence of argument lists for each nested call
}

/// Represents a quantifier.
#[derive(Debug, Clone, PartialEq)]
pub enum Quantifier {
    ForAll,
    Exists,
    ExistsUnique, // New: $exists!
}

/// Represents nested set comprehensions
#[derive(Debug, Clone, PartialEq)]
pub struct NestedComprehension {
    pub expr: Box<Expression>,
    pub clauses: Vec<ComprehensionClause>,
}

/// Represents a comprehension clause in a set comprehension
#[derive(Debug, Clone, PartialEq)]
pub enum ComprehensionClause {
    Generator {
        var: String,
        domain: Box<Expression>,
    },
    Constraint(Box<Expression>),
    Nested(Box<NestedComprehension>),
}

/// Represents a primitive block definition.
#[derive(Debug, Clone)]
pub struct PrimitiveDef {
    pub name: String,
    pub elements: Vec<PrimitiveElement>, // Assuming elements are like category elements for now
}

/// Represents elements within a primitive block.
/// Currently using MappingDecl, adjust if primitives have different structure.
#[derive(Debug, Clone)]
pub enum PrimitiveElement {
    Mapping(MappingDecl),
    // Add other primitive element types if necessary
}

/// Represents a parsed type expression.
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
    /// A specialized arrow type with transitive closure.
    TypeArrowEx {
        lhs: Box<TypeExpr>,
        rhs: Box<TypeExpr>,
        transitive: bool, // true for ->+, false for ->
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
        params: Vec<TypeExpr>,
        return_type: Box<TypeExpr>,
        constraint: Option<Box<Expression>>,
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
}

/// Represents the atomic building blocks of expressions.
#[derive(Debug, Clone, PartialEq)]
pub enum Atom {
    Identifier(String),
    DollarIdentifier(String), // Added for $identifiers
    Integer(i64),
    Boolean(bool), // Added Boolean
    Symbol(String),
    Tuple(Vec<Expression>), // Allow variable length tuples
    Set(Box<SetExpr>),      // Added Set variant
    EmptySet,               // Empty set literal {}
    Paren(Box<Expression>), // Parenthesized expression
    StringLiteral(String),  // For `"ident"` in grammar
    LawIdentifier(String),  // For law.name identifiers
}

/// Placeholder for Quantifier expressions
#[derive(Debug, Clone, PartialEq)]
pub struct QuantifierExpr {
    pub quantifier: Quantifier, // Changed from String to Quantifier enum
    pub vars: Vec<String>,
    pub domain: Box<Expression>,
    pub condition: Option<Box<Expression>>, // Constraint expression
    pub body: Box<Expression>,
}

/// Placeholder for Type Calculation expressions |{...}|
#[derive(Debug, Clone, PartialEq)]
pub struct TypeCalculationExpr {
    pub set: SetExpr, // Contains the inner set expression
}
