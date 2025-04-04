# Pratt Parser Test Plan

This document outlines a comprehensive test plan for the Borf language's Pratt parser implementation, focusing on operator precedence and associativity rules.

## Current Status

The Borf parser is currently in transition, with the grammar file (borf.pest) updated to use a flattened structure for expressions, relying on a Pratt parser implemented in Rust to handle operator precedence and associativity. However, many of the parser implementation files still reference grammar rules that have been removed or renamed.

## Test Categories

Once the codebase stabilizes, the following test categories should be implemented:

### 1. Basic Parsing Tests

- Test successful parsing of different expression types
- Test error handling for invalid expressions
- Test basic recognition of all supported operators

### 2. Operator Precedence Tests

Verify that the parser handles precedence correctly for:

- Postfix operators (highest precedence: function calls, field access, indexing)
- Prefix operators ($not, ?, !)
- Multiplicative operators (*, /)
- Additive operators (+, -)
- Composition operators (., >>, |>)
- Set operators ($cup, $cap, $subseteq)
- Comparison operators (=, ==, ===, $teq, $veq, $seq, $ceq, <::, >, <, >=, <=, $in, $omega)
- Logical AND ($and)
- Logical OR ($or)
- Implication (=>)
- Equivalence ($iff)
- Ternary operator (? :)

### 3. Associativity Tests

Verify correct associativity for:

- Left-associative operators (+, -, *, /, $cup, $cap, $and, $or)
- Right-associative operators (=>, $iff, ->)
- Composition operators (., >>, |>)

### 4. Parenthesized Expressions

- Test that parentheses override default precedence
- Test nested parentheses
- Test combinations of parentheses with different operator types

### 5. Mixing Different Operator Types

- Test expressions mixing arithmetic and logical operators
- Test expressions mixing comparison and logical operators
- Test expressions mixing set operators with other operator types
- Test complex expressions with multiple precedence levels

### 6. Type Expression Tests

- Test basic type expressions (A, B)
- Test product types (A * B)
- Test function types (A -> B)
- Test set types ({A})
- Test list types ([A])
- Test optional and linear types (?A, !A)
- Test nested type expressions (A -> B -> C, A * B * C)
- Test parenthesized type expressions ((A -> B) -> C)

### 7. Special Expression Types

- Test lambda expressions
- Test if-then-else expressions
- Test let-rec expressions
- Test quantifier expressions ($forall, $exists)
- Test set expressions and comprehensions

## Test Implementation Strategy

For each test category, implement tests that:

1. Verify successful parsing of valid expressions
2. Check that the resulting AST structure matches the expected operator precedence
3. Validate associativity by examining the nesting structure of the AST

### AST Structure Validation

For complex expressions, validate the AST structure using one of these approaches:

1. Pattern matching on the AST structure to verify the expected shape
2. Serializing the AST to a string and comparing with expected structure
3. Writing a custom AST matcher function that checks specific properties

## Example Test Structure

Here's an example of how to structure precedence tests:

```rust
#[test]
fn test_arithmetic_precedence() {
    // Verify that multiplication has higher precedence than addition
    let expr = parse_expr("a + b * c");
    assert_ast_shape!(expr, 
        InfixOp {
            op: Add,
            lhs: Identifier("a"),
            rhs: InfixOp {
                op: Multiply,
                lhs: Identifier("b"),
                rhs: Identifier("c")
            }
        }
    );
}
```

## Tracking Test Coverage

Track test coverage to ensure all operators and combinations are tested:

1. All individual operators should be tested
2. All operator precedence relationships should be tested
3. All associativity rules should be verified
4. All expression types should have test cases

## Implementation Plan

1. First, stabilize the parser code to match the updated grammar
2. Implement basic parsing tests for each operator type
3. Implement precedence tests for each operator category
4. Implement associativity tests
5. Add tests for complex expressions and special cases
6. Add tests for type expressions
7. Continuously measure test coverage and add tests for gaps 