use borf::parser::ast::{
    Declaration, Expression, Identifier, LawBody, ModuleDef, ModuleElement, PrimitiveDecl,
    PrimitiveElement, Quantifier, TopLevelItem, TypeExpr,
};
use borf::parser::{parse_program, SourceCache};
use std::fmt::Write as FmtWrite; // Use Write trait for building string
use std::fs;
use std::path::Path;

const INDENT_SIZE: usize = 2;

fn main() {
    println!("Prelude Formatter Tool");
    println!("=====================");

    let source_path = Path::new("src/prelude/mod.borf");
    let dest_path = Path::new("src/prelude/mod.formatted.borf");

    println!("Reading source: {:?}", source_path);
    let prelude_content = match fs::read_to_string(source_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading prelude file '{:?}': {}", source_path, e);
            return;
        }
    };

    println!("Parsing prelude...");
    let _cache = SourceCache::new();
    let ast = match parse_program(&prelude_content) {
        Ok(items) => items,
        Err(err) => {
            eprintln!("Failed to parse prelude:");
            eprintln!("{:?}", err);
            return;
        }
    };

    println!("Formatting AST...");
    let formatted_content = match format_ast(&ast) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error formatting AST: {}", e);
            return;
        }
    };

    println!("Writing formatted output to {:?}", dest_path);
    if let Err(e) = fs::write(dest_path, &formatted_content) {
        eprintln!("Error writing formatted prelude: {}", e);
        return;
    }

    println!("Formatting complete.");
    let original_lines = prelude_content.lines().count();
    let formatted_lines = formatted_content.lines().count();
    println!("  Original line count: {}", original_lines);
    println!("  Formatted line count: {}", formatted_lines);
    // Note: line count difference isn't always meaningful
}

// --- AST Formatting Logic ---

fn make_indent(level: usize) -> String {
    " ".repeat(level * INDENT_SIZE)
}

// Main entry point for formatting
fn format_ast(items: &[TopLevelItem]) -> Result<String, std::fmt::Error> {
    let mut output = String::new();
    for item in items {
        format_top_level_item(&mut output, item, 0)?;
        writeln!(output)?;
        writeln!(output)?; // Add an extra blank line between top-level items
    }
    Ok(output)
}

// Format individual top-level items
fn format_top_level_item(
    output: &mut String,
    item: &TopLevelItem,
    indent_level: usize,
) -> Result<(), std::fmt::Error> {
    let indent = make_indent(indent_level);
    match item {
        TopLevelItem::Module(module_def) => format_module(output, module_def, indent_level)?,
        TopLevelItem::Primitive(primitive_def) => {
            format_primitive(output, primitive_def, indent_level)?
        }
        TopLevelItem::Export(name) => write!(output, "{}export {}", indent, name)?,
        TopLevelItem::Import(name) => write!(output, "{}import {}", indent, name)?,
        TopLevelItem::ExpressionStatement(expr) => {
            write!(output, "{}", indent)?;
            format_expression(output, expr, indent_level)?;
            // No semicolon
        }
    }
    Ok(())
}

// Format a module definition
fn format_module(
    output: &mut String,
    module: &ModuleDef,
    indent_level: usize,
) -> Result<(), std::fmt::Error> {
    let indent = make_indent(indent_level);
    write!(output, "{}@{} ", indent, module.name)?;
    if let Some(param) = &module.type_param {
        write!(output, "<{}> ", param)?;
    }
    writeln!(output, ": {{")?;

    for element in &module.elements {
        format_module_element(output, element, indent_level + 1)?;
        writeln!(output)?;
    }

    write!(output, "{}}}", indent)?; // No semicolon
    Ok(())
}

// Format a primitive definition
fn format_primitive(
    output: &mut String,
    primitive: &PrimitiveDecl,
    indent_level: usize,
) -> Result<(), std::fmt::Error> {
    let indent = make_indent(indent_level);
    writeln!(output, "{}@Primitives: {{", indent)?; // Hardcoded name for now

    for element in &primitive.elements {
        format_primitive_element(output, element, indent_level + 1)?;
        writeln!(output)?;
    }

    write!(output, "{}}}", indent)?; // No semicolon
    Ok(())
}

// Format module elements
fn format_module_element(
    output: &mut String,
    element: &ModuleElement,
    indent_level: usize,
) -> Result<(), std::fmt::Error> {
    match element {
        ModuleElement::Declaration(decl) => format_declaration(output, decl, indent_level)?,
        // Add cases for comments or other element types if they exist in AST
    }
    Ok(())
}

// Format primitive elements
fn format_primitive_element(
    output: &mut String,
    element: &PrimitiveElement,
    indent_level: usize,
) -> Result<(), std::fmt::Error> {
    match element {
        PrimitiveElement::Declaration(decl) => format_declaration(output, decl, indent_level)?,
        // Add cases for comments or other element types if they exist in AST
    }
    Ok(())
}

// Format declarations
fn format_declaration(
    output: &mut String,
    declaration: &Declaration,
    indent_level: usize,
) -> Result<(), std::fmt::Error> {
    let indent = make_indent(indent_level);
    write!(output, "{}", indent)?;
    match declaration {
        Declaration::ObjectDecl {
            name,
            type_constraint,
        } => {
            write!(output, "{}", name)?;
            if let Some(constraint) = type_constraint {
                write!(output, " $in ")?;
                format_type_expr(output, constraint, indent_level)?;
            }
            // No semicolon
        }
        Declaration::MappingDecl {
            name,
            type_constraint,
            value,
            constraint,
        } => {
            write!(output, "{}", name)?;
            if let Some(tc) = type_constraint {
                write!(output, ": ")?;
                format_type_expr(output, tc, indent_level)?;
            }
            if let Some(val) = value {
                write!(output, " = ")?;
                format_expression(output, val, indent_level)?;
            }
            if let Some(con) = constraint {
                write!(output, " | ")?;
                format_expression(output, con, indent_level)?;
            }
            // No semicolon
        }
        Declaration::LawDecl { name, body } => {
            write!(output, "{} = ", name)?;
            match body {
                LawBody::Expression(expr) => {
                    format_expression(output, expr, indent_level)?;
                }
                LawBody::Block {
                    quantifier,
                    variable,
                    domain,
                    elements,
                } => {
                    let q_str = match quantifier {
                        Quantifier::Forall => "$forall",
                        Quantifier::Exists => "$exists",
                        Quantifier::ExistsUnique => "$exists!", // Assuming this exists
                    };
                    writeln!(output, "{} {} $in {} {{", q_str, variable, domain)?;
                    for element in elements {
                        format_module_element(output, element, indent_level + 1)?;
                        writeln!(output)?;
                    }
                    write!(output, "{}}}", indent)?; // No semicolon after block
                }
            }
        }
    }
    Ok(())
}

// Format type expressions (Simplified initial version)
fn format_type_expr(
    output: &mut String,
    type_expr: &TypeExpr,
    _indent_level: usize, // Not used yet, but keep for future multi-line formatting
) -> Result<(), std::fmt::Error> {
    match type_expr {
        TypeExpr::TypeName(name) => write!(output, "{}", name)?,
        TypeExpr::DollarName(name) => write!(output, "{}", name)?,
        TypeExpr::QualifiedName(mod_name, type_name) => {
            write!(output, "{}.{}", mod_name, type_name)?
        }
        TypeExpr::Product(left, right) => {
            format_type_expr(output, left, _indent_level)?;
            write!(output, " * ")?;
            format_type_expr(output, right, _indent_level)?;
        }
        TypeExpr::Sum(left, right) => {
            format_type_expr(output, left, _indent_level)?;
            write!(output, " + ")?;
            format_type_expr(output, right, _indent_level)?;
        }
        TypeExpr::Function(domain, codomain) => {
            // Add parens for nested functions? (Maybe later)
            format_type_expr(output, domain, _indent_level)?;
            write!(output, " -> ")?;
            format_type_expr(output, codomain, _indent_level)?;
        }
        TypeExpr::Set(inner) => {
            write!(output, "{{")?;
            format_type_expr(output, inner, _indent_level)?;
            write!(output, "}}")?;
        }
        TypeExpr::List(inner) => {
            write!(output, "[")?;
            format_type_expr(output, inner, _indent_level)?;
            write!(output, "]")?;
        }
        TypeExpr::Record(fields) => {
            write!(output, "{{ ")?;
            for (i, (name, ty)) in fields.iter().enumerate() {
                if i > 0 {
                    write!(output, ", ")?;
                }
                write!(output, "{}: ", name)?;
                format_type_expr(output, ty, _indent_level)?;
            }
            write!(output, " }}")?;
        }
        TypeExpr::Optional(inner) => {
            write!(output, "?")?;
            format_type_expr(output, inner, _indent_level)?;
        }
        TypeExpr::Linear(inner) => {
            write!(output, "!")?;
            format_type_expr(output, inner, _indent_level)?;
        }
        TypeExpr::Tuple(elements) => {
            write!(output, "(")?;
            for (i, elem) in elements.iter().enumerate() {
                if i > 0 {
                    write!(output, ", ")?;
                }
                format_type_expr(output, elem, _indent_level)?;
            }
            write!(output, ")")?;
        }
        TypeExpr::Union(left, right) => {
            format_type_expr(output, left, _indent_level)?;
            write!(output, " $cup ")?;
            format_type_expr(output, right, _indent_level)?;
        }
        TypeExpr::Intersection(left, right) => {
            format_type_expr(output, left, _indent_level)?;
            write!(output, " $cap ")?;
            format_type_expr(output, right, _indent_level)?;
        }
        TypeExpr::Range(name, modifier) => write!(output, "{}{}", name, modifier)?,
        TypeExpr::MultiParamFunction(params, ret) => {
            write!(output, "(")?;
            for (i, param) in params.iter().enumerate() {
                if i > 0 {
                    write!(output, ",")?;
                }
                format_type_expr(output, param, _indent_level)?;
            }
            write!(output, ") -> ")?;
            format_type_expr(output, ret, _indent_level)?;
        }
        TypeExpr::Constrained(ty, expr) => {
            format_type_expr(output, ty, _indent_level)?;
            write!(output, " | ")?;
            format_expression(output, expr, _indent_level)?;
        } // Add other TypeExpr variants as needed
    }
    Ok(())
}

// Format expressions (Simplified initial version - does not handle precedence/parens)
fn format_expression(
    output: &mut String,
    expr: &Expression,
    _indent_level: usize, // Not used yet
) -> Result<(), std::fmt::Error> {
    match expr {
        Expression::Identifier(Identifier(name)) => write!(output, "{}", name)?,
        Expression::DollarIdentifier(name) => write!(output, "{}", name)?,
        Expression::QualifiedName(mod_name, name) => write!(output, "{}.{}", mod_name, name)?,
        Expression::LawIdentifier(name) => write!(output, "{}", name)?,
        Expression::IntLiteral(val) => write!(output, "{}", val)?,
        Expression::BoolLiteral(val) => write!(output, "{}", val)?,
        Expression::StringLiteral(val) => write!(output, "\"{}\"", val)?,
        Expression::SymbolLiteral(val) => write!(output, ":{}", val)?,
        Expression::OperatorName(op) => write!(output, "{}", op)?,
        Expression::BinaryOp { left, op, right } => {
            format_expression(output, left, _indent_level)?;
            write!(output, " {} ", op)?;
            format_expression(output, right, _indent_level)?;
        }
        Expression::PrefixOp { op, expr } => {
            write!(output, "{}", op)?;
            // Add space maybe? Depends on operator
            if op == "$not" {
                write!(output, " ")?;
            }
            format_expression(output, expr, _indent_level)?;
        }
        Expression::FunctionCall { func, args } => {
            format_expression(output, func, _indent_level)?;
            write!(output, "(")?;
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    write!(output, ", ")?;
                }
                format_expression(output, arg, _indent_level)?;
            }
            write!(output, ")")?;
        }
        Expression::FieldAccess { base, field } => {
            format_expression(output, base, _indent_level)?;
            write!(output, ".{}", field)?;
        }
        Expression::IndexAccess { base, index } => {
            format_expression(output, base, _indent_level)?;
            write!(output, "[")?;
            format_expression(output, index, _indent_level)?;
            write!(output, "]")?;
        }
        Expression::Lambda { params, body } => {
            write!(output, "\\{}", params.join(", "))?;
            write!(output, ".")?;
            // Maybe add space?
            format_expression(output, body, _indent_level)?;
        }
        Expression::IfThenElse {
            condition,
            then_branch,
            else_branch,
        } => {
            write!(output, "if ")?;
            format_expression(output, condition, _indent_level)?;
            write!(output, " then ")?;
            format_expression(output, then_branch, _indent_level)?;
            write!(output, " else ")?;
            format_expression(output, else_branch, _indent_level)?;
        }
        Expression::Conditional {
            condition,
            then_expr,
            else_expr,
        } => {
            format_expression(output, condition, _indent_level)?;
            write!(output, " ? ")?;
            format_expression(output, then_expr, _indent_level)?;
            write!(output, " : ")?;
            format_expression(output, else_expr, _indent_level)?;
        }
        Expression::LetRec { bindings, body } => {
            write!(output, "let rec ")?;
            // Simple formatting for now
            for (i, (name, val)) in bindings.iter().enumerate() {
                if i > 0 {
                    write!(output, " and ")?;
                }
                write!(output, "{} = ", name)?;
                format_expression(output, val, _indent_level)?;
            }
            write!(output, " in ")?;
            format_expression(output, body, _indent_level)?;
        }
        Expression::SetLiteral(elements) => {
            write!(output, "{{")?;
            for (i, elem) in elements.iter().enumerate() {
                if i > 0 {
                    write!(output, ", ")?;
                }
                format_expression(output, elem, _indent_level)?;
            }
            write!(output, "}}")?;
        }
        Expression::EmptySet => write!(output, "{{}}")?,
        Expression::EmptyList => write!(output, "[]")?,
        Expression::Tuple(elements) => {
            write!(output, "(")?;
            for (i, elem) in elements.iter().enumerate() {
                if i > 0 {
                    write!(output, ", ")?;
                }
                format_expression(output, elem, _indent_level)?;
            }
            write!(output, ")")?;
        }
        Expression::Quantified {
            quantifier,
            variables,
            domain,
            body,
            ..
        } => {
            let q_str = match quantifier {
                Quantifier::Forall => "$forall",
                Quantifier::Exists => "$exists",
                Quantifier::ExistsUnique => "$exists!",
            };
            write!(output, "{} {} $in ", q_str, variables.join(", "))?;
            format_expression(output, domain, _indent_level)?;
            // Handle optional domain/constraint later if needed
            write!(output, ": ")?;
            format_expression(output, body, _indent_level)?;
        }

        // Add other Expression variants here as needed
        _ => write!(output, "/* unformatted {:?} */", expr)?, // Placeholder for unhandled variants
    }
    Ok(())
}
