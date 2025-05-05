use crate::ast::Expr;
use chumsky::prelude::*;
use chumsky::text::{digits, int};

/// Build a parser for Q expressions (ints, floats, +, -, *, /, parentheses).
pub fn expr_parser() -> impl Parser<char, Expr, Error = Simple<char>> {
    recursive(|expr| {
        // Floating-point: digits.digits
        let float = int(10)
            .then_ignore(just('.'))
            .then(digits(10))
            .map(|(int_part, frac): (String, String)| {
                Expr::Float(format!("{}.{}", int_part, frac).parse().unwrap())
            })
            .padded();

        // Integer literal
        let integer = int(10)
            .map(|s: String| Expr::Int(s.parse().unwrap()))
            .padded();

        // Parenthesized expression
        let parenthesized = just('(')
            .padded()
            .ignore_then(expr.clone())
            .then_ignore(just(')').padded());

        // Atom: float, integer, or parenthesized
        let atom = float.or(integer).or(parenthesized);

        // * or / operator parser yielding the operator char
        // * or / operator parser yielding the operator char
        let op_mul_div = just('*').or(just('/')).padded();

        // Product: left-associative multiplication/division
        let product = atom
            .clone()
            .then(op_mul_div.then(atom.clone()).repeated())
            .foldl(|l, (op, r)| match op {
                '*' => Expr::Mul(Box::new(l), Box::new(r)),
                '/' => Expr::Div(Box::new(l), Box::new(r)),
                _ => unreachable!(),
            });

        // + or - operator parser yielding the operator char
        let op_add_sub = just('+').or(just('-')).padded();

        // Sum: left-associative addition/subtraction
        product
            .clone()
            .then(op_add_sub.then(product.clone()).repeated())
            .foldl(|l, (op, r)| match op {
                '+' => Expr::Add(Box::new(l), Box::new(r)),
                '-' => Expr::Sub(Box::new(l), Box::new(r)),
                _ => unreachable!(),
            })
    })
}

// (a top-level `parse` function is provided in lib.rs)

/// Parse a Q expression from the input string.
/// Returns the AST on success, or a stringified error.
pub fn parse_expr(input: &str) -> Result<Expr, String> {
    expr_parser().parse(input).map_err(|errs| {
        errs.into_iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    })
}
