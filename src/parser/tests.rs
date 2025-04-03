use super::parse_program;

#[test]
fn test_parse_empty() {
    let input = "";
    assert!(parse_program(input).is_ok());
    assert!(parse_program(input).unwrap().is_empty());
}

#[test]
fn test_parse_comment_only() {
    let input = "-- this is a comment
--[[ block comment ]]--";
    assert!(parse_program(input).is_ok());
    assert!(parse_program(input).unwrap().is_empty());
}

#[test]
fn test_parse_basic_category() {
    let input = "@MyCat: {}";
    let result = parse_program(input);
    assert!(
        result.is_ok(),
        "Failed to parse basic category: {:?}",
        result.err()
    );
    let items = result.unwrap();
    assert_eq!(items.len(), 1);
    // Add more specific AST checks later if needed
}

#[test]
fn test_parse_category_with_object() {
    let input = "@MyCat: { Obj; }";
    let result = parse_program(input);
    assert!(
        result.is_ok(),
        "Failed to parse category with object: {:?}",
        result.err()
    );
    // Further AST checks...
}

#[test]
fn test_parse_category_with_mapping() {
    let input = "@MyCat: { map: A -> B; }";
    let result = parse_program(input);
    assert!(
        result.is_ok(),
        "Failed to parse category with mapping: {:?}",
        result.err()
    );
    // Further AST checks...
}

#[test]
fn test_parse_category_with_law() {
    let input = "@MyCat: { law.refl = $forall x $in X: x $veq x; }";
    let result = parse_program(input);
    assert!(
        result.is_ok(),
        "Failed to parse category with law: {:?}",
        result.err()
    );
    // Further AST checks...
}

#[test]
fn test_parse_category_with_func_def() {
    let input = "@MyCat: { my_func: X -> Y = \\x. x; }";
    let result = parse_program(input);
    assert!(
        result.is_ok(),
        "Failed to parse category with func def: {:?}",
        result.err()
    );
    // Further AST checks...
}

#[test]
fn test_parse_pipeline_def() {
    let input = "@MyPipeline: input In output Out steps { Step1, Step2 }";
    let result = parse_program(input);
    assert!(
        result.is_ok(),
        "Failed to parse pipeline def: {:?}",
        result.err()
    );
    // Further AST checks...
}

#[test]
fn test_parse_pipe_expr() {
    let input = "initial_step |> next_step |> final_step";
    let result = parse_program(input);
    assert!(
        result.is_ok(),
        "Failed to parse pipe expr: {:?}",
        result.err()
    );
    // Further AST checks...
}

#[test]
fn test_parse_app_expr() {
    let input = "my_function(arg)";
    let result = parse_program(input);
    assert!(
        result.is_ok(),
        "Failed to parse app expr: {:?}",
        result.err()
    );
    // Further AST checks...
}

#[test]
fn test_parse_composition_statement() {
    // Note: Grammar is `ident = ident . ident . ... ( ident )`
    // This seems unusual, let's test exactly what the grammar specifies.
    let input = "result = f.g(input)";
    let result = parse_program(input);
    assert!(
        result.is_ok(),
        "Failed to parse composition statement: {:?}",
        result.err()
    );
    // Further AST checks...
}

#[test]
fn test_parse_export_statement() {
    let input = "export MyModule;";
    let result = parse_program(input);
    assert!(
        result.is_ok(),
        "Failed to parse export statement: {:?}",
        result.err()
    );
    // Further AST checks...
}

#[test]
fn test_parse_import_statement() {
    let input = r#"import "./my_other_module.borf";"#; // Use raw string literal
    let result = parse_program(input);
    assert!(
        result.is_ok(),
        "Failed to parse import statement: {:?}",
        result.err()
    );
    // Further AST checks...
}

#[test]
fn test_parse_symbol_literal_in_expr() {
    // Assuming symbols can appear in places like set literals or function args
    let input = "@Data: { stuff = {:Symbol1, :Symbol2}; }";
    let result = parse_program(input);
    assert!(
        result.is_ok(),
        "Failed to parse symbol literal: {:?}",
        result.err()
    );
    // Further AST checks...
}

#[test]
fn test_parse_set_comprehension() {
    let input = "@Sets: { subset = {x $in FullSet | filter(x)}; }";
    let result = parse_program(input);
    assert!(
        result.is_ok(),
        "Failed to parse set comprehension: {:?}",
        result.err()
    );
    // Further AST checks...
}

#[test]
fn test_parse_lambda_expr() {
    let input = "@Funcs: { double = \\x. mul(x, 2); }"; // Simplified example, escaped backslash
    let result = parse_program(input);
    assert!(
        result.is_ok(),
        "Failed to parse lambda expr: {:?}",
        result.err()
    );
    // Further AST checks...
}

#[test]
fn test_parse_invalid_syntax() {
    let input = "@Invalid {"; // Missing colon and closing brace
    assert!(parse_program(input).is_err());
}

#[test]
fn test_parse_multiple_statements() {
    // Revert back to raw string literal
    let input = r#"
        @CatA: { A; }
        export CatA;
        @CatB: { B; f: A -> B; }
        import "cat_a.borf";
    "#;
    let result = parse_program(input);
    assert!(
        result.is_ok(),
        "Failed to parse multiple statements: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap().len(), 4);
}
