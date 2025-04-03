//! Abstract Syntax Tree definitions for the Borf language.
//!
//! This module contains the type definitions that make up the AST for Borf,
//! including top-level items, category elements, constraints, and expressions.

/// Represents a top-level item in the Borf program.
///
/// Top-level items include category definitions, pipeline definitions,
/// pipe expressions, application expressions, composition expressions,
/// and export/import directives.
#[derive(Debug, Clone)]
pub enum TopLevelItem {
    /// A category definition with objects, mappings, laws, and other elements
    Category(CategoryDef),
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
    /// Name of the mapping
    pub name: String,
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
    /// Standard mapping relation ($to)
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
}

/// Represents a law within a category.
///
/// Laws define mathematical properties and constraints that should be
/// upheld by objects and mappings within a category. They can be expressed
/// as composition laws, universal quantifications (forall), or existential
/// quantifications (exists).
#[derive(Debug, Clone)]
pub enum Law {
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
#[derive(Debug, Clone)]
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
}

/// Represents a condition in a set comprehension.
///
/// Set conditions specify constraints on the elements in a set comprehension.
#[derive(Debug, Clone)]
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
    pub body: String,
}
