use borf::error_reporting::parse_string_to_module_with_enhanced_errors;
use borf::parser::{parse_repl_input, BorfParser, Rule};
use insta::assert_debug_snapshot;
use pest::Parser;

// Helper function to parse and capture AST for modules
fn parse_and_snapshot(name: &str, input: &str) {
    let result = parse_string_to_module_with_enhanced_errors(input, Some(name.to_string()));
    assert_debug_snapshot!(name, result);
}

// Helper for testing error cases
fn parse_and_snapshot_error(name: &str, input: &str) {
    let result = parse_string_to_module_with_enhanced_errors(input, Some(name.to_string()));
    assert_debug_snapshot!(name, result);
}

// Helper for REPL input testing
#[allow(dead_code)]
fn parse_repl_and_snapshot(name: &str, input: &str) {
    let result = parse_repl_input(input, Some(name.to_string()));
    assert_debug_snapshot!(name, result);
}

// Helper for direct grammar rule testing
fn parse_rule_and_snapshot(name: &str, rule: Rule, input: &str) {
    let result = BorfParser::parse(rule, input);
    assert_debug_snapshot!(name, result);
}

#[test]
#[ignore = "Parser is still being developed, snapshots are unstable"]
fn test_module_declaration() {
    parse_and_snapshot(
        "simple_module",
        r#"
@MyModule: {
    typ: {Int String Bool}
    op: {add subtract}
    fn: {main helper}
}
"#,
    );
}

#[test]
#[ignore = "Parser is still being developed, snapshots are unstable"]
fn test_expressions() {
    // Test literal expressions - make sure they're complete expressions
    parse_rule_and_snapshot("integer_literal_rule", Rule::expr, "42");
    parse_rule_and_snapshot("float_literal_rule", Rule::expr, "3.14");
    parse_rule_and_snapshot("string_literal_rule", Rule::expr, "\"hello\"");
    parse_rule_and_snapshot("boolean_literal_rule", Rule::expr, "true");

    // Collection literals
    parse_rule_and_snapshot("list_literal_rule", Rule::expr, "[1, 2, 3]");
    parse_rule_and_snapshot("set_literal_rule", Rule::expr, "{1, 2, 3}");
    parse_rule_and_snapshot("map_literal_expr", Rule::expr, "{\"key1\": 1, \"key2\": 2}");
}

#[test]
#[ignore = "Parser is still being developed, snapshots are unstable"]
fn test_lambda_and_application() {
    parse_rule_and_snapshot("lambda_expr", Rule::expr, "[x] x + 1");
    parse_rule_and_snapshot("application_expr", Rule::expr, "(add 1 2)");
}

#[test]
#[ignore = "Parser is still being developed, snapshots are unstable"]
fn test_let_expressions() {
    parse_rule_and_snapshot("let_expression_rule", Rule::expr, "let x = 5 in x * 2");
}

#[test]
#[ignore = "Parser is still being developed, snapshots are unstable"]
fn test_ternary_expressions() {
    parse_rule_and_snapshot("ternary_rule", Rule::expr, "x iff x > 0 or_else 0");
}

#[test]
#[ignore = "Parser is still being developed, snapshots are unstable"]
fn test_quoting() {
    parse_rule_and_snapshot("quote_expr", Rule::expr, "'(x + 1)");
    parse_rule_and_snapshot("unquote_expr", Rule::expr, "~x");
    parse_rule_and_snapshot("unquote_splice_expr", Rule::expr, "~@list");
    parse_rule_and_snapshot("quasiquote_expr", Rule::expr, "`(x + ~y)");
}

#[test]
#[ignore = "Parser is still being developed, snapshots are unstable"]
fn test_patterns() {
    // Test patterns within a lambda context
    parse_rule_and_snapshot("lambda_with_variable", Rule::expr, "[x] x");
    parse_rule_and_snapshot("lambda_with_wildcard", Rule::expr, "[_] 42");
    parse_rule_and_snapshot("lambda_with_list_pattern", Rule::expr, "[[x, y]] x + y");
    parse_rule_and_snapshot("lambda_with_set_pattern", Rule::expr, "[{x, y}] x + y");
    parse_rule_and_snapshot(
        "lambda_with_map_pattern",
        Rule::expr,
        "[{\"key\": value}] value",
    );
    parse_rule_and_snapshot("lambda_with_type_annotation", Rule::expr, "[x: Int] x + 1");
}

#[test]
#[ignore = "Parser is still being developed, snapshots are unstable"]
fn test_type_expressions() {
    // Type expressions in a module
    parse_and_snapshot(
        "type_expressions",
        r#"
@TypeModule: {
    typ: {Int String List Option}

    // Type declarations with different type expressions
    Int -> Int;  // Function type
    Int * String;  // Product type
    Int + String;  // Sum type
    [Int];  // List type
    {Int};  // Set type
    ?Int;  // Option type
}
"#,
    );
}

// Error case tests
#[test]
#[ignore = "Parser is still being developed, snapshots are unstable"]
fn test_syntax_errors() {
    // Missing closing brace
    parse_and_snapshot_error(
        "missing_closing_brace",
        r#"
@ErrorModule: {
    typ: {Int}
}
"#,
    );

    // Invalid token in module declaration
    parse_and_snapshot_error(
        "invalid_token_in_module",
        r#"
@ErrorModule: {
    invalid_section: {something}
}
"#,
    );

    // Missing parameter in lambda
    parse_rule_and_snapshot(
        "missing_param_in_lambda",
        Rule::expr,
        "[] x + 1", // Missing parameter
    );

    // Unclosed string literal (should be caught by the parser)
    parse_rule_and_snapshot("unclosed_string_rule", Rule::string, "\"unclosed string");
}

// Direct rule testing
#[test]
#[ignore = "Parser is still being developed, snapshots are unstable"]
fn test_specific_rules() {
    // Test identifier rule
    parse_rule_and_snapshot("identifier_rule", Rule::identifier, "valid_identifier");
    parse_rule_and_snapshot(
        "identifier_with_symbols",
        Rule::identifier,
        "valid_identifier_123'?",
    );

    // Test literal rules
    parse_rule_and_snapshot("integer_rule", Rule::integer, "42");
    parse_rule_and_snapshot("float_rule", Rule::float, "3.14159");
    parse_rule_and_snapshot("string_rule", Rule::string, "\"hello world\"");
    parse_rule_and_snapshot("boolean_rule", Rule::boolean, "true");

    // Test collection rules directly
    parse_rule_and_snapshot(
        "list_literal_collection",
        Rule::collection_literal,
        "[1, 2, 3]",
    );
    parse_rule_and_snapshot(
        "map_literal_collection",
        Rule::collection_literal,
        "{\"key\": value}",
    );
    parse_rule_and_snapshot(
        "set_literal_collection",
        Rule::collection_literal,
        "{1, 2, 3}",
    );
}
