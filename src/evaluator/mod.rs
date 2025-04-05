use crate::parser::ast::{self, Declaration, Expr, Literal, Pattern, SmallVec8, TypeExpr};
use rustc_hash::FxHashMap;
use smallvec::smallvec;
use std::cell::RefCell;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use thiserror::Error;

/// The Borf value type representing values in the language
#[derive(Clone, Debug)]
pub enum Value {
    Integer(i64),
    Float(f64),
    String(Rc<String>), // Use Rc for efficient string sharing
    Boolean(bool),
    Symbol(Rc<String>),          // Symbols are distinct from strings at runtime
    List(Box<SmallVec8<Value>>), // Box the collection
    #[allow(clippy::mutable_key_type)] // Allow Value as key, checked at runtime
    Map(Box<Rc<FxHashMap<Value, Value>>>), // Box the Rc containing the map
    #[allow(clippy::mutable_key_type)] // Allow Value as key, checked at runtime
    Set(Box<Rc<FxHashMap<Value, ()>>>), // Box the Rc containing the set
    Function(Box<BorfFunction>), // Box the function
    Module(String),              // Just store the module name for now
    Quote(Box<Rc<Expr>>),        // Box the Rc containing the Expr
    Null,
    Void, // Representing the absence of a value (distinct from Null?)
}

/// Function representation
#[derive(Debug, Clone)]
pub enum BorfFunction {
    Native(String), // Reference to a native function by name
    Closure {
        params: Vec<Pattern>, // Parameter patterns from the lambda AST
        body: Rc<Expr>,       // Use Rc<Expr> for shared body
        env: EnvironmentRef,  // Use EnvironmentRef (Rc<RefCell<Environment>>)
    },
}

// Custom PartialEq implementation for BorfFunction that ignores environment comparison
impl PartialEq for BorfFunction {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (BorfFunction::Native(a), BorfFunction::Native(b)) => a == b,
            (
                BorfFunction::Closure {
                    params: p1,
                    body: b1,
                    ..
                },
                BorfFunction::Closure {
                    params: p2,
                    body: b2,
                    ..
                },
            ) => p1 == p2 && **b1 == **b2, // Compare patterns and dereferenced Expr
            _ => false,
        }
    }
}

/// Environment for evaluation
pub type EnvironmentRef = Rc<RefCell<Environment>>;

#[derive(Debug, Clone, Default)]
pub struct Environment {
    // Use FxHashMap for potentially better performance
    bindings: FxHashMap<String, Value>,
    // Parent environment for lexical scoping using the type alias
    outer: Option<EnvironmentRef>,
}

impl Environment {
    /// Create a new empty environment.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new, empty global environment.
    pub fn new_global() -> EnvironmentRef {
        Rc::new(RefCell::new(Environment {
            bindings: FxHashMap::default(),
            outer: None,
        }))
    }

    /// Creates a new environment that extends an outer one.
    pub fn new_extending(outer: EnvironmentRef) -> EnvironmentRef {
        Rc::new(RefCell::new(Environment {
            bindings: FxHashMap::default(),
            outer: Some(outer),
        }))
    }

    /// Define a variable in the *current* environment frame.
    pub fn define(&mut self, name: String, value: Value) {
        self.bindings.insert(name, value);
    }

    /// Looks up a variable, searching outwards through enclosing environments.
    pub fn lookup(&self, name: &str) -> Option<Value> {
        if let Some(value) = self.bindings.get(name) {
            Some(value.clone()) // Clone the value to return ownership
        } else if let Some(outer_env) = &self.outer {
            outer_env.borrow().lookup(name) // Recursively search outer scope
        } else {
            None // Not found in any scope
        }
    }

    /// Sets the value of an *existing* variable in the nearest environment where it's defined.
    /// Returns true if the variable was found and set, false otherwise.
    pub fn set(&mut self, name: &str, value: Value) -> bool {
        if self.bindings.contains_key(name) {
            self.bindings.insert(name.to_string(), value);
            true
        } else if let Some(outer_env) = &self.outer {
            outer_env.borrow_mut().set(name, value) // Try setting in outer scope
        } else {
            false // Variable not found to set
        }
    }
}

/// The evaluator for Borf code
pub struct Evaluator {
    pub global_env: EnvironmentRef, // Make field public
}

impl Evaluator {
    /// Create a new evaluator with basic Primitives and prelude loaded.
    pub fn new() -> Self {
        // Create the initial global environment using Rc<RefCell>
        let global_env = Environment::new_global();

        // Populate with core primitives *first* so prelude files can use them
        populate_global_env(Rc::clone(&global_env));

        // Call the helper function from lib.rs to load prelude files
        match crate::process_prelude_directory_internal("src/prelude", Rc::clone(&global_env)) {
            Ok(_) => println!("Prelude directory processed successfully."),
            Err(e) => {
                eprintln!("Error processing prelude directory: {}", e);
                // Decide how to handle prelude errors (e.g., continue without prelude?)
            }
        }

        Self { global_env }
    }

    /// Evaluate a module
    pub fn evaluate_module(&self, module: &ast::Module) -> Result<Value, EvalError> {
        // Create an environment for this module, inheriting from the *final* global scope
        let module_env = Environment::new_extending(Rc::clone(&self.global_env));

        // Evaluate each declaration within the module
        for decl in &module.declarations {
            // Use the standalone evaluate_declaration helper, passing the module env
            evaluate_declaration(decl, Rc::clone(&module_env))?;
        }
        Ok(Value::Module(module.name.clone()))
    }

    // NOTE: evaluate_expr, evaluate_literal, apply_function, and bind_pattern
    // are defined as standalone functions later in this file and used by evaluate_declaration.
    // The method versions below were redundant and are now removed.
}

// --- Evaluation Error ---
#[derive(Debug, Error, Clone, PartialEq)]
pub enum EvalError {
    #[error("Unbound variable: {0}")]
    UnboundVariable(String),
    #[error("Type error: Expected {expected}, found {found_type} ({value})")]
    TypeError {
        expected: String,
        found_type: String,
        value: String, // Include string representation of the problematic value
    },
    #[error("Arity mismatch: {context} expected {expected} arguments, found {found}")]
    ArityMismatch {
        context: String,  // e.g., function name or operation
        expected: String, // Can be specific number or range "at least N"
        found: usize,
    },
    #[error("Invalid function application: {0} is not a function")]
    NotAFunction(String), // Value represented as String
    #[error("Division by zero")]
    DivisionByZero,
    #[error("Invalid arguments for {context}: {message}")]
    InvalidArguments { context: String, message: String },
    #[error("Pattern match failed for value: {value} against pattern {pattern}")]
    PatternMatchFailed { value: String, pattern: String },
    #[error("Cannot set undefined variable: {0}")]
    CannotSetUndefined(String),
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    #[error("Quasiquote error: {0}")]
    QuasiquoteError(String),
    #[error("Prelude loading error: {0}")]
    PreludeError(String),
    #[error("Parse error during evaluation: {0}")]
    // For errors originating from parser called by eval
    ParseError(String),
    #[error("Hash error: Cannot use value {value} of type {type_name} as hash key")]
    HashingError { value: String, type_name: String },
    #[error("Complex type mismatch: {message}")]
    ComplexTypeMismatch { message: String },
}

// Add Default impl as suggested by clippy
impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}

// --- Initialization & Native Functions (Standalone Functions) ---

// Dispatcher for native function calls (remains standalone)
fn apply_native_function(name: &str, args: Vec<Value>) -> Result<Value, EvalError> {
    // Macro to check arity and types
    macro_rules! check_arity {
        ($fn_name:expr_2021, $args:expr_2021, $expected:expr_2021) => {
            if $args.len() != $expected {
                return Err(EvalError::ArityMismatch {
                    context: $fn_name.to_string(),
                    expected: $expected.to_string(),
                    found: $args.len(),
                });
            }
        };
        ($fn_name:expr_2021, $args:expr_2021, $min:expr_2021, $max:expr_2021) => {
            if $args.len() < $min || $args.len() > $max {
                return Err(EvalError::ArityMismatch {
                    context: $fn_name.to_string(),
                    expected: format!("between {} and {}", $min, $max),
                    found: $args.len(),
                });
            }
        };
    }

    macro_rules! expect_type {
        ($arg:expr_2021, $pattern:pat => $extracted_value:expr_2021, $fn_name:expr_2021, $arg_num:expr_2021, $expected_type:expr_2021) => {
            match $arg {
                $pattern => $extracted_value,
                other => {
                    return Err(EvalError::TypeError {
                        expected: $expected_type.to_string(),
                        found_type: other.type_name().to_string(),
                        value: format!("{}", other),
                    })
                }
            }
        };
    }

    match name {
        "print" => {
            for (i, arg) in args.iter().enumerate() {
                print!("{}{}", if i > 0 { " " } else { "" }, arg); // Use Display
            }
            println!();
            Ok(Value::Void)
        }
        "+" => {
            check_arity!("+", args, 2);
            match (&args[0], &args[1]) {
                (Value::Integer(a), Value::Integer(b)) => {
                    Ok(Value::Integer(a.checked_add(*b).ok_or_else(|| {
                        EvalError::InvalidOperation("Integer overflow in +".to_string())
                    })?))
                }
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
                (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a + (*b as f64))),
                (Value::Integer(a), Value::Float(b)) => Ok(Value::Float((*a as f64) + b)),
                (Value::String(a), Value::String(b)) => {
                    Ok(Value::String(Rc::new(format!("{}{}", a, b))))
                } // Wrap in Rc
                _ => Err(EvalError::InvalidArguments {
                    context: "+".to_string(),
                    message: format!(
                        "Unsupported types: {}, {}",
                        args[0].type_name(),
                        args[1].type_name()
                    ),
                }),
            }
        }
        "-" => {
            check_arity!("-", args, 2);
            match (&args[0], &args[1]) {
                (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a - b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
                (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a - (*b as f64))),
                (Value::Integer(a), Value::Float(b)) => Ok(Value::Float((*a as f64) - b)),
                _ => Err(EvalError::InvalidArguments {
                    context: "-".to_string(),
                    message: format!(
                        "Unsupported types: {}, {}",
                        args[0].type_name(),
                        args[1].type_name()
                    ),
                }),
            }
        }
        "*" => {
            check_arity!("*", args, 2);
            match (&args[0], &args[1]) {
                (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a * b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
                (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a * (*b as f64))),
                (Value::Integer(a), Value::Float(b)) => Ok(Value::Float((*a as f64) * b)),
                _ => Err(EvalError::InvalidArguments {
                    context: "*".to_string(),
                    message: format!(
                        "Unsupported types: {}, {}",
                        args[0].type_name(),
                        args[1].type_name()
                    ),
                }),
            }
        }
        "/" => {
            check_arity!("/", args, 2);
            match (&args[0], &args[1]) {
                (_, Value::Integer(0)) => Err(EvalError::DivisionByZero),
                (_, Value::Float(f)) if f.abs() < f64::EPSILON => Err(EvalError::DivisionByZero),
                (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a / b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a / b)),
                (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a / (*b as f64))),
                (Value::Integer(a), Value::Float(b)) => Ok(Value::Float((*a as f64) / b)),
                _ => Err(EvalError::InvalidArguments {
                    context: "/".to_string(),
                    message: format!(
                        "Unsupported types: {}, {}",
                        args[0].type_name(),
                        args[1].type_name()
                    ),
                }),
            }
        }
        "==" => {
            check_arity!("==", args, 2);
            Ok(Value::Boolean(args[0] == args[1])) // Use derived PartialEq
        }
        "!=" => {
            check_arity!("!=", args, 2);
            Ok(Value::Boolean(args[0] != args[1])) // Use derived PartialEq
        }
        "<" => {
            check_arity!("<", args, 2);
            match (&args[0], &args[1]) {
                (Value::Integer(a), Value::Integer(b)) => Ok(Value::Boolean(a < b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Boolean(a < b)),
                (Value::Float(a), Value::Integer(b)) => Ok(Value::Boolean(a < &(*b as f64))),
                (Value::Integer(a), Value::Float(b)) => Ok(Value::Boolean(&(*a as f64) < b)),
                (Value::String(a), Value::String(b)) => Ok(Value::Boolean(a < b)),
                _ => Err(EvalError::InvalidArguments {
                    context: "<".to_string(),
                    message: format!(
                        "Cannot compare types: {}, {}",
                        args[0].type_name(),
                        args[1].type_name()
                    ),
                }),
            }
        }
        ">" => {
            check_arity!(">", args, 2);
            match (&args[0], &args[1]) {
                (Value::Integer(a), Value::Integer(b)) => Ok(Value::Boolean(a > b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Boolean(a > b)),
                (Value::Float(a), Value::Integer(b)) => Ok(Value::Boolean(a > &(*b as f64))),
                (Value::Integer(a), Value::Float(b)) => Ok(Value::Boolean(&(*a as f64) > b)),
                (Value::String(a), Value::String(b)) => Ok(Value::Boolean(a > b)),
                _ => Err(EvalError::InvalidArguments {
                    context: ">".to_string(),
                    message: format!(
                        "Cannot compare types: {}, {}",
                        args[0].type_name(),
                        args[1].type_name()
                    ),
                }),
            }
        }
        "<=" => {
            check_arity!("<=", args, 2);
            match (&args[0], &args[1]) {
                (Value::Integer(a), Value::Integer(b)) => Ok(Value::Boolean(a <= b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Boolean(a <= b)),
                (Value::Float(a), Value::Integer(b)) => Ok(Value::Boolean(a <= &(*b as f64))),
                (Value::Integer(a), Value::Float(b)) => Ok(Value::Boolean(&(*a as f64) <= b)),
                (Value::String(a), Value::String(b)) => Ok(Value::Boolean(a <= b)),
                _ => Err(EvalError::InvalidArguments {
                    context: "<=".to_string(),
                    message: format!(
                        "Cannot compare types: {}, {}",
                        args[0].type_name(),
                        args[1].type_name()
                    ),
                }),
            }
        }
        ">=" => {
            check_arity!(">=", args, 2);
            match (&args[0], &args[1]) {
                (Value::Integer(a), Value::Integer(b)) => Ok(Value::Boolean(a >= b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Boolean(a >= b)),
                (Value::Float(a), Value::Integer(b)) => Ok(Value::Boolean(a >= &(*b as f64))),
                (Value::Integer(a), Value::Float(b)) => Ok(Value::Boolean(&(*a as f64) >= b)),
                (Value::String(a), Value::String(b)) => Ok(Value::Boolean(a >= b)),
                _ => Err(EvalError::InvalidArguments {
                    context: ">=".to_string(),
                    message: format!(
                        "Cannot compare types: {}, {}",
                        args[0].type_name(),
                        args[1].type_name()
                    ),
                }),
            }
        }
        "and" => {
            check_arity!("and", args, 2);
            let left = expect_type!(&args[0], Value::Boolean(b) => *b, "and", 1, "Boolean");
            let right = expect_type!(&args[1], Value::Boolean(b) => *b, "and", 2, "Boolean");
            Ok(Value::Boolean(left && right))
        }
        "or" => {
            check_arity!("or", args, 2);
            let left = expect_type!(&args[0], Value::Boolean(b) => *b, "or", 1, "Boolean");
            let right = expect_type!(&args[1], Value::Boolean(b) => *b, "or", 2, "Boolean");
            Ok(Value::Boolean(left || right))
        }
        "not" => {
            check_arity!("not", args, 1);
            let val = expect_type!(&args[0], Value::Boolean(b) => *b, "not", 1, "Boolean");
            Ok(Value::Boolean(!val))
        }
        "mod" => {
            check_arity!("mod", args, 2);
            let a = expect_type!(&args[0], Value::Integer(i) => *i, "mod", 1, "Integer");
            let b = expect_type!(&args[1], Value::Integer(i) => *i, "mod", 2, "Integer");
            if b == 0 {
                Err(EvalError::DivisionByZero)
            } else {
                Ok(Value::Integer(a % b))
            }
        }
        "cons" => {
            check_arity!("cons", args, 2);
            let head = args[0].clone();
            let tail = match &args[1] {
                Value::List(boxed_list) => (**boxed_list).clone(),
                other => {
                    return Err(EvalError::TypeError {
                        expected: "List".to_string(),
                        found_type: other.type_name().to_string(),
                        value: format!("{}", other),
                    })
                }
            };
            let mut new_list = SmallVec8::new();
            new_list.push(head);
            new_list.extend(tail);
            Ok(Value::List(Box::new(new_list)))
        }
        "car" => {
            check_arity!("car", args, 1);
            let list = expect_type!(&args[0], Value::List(l) => l, "car", 1, "List");
            if list.is_empty() {
                Err(EvalError::InvalidArguments {
                    context: "car".to_string(),
                    message: "Cannot take car of empty list".to_string(),
                })
            } else {
                Ok(list[0].clone())
            }
        }
        "cdr" => {
            check_arity!("cdr", args, 1);
            let list = match &args[0] {
                Value::List(boxed_list) => &**boxed_list,
                other => {
                    return Err(EvalError::TypeError {
                        expected: "List".to_string(),
                        found_type: other.type_name().to_string(),
                        value: format!("{}", other),
                    })
                }
            };
            if list.is_empty() {
                Err(EvalError::InvalidArguments {
                    context: "cdr".to_string(),
                    message: "Cannot take cdr of empty list".to_string(),
                })
            } else {
                // Collect to SmallVec explicitly first
                let collected_cdr: SmallVec8<Value> = list[1..].iter().cloned().collect();
                Ok(Value::List(Box::new(collected_cdr))) // Box the result
            }
        }
        "list" => {
            // Arity can be 0 or more
            let collected_list: SmallVec8<Value> = args.into_iter().collect();
            Ok(Value::List(Box::new(collected_list))) // Box the result
        }
        "typeof" => {
            check_arity!("typeof", args, 1);
            Ok(Value::Symbol(Rc::new(args[0].type_name().to_string())))
        }
        "null?" => {
            check_arity!("null?", args, 1);
            Ok(Value::Boolean(match &args[0] {
                Value::List(l) => l.is_empty(),
                Value::Null => true,
                _ => false,
            }))
        }
        "len" => {
            check_arity!("len", args, 1);
            match &args[0] {
                Value::List(l) => Ok(Value::Integer(l.len() as i64)),
                Value::String(s) => Ok(Value::Integer(s.chars().count() as i64)), // Count chars for string length
                Value::Map(m) => Ok(Value::Integer(m.len() as i64)),
                Value::Set(s) => Ok(Value::Integer(s.len() as i64)),
                other => Err(EvalError::TypeError {
                    expected: "List, String, Map, or Set".to_string(),
                    found_type: other.type_name().to_string(),
                    value: format!("{}", other),
                }),
            }
        }
        // Type Predicates
        "integer?" => {
            check_arity!("integer?", args, 1);
            Ok(Value::Boolean(matches!(args[0], Value::Integer(_))))
        }
        "float?" => {
            check_arity!("float?", args, 1);
            Ok(Value::Boolean(matches!(args[0], Value::Float(_))))
        }
        "boolean?" => {
            check_arity!("boolean?", args, 1);
            Ok(Value::Boolean(matches!(args[0], Value::Boolean(_))))
        }
        "string?" => {
            check_arity!("string?", args, 1);
            Ok(Value::Boolean(matches!(args[0], Value::String(_))))
        }
        "symbol?" => {
            check_arity!("symbol?", args, 1);
            Ok(Value::Boolean(matches!(args[0], Value::Symbol(_))))
        }
        "list?" => {
            check_arity!("list?", args, 1);
            Ok(Value::Boolean(matches!(args[0], Value::List(_))))
        }
        "map?" => {
            check_arity!("map?", args, 1);
            Ok(Value::Boolean(matches!(args[0], Value::Map(_))))
        }
        "set?" => {
            check_arity!("set?", args, 1);
            Ok(Value::Boolean(matches!(args[0], Value::Set(_))))
        }
        "function?" => {
            check_arity!("function?", args, 1);
            Ok(Value::Boolean(matches!(args[0], Value::Function(_))))
        }
        "primitive?" => {
            check_arity!("primitive?", args, 1);
            Ok(Value::Boolean(matches!(&args[0],
                Value::Function(boxed_fn) if matches!(**boxed_fn, BorfFunction::Native(_)))))
        }
        _ => Err(EvalError::InvalidOperation(format!(
            "Unknown native function: {}",
            name
        ))),
    }
}

// Evaluate a single declaration and update the *given* environment
pub fn evaluate_declaration(
    declaration: &Declaration,
    env: EnvironmentRef, // Takes env ref
) -> Result<(), EvalError> {
    let mut env_borrow = env.borrow_mut(); // Borrow mutably once
    match declaration {
        Declaration::Entity(name, type_expr, value_opt, _) => {
            // Drop mutable borrow before potentially calling eval recursively
            drop(env_borrow);
            let value = match value_opt {
                Some(expr) => eval(expr, Rc::clone(&env))?,
                None => Value::Null,
            };
            // Re-borrow to define
            env.borrow_mut().define(name.clone(), value);
            println!(
                "Declared entity '{}'. Type: {:?}", // Removed env ptr
                name, type_expr
            );
        }
        Declaration::Function(name, signature, body, _) => {
            let params = extract_params_from_signature(&Some(signature.clone()));
            let body_rc = Rc::new(*body.clone()); // Clone Box<Expr> -> Expr, then Rc::new
            let closure = Value::Function(Box::new(BorfFunction::Closure {
                params,
                body: body_rc,
                env: Rc::clone(&env), // Capture *current* env ref
            }));
            env_borrow.define(name.clone(), closure);
            println!("Defined function '{}'", name); // Removed env ptr
        }
        Declaration::Type(name, _type_expr, _) => {
            env_borrow.define(
                name.clone(),
                Value::Symbol(Rc::new(format!("type {}", name))),
            ); // Wrap symbol string in Rc
            println!("Declared type '{}'", name);
        }
        Declaration::Operation(name, _type_expr, _) => {
            env_borrow.define(name.clone(), Value::Symbol(Rc::new(format!("op {}", name)))); // Wrap symbol string in Rc
            println!("Declared operator '{}'", name);
        }
        Declaration::Dependency(import, _export, _direct, _) => {
            println!("TODO: Handle dependency declaration: import '{}'", import);
        }
    }
    Ok(())
}

// Main evaluation function (recursive)
pub fn eval(expr: &Expr, env: EnvironmentRef) -> Result<Value, EvalError> {
    match expr {
        Expr::Literal(lit, _) => eval_literal(lit),
        Expr::Variable(name, _) => env
            .borrow()
            .lookup(name)
            .ok_or(EvalError::UnboundVariable(name.clone())),
        Expr::QualifiedName(parts, _) => {
            let name = parts.join("::");
            env.borrow()
                .lookup(&name)
                .ok_or(EvalError::UnboundVariable(name))
        }
        Expr::Set(elements, _) => {
            #[allow(clippy::mutable_key_type)] // Allow local map with Value keys
            let mut evaluated_set = FxHashMap::default();
            for elem_expr in elements {
                let value = eval(elem_expr, Rc::clone(&env))?;
                if !is_hashable(&value) {
                    return Err(EvalError::HashingError {
                        value: format!("{}", value),
                        type_name: value.type_name().to_string(),
                    });
                }
                evaluated_set.insert(value, ());
            }
            Ok(Value::Set(Box::new(Rc::new(evaluated_set))))
        }
        Expr::List(elements, _) => {
            let mut evaluated_elements = SmallVec8::new();
            for elem_expr in elements {
                evaluated_elements.push(eval(elem_expr, Rc::clone(&env))?);
            }
            Ok(Value::List(Box::new(evaluated_elements)))
        }
        Expr::Map(entries, _) => {
            #[allow(clippy::mutable_key_type)] // Allow local map with Value keys
            let mut evaluated_map = FxHashMap::default();
            for (key_str, val_expr) in entries {
                let key_value = Value::String(Rc::new(key_str.clone()));
                if !is_hashable(&key_value) {
                    return Err(EvalError::HashingError {
                        value: format!("{}", key_value),
                        type_name: key_value.type_name().to_string(),
                    });
                }
                let value = eval(val_expr, Rc::clone(&env))?;
                evaluated_map.insert(key_value, value);
            }
            Ok(Value::Map(Box::new(Rc::new(evaluated_map))))
        }
        Expr::Lambda(params, body, _) => {
            let body_rc = Rc::new(*body.clone());
            Ok(Value::Function(Box::new(BorfFunction::Closure {
                params: params.iter().map(|p| *(*p).clone()).collect(),
                body: body_rc,
                env: Rc::clone(&env),
            })))
        }
        Expr::Application(function, arguments, _) => {
            let func_value = eval(function, Rc::clone(&env))?;
            let mut args = SmallVec8::new();
            for arg_expr in arguments {
                args.push(eval(arg_expr, Rc::clone(&env))?);
            }
            apply_function(func_value, args)
        }
        Expr::Let(pattern, value_expr, body_expr, _) => {
            let value = eval(value_expr, Rc::clone(&env))?;
            let let_env = Environment::new_extending(Rc::clone(&env));
            {
                let mut bindings = FxHashMap::default();
                if bind_pattern(pattern, &value, &mut bindings)? {
                    let mut let_env_mut = let_env.borrow_mut();
                    for (name, bound_value) in bindings {
                        let_env_mut.define(name, bound_value);
                    }
                } else {
                    return Err(EvalError::PatternMatchFailed {
                        value: format!("{}", value),
                        pattern: format!("{:?}", pattern),
                    });
                }
            }
            eval(body_expr, let_env)
        }
        Expr::If(cond, then_expr, else_expr, _) => {
            let cond_val = eval(cond, Rc::clone(&env))?;
            match cond_val {
                Value::Boolean(true) => eval(then_expr, Rc::clone(&env)),
                Value::Boolean(false) => eval(else_expr, Rc::clone(&env)),
                other => Err(EvalError::TypeError {
                    expected: "Boolean".to_string(),
                    found_type: other.type_name().to_string(),
                    value: format!("{}", other),
                }),
            }
        }
        Expr::BinaryOp(operator, left, right, _) => {
            if operator == "|>" {
                let left_val = eval(left, Rc::clone(&env))?;
                let func_value = eval(right, Rc::clone(&env))?;
                apply_function(func_value, smallvec![left_val])
            } else {
                let left_val = eval(left, Rc::clone(&env))?;
                let right_val = eval(right, Rc::clone(&env))?;
                // Look up operator, store the resulting Value
                let op_lookup_result = env.borrow().lookup(operator);
                match op_lookup_result {
                    // Pass the found Value (already contains Box) directly to apply_function
                    Some(func_val @ Value::Function(_)) => {
                        apply_function(func_val.clone(), smallvec![left_val, right_val])
                    }
                    Some(other) => Err(EvalError::NotAFunction(format!("{}", other))),
                    None => apply_native_function(operator, vec![left_val, right_val]),
                }
            }
        }
        Expr::UnaryOp(operator, operand, _) => {
            let operand_val = eval(operand, Rc::clone(&env))?;
            // Look up operator, store the resulting Value
            let op_lookup_result = env.borrow().lookup(operator);
            match op_lookup_result {
                // Pass the found Value (already contains Box) directly to apply_function
                Some(func_val @ Value::Function(_)) => {
                    apply_function(func_val.clone(), smallvec![operand_val])
                }
                Some(other) => Err(EvalError::NotAFunction(format!("{}", other))),
                None => apply_native_function(operator, vec![operand_val]),
            }
        }
        Expr::Quote(inner_expr, _) => {
            let inner_expr_rc = Rc::new(*inner_expr.clone());
            Ok(Value::Quote(Box::new(inner_expr_rc)))
        }
        Expr::Unquote(_, _) => Err(EvalError::QuasiquoteError(
            "Unquote ('~') is only valid inside a quasiquote ('`')".to_string(),
        )),
        Expr::UnquoteSplice(_, _) => Err(EvalError::QuasiquoteError(
            "Unquote-splice ('~@') is only valid inside a quasiquote ('`')".to_string(),
        )),
        Expr::Quasiquote(expr_to_process, _) => {
            evaluate_quasiquote(expr_to_process, Rc::clone(&env))
        }
    }
}

// Helper to evaluate literal values (standalone)
fn eval_literal(literal: &Literal) -> Result<Value, EvalError> {
    Ok(match literal {
        Literal::Integer(i) => Value::Integer(*i),
        Literal::Float(f) => Value::Float(*f),
        Literal::String(s) => Value::String(Rc::new(s.clone())),
        Literal::Boolean(b) => Value::Boolean(*b),
    })
}

// Helper to apply a function value (takes SmallVec8)
fn apply_function(func: Value, args: SmallVec8<Value>) -> Result<Value, EvalError> {
    match func {
        // Match on the Boxed function variant
        Value::Function(boxed_fn) => {
            // Match on the BorfFunction inside the Box
            match *boxed_fn {
                BorfFunction::Closure {
                    params,
                    body,
                    env: captured_env,
                } => {
                    let expected_arity = params.len();
                    let found_arity = args.len();
                    if expected_arity != found_arity {
                        return Err(EvalError::ArityMismatch {
                            context: "Closure".to_string(),
                            expected: expected_arity.to_string(),
                            found: found_arity,
                        });
                    }
                    let call_env = Environment::new_extending(captured_env);
                    {
                        let mut bindings = FxHashMap::default();
                        {
                            let mut call_env_mut = call_env.borrow_mut();
                            for (param_pattern, arg_value) in params.iter().zip(args.iter()) {
                                if !bind_pattern(param_pattern, arg_value, &mut bindings)? {
                                    return Err(EvalError::PatternMatchFailed {
                                        value: format!("{}", arg_value),
                                        pattern: format!("{:?}", param_pattern),
                                    });
                                }
                            }
                            for (name, value) in bindings {
                                call_env_mut.define(name, value);
                            }
                        }
                    }
                    eval(&body, call_env)
                }
                BorfFunction::Native(name) => apply_native_function(&name, args.into_vec()),
            }
        }
        other => Err(EvalError::NotAFunction(format!("{}", other))),
    }
}

// Placeholder helper function
fn extract_params_from_signature(_signature: &Option<TypeExpr>) -> Vec<Pattern> {
    // TODO: Implement properly based on TypeExpr structure
    Vec::new()
}

// Pattern matching helper - matches a pattern against a value, updating bindings map
fn bind_pattern(
    pattern: &Pattern,
    value: &Value,
    bindings: &mut FxHashMap<String, Value>,
) -> Result<bool, EvalError> {
    match pattern {
        Pattern::Literal(lit, _) => match (lit, value) {
            (Literal::Integer(i), Value::Integer(v)) => Ok(i == v),
            (Literal::Float(f), Value::Float(v)) => Ok(f == v),
            (Literal::String(s), Value::String(v)) => Ok(s == v.as_ref()),
            (Literal::Boolean(b), Value::Boolean(v)) => Ok(b == v),
            _ => Ok(false),
        },
        Pattern::Wildcard(_) => Ok(true),
        Pattern::Variable(var, _) => {
            bindings.insert(var.clone(), value.clone());
            Ok(true)
        }
        Pattern::List(list_pat, _) => {
            if let Value::List(list_val_box) = value {
                let list_val = &**list_val_box;
                if list_pat.len() != list_val.len() {
                    return Ok(false);
                }
                let mut temp_bindings = FxHashMap::default();
                for (pat_elem, val_elem) in list_pat.iter().zip(list_val.iter()) {
                    if !bind_pattern(pat_elem, val_elem, &mut temp_bindings)? {
                        return Ok(false);
                    }
                }
                bindings.extend(temp_bindings);
                Ok(true)
            } else {
                Ok(false)
            }
        }
        Pattern::Set(set_pat, _) => {
            if let Value::Set(set_val_box) = value {
                #[allow(clippy::mutable_key_type)] // Allow local map ref with Value keys
                let set_val = &***set_val_box;
                if set_pat.len() != set_val.len() {
                    return Ok(false);
                }
                Err(EvalError::InvalidOperation(
                    "Set pattern matching not fully implemented".to_string(),
                ))
            } else {
                Ok(false)
            }
        }
        Pattern::Map(map_pat, _) => {
            if let Value::Map(map_val_box) = value {
                #[allow(clippy::mutable_key_type)] // Allow local map ref with Value keys
                let map_val = &***map_val_box;
                if map_pat.len() != map_val.len() {
                    return Ok(false);
                }
                let mut temp_bindings = FxHashMap::default();
                for (key_str, pat_val) in map_pat {
                    let key_value = Value::String(Rc::new(key_str.clone()));
                    if !is_hashable(&key_value) {
                        return Err(EvalError::HashingError {
                            value: format!("{}", key_value),
                            type_name: key_value.type_name().to_string(),
                        });
                    }
                    match map_val.get(&key_value) {
                        Some(val_val) => {
                            if !bind_pattern(pat_val, val_val, &mut temp_bindings)? {
                                return Ok(false);
                            }
                        }
                        None => return Ok(false),
                    }
                }
                bindings.extend(temp_bindings);
                Ok(true)
            } else {
                Ok(false)
            }
        }
        Pattern::TypeAnnotated(inner_pat, _type_expr, _) => {
            bind_pattern(inner_pat, value, bindings)
        }
    }
}

// Placeholder for quasiquote evaluation
fn evaluate_quasiquote(expr: &Expr, env: EnvironmentRef) -> Result<Value, EvalError> {
    match expr {
        Expr::Literal(lit, _) => Ok(convert_literal_to_value(lit)),
        Expr::Variable(name, _) => Ok(Value::Symbol(Rc::new(name.clone()))),
        Expr::QualifiedName(parts, _) => Ok(Value::Symbol(Rc::new(parts.join("::")))),
        Expr::List(items, _) => {
            let mut result_list = SmallVec8::new();
            for item in items {
                match &**item {
                    Expr::UnquoteSplice(inner_expr, _) => {
                        let evaluated = eval(inner_expr, Rc::clone(&env))?;
                        match evaluated {
                            Value::List(spliced_items_box) => {
                                result_list.extend((**spliced_items_box).iter().cloned())
                            }
                            Value::Set(_) => {
                                return Err(EvalError::QuasiquoteError(
                                    "Cannot splice Set (~@) into List ('`[])".to_string(),
                                ))
                            }
                            other => {
                                return Err(EvalError::QuasiquoteError(format!(
                                    "Cannot splice value of type {} (need a List)",
                                    other.type_name()
                                )));
                            }
                        }
                    }
                    Expr::Unquote(inner_expr, _) => {
                        result_list.push(eval(inner_expr, Rc::clone(&env))?);
                    }
                    _ => {
                        result_list.push(evaluate_quasiquote(item, Rc::clone(&env))?);
                    }
                }
            }
            Ok(Value::List(Box::new(result_list)))
        }
        Expr::Set(items, _) => {
            #[allow(clippy::mutable_key_type)] // Allow local map with Value keys
            let mut result_set = FxHashMap::default();
            for item in items {
                match &**item {
                    Expr::UnquoteSplice(inner_expr, _) => {
                        let evaluated = eval(inner_expr, Rc::clone(&env))?;
                        match evaluated {
                            Value::Set(spliced_items_box) => {
                                let map_rc: &Rc<FxHashMap<Value, ()>> = &spliced_items_box;
                                #[allow(clippy::mutable_key_type)]
                                // Allow local map ref with Value keys
                                let the_map: &FxHashMap<Value, ()> = map_rc;
                                for k in the_map.keys() {
                                    if !is_hashable(k) {
                                        return Err(EvalError::QuasiquoteError(
                                            "~@ Set in Set requires hashable elements".to_string(),
                                        ));
                                    }
                                    result_set.insert(k.clone(), ());
                                }
                            }
                            Value::List(spliced_items_box) => {
                                for k in (**spliced_items_box).iter() {
                                    if !is_hashable(k) {
                                        return Err(EvalError::QuasiquoteError(
                                            "~@ List into Set requires hashable elements"
                                                .to_string(),
                                        ));
                                    }
                                    result_set.insert(k.clone(), ());
                                }
                            }
                            other => {
                                return Err(EvalError::QuasiquoteError(format!(
                                    "~@ in Set requires a Set or List value, found {}",
                                    other.type_name()
                                )))
                            }
                        }
                    }
                    Expr::Unquote(inner_expr, _) => {
                        let val = eval(inner_expr, Rc::clone(&env))?;
                        if !is_hashable(&val) {
                            return Err(EvalError::HashingError {
                                value: format!("{}", val),
                                type_name: val.type_name().to_string(),
                            });
                        }
                        result_set.insert(val, ());
                    }
                    _ => {
                        let val = evaluate_quasiquote(item, Rc::clone(&env))?;
                        if !is_hashable(&val) {
                            return Err(EvalError::HashingError {
                                value: format!("{}", val),
                                type_name: val.type_name().to_string(),
                            });
                        }
                        result_set.insert(val, ());
                    }
                }
            }
            Ok(Value::Set(Box::new(Rc::new(result_set))))
        }
        Expr::Map(entries, _) => {
            #[allow(clippy::mutable_key_type)] // Allow local map with Value keys
            let mut result_map = FxHashMap::default();
            for (key_str, value_expr) in entries {
                let key_val = Value::Symbol(Rc::new(key_str.clone()));
                if !is_hashable(&key_val) {
                    return Err(EvalError::HashingError {
                        value: format!("{}", key_val),
                        type_name: key_val.type_name().to_string(),
                    });
                }
                match &**value_expr {
                    Expr::Unquote(inner_expr, _) => {
                        let val = eval(inner_expr, Rc::clone(&env))?;
                        result_map.insert(key_val, val);
                    }
                    Expr::UnquoteSplice(_, _) => {
                        return Err(EvalError::QuasiquoteError(
                            "Unquote-splice (~@) not supported directly as map value in quasiquote"
                                .to_string(),
                        ))
                    }
                    _ => {
                        let val = evaluate_quasiquote(value_expr, Rc::clone(&env))?;
                        result_map.insert(key_val, val);
                    }
                }
            }
            Ok(Value::Map(Box::new(Rc::new(result_map))))
        }
        Expr::Unquote(inner_expr, _) => eval(inner_expr, Rc::clone(&env)),
        Expr::UnquoteSplice(_, _) => Err(EvalError::QuasiquoteError(
            "Unquote-splice (~@) cannot be at the top level of a quasiquote".to_string(),
        )),
        Expr::BinaryOp(op, l, r, _) => Ok(Value::List(Box::new(smallvec![
            Value::Symbol(Rc::new(op.clone())),
            evaluate_quasiquote(l, Rc::clone(&env))?,
            evaluate_quasiquote(r, Rc::clone(&env))?
        ]))),
        Expr::UnaryOp(op, o, _) => Ok(Value::List(Box::new(smallvec![
            Value::Symbol(Rc::new(op.clone())),
            evaluate_quasiquote(o, Rc::clone(&env))?
        ]))),
        Expr::If(cond, then, els, _) => Ok(Value::List(Box::new(smallvec![
            Value::Symbol(Rc::new("if".to_string())),
            evaluate_quasiquote(cond, Rc::clone(&env))?,
            evaluate_quasiquote(then, Rc::clone(&env))?,
            evaluate_quasiquote(els, Rc::clone(&env))?
        ]))),
        Expr::Let(pat, val, body, _) => Ok(Value::List(Box::new(smallvec![
            Value::Symbol(Rc::new("let".to_string())),
            convert_pattern_to_value(pat)?,
            evaluate_quasiquote(val, Rc::clone(&env))?,
            evaluate_quasiquote(body, Rc::clone(&env))?
        ]))),
        Expr::Lambda(params, body, _) => {
            // Collect pattern values into a SmallVec first
            let pattern_values_vec = params
                .iter()
                .map(|p| convert_pattern_to_value(p))
                .collect::<Result<SmallVec8<_>, _>>()?;
            // Build the final list value, boxing the SmallVec
            Ok(Value::List(Box::new(smallvec![
                Value::Symbol(Rc::new("lambda".to_string())),
                Value::List(Box::new(pattern_values_vec)), // Box the collected patterns list
                evaluate_quasiquote(body, Rc::clone(&env))?
            ])))
        }
        Expr::Application(func, args, _) => {
            let mut app_list = smallvec![evaluate_quasiquote(func, Rc::clone(&env))?];
            for arg in args {
                app_list.push(evaluate_quasiquote(arg, Rc::clone(&env))?);
            }
            Ok(Value::List(Box::new(app_list)))
        }
        Expr::Quote(inner, _) => Ok(Value::List(Box::new(smallvec![
            Value::Symbol(Rc::new("quote".to_string())),
            evaluate_quasiquote(inner, Rc::clone(&env))?
        ]))),
        Expr::Quasiquote(inner, _) => Ok(Value::List(Box::new(smallvec![
            Value::Symbol(Rc::new("quasiquote".to_string())),
            evaluate_quasiquote(inner, Rc::clone(&env))?
        ]))),
    }
}

// Helper to convert AST Literal to Value (existing)
fn convert_literal_to_value(literal: &Literal) -> Value {
    match literal {
        Literal::Integer(i) => Value::Integer(*i),
        Literal::Float(f) => Value::Float(*f),
        Literal::String(s) => Value::String(Rc::new(s.clone())),
        Literal::Boolean(b) => Value::Boolean(*b),
    }
}

// TODO: Helper function to convert Pattern AST to Value for quasiquote
fn convert_pattern_to_value(pattern: &Pattern) -> Result<Value, EvalError> {
    match pattern {
        Pattern::Variable(name, _) => Ok(Value::Symbol(Rc::new(name.clone()))),
        Pattern::Literal(lit, _) => Ok(convert_literal_to_value(lit)),
        Pattern::Wildcard(_) => Ok(Value::Symbol(Rc::new("_".to_string()))),
        Pattern::List(pats, _) => {
            let val_list = pats
                .iter()
                .map(|p| convert_pattern_to_value(p))
                .collect::<Result<SmallVec8<_>, _>>()?;
            Ok(Value::List(Box::new(val_list)))
        }
        Pattern::Set(pats, _) => {
            #[allow(clippy::mutable_key_type)] // Allow local map with Value keys
            let mut val_set_map = FxHashMap::default();
            for p in pats {
                let val = convert_pattern_to_value(p)?;
                if !is_hashable(&val) {
                    return Err(EvalError::HashingError {
                        value: format!("{}", val),
                        type_name: val.type_name().to_string(),
                    });
                }
                val_set_map.insert(val, ());
            }
            Ok(Value::Set(Box::new(Rc::new(val_set_map))))
        }
        Pattern::Map(pats, _) => {
            #[allow(clippy::mutable_key_type)] // Allow local map with Value keys
            let mut val_map = FxHashMap::default();
            for (key_str, pat_val) in pats {
                let key_value = Value::Symbol(Rc::new(key_str.clone()));
                if !is_hashable(&key_value) {
                    return Err(EvalError::HashingError {
                        value: format!("{}", key_value),
                        type_name: key_value.type_name().to_string(),
                    });
                }
                let value = convert_pattern_to_value(pat_val)?;
                val_map.insert(key_value, value);
            }
            Ok(Value::Map(Box::new(Rc::new(val_map))))
        }
        Pattern::TypeAnnotated(inner_pat, _type_expr, _) => convert_pattern_to_value(inner_pat),
    }
}

// Helper to populate global environment with primitives
pub fn populate_global_env(env: EnvironmentRef) {
    let mut env_mut = env.borrow_mut();
    // Use BorfFunction::Native
    env_mut.define(
        "+".to_string(),
        Value::Function(Box::new(BorfFunction::Native("+".to_string()))),
    );
    env_mut.define(
        "-".to_string(),
        Value::Function(Box::new(BorfFunction::Native("-".to_string()))),
    );
    env_mut.define(
        "*".to_string(),
        Value::Function(Box::new(BorfFunction::Native("*".to_string()))),
    );
    env_mut.define(
        "/".to_string(),
        Value::Function(Box::new(BorfFunction::Native("/".to_string()))),
    );
    env_mut.define(
        "==".to_string(),
        Value::Function(Box::new(BorfFunction::Native("==".to_string()))),
    );
    env_mut.define(
        "!=".to_string(),
        Value::Function(Box::new(BorfFunction::Native("!=".to_string()))),
    );
    env_mut.define(
        "<".to_string(),
        Value::Function(Box::new(BorfFunction::Native("<".to_string()))),
    );
    env_mut.define(
        ">".to_string(),
        Value::Function(Box::new(BorfFunction::Native(">".to_string()))),
    );
    env_mut.define(
        "<=".to_string(),
        Value::Function(Box::new(BorfFunction::Native("<=".to_string()))),
    );
    env_mut.define(
        ">=".to_string(),
        Value::Function(Box::new(BorfFunction::Native(">=".to_string()))),
    );
    env_mut.define(
        "print".to_string(),
        Value::Function(Box::new(BorfFunction::Native("print".to_string()))),
    );
    // Add cons, car, cdr, list, typeof etc.
}

// --- Add Display/Debug impls for needed AST types if not already present ---
// Ensure ast::Expr and ast::Literal implement fmt::Display for Value::Quote formatting
// (Assuming they might be missing or need adjustment)

// Add Display impl for Expr (example, adjust based on actual AST structure)
impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Literal(lit, _) => write!(f, "{}", lit),
            Expr::Variable(name, _) => write!(f, "{}", name),
            Expr::QualifiedName(parts, _) => write!(f, "{}", parts.join("::")),
            Expr::Lambda(params, body, _) => {
                write!(
                    f,
                    "(lambda ({}) {})",
                    params
                        .iter()
                        .map(|p| format!("{:?}", p))
                        .collect::<Vec<_>>()
                        .join(" "),
                    body
                )
            }
            Expr::Application(func, args, _) => {
                write!(f, "({}", func)?;
                for arg in args {
                    write!(f, " {}", arg)?;
                }
                write!(f, ")")
            }
            Expr::Let(pat, val, body, _) => write!(f, "(let (({:?}) {}) {})", pat, val, body),
            Expr::If(cond, then, els, _) => write!(f, "(if {} {} {})", cond, then, els),
            Expr::BinaryOp(op, l, r, _) => write!(f, "({} {} {})", op, l, r),
            Expr::UnaryOp(op, o, _) => write!(f, "({} {})", op, o),
            Expr::List(items, _) => {
                write!(f, "(list")?;
                for item in items {
                    write!(f, " {}", item)?;
                }
                write!(f, ")")
            }
            Expr::Set(items, _) => {
                write!(f, "(set")?;
                for item in items {
                    write!(f, " {}", item)?;
                }
                write!(f, ")")
            }
            Expr::Map(items, _) => {
                write!(f, "(map")?;
                for (k, v) in items {
                    write!(f, " ({:?} {})", k, v)?;
                }
                write!(f, ")")
            }
            Expr::Quote(e, _) => write!(f, "(quote {})", e),
            Expr::Unquote(e, _) => write!(f, "~{}", e),
            Expr::UnquoteSplice(e, _) => write!(f, "~@{}", e),
            Expr::Quasiquote(e, _) => write!(f, "`{}", e),
        }
    }
}

// Display impl for Literal (seems ok, ensure quotes for strings)
impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::Integer(i) => write!(f, "{}", i),
            Literal::Float(fl) => write!(f, "{}", fl),
            Literal::Boolean(b) => write!(f, "{}", b),
            Literal::String(s) => write!(f, "\"{}\"", s),
        }
    }
}

// Helper to check if a value can be hashed (needed for Map/Set keys)
fn is_hashable(value: &Value) -> bool {
    match value {
        Value::Integer(_)
        | Value::Float(_)
        | Value::Boolean(_)
        | Value::String(_)
        | Value::Symbol(_)
        | Value::Null
        | Value::Void => true,
        Value::List(l) => l.iter().all(is_hashable),
        Value::Map(m) => m.iter().all(|(k, v)| is_hashable(k) && is_hashable(v)),
        Value::Set(s) => s.keys().all(is_hashable),
        Value::Function(_) | Value::Module(_) | Value::Quote(_) => false,
    }
}

// Add a helper to get type name string for errors
impl Value {
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Integer(_) => "Integer",
            Value::Float(_) => "Float",
            Value::String(_) => "String",
            Value::Boolean(_) => "Boolean",
            Value::Symbol(_) => "Symbol",
            Value::List(_) => "List",
            Value::Map(_) => "Map",
            Value::Set(_) => "Set",
            Value::Function(_) => "Function",
            Value::Module(_) => "Module",
            Value::Quote(_) => "Quote",
            Value::Null => "Null",
            Value::Void => "Void",
        }
    }
}

// --- Value Implementations (PartialEq, Eq, Hash, Display) ---

// Add back PartialEq for Value
#[allow(clippy::mutable_key_type)] // Allow comparison involving Value keys, checked at runtime
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => {
                if a.is_nan() && b.is_nan() {
                    true
                } else {
                    a == b
                }
            }
            (Value::Integer(a), Value::Float(b)) => (*a as f64) == *b,
            (Value::Float(a), Value::Integer(b)) => *a == (*b as f64),
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Symbol(a), Value::Symbol(b)) => a == b,
            (Value::List(a), Value::List(b)) => **a == **b,
            (Value::Map(a), Value::Map(b)) => **a == **b,
            (Value::Set(a), Value::Set(b)) => {
                let map_a = &***a;
                let map_b = &***b;
                if map_a.len() != map_b.len() {
                    return false;
                }
                map_a.keys().all(|k| map_b.contains_key(k))
            }
            (Value::Function(a), Value::Function(b)) => **a == **b,
            (Value::Module(a), Value::Module(b)) => a == b,
            (Value::Quote(a), Value::Quote(b)) => **a == **b,
            (Value::Null, Value::Null) => true,
            (Value::Void, Value::Void) => true,
            _ => false,
        }
    }
}

// Add back Eq for Value
impl Eq for Value {}

// Add back Hash for Value
#[allow(clippy::mutable_key_type)] // Allow hashing involving Value keys, checked at runtime
impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            Value::Integer(i) => i.hash(state),
            Value::Float(f) => {
                if f.is_nan() {
                    0f64.to_bits().hash(state);
                } else {
                    f.to_bits().hash(state);
                }
            }
            Value::Boolean(b) => b.hash(state),
            Value::String(s) => s.hash(state),
            Value::Symbol(s) => s.hash(state),
            Value::List(l) => l.hash(state),
            Value::Map(m) => {
                let inner_map = &***m;
                inner_map.len().hash(state);
                let mut xor_sum = 0u64;
                for (k, v) in inner_map.iter() {
                    let mut hasher = rustc_hash::FxHasher::default();
                    k.hash(&mut hasher);
                    v.hash(&mut hasher);
                    xor_sum ^= hasher.finish();
                }
                xor_sum.hash(state);
            }
            Value::Set(s) => {
                let inner_set = &***s;
                inner_set.len().hash(state);
                let mut xor_sum = 0u64;
                for k in inner_set.keys() {
                    let mut hasher = rustc_hash::FxHasher::default();
                    k.hash(&mut hasher);
                    xor_sum ^= hasher.finish();
                }
                xor_sum.hash(state);
            }
            Value::Function(_) => {
                // Functions are not hashable
                panic!("Attempted to hash non-hashable type: Function");
            }
            Value::Module(_name) => {
                // Modules are not hashable by value, only by name maybe?
                // For now, treat as non-hashable in sets/maps.
                panic!("Attempted to hash non-hashable type: Module");
            }
            Value::Quote(_) => {
                // Quoted expressions are not hashable
                panic!("Attempted to hash non-hashable type: Quote");
            }
            Value::Null => 2u64.hash(state),
            Value::Void => 3u64.hash(state),
        }
    }
}

// Add back Display for Value
impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Integer(i) => write!(f, "{}", i),
            Value::Float(fl) => {
                if fl.is_nan() {
                    write!(f, "NaN")
                } else if fl.is_infinite() {
                    write!(
                        f,
                        "{}Infinity",
                        if fl.is_sign_positive() { "" } else { "-" }
                    )
                } else {
                    write!(f, "{}", fl)
                }
            }
            Value::Boolean(b) => write!(f, "{}", b),
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::Symbol(s) => write!(f, "{}", s),
            Value::List(vals_box) => {
                write!(f, "(list")?;
                for v in vals_box.iter() {
                    write!(f, " {}", v)?;
                }
                write!(f, ")")
            }
            Value::Map(map_box) => {
                #[allow(clippy::mutable_key_type)] // Allow local map ref with Value keys
                let map = &***map_box;
                write!(f, "{{")?;
                let mut first = true;
                for (k, v) in map.iter() {
                    if !first {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", k, v)?;
                    first = false;
                }
                write!(f, "}}")
            }
            Value::Set(set_box) => {
                #[allow(clippy::mutable_key_type)] // Allow local map ref with Value keys
                let set = &***set_box;
                write!(f, "#{{")?;
                let mut first = true;
                for k in set.keys() {
                    if !first {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", k)?;
                    first = false;
                }
                write!(f, "}}")
            }
            Value::Function(_) => write!(f, "<function>"),
            Value::Module(name) => write!(f, "<module:{}>", name),
            Value::Quote(expr_box) => write!(f, "'{}", expr_box),
            Value::Null => write!(f, "null"),
            Value::Void => write!(f, "void"),
        }
    }
}
