# Comprehensive Pratt Parser Test Suite

## Overview

This PR adds a comprehensive test suite for the Borf language's Pratt parser implementation, focusing on operator precedence and associativity rules. It provides thorough testing of the parser's ability to correctly handle expressions with diverse operators and precedence levels.

## Changes

- Created a new test module `src/parser/pratt_tests.rs`
- Added helper functions for expression and type parsing
- Added an `assert_expr_matches!` macro for more readable test assertions
- Implemented a comprehensive set of tests in the following categories:
  - Basic parsing and error handling
  - Postfix operator precedence
  - Prefix operator precedence
  - Arithmetic operator precedence
  - Logical operator precedence
  - Comparison operator precedence
  - Set operator precedence
  - Composition operator precedence
  - Parentheses and nesting
  - Operator associativity (left vs right)
  - Complex mixed expressions
  - Type expressions

## Testing Strategy

The test suite verifies:
1. That expressions parse correctly
2. That operator precedence is respected in the resulting AST
3. That associativity rules are correctly applied
4. That special expressions and type expressions parse correctly

Each test follows a pattern of:
1. Parse an expression containing operators with different precedence levels
2. Verify the top-level structure matches expectations
3. Examine the nested structure to confirm operators are grouped correctly

## Future Work

- Once the parser implementation is stabilized, additional tests can be added for:
  - Ternary operators
  - More complex type expressions
  - Edge cases for all operator types
  - Custom AST matchers for more concise assertions

## Documentation

The included `docs/pratt_parser_test_plan.md` provides a detailed roadmap for further test development and identifies all the operator precedence relationships that should be tested. 