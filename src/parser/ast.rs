use std::fmt;

/// Top-level items that can appear in a Borf program
#[derive(Debug, Clone)]
pub enum TopLevelItem {
    Module(ModuleDef),
    Primitive(PrimitiveDecl),
    Export(String),
    Import(String),
    ExpressionStatement(Expression),
}

/// A module definition (category)
#[derive(Debug, Clone)]
pub struct ModuleDef {
    pub name: String,
    pub type_param: Option<String>,
    pub elements: Vec<ModuleElement>,
}

/// Elements within a module
#[derive(Debug, Clone, PartialEq)]
pub enum ModuleElement {
    Declaration(Declaration),
}

/// A primitive declaration
#[derive(Debug, Clone)]
pub struct PrimitiveDecl {
    pub name: String,
    pub elements: Vec<PrimitiveElement>,
}

/// Elements within a primitive declaration
#[derive(Debug, Clone)]
pub enum PrimitiveElement {
    Declaration(Declaration),
}

/// A declaration within a module or primitive
#[derive(Debug, Clone, PartialEq)]
pub enum Declaration {
    /// Object declaration (e.g., "A;", or "A $in T;")
    ObjectDecl {
        name: String,
        type_constraint: Option<TypeExpr>,
    },
    /// Mapping declaration (e.g., "map: A -> B;", "map = \x.x;", "law.assoc = $forall x $in M: ...")
    MappingDecl {
        name: String,
        type_constraint: Option<TypeExpr>,
        value: Option<Expression>,
        constraint: Option<Expression>,
    },
    /// Law declaration (e.g., "law.assoc = $forall x $in M: ..." or "law.constr = $forall a in T { ... }")
    LawDecl { name: String, body: LawBody },
}

/// Represents the body of a law declaration
#[derive(Debug, Clone, PartialEq)]
pub enum LawBody {
    /// The law is defined by a single expression (possibly quantified)
    Expression(Expression),
    /// The law is defined by a block of declarations under a quantifier
    Block {
        quantifier: Quantifier,
        variable: String,             // e.g., 'a' in $forall a in T
        domain: String,               // e.g., 'T' in $forall a in T
        elements: Vec<ModuleElement>, // The declarations inside the {}
    },
}

/// Type expressions that can appear in Borf code
#[derive(Debug, Clone, PartialEq)]
pub enum TypeExpr {
    /// A simple type name (e.g., "T", "N", "Z", etc.)
    TypeName(String),
    /// A dollar-prefixed type name (e.g., "$alpha", "$rho", etc.)
    DollarName(String),
    /// A qualified type name (e.g., "Cat.O", "Mod.M", etc.)
    QualifiedName(String, String),
    /// A product type (e.g., "T * T")
    Product(Box<TypeExpr>, Box<TypeExpr>),
    /// A sum type (e.g., "T + T")
    Sum(Box<TypeExpr>, Box<TypeExpr>),
    /// A function type (e.g., "A -> B")
    Function(Box<TypeExpr>, Box<TypeExpr>),
    /// A map type (e.g., "A :-> B")
    Map(Box<TypeExpr>, Box<TypeExpr>),
    /// A set type (e.g., "{T}")
    Set(Box<TypeExpr>),
    /// A list type (e.g., "[T]")
    List(Box<TypeExpr>),
    /// A record type (e.g., "{x: A, y: B}")
    Record(Vec<(String, TypeExpr)>),
    /// An optional type (e.g., "?T")
    Optional(Box<TypeExpr>),
    /// A linear type (e.g., "!T")
    Linear(Box<TypeExpr>),
    /// A tuple type (e.g., "(A, B, C)")
    Tuple(Vec<TypeExpr>),
    /// A union type (e.g., "A $cup B")
    Union(Box<TypeExpr>, Box<TypeExpr>),
    /// An intersection type (e.g., "A $cap B")
    Intersection(Box<TypeExpr>, Box<TypeExpr>),
    /// A type range (e.g., "Z+", "R-")
    Range(String, String),
    /// A multi-parameter function (e.g., "(a,b,c)->d")
    MultiParamFunction(Vec<TypeExpr>, Box<TypeExpr>),
    /// A sequence type (e.g., "Seq T")
    Sequence(Box<TypeExpr>),
    /// A type with a constraint (e.g., "T | condition")
    Constrained(Box<TypeExpr>, Box<Expression>),
    /// A linear function type (e.g., "A -o B")
    LinearFunction(Box<TypeExpr>, Box<TypeExpr>),
    /// A type constructor application (e.g., "!Result a")
    TypeConstructor {
        is_linear: bool,      // Whether the type constructor has '!' prefix
        is_optional: bool,    // Whether the type constructor has '?' prefix
        constructor: String,  // Name of the type constructor
        param: Box<TypeExpr>, // Type parameter
    },
}

/// Expression values that can appear in Borf code
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    /// An identifier (e.g., "x", "my_value")
    Identifier(Identifier),
    /// A dollar identifier (e.g., "$x", "$alpha")
    DollarIdentifier(String),
    /// A qualified name (e.g., "Cat.id", "Mod.E")
    QualifiedName(String, String),
    /// A law identifier (e.g., "law.identity")
    LawIdentifier(String),
    /// An integer literal (e.g., "42")
    IntLiteral(i64),
    /// A boolean literal (e.g., "true", "false")
    BoolLiteral(bool),
    /// A string literal (e.g., "\"hello\"")
    StringLiteral(String),
    /// A symbol literal (e.g., ":Symbol")
    SymbolLiteral(String),
    /// An operator name (e.g., "$cup", "$in")
    OperatorName(String),
    /// A binary operation (e.g., "a + b", "x $in Y")
    BinaryOp {
        left: Box<Expression>,
        op: String,
        right: Box<Expression>,
    },
    /// A prefix operation (e.g., "$not x", "|S|")
    PrefixOp { op: String, expr: Box<Expression> },
    /// A function call (e.g., "f(x)")
    FunctionCall {
        func: Box<Expression>,
        args: Vec<Expression>,
    },
    /// A field access (e.g., "x.y")
    FieldAccess {
        base: Box<Expression>,
        field: String,
    },
    /// Index access (e.g., "arr[i]")
    IndexAccess {
        base: Box<Expression>,
        index: Box<Expression>,
    },
    /// Lambda expression (e.g., "\x.x", "\x,y.x+y")
    Lambda {
        params: Vec<String>,
        body: Box<Expression>,
    },
    /// If-then-else expression
    IfThenElse {
        condition: Box<Expression>,
        then_branch: Box<Expression>,
        else_branch: Box<Expression>,
    },
    /// Conditional expression (ternary) (e.g., "a ? b : c")
    Conditional {
        condition: Box<Expression>,
        then_expr: Box<Expression>,
        else_expr: Box<Expression>,
    },
    /// Let-rec expression
    LetRec {
        bindings: Vec<(String, Expression)>,
        body: Box<Expression>,
    },
    /// A set literal (e.g., "{a, b, c}")
    SetLiteral(Vec<Expression>),
    /// An empty set literal (e.g., "{}")
    EmptySet,
    /// An empty list literal (e.g., "[]")
    EmptyList,
    /// A set comprehension (e.g., "{x $in S | p(x)}")
    SetComprehension {
        expr: Box<Expression>,
        clauses: Vec<ComprehensionClause>,
    },
    /// A tuple expression (e.g., "(a, b, c)")
    Tuple(Vec<Expression>),
    /// A quantified expression (e.g., "$forall x $in X: p(x)")
    Quantified {
        quantifier: Quantifier,
        variables: Vec<String>,
        domain: Box<Expression>,
        optional_domain: Option<Box<Expression>>,
        body: Box<Expression>,
        constraint: Option<Box<Expression>>,
    },
    /// Module access (e.g., "x:s.O")
    ModuleAccess { var: String, module_path: String },
    /// Function chain call (e.g., "f(g(h(x)))")
    FunctionChain {
        functions: Vec<(String, Vec<Expression>)>,
    },
    /// Cardinality expression (e.g., "|S|")
    Cardinality(Box<Expression>),
    /// Type calculation (e.g., "|{x $in T}|")
    TypeCalculation(Box<Expression>),
    /// A block expression (e.g., "{ expr1; expr2; expr3 }")
    BlockExpression(Vec<Expression>),
    /// An empty sequence literal (e.g., "<>")
    EmptySequence,
    /// A fallible operation (e.g., "$seq(r1, \n. ...)")
    FallibleOp {
        op: String,            // Operation name ($seq, $alt, etc.)
        args: Vec<Expression>, // Arguments to the operation
    },
}

/// A comprehension clause in a set comprehension
#[derive(Debug, Clone, PartialEq)]
pub enum ComprehensionClause {
    /// A generator clause (e.g., "x $in S")
    Generator(String, Expression),
    /// A constraint clause (e.g., "p(x)")
    Constraint(Expression),
    /// A nested comprehension
    Nested(Box<Expression>, Vec<ComprehensionClause>),
    /// A dynamic predicate
    DynamicPredicate(Expression),
}

/// Quantifiers for quantified expressions
#[derive(Debug, Clone, PartialEq)]
pub enum Quantifier {
    /// Universal quantifier "$forall"
    Forall,
    /// Existential quantifier "$exists"
    Exists,
    /// Unique existential quantifier "$exists!"
    ExistsUnique,
}

/// A simple wrapper for identifiers
#[derive(Debug, Clone, PartialEq)]
pub struct Identifier(pub String);

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TypeExpr {
    /// Create a simple type name
    pub fn type_name(name: &str) -> Self {
        TypeExpr::TypeName(name.to_string())
    }

    /// Create a dollar-prefixed type name
    pub fn dollar_name(name: &str) -> Self {
        TypeExpr::DollarName(name.to_string())
    }

    /// Create a function type
    pub fn function(domain: TypeExpr, codomain: TypeExpr) -> Self {
        TypeExpr::Function(Box::new(domain), Box::new(codomain))
    }

    /// Create a product type
    pub fn product(left: TypeExpr, right: TypeExpr) -> Self {
        TypeExpr::Product(Box::new(left), Box::new(right))
    }

    /// Create a sum type
    pub fn sum(left: TypeExpr, right: TypeExpr) -> Self {
        TypeExpr::Sum(Box::new(left), Box::new(right))
    }

    /// Create a map type
    pub fn map(key: TypeExpr, value: TypeExpr) -> Self {
        TypeExpr::Map(Box::new(key), Box::new(value))
    }

    /// Create a set type
    pub fn set(inner: TypeExpr) -> Self {
        TypeExpr::Set(Box::new(inner))
    }

    /// Create a list type
    pub fn list(inner: TypeExpr) -> Self {
        TypeExpr::List(Box::new(inner))
    }

    /// Create a sequence type
    pub fn sequence(inner: TypeExpr) -> Self {
        TypeExpr::Sequence(Box::new(inner))
    }

    /// Create an optional type
    pub fn optional(inner: TypeExpr) -> Self {
        TypeExpr::Optional(Box::new(inner))
    }

    /// Create a linear type
    pub fn linear(inner: TypeExpr) -> Self {
        TypeExpr::Linear(Box::new(inner))
    }
}

impl Expression {
    /// Create an identifier expression
    pub fn identifier(name: &str) -> Self {
        Expression::Identifier(Identifier(name.to_string()))
    }

    /// Create a dollar identifier expression
    pub fn dollar_identifier(name: &str) -> Self {
        Expression::DollarIdentifier(name.to_string())
    }

    /// Create a binary operation expression
    pub fn binary_op(left: Expression, op: &str, right: Expression) -> Self {
        Expression::BinaryOp {
            left: Box::new(left),
            op: op.to_string(),
            right: Box::new(right),
        }
    }

    /// Create a prefix operation expression
    pub fn prefix_op(op: &str, expr: Expression) -> Self {
        Expression::PrefixOp {
            op: op.to_string(),
            expr: Box::new(expr),
        }
    }

    /// Create a function call expression
    pub fn function_call(func: Expression, args: Vec<Expression>) -> Self {
        Expression::FunctionCall {
            func: Box::new(func),
            args,
        }
    }

    /// Create a lambda expression
    pub fn lambda(params: Vec<String>, body: Expression) -> Self {
        Expression::Lambda {
            params,
            body: Box::new(body),
        }
    }

    /// Create an if-then-else expression
    pub fn if_then_else(
        condition: Expression,
        then_branch: Expression,
        else_branch: Expression,
    ) -> Self {
        Expression::IfThenElse {
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_branch: Box::new(else_branch),
        }
    }

    /// Create a conditional (ternary) expression
    pub fn conditional(
        condition: Expression,
        then_expr: Expression,
        else_expr: Expression,
    ) -> Self {
        Expression::Conditional {
            condition: Box::new(condition),
            then_expr: Box::new(then_expr),
            else_expr: Box::new(else_expr),
        }
    }

    /// Create a set literal expression
    pub fn set_literal(elements: Vec<Expression>) -> Self {
        Expression::SetLiteral(elements)
    }

    /// Create a quantified expression
    pub fn quantified(
        quantifier: Quantifier,
        variables: Vec<String>,
        domain: Expression,
        body: Expression,
    ) -> Self {
        Expression::Quantified {
            quantifier,
            variables,
            domain: Box::new(domain),
            optional_domain: None,
            body: Box::new(body),
            constraint: None,
        }
    }
}
