// Tests for Pratt parser and operator precedence
use super::build_expr_ast;
use crate::parser::ast::{
    Atom, Expression, InfixOperator, PostfixOperator, PrefixOperator, TypeExpr,
};
use crate::parser::error::{BorfError, SourceSpan};
use crate::parser::{build_type_ast, BorfParser, Rule};
use pest::Parser;

// Helper macro for more clear assertions on AST structure
macro_rules! assert_expr_matches {
    ($expr:expr, $pattern:pat $(if $guard:expr)? $(,)?) => {
        assert!(
            matches!($expr, $pattern $(if $guard)?),
            "Expression didn't match expected pattern.\nExpected: {}\nGot: {:?}",
            stringify!($pattern $(if $guard)?),
            $expr
        );
    };
}

// Helper function to parse expression text to AST
fn parse_expr(input: &str) -> Result<Expression, Box<BorfError>> {
    match BorfParser::parse(Rule::expression, input) {
        Ok(mut pairs) => {
            let pairs = pairs.next().unwrap().into_inner();
            build_expr_ast(pairs)
        }
        Err(e) => {
            // Create a custom error instead of using From trait
            Err(Box::new(BorfError::SyntaxError(
                crate::parser::error::SyntaxError::new(
                    &format!("Failed to parse expression: {}", e),
                    crate::parser::get_named_source(input),
                    SourceSpan::new(0.into(), input.len()),
                    "Check that the expression syntax is valid",
                    "Parse Error",
                ),
            )))
        }
    }
}

// Helper function to parse type expression text to AST
fn parse_type(input: &str) -> Result<TypeExpr, Box<BorfError>> {
    match BorfParser::parse(Rule::type_expr, input) {
        Ok(mut pairs) => {
            let pairs = pairs.next().unwrap().into_inner();
            build_type_ast(pairs)
        }
        Err(e) => Err(Box::new(BorfError::SyntaxError(
            crate::parser::error::SyntaxError::new(
                &format!("Failed to parse type expression: {}", e),
                crate::parser::get_named_source(input),
                SourceSpan::new(0.into(), input.len()),
                "Check that the type expression syntax is valid",
                "Parse Error",
            ),
        ))),
    }
}

//------------------------------------------------------------------
// BASIC PARSING TESTS
//------------------------------------------------------------------

#[test]
fn test_basic_parsing() {
    // Test a variety of basic expressions to ensure they parse
    let exprs = [
        "a",
        "a + b",
        "123",
        "true",
        "false",
        ":symbol",
        "\"string\"",
        "$some_dollar_id",
        "a * (b + c)",
        "a $and b",
        "$not a",
        "a => b",
        "a.field",
        "func(arg)",
        "a[i]",
    ];

    for expr in exprs {
        let result = parse_expr(expr);
        assert!(
            result.is_ok(),
            "Failed to parse basic expression '{}': {:?}",
            expr,
            result.err()
        );
    }
}

#[test]
fn test_error_handling() {
    // Test malformed expressions
    let bad_exprs = [
        "@",     // Module declaration, not expression
        "a +",   // Missing right operand
        "a ($",  // Malformed function call
        "a ? b", // Incomplete ternary
    ];

    for expr in bad_exprs {
        let result = parse_expr(expr);
        assert!(
            result.is_err(),
            "Expected parsing error for '{}', but got success",
            expr
        );
    }
}

//------------------------------------------------------------------
// POSTFIX OPERATOR PRECEDENCE TESTS
//------------------------------------------------------------------

#[test]
fn test_postfix_call_precedence() {
    // Function calls should have high precedence
    let expr = parse_expr("a + func(b)").unwrap();

    assert_expr_matches!(
        expr,
        Expression::InfixOp {
            op: InfixOperator::Add,
            ..
        }
    );

    if let Expression::InfixOp {
        ref lhs, ref rhs, ..
    } = expr
    {
        assert_expr_matches!(**lhs, Expression::AtomExpr(Atom::Identifier(..)));

        // Right side should be a function call, not an atom
        assert_expr_matches!(
            **rhs,
            Expression::PostfixOp {
                op: PostfixOperator::FunctionCall(..),
                ..
            }
        );
    }
}

#[test]
fn test_postfix_index_precedence() {
    // Array indexing should have high precedence
    let expr = parse_expr("a + b[c]").unwrap();

    assert_expr_matches!(
        expr,
        Expression::InfixOp {
            op: InfixOperator::Add,
            ..
        }
    );

    if let Expression::InfixOp {
        ref lhs, ref rhs, ..
    } = expr
    {
        assert_expr_matches!(**lhs, Expression::AtomExpr(Atom::Identifier(..)));

        // Right side should be an indexing operation, not an atom
        assert_expr_matches!(
            **rhs,
            Expression::PostfixOp {
                op: PostfixOperator::Index(..),
                ..
            }
        );
    }
}

#[test]
fn test_postfix_field_access_precedence() {
    // Field access should have high precedence
    let expr = parse_expr("a + b.field").unwrap();

    assert_expr_matches!(
        expr,
        Expression::InfixOp {
            op: InfixOperator::Add,
            ..
        }
    );

    if let Expression::InfixOp {
        ref lhs, ref rhs, ..
    } = expr
    {
        assert_expr_matches!(**lhs, Expression::AtomExpr(Atom::Identifier(..)));

        // Right side should be a field access, not an atom
        assert_expr_matches!(
            **rhs,
            Expression::PostfixOp {
                op: PostfixOperator::FieldAccess(..),
                ..
            }
        );
    }
}

//------------------------------------------------------------------
// PREFIX OPERATOR PRECEDENCE TESTS
//------------------------------------------------------------------

#[test]
fn test_prefix_not_precedence() {
    // Prefix $not should have higher precedence than infix operators
    let expr = parse_expr("$not a $and b").unwrap();

    assert_expr_matches!(
        expr,
        Expression::InfixOp {
            op: InfixOperator::And,
            ..
        }
    );

    if let Expression::InfixOp {
        ref lhs, ref rhs, ..
    } = expr
    {
        // Left side should be a prefix operation, not an atom
        assert_expr_matches!(
            **lhs,
            Expression::PrefixOp {
                op: PrefixOperator::Not,
                ..
            }
        );

        assert_expr_matches!(**rhs, Expression::AtomExpr(Atom::Identifier(..)));
    }
}

//------------------------------------------------------------------
// ARITHMETIC OPERATOR PRECEDENCE TESTS
//------------------------------------------------------------------

#[test]
fn test_multiplication_over_addition() {
    // Multiplication should have higher precedence than addition
    let expr = parse_expr("a + b * c").unwrap();

    assert_expr_matches!(
        expr,
        Expression::InfixOp {
            op: InfixOperator::Add,
            ..
        }
    );

    if let Expression::InfixOp {
        ref lhs, ref rhs, ..
    } = expr
    {
        assert_expr_matches!(**lhs, Expression::AtomExpr(Atom::Identifier(..)));

        // Right side should be a multiplication, not an atom
        assert_expr_matches!(
            **rhs,
            Expression::InfixOp {
                op: InfixOperator::Multiply,
                ..
            }
        );
    }
}

#[test]
fn test_division_over_addition() {
    // Division should have higher precedence than addition
    let expr = parse_expr("a + b / c").unwrap();

    assert_expr_matches!(
        expr,
        Expression::InfixOp {
            op: InfixOperator::Add,
            ..
        }
    );

    if let Expression::InfixOp {
        ref lhs, ref rhs, ..
    } = expr
    {
        assert_expr_matches!(**lhs, Expression::AtomExpr(Atom::Identifier(..)));

        // Right side should be a division, not an atom
        assert_expr_matches!(
            **rhs,
            Expression::InfixOp {
                op: InfixOperator::Divide,
                ..
            }
        );
    }
}

//------------------------------------------------------------------
// LOGICAL OPERATOR PRECEDENCE TESTS
//------------------------------------------------------------------

#[test]
fn test_and_over_or() {
    // Logical AND should have higher precedence than OR
    let expr = parse_expr("a $or b $and c").unwrap();

    assert_expr_matches!(
        expr,
        Expression::InfixOp {
            op: InfixOperator::Or,
            ..
        }
    );

    if let Expression::InfixOp {
        ref lhs, ref rhs, ..
    } = expr
    {
        assert_expr_matches!(**lhs, Expression::AtomExpr(Atom::Identifier(..)));

        // Right side should be an AND operation
        assert_expr_matches!(
            **rhs,
            Expression::InfixOp {
                op: InfixOperator::And,
                ..
            }
        );
    }
}

#[test]
fn test_and_over_implication() {
    // Logical AND should have higher precedence than implication
    let expr = parse_expr("a $and b => c").unwrap();

    assert_expr_matches!(
        expr,
        Expression::InfixOp {
            op: InfixOperator::Implies,
            ..
        }
    );

    if let Expression::InfixOp {
        ref lhs, ref rhs, ..
    } = expr
    {
        // Left side should be an AND operation
        assert_expr_matches!(
            **lhs,
            Expression::InfixOp {
                op: InfixOperator::And,
                ..
            }
        );

        assert_expr_matches!(**rhs, Expression::AtomExpr(Atom::Identifier(..)));
    }
}

#[test]
fn test_or_over_implication() {
    // Logical OR should have higher precedence than implication
    let expr = parse_expr("a $or b => c").unwrap();

    assert_expr_matches!(
        expr,
        Expression::InfixOp {
            op: InfixOperator::Implies,
            ..
        }
    );

    if let Expression::InfixOp {
        ref lhs, ref rhs, ..
    } = expr
    {
        // Left side should be an OR operation
        assert_expr_matches!(
            **lhs,
            Expression::InfixOp {
                op: InfixOperator::Or,
                ..
            }
        );

        assert_expr_matches!(**rhs, Expression::AtomExpr(Atom::Identifier(..)));
    }
}

//------------------------------------------------------------------
// COMPARISON OPERATOR PRECEDENCE TESTS
//------------------------------------------------------------------

#[test]
fn test_comparison_over_logical() {
    // Comparison operators should have higher precedence than logical operators
    let expr = parse_expr("a $and b < c").unwrap();

    assert_expr_matches!(
        expr,
        Expression::InfixOp {
            op: InfixOperator::And,
            ..
        }
    );

    if let Expression::InfixOp {
        ref lhs, ref rhs, ..
    } = expr
    {
        assert_expr_matches!(**lhs, Expression::AtomExpr(Atom::Identifier(..)));

        // Right side should be a comparison
        assert_expr_matches!(
            **rhs,
            Expression::InfixOp {
                op: InfixOperator::LessThan,
                ..
            }
        );
    }
}

#[test]
fn test_multiple_comparisons() {
    // Multiple comparison operators should be parsed as separate operations
    let expr = parse_expr("a < b = c").unwrap();

    assert_expr_matches!(
        expr,
        Expression::InfixOp {
            op: InfixOperator::Equal,
            ..
        }
    );

    if let Expression::InfixOp {
        ref lhs, ref rhs, ..
    } = expr
    {
        // Left side should be a comparison
        assert_expr_matches!(
            **lhs,
            Expression::InfixOp {
                op: InfixOperator::LessThan,
                ..
            }
        );

        assert_expr_matches!(**rhs, Expression::AtomExpr(Atom::Identifier(..)));
    }
}

//------------------------------------------------------------------
// SET OPERATOR PRECEDENCE TESTS
//------------------------------------------------------------------

#[test]
fn test_set_operators() {
    // Set union should have correct precedence
    let expr = parse_expr("a $cup b + c").unwrap();

    assert_expr_matches!(
        expr,
        Expression::InfixOp {
            op: InfixOperator::Union,
            ..
        }
    );

    if let Expression::InfixOp {
        ref lhs, ref rhs, ..
    } = expr
    {
        assert_expr_matches!(**lhs, Expression::AtomExpr(Atom::Identifier(..)));

        // Right side should be an addition
        assert_expr_matches!(
            **rhs,
            Expression::InfixOp {
                op: InfixOperator::Add,
                ..
            }
        );
    }

    // Set intersection should have higher precedence than union
    let expr = parse_expr("a $cup b $cap c").unwrap();

    assert_expr_matches!(
        expr,
        Expression::InfixOp {
            op: InfixOperator::Union,
            ..
        }
    );

    if let Expression::InfixOp {
        ref lhs, ref rhs, ..
    } = expr
    {
        assert_expr_matches!(**lhs, Expression::AtomExpr(Atom::Identifier(..)));

        // Right side should be a intersection
        assert_expr_matches!(
            **rhs,
            Expression::InfixOp {
                op: InfixOperator::Intersect,
                ..
            }
        );
    }
}

//------------------------------------------------------------------
// COMPOSITION OPERATOR PRECEDENCE TESTS
//------------------------------------------------------------------

#[test]
fn test_composition_operators() {
    // Dot composition should have high precedence
    let expr = parse_expr("a . b $and c").unwrap();

    assert_expr_matches!(
        expr,
        Expression::InfixOp {
            op: InfixOperator::And,
            ..
        }
    );

    if let Expression::InfixOp {
        ref lhs, ref rhs, ..
    } = expr
    {
        // Left side should be a composition
        assert_expr_matches!(
            **lhs,
            Expression::InfixOp {
                op: InfixOperator::Compose,
                ..
            }
        );

        assert_expr_matches!(**rhs, Expression::AtomExpr(Atom::Identifier(..)));
    }

    // Piping should have correct precedence
    let expr = parse_expr("a |> b . c").unwrap();

    assert_expr_matches!(
        expr,
        Expression::InfixOp {
            op: InfixOperator::Pipe,
            ..
        }
    );

    if let Expression::InfixOp {
        ref lhs, ref rhs, ..
    } = expr
    {
        assert_expr_matches!(**lhs, Expression::AtomExpr(Atom::Identifier(..)));

        // Right side should be a composition
        assert_expr_matches!(
            **rhs,
            Expression::InfixOp {
                op: InfixOperator::Compose,
                ..
            }
        );
    }
}

//------------------------------------------------------------------
// PARENTHESES TESTS
//------------------------------------------------------------------

#[test]
fn test_parentheses_override_precedence() {
    // Parentheses should override default precedence
    // Without parentheses: a + (b * c)
    // With parentheses: (a + b) * c
    let expr = parse_expr("(a + b) * c").unwrap();

    assert_expr_matches!(
        expr,
        Expression::InfixOp {
            op: InfixOperator::Multiply,
            ..
        }
    );

    if let Expression::InfixOp {
        ref lhs, ref rhs, ..
    } = expr
    {
        // Left side should be a parenthesized expression
        assert_expr_matches!(**lhs, Expression::AtomExpr(Atom::Paren(..)));

        if let Expression::AtomExpr(Atom::Paren(ref inner)) = **lhs {
            // Inside the parentheses should be an addition
            assert_expr_matches!(
                **inner,
                Expression::InfixOp {
                    op: InfixOperator::Add,
                    ..
                }
            );
        }
    }
}

#[test]
fn test_nested_parentheses() {
    // Test nested parentheses
    let expr = parse_expr("(a * (b + c)) * d").unwrap();

    assert_expr_matches!(
        expr,
        Expression::InfixOp {
            op: InfixOperator::Multiply,
            ..
        }
    );

    // Verify the nesting structure
    if let Expression::InfixOp { ref lhs, .. } = expr {
        assert_expr_matches!(**lhs, Expression::AtomExpr(Atom::Paren(..)));

        if let Expression::AtomExpr(Atom::Paren(ref outer)) = **lhs {
            assert_expr_matches!(
                **outer,
                Expression::InfixOp {
                    op: InfixOperator::Multiply,
                    ..
                }
            );

            if let Expression::InfixOp { ref rhs, .. } = **outer {
                assert_expr_matches!(**rhs, Expression::AtomExpr(Atom::Paren(..)));

                if let Expression::AtomExpr(Atom::Paren(ref inner)) = **rhs {
                    assert_expr_matches!(
                        **inner,
                        Expression::InfixOp {
                            op: InfixOperator::Add,
                            ..
                        }
                    );
                }
            }
        }
    }
}

//------------------------------------------------------------------
// ASSOCIATIVITY TESTS
//------------------------------------------------------------------

#[test]
fn test_left_associativity() {
    // Test left associative operators (a + b + c) = ((a + b) + c)
    let expr = parse_expr("a + b + c").unwrap();

    assert_expr_matches!(
        expr,
        Expression::InfixOp {
            op: InfixOperator::Add,
            ..
        }
    );

    if let Expression::InfixOp { ref lhs, .. } = expr {
        // Left side should be another addition
        assert_expr_matches!(
            **lhs,
            Expression::InfixOp {
                op: InfixOperator::Add,
                ..
            }
        );
    }

    // Test another left-associative operator
    let expr = parse_expr("a $and b $and c").unwrap();

    if let Expression::InfixOp { ref lhs, .. } = expr {
        // Left side should be another AND
        assert_expr_matches!(
            **lhs,
            Expression::InfixOp {
                op: InfixOperator::And,
                ..
            }
        );
    }
}

#[test]
fn test_right_associativity() {
    // Test right associative operators (a => b => c) = (a => (b => c))
    let expr = parse_expr("a => b => c").unwrap();

    assert_expr_matches!(
        expr,
        Expression::InfixOp {
            op: InfixOperator::Implies,
            ..
        }
    );

    if let Expression::InfixOp { ref rhs, .. } = expr {
        // Right side should be another implication
        assert_expr_matches!(
            **rhs,
            Expression::InfixOp {
                op: InfixOperator::Implies,
                ..
            }
        );
    }

    // Test equivalence (iff) which should also be right associative
    let expr = parse_expr("a $iff b $iff c").unwrap();

    if let Expression::InfixOp { ref rhs, .. } = expr {
        // Right side should be another iff
        assert_expr_matches!(
            **rhs,
            Expression::InfixOp {
                op: InfixOperator::Iff,
                ..
            }
        );
    }
}

//------------------------------------------------------------------
// MIXED OPERATOR TESTS
//------------------------------------------------------------------

#[test]
fn test_complex_expressions() {
    // Test a complex expression with multiple precedence levels
    let expr = parse_expr("a + b * c $and d < e => f $or g . h").unwrap();

    // Top level should be an implication
    assert_expr_matches!(
        expr,
        Expression::InfixOp {
            op: InfixOperator::Implies,
            ..
        }
    );

    if let Expression::InfixOp {
        ref lhs, ref rhs, ..
    } = expr
    {
        // Left side should be an AND
        assert_expr_matches!(
            **lhs,
            Expression::InfixOp {
                op: InfixOperator::And,
                ..
            }
        );

        // Right side should be an OR
        assert_expr_matches!(
            **rhs,
            Expression::InfixOp {
                op: InfixOperator::Or,
                ..
            }
        );
    }
}

//------------------------------------------------------------------
// TYPE EXPRESSION TESTS
//------------------------------------------------------------------

#[test]
fn test_basic_type_expressions() {
    // Test simple base types
    let type_expr = parse_type("A").unwrap();
    assert_expr_matches!(type_expr, TypeExpr::Base(..));

    // Test dollar identifier types
    let type_expr = parse_type("$T").unwrap();
    assert_expr_matches!(type_expr, TypeExpr::Base(..));

    // Test Sym type
    let type_expr = parse_type("Sym").unwrap();
    assert_expr_matches!(type_expr, TypeExpr::Sym);
}

#[test]
fn test_product_types() {
    // Test product types
    let type_expr = parse_type("A * B").unwrap();
    assert_expr_matches!(type_expr, TypeExpr::Product(..));

    // Test left associativity of product types
    let type_expr = parse_type("A * B * C").unwrap();
    assert_expr_matches!(type_expr, TypeExpr::Product(..));

    if let TypeExpr::Product(ref lhs, _) = type_expr {
        assert_expr_matches!(**lhs, TypeExpr::Product(..));
    }
}

#[test]
fn test_arrow_types() {
    // Test arrow types
    let type_expr = parse_type("A -> B").unwrap();
    assert_expr_matches!(type_expr, TypeExpr::Arrow(..));

    // Test right associativity of arrow types
    let type_expr = parse_type("A -> B -> C").unwrap();
    assert_expr_matches!(type_expr, TypeExpr::Arrow(..));

    if let TypeExpr::Arrow(_, ref rhs) = type_expr {
        assert_expr_matches!(**rhs, TypeExpr::Arrow(..));
    }
}

#[test]
fn test_set_list_types() {
    // Test set types
    let type_expr = parse_type("{A}").unwrap();
    assert_expr_matches!(type_expr, TypeExpr::Set(..));

    // Test list types
    let type_expr = parse_type("[A]").unwrap();
    assert_expr_matches!(type_expr, TypeExpr::List(..));
}

#[test]
fn test_prefix_type_operators() {
    // Test optional types
    let type_expr = parse_type("?A").unwrap();
    assert_expr_matches!(type_expr, TypeExpr::Optional(..));

    // Test linear types
    let type_expr = parse_type("!A").unwrap();
    assert_expr_matches!(type_expr, TypeExpr::Linear(..));
}

#[test]
fn test_type_precedence() {
    // Test that arrow has lower precedence than product
    let type_expr = parse_type("A * B -> C").unwrap();
    assert_expr_matches!(type_expr, TypeExpr::Arrow(..));

    if let TypeExpr::Arrow(ref lhs, _) = type_expr {
        assert_expr_matches!(**lhs, TypeExpr::Product(..));
    }

    // Test that prefix operators have highest precedence
    let type_expr = parse_type("?A * B").unwrap();
    assert_expr_matches!(type_expr, TypeExpr::Product(..));

    if let TypeExpr::Product(ref lhs, _) = type_expr {
        assert_expr_matches!(**lhs, TypeExpr::Optional(..));
    }
}

#[test]
fn test_parenthesized_type_expressions() {
    // Test parenthesized type expressions
    let type_expr = parse_type("(A -> B) -> C").unwrap();
    assert_expr_matches!(type_expr, TypeExpr::Arrow(..));

    if let TypeExpr::Arrow(ref lhs, _) = type_expr {
        assert_expr_matches!(**lhs, TypeExpr::Arrow(..));
    }
}
