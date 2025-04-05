use crate::parser::ast::{self, Declaration, Expr, Literal, Pattern, TypeExpr};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

/// The Borf value type representing values in the language
#[derive(Debug, Clone)]
pub enum Value {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Symbol(String),
    List(Vec<Value>),
    Set(Vec<Value>), // Using Vec for simplicity, a real implementation would use a proper Set type
    Map(HashMap<String, Value>),
    Function(BorfFunction),
    Module(String), // Just store the module name for now
    Null,
    Void, // Representing the absence of a value (distinct from Null?)
}

/// Function representation
#[derive(Debug, Clone)]
pub enum BorfFunction {
    Native(String), // Reference to a native function by name
    Closure {
        params: Vec<Pattern>, // Parameter patterns from the lambda AST
        body: Box<Expr>,      // Body expression AST node
        env: Rc<Environment>, // Captured environment (using Rc for sharing)
    },
}

/// Environment for evaluation
#[derive(Debug, Clone, Default)]
pub struct Environment {
    // Map module and variable names to values
    values: HashMap<String, Value>,

    // Parent environment for lexical scoping
    parent: Option<Rc<Environment>>,
}

impl Environment {
    /// Create a new empty environment
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new environment with a parent scope
    pub fn with_parent(parent: Rc<Environment>) -> Self {
        Self {
            values: HashMap::new(),
            parent: Some(parent),
        }
    }

    /// Define a variable in the current environment
    pub fn define(&mut self, name: &str, value: Value) {
        self.values.insert(name.to_string(), value);
    }

    /// Get a variable value from the environment or its parents
    pub fn get(&self, name: &str) -> Option<Value> {
        self.values
            .get(name)
            .cloned()
            .or_else(|| self.parent.as_ref().and_then(|p| p.get(name)))
    }
}

/// The evaluator for Borf code
pub struct Evaluator {
    global_env: Rc<Environment>,
}

impl Evaluator {
    /// Create a new evaluator with basic Prim
    pub fn new() -> Self {
        // Create the initial global environment
        let mut initial_env = Environment::new();

        // Initialize environment with primitives and prelude files
        match initialize_prelude_and_load_files(&mut initial_env) {
            Ok(_) => println!("Prelude loaded successfully."),
            Err(e) => eprintln!("Error loading prelude: {}", e),
            // Consider how to handle prelude loading errors more robustly
        }

        Self {
            global_env: Rc::new(initial_env),
        }
    }

    /// Evaluate a module
    pub fn evaluate_module(&self, module: &ast::Module) -> Result<Value, EvalError> {
        // Create an environment for this module, inheriting from the *final* global scope
        let mut module_env = Environment::with_parent(Rc::clone(&self.global_env));

        // Evaluate each declaration within the module
        for decl in &module.declarations {
            // Use the standalone evaluate_declaration helper, passing the module env
            evaluate_declaration(decl, &mut module_env)?;
        }
        Ok(Value::Module(module.name.clone()))
    }

    // NOTE: evaluate_expr, evaluate_literal, apply_function, and bind_pattern
    // are defined as standalone functions later in this file and used by evaluate_declaration.
    // The method versions below were redundant and are now removed.
}

// --- Evaluation Error ---
#[derive(Debug, Clone)]
pub struct EvalError(String); // Ensure this is public

impl std::fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Evaluation Error: {}", self.0)
    }
}

impl std::error::Error for EvalError {}

// Add Default impl as suggested by clippy
impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}

// --- Initialization & Native Functions (Standalone Functions) ---

// Initialize prelude: Define built-ins AND load prelude files
fn initialize_prelude_and_load_files(
    env: &mut Environment,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Initializing prelude...");
    // Define hardcoded primitive values
    env.define("true", Value::Boolean(true));
    env.define("false", Value::Boolean(false));
    env.define("null", Value::Null);
    env.define("void", Value::Void);

    // Define hardcoded primitive native functions (operators, print, etc.)
    let primitives = [
        "print", "+", "-", "*", "/", "==", "!=", "<", "<=", ">", ">=", "and", "or", "not",
    ];
    for name in primitives {
        env.define(
            name,
            Value::Function(BorfFunction::Native(name.to_string())),
        );
    }
    println!("Primitives defined.");

    // Load and evaluate prelude files from src/prelude
    let prelude_dir = PathBuf::from("src/prelude");
    println!(
        "Looking for prelude files in: {:?}",
        prelude_dir.canonicalize()?
    );

    // Read directory entries
    let entries = fs::read_dir(prelude_dir)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        // Check if it's a .borf file
        if path.is_file() && path.extension().is_some_and(|ext| ext == "borf") {
            println!("Loading prelude file: {:?}", path);

            // TODO: Use proper parsing function once circular dependencies are resolved
            println!("Skipping prelude file for now - parser integration needed");
            /*
            // Parse the file
            // Handle parse errors gracefully
            let module_ast = match parser::parse_file(&path) {
                Ok(ast) => ast,
                Err(e) => {
                    eprintln!(
                        "Failed to parse prelude file {:?}: {}\nSkipping this file.",
                        path, e
                    );
                    continue; // Skip to next file on parse error
                }
            };

            println!(
                "Parsed prelude file: {}, found {} declarations.",
                module_ast.name,
                module_ast.declarations.len()
            );

            // Evaluate declarations directly into the *initial* prelude environment
            for decl in &module_ast.declarations {
                // Use the standalone evaluate_declaration helper
                match evaluate_declaration(decl, env) {
                    Ok(_) => (),
                    Err(e) => {
                        eprintln!("Error evaluating declaration {:?} in prelude file {:?}: {}\nStopping prelude load.", decl, path, e);
                        // Decide whether to stop all loading or just skip the decl/file
                        return Err(Box::new(e)); // Stop all loading on eval error
                    }
                }
            }
            println!("Finished evaluating prelude file: {}", module_ast.name);
            */
        }
    }

    println!("Prelude initialization complete.");
    Ok(())
}

// Dispatcher for native function calls (remains standalone)
fn apply_native_function(name: &str, args: Vec<Value>) -> Result<Value, EvalError> {
    // Macro to check arity easily
    macro_rules! check_arity {
        ($fn_name:expr, $args:expr, $expected:expr) => {
            if $args.len() != $expected {
                return Err(EvalError(format!(
                    "Native function '{}' expected {} arguments, got {}",
                    $fn_name,
                    $expected,
                    $args.len()
                )));
            }
        };
        ($fn_name:expr, $args:expr, $min:expr, $max:expr) => {
            if $args.len() < $min || $args.len() > $max {
                return Err(EvalError(format!(
                    "Native function '{}' expected between {} and {} arguments, got {}",
                    $fn_name,
                    $min,
                    $max,
                    $args.len()
                )));
            }
        };
    }
    // Macro to expect boolean
    macro_rules! expect_boolean {
        ($arg:expr, $fn_name:expr, $arg_num:expr) => {
            match $arg {
                Value::Boolean(b) => *b, // Return the boolean value directly
                _ => {
                    return Err(EvalError(format!(
                        "Native function '{}' expects argument {} to be Boolean, got {:?}",
                        $fn_name, $arg_num, $arg
                    )))
                }
            }
        };
    }

    match name {
        "print" => {
            for (i, arg) in args.iter().enumerate() {
                print!("{}{:?}", if i > 0 { " " } else { "" }, arg);
            }
            println!();
            Ok(Value::Void)
        }
        "+" => {
            check_arity!("+", args, 2);
            match (&args[0], &args[1]) {
                (Value::Integer(a), Value::Integer(b)) => {
                    Ok(Value::Integer(a.checked_add(*b).ok_or_else(|| {
                        EvalError("Integer overflow in +".to_string())
                    })?))
                }
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
                (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a + (*b as f64))),
                (Value::Integer(a), Value::Float(b)) => Ok(Value::Float((*a as f64) + b)),
                (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
                _ => Err(EvalError(format!(
                    "Invalid arguments for '+': {:?}, {:?}",
                    args[0], args[1]
                ))),
            }
        }
        "-" => {
            check_arity!("-", args, 2);
            match (&args[0], &args[1]) {
                (Value::Integer(a), Value::Integer(b)) => {
                    Ok(Value::Integer(a.checked_sub(*b).ok_or_else(|| {
                        EvalError("Integer overflow in -".to_string())
                    })?))
                }
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
                (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a - (*b as f64))),
                (Value::Integer(a), Value::Float(b)) => Ok(Value::Float((*a as f64) - b)),
                _ => Err(EvalError(format!(
                    "Invalid arguments for '-': {:?}, {:?}",
                    args[0], args[1]
                ))),
            }
        }
        "*" => {
            check_arity!("*", args, 2);
            match (&args[0], &args[1]) {
                (Value::Integer(a), Value::Integer(b)) => {
                    Ok(Value::Integer(a.checked_mul(*b).ok_or_else(|| {
                        EvalError("Integer overflow in *".to_string())
                    })?))
                }
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
                (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a * (*b as f64))),
                (Value::Integer(a), Value::Float(b)) => Ok(Value::Float((*a as f64) * b)),
                _ => Err(EvalError(format!(
                    "Invalid arguments for '*': {:?}, {:?}",
                    args[0], args[1]
                ))),
            }
        }
        "/" => {
            check_arity!("/", args, 2);
            match (&args[0], &args[1]) {
                (_, Value::Integer(0)) => Err(EvalError("Integer division by zero".to_string())),
                (_, Value::Float(f)) if f.abs() < f64::EPSILON => {
                    Err(EvalError("Float division by zero".to_string()))
                }
                (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a / b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a / b)),
                (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a / (*b as f64))),
                (Value::Integer(a), Value::Float(b)) => Ok(Value::Float((*a as f64) / b)),
                _ => Err(EvalError(format!(
                    "Invalid arguments for '/': {:?}, {:?}",
                    args[0], args[1]
                ))),
            }
        }
        "==" => {
            check_arity!("==", args, 2);
            println!("Warning: Equality comparison (==) is naive (uses Debug format).");
            Ok(Value::Boolean(
                format!("{:?}", args[0]) == format!("{:?}", args[1]),
            ))
        }
        "!=" => {
            check_arity!("!=", args, 2);
            println!("Warning: Inequality comparison (!=) is naive (uses Debug format).");
            Ok(Value::Boolean(
                format!("{:?}", args[0]) != format!("{:?}", args[1]),
            ))
        }
        "<" => {
            check_arity!("<", args, 2);
            match (&args[0], &args[1]) {
                (Value::Integer(a), Value::Integer(b)) => Ok(Value::Boolean(a < b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Boolean(a < b)),
                (Value::Float(a), Value::Integer(b)) => Ok(Value::Boolean(a < &(*b as f64))),
                (Value::Integer(a), Value::Float(b)) => Ok(Value::Boolean(&(*a as f64) < b)),
                (Value::String(a), Value::String(b)) => Ok(Value::Boolean(a < b)),
                _ => Err(EvalError(format!(
                    "Cannot compare arguments with '<': {:?}, {:?}",
                    args[0], args[1]
                ))),
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
                _ => Err(EvalError(format!(
                    "Cannot compare arguments with '<=': {:?}, {:?}",
                    args[0], args[1]
                ))),
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
                _ => Err(EvalError(format!(
                    "Cannot compare arguments with '>': {:?}, {:?}",
                    args[0], args[1]
                ))),
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
                _ => Err(EvalError(format!(
                    "Cannot compare arguments with '>=': {:?}, {:?}",
                    args[0], args[1]
                ))),
            }
        }
        "and" => {
            check_arity!("and", args, 2);
            let left = expect_boolean!(&args[0], "and", 1);
            let right = expect_boolean!(&args[1], "and", 2);
            Ok(Value::Boolean(left && right))
        }
        "or" => {
            check_arity!("or", args, 2);
            let left = expect_boolean!(&args[0], "or", 1);
            let right = expect_boolean!(&args[1], "or", 2);
            Ok(Value::Boolean(left || right))
        }
        "not" => {
            check_arity!("not", args, 1);
            let val = expect_boolean!(&args[0], "not", 1);
            Ok(Value::Boolean(!val))
        }
        _ => Err(EvalError(format!("Unknown native function: {}", name))),
    }
}

// Evaluate a single declaration and update the *given* environment
fn evaluate_declaration(
    declaration: &Declaration,
    env: &mut Environment, // Takes env mutably
) -> Result<(), EvalError> {
    match declaration {
        Declaration::EntityDecl(entity) => {
            let value = match &entity.value {
                Some(expr) => evaluate_expr(expr, env)?,
                None => Value::Null,
            };
            env.define(&entity.name, value);
            println!(
                "Declared entity '{}' in env {:p}. Type: {:?}",
                entity.name, env, entity.type_expr
            );
        }
        Declaration::FnImpl(fn_impl) => {
            let params = extract_params_from_signature(&fn_impl.signature);
            let closure = Value::Function(BorfFunction::Closure {
                params,
                body: Box::new(fn_impl.body.clone()),
                env: Rc::new(env.clone()), // Capture *this* environment
            });
            env.define(&fn_impl.name, closure);
            println!(
                "Defined function '{}' (impl) in env {:p}",
                fn_impl.name, env
            );
        }
        Declaration::FnDecl(fn_decl) => {
            if let Some(body_expr) = &fn_decl.impl_body {
                let params = extract_params_from_signature(&fn_decl.signature);
                let closure = Value::Function(BorfFunction::Closure {
                    params,
                    body: Box::new(body_expr.clone()),
                    env: Rc::new(env.clone()), // Capture *this* environment
                });
                env.define(&fn_decl.name, closure);
                println!(
                    "Defined function '{}' (decl+body) in env {:p}",
                    fn_decl.name, env
                );
            } else {
                println!(
                    "Declared function '{}' (sig only) in env {:p}. Sig: {:?}",
                    fn_decl.name, env, fn_decl.signature
                );
                env.define(&fn_decl.name, Value::Null);
            }
        }
        Declaration::TypeDecl(type_decl) => {
            for name in &type_decl.names {
                let name_str = name.clone(); // Clone to get owned String
                env.define(&name_str, Value::Symbol(name_str.clone()));
                println!("Declared type '{}' in env {:p}", name, env);
            }
        }
        Declaration::OpDecl(op_decl) => {
            for name in &op_decl.names {
                let name_str = name.clone(); // Clone to get owned String
                env.define(&name_str, Value::Symbol(name_str.clone()));
                println!("Declared operator '{}' in env {:p}", name, env);
            }
        }
        Declaration::DepsDecl(deps_decl) => {
            println!(
                "TODO: Handle deps declaration: {:?}",
                deps_decl.dependencies
            );
        }
    }
    Ok(())
}

// Evaluate an expression within a given environment (standalone)
fn evaluate_expr(expr: &Expr, env: &Environment) -> Result<Value, EvalError> {
    match expr {
        Expr::Literal(lit) => evaluate_literal(lit), // Use standalone helper
        Expr::Variable(var) => env
            .get(&var.name)
            .ok_or_else(|| EvalError(format!("Undefined variable: {}", var.name))),
        Expr::Set(set_expr) => {
            let elements = set_expr
                .elements
                .iter()
                .map(|el| evaluate_expr(el, env))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Value::Set(elements))
        }
        Expr::List(list_expr) => {
            let elements = list_expr
                .elements
                .iter()
                .map(|el| evaluate_expr(el, env))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Value::List(elements))
        }
        Expr::Map(map_expr) => {
            let mut entries = HashMap::new();
            for (key, val_expr) in &map_expr.entries {
                let value = evaluate_expr(val_expr, env)?;
                entries.insert(key.clone(), value);
            }
            Ok(Value::Map(entries))
        }
        Expr::Lambda(lambda) => {
            Ok(Value::Function(BorfFunction::Closure {
                params: lambda.params.clone(),
                body: lambda.body.clone(),
                env: Rc::new(env.clone()), // Capture current env
            }))
        }
        Expr::FnApp(app) => {
            let func_value = evaluate_expr(&app.function, env)?;
            let args = app
                .arguments
                .iter()
                .map(|arg| evaluate_expr(arg, env))
                .collect::<Result<Vec<_>, _>>()?;
            apply_function(func_value, args) // Use standalone apply
        }
        Expr::InfixOp(op) => {
            let left_val = evaluate_expr(&op.left, env)?;
            let right_val = evaluate_expr(&op.right, env)?;
            match env.get(&op.operator) {
                Some(op_func) => apply_function(op_func, vec![left_val, right_val]), // Use standalone apply
                None => Err(EvalError(format!(
                    "Undefined operator/function: {}",
                    op.operator
                ))),
            }
        }
        Expr::Pipe(pipe) => {
            let left_val = evaluate_expr(&pipe.left, env)?;
            let func_value = evaluate_expr(&pipe.right, env)?;
            apply_function(func_value, vec![left_val]) // Use standalone apply
        }
        Expr::Match(match_expr) => {
            let value_to_match = evaluate_expr(&match_expr.value, env)?;
            for arm in &match_expr.arms {
                let mut bindings = HashMap::new();
                if bind_pattern(&arm.pattern, &value_to_match, &mut bindings)? {
                    let arm_env = Environment {
                        values: bindings,
                        parent: Some(Rc::new(env.clone())), // Inherit from match scope
                    };
                    let guard_passed = match &arm.guard {
                        Some(guard_expr) => {
                            let guard_result = evaluate_expr(guard_expr, &arm_env)?;
                            match guard_result {
                                Value::Boolean(true) => true,
                                Value::Boolean(false) => false,
                                _ => {
                                    return Err(EvalError(
                                        "Match guard must evaluate to a Boolean".to_string(),
                                    ))
                                }
                            }
                        }
                        None => true,
                    };
                    if guard_passed {
                        return evaluate_expr(&arm.result, &arm_env); // Evaluate result in arm's env
                    }
                }
            }
            Err(EvalError(format!(
                "Match failed: No arm matched value {:?}",
                value_to_match
            )))
        }
    }
}

// Helper to evaluate literal values (standalone)
fn evaluate_literal(literal: &Literal) -> Result<Value, EvalError> {
    // Return Result for consistency
    match literal {
        Literal::Integer(i) => Ok(Value::Integer(*i)),
        Literal::Float(f) => Ok(Value::Float(*f)),
        Literal::String(s) => Ok(Value::String(s.clone())),
        Literal::Boolean(b) => Ok(Value::Boolean(*b)),
    }
}

// Helper to apply a function value (standalone)
fn apply_function(func: Value, args: Vec<Value>) -> Result<Value, EvalError> {
    match func {
        Value::Function(BorfFunction::Closure {
            params,
            body,
            env: captured_env,
        }) => {
            let mut call_env = Environment::with_parent(Rc::clone(&captured_env));
            if params.len() != args.len() {
                return Err(EvalError(format!(
                    "Arity mismatch: Function expected {} arguments, got {}",
                    params.len(),
                    args.len()
                )));
            }
            let mut bindings = HashMap::new();
            for (param_pattern, arg_value) in params.iter().zip(args.iter()) {
                if !bind_pattern(param_pattern, arg_value, &mut bindings)? {
                    return Err(EvalError(format!(
                        "Argument pattern mismatch: Pattern {:?} does not match value {:?}",
                        param_pattern, arg_value
                    )));
                }
            }
            for (name, value) in bindings {
                call_env.define(&name, value);
            }
            evaluate_expr(&body, &call_env) // Use standalone evaluate_expr
        }
        Value::Function(BorfFunction::Native(name)) => {
            apply_native_function(&name, args) // Use standalone native dispatcher
        }
        _ => Err(EvalError(format!(
            "Attempted to call a non-function value: {:?}",
            func
        ))),
    }
}

// Placeholder helper function (moved outside impl)
fn extract_params_from_signature(_signature: &Option<TypeExpr>) -> Vec<Pattern> {
    // TODO: This needs to parse the TypeExpr::Function variant if it exists
    // and somehow convert type expressions into patterns (likely just variable patterns).
    println!(
        "Warning: Parameter extraction from type signature not implemented. Assuming no params."
    );
    Vec::new()
}

// Pattern matching helper - matches a pattern against a value, updating bindings map
fn bind_pattern(
    pattern: &Pattern,
    value: &Value,
    bindings: &mut HashMap<String, Value>,
) -> Result<bool, EvalError> {
    // Match pattern against value, return true if successful, false if no match
    match pattern {
        // Literal patterns - match exact values
        Pattern::Literal(lit) => match (lit, value) {
            (Literal::Integer(i), Value::Integer(v)) => Ok(i == v),
            (Literal::Float(f), Value::Float(v)) => Ok(f == v),
            (Literal::String(s), Value::String(v)) => Ok(s == v),
            (Literal::Boolean(b), Value::Boolean(v)) => Ok(b == v),
            _ => Ok(false), // Type mismatch = no match
        },

        // Wildcard - matches any value
        Pattern::Wildcard => Ok(true),

        // Variable binding - always matches and binds the value
        Pattern::Variable(var) => {
            bindings.insert(var.name.clone(), value.clone());
            Ok(true)
        }

        // List pattern - match length and each element
        Pattern::List(list_pat) => {
            if let Value::List(value_list) = value {
                if list_pat.elements.len() != value_list.len() {
                    return Ok(false); // Length mismatch
                }

                // Match each element
                for (pat, val) in list_pat.elements.iter().zip(value_list.iter()) {
                    if !bind_pattern(pat, val, bindings)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            } else {
                Ok(false) // Not a list
            }
        }

        // Set pattern - similar to list but order doesn't matter
        // For simplicity, we'll implement the exact same list matching logic
        // In a real implementation, we'd do more sophisticated set matching
        Pattern::Set(set_pat) => {
            if let Value::Set(value_set) = value {
                if set_pat.elements.len() != value_set.len() {
                    return Ok(false); // Size mismatch
                }

                // This is a simplified approach - a real implementation would
                // handle matching elements regardless of order
                println!("Warning: Set pattern matching is naive (treats sets like lists)");
                for (pat, val) in set_pat.elements.iter().zip(value_set.iter()) {
                    if !bind_pattern(pat, val, bindings)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            } else {
                Ok(false) // Not a set
            }
        }
    }
}

// Remove the old placeholder module if present
// pub mod expr { ... }
