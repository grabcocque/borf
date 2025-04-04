use pest::error::LineColLocation;
use pest::Parser;
use std::fs;
use std::path::Path;

use super::BorfParser;
use super::Rule;

// Simple test to verify Pest grammar can parse the prelude file
#[test]
fn test_prelude_parsing() {
    // Get the prelude file path
    let prelude_path = Path::new("src/prelude/mod.borf");

    // Read the prelude file
    let prelude_content =
        fs::read_to_string(prelude_path).expect("Should be able to read the prelude file");

    // Try parsing with our Pest grammar
    let parse_result = BorfParser::parse(Rule::program, &prelude_content);

    // Print detailed error if parsing fails
    if let Err(err) = &parse_result {
        eprintln!("Parsing error: {}", err);

        // Extract line and column information
        match &err.line_col {
            LineColLocation::Pos((line, col)) => {
                // Get the problematic line
                let lines: Vec<&str> = prelude_content.lines().collect();
                let problematic_line = if *line > 0 && *line <= lines.len() {
                    lines[*line - 1] // line is 1-indexed
                } else {
                    "(line out of range)"
                };

                eprintln!("Error at line {}, column {}:", line, col);
                eprintln!("{}", problematic_line);
                eprintln!("{}^", " ".repeat(col.saturating_sub(1))); // Create pointer to error position
            }
            LineColLocation::Span((start_line, start_col), (end_line, end_col)) => {
                eprintln!(
                    "Error spanning from line {}, column {} to line {}, column {}",
                    start_line, start_col, end_line, end_col
                );

                // Print the affected lines
                let lines: Vec<&str> = prelude_content.lines().collect();
                for line_num in *start_line..=*end_line {
                    if line_num > 0 && line_num <= lines.len() {
                        eprintln!("{}: {}", line_num, lines[line_num - 1]);
                    }
                }
            }
        }

        // If we have expected tokens, print them
        match &err.variant {
            pest::error::ErrorVariant::ParsingError {
                positives,
                negatives,
            } => {
                if !positives.is_empty() {
                    eprintln!("Expected one of: {:?}", positives);
                }
                if !negatives.is_empty() {
                    eprintln!("Unexpected: {:?}", negatives);
                }
            }
            _ => {}
        }
    }

    // Assert that parsing succeeded
    assert!(
        parse_result.is_ok(),
        "Failed to parse prelude with pest grammar"
    );

    // Optionally: print some info about the parsed structure
    if let Ok(pairs) = parse_result {
        let count = pairs.clone().count();
        println!("Successfully parsed prelude with {} top-level pairs", count);

        // Print the first few rules (for debugging)
        for (i, pair) in pairs.enumerate().take(5) {
            println!(
                "Rule {}: {:?} spanning {:?}",
                i,
                pair.as_rule(),
                pair.as_span()
            );
        }
    }
}

// Helper function to wrap test content in a proper program structure
fn wrap_as_program(content: &str) -> String {
    format!("{}\n", content)
}

#[test]
fn test_parse_primitives_module() {
    let module = r#"@Primitives: {
      use_once: !Any -> Bool;
      matches: Pattern*Net -> Bool;
      a_uses: $alpha*B -> Bool;
      processed: B -> Bool;
      extract_data: Net -> S;
      safe_pipeline: S -> Net;
      apply: $rho*Net -> Net;
      $append: [Any]*Any -> [Any];
      $ne: {Any} -> Bool;
    }"#;

    let parse_result = BorfParser::parse(Rule::module_declaration, module);
    assert!(
        parse_result.is_ok(),
        "Failed to parse Primitives module: {:?}",
        parse_result.err()
    );
}

#[test]
fn test_parse_type_system_module() {
    let module = r#"@T: {  -- Type universe
      T;                              -- Set of all types
      <:: T*T->Bool;                  -- Subtyping relation (infix)
      ~: Any->Bool;                   -- Type predicate
      $teq: T*T->Bool;                -- Type equivalence
      law.refl = $forall t $in T: t<::t;
      law.trans = $forall a,b,c $in T: a<::b $and b<::c => a<::c;
      U $in T; B $in T; N $in T; Z $in T; Q $in T; R $in T; C $in T; H $in T; S $in T; Sym $in T;
      $forall a,b $in T: a*b $in T; a+b $in T; a->b $in T;
      $forall a $in T: [a] $in T; {a} $in T; ?a $in T; !a $in T;
    }"#;

    let parse_result = BorfParser::parse(Rule::module_declaration, module);
    assert!(
        parse_result.is_ok(),
        "Failed to parse Type system module: {:?}",
        parse_result.err()
    );
}

#[test]
fn test_parse_categorical_module() {
    let module = r#"@Cat: {  -- Category
      O;M;                            -- Objects and morphisms
      dom,cod: M->O;                  -- Domain/codomain maps
      id: O->M;                       -- Identity morphisms
      .: M*M->M | cod(g) $veq dom(f); -- Composition (infix), fixed condition syntax
      $ceq: O*O->Bool;                -- Categorical equivalence
      law.id_type = $forall o $in O: dom(id(o)) $veq o $and cod(id(o)) $veq o;
      law.id_r = $forall f $in M: id(cod(f)).f $seq f;
      composable: M*M*M->Bool = \f,g,h.cod(g) $veq dom(f) $and cod(h) $veq dom(g);
    }"#;

    let parse_result = BorfParser::parse(Rule::module_declaration, module);
    assert!(
        parse_result.is_ok(),
        "Failed to parse Categorical module: {:?}",
        parse_result.err()
    );
}

#[test]
fn test_parse_complex_law() {
    let law = r#"@Test: {
      law.ceq_iso = $forall a,b $in O: a $ceq b $iff
        ($exists f,g $in M: dom(f) $veq a $and cod(f) $veq b $and
         dom(g) $veq b $and cod(g) $veq a $and
         g.f $seq id(a) $and f.g $seq id(b));
    }"#;

    let parse_result = BorfParser::parse(Rule::module_declaration, law);
    assert!(
        parse_result.is_ok(),
        "Failed to parse complex law: {:?}",
        parse_result.err()
    );
}

#[test]
fn test_parse_lambda_expressions() {
    let lambdas = r#"@Lambda: {
      id = \x.x;
      pair = \x,y.(x,y);
      compose = \f,g.\x.f(g(x));
      node_eq: N*N->Bool = \a,b.$lambdaN(a) $seq $lambdaN(b);
    }"#;

    let parse_result = BorfParser::parse(Rule::module_declaration, lambdas);
    assert!(
        parse_result.is_ok(),
        "Failed to parse lambda expressions: {:?}",
        parse_result.err()
    );
}

#[test]
fn test_parse_set_comprehensions() {
    let sets = r#"@Sets: {
      filtered = {x $in S | p(x)};
      complex = {(x,y) | x $in A, y $in B, x $veq y};
      M = {(a,b) | a $in E $and b $in E $and $delta(a,b)};
      typ = {e $in E | $tau(e) $veq TypeSym};
      hom = \a,b.{f $in M | dom(f) $veq a $and cod(f) $veq b};
    }"#;

    let parse_result = BorfParser::parse(Rule::module_declaration, sets);
    assert!(
        parse_result.is_ok(),
        "Failed to parse set comprehensions: {:?}",
        parse_result.err()
    );
}

#[test]
fn test_parse_complex_statements() {
    let complex = r#"@Complex: {
      term = \n.normal(n) ? n : red(step(n));
      let_example = let rec build_hist current_n history =
            if normal(current_n) then Primitives.$append(history, current_n)
            else build_hist (step current_n) (Primitives.$append(history, current_n))
          in build_hist n [];
      complex_fn = \f.g(~(f));
    }"#;

    let parse_result = BorfParser::parse(Rule::module_declaration, complex);
    assert!(
        parse_result.is_ok(),
        "Failed to parse complex statements: {:?}",
        parse_result.err()
    );
}

#[test]
fn test_parse_export() {
    let export = "export Core;";

    let parse_result = BorfParser::parse(Rule::program, export);
    assert!(
        parse_result.is_ok(),
        "Failed to parse export statement: {:?}",
        parse_result.err()
    );
}

// Parse individual elements to identify which parts of the grammar might cause issues
#[test]
fn test_parse_individual_elements() {
    // Define a helper to test a specific rule
    let test_rule = |rule: Rule, input: &str, description: &str| {
        let result = BorfParser::parse(rule, input);
        assert!(
            result.is_ok(),
            "Failed to parse {} with rule {:?}: {:?}",
            description,
            rule,
            result.err()
        );
    };

    // Test basic elements
    test_rule(Rule::ident, "x", "identifier");
    test_rule(Rule::ident, "foo", "identifier");
    test_rule(Rule::dollar_ident, "$x", "dollar identifier");
    test_rule(Rule::int, "42", "integer");
    test_rule(Rule::string_literal, "\"hello\"", "string literal");
    test_rule(Rule::symbol_literal, ":sym", "symbol literal");
    test_rule(Rule::boolean_literal, "true", "boolean literal");
    test_rule(Rule::boolean_literal, "false", "boolean literal");

    // Test type expressions
    test_rule(Rule::type_expr, "T", "simple type");
    test_rule(Rule::type_expr, "T*U", "product type");
    test_rule(Rule::type_expr, "T+U", "sum type");
    test_rule(Rule::type_expr, "T->U", "function type");
    test_rule(Rule::type_expr, "{T}", "set type");
    test_rule(Rule::type_expr, "[T]", "list type");
    test_rule(Rule::type_expr, "!A", "linear type");
    test_rule(Rule::type_expr, "?A", "optional type");

    // Test expressions
    test_rule(Rule::expr_term, "a", "identifier expression");
    test_rule(Rule::expr_term, "42", "integer expression");
    test_rule(Rule::expr_term, "\"hello\"", "string expression");

    // Test expressions with operators
    test_rule(Rule::expression, "a $cup b", "set union expression");
    test_rule(
        Rule::expression,
        "$forall x $in X: p(x)",
        "quantified expression",
    );

    // Special cases that need preprocessing
    let result = BorfParser::parse(Rule::expression, "|S|");
    assert!(
        result.is_ok(),
        "Failed to parse cardinality expression with rule expression: {:?}",
        result.err()
    );

    test_rule(
        Rule::expression,
        "let rec f x = g(x) in f(a)",
        "let-rec expression",
    );
    test_rule(
        Rule::set_comprehension,
        "{x $in S | p(x)}",
        "set comprehension expression",
    );
}

// Helper function to format the error message with context
fn format_pest_error_with_context(input: &str, e: pest::error::Error<Rule>) -> String {
    let (line, col) = match e.line_col {
        pest::error::LineColLocation::Pos(pos) => pos,
        pest::error::LineColLocation::Span(start, _) => start,
    };
    let lines: Vec<&str> = input.lines().collect();
    let problematic_line = if line > 0 && line <= lines.len() {
        lines[line - 1]
    } else {
        "(line out of range)"
    };

    // Add context lines
    let start_context = line.saturating_sub(2);
    let end_context = (line + 2).min(lines.len());

    let mut context_lines = Vec::new();
    for line_num in start_context..end_context {
        context_lines.push(format!("{:>4} | {}", line_num + 1, lines[line_num]));
        if line_num == line - 1 {
            context_lines.push(format!("{}^", " ".repeat(col.saturating_sub(1))));
        }
    }

    let context_str = context_lines.join("\n");
    format!("Error at line {}, column {}:", line, col)
        + "\n"
        + &problematic_line
        + "\n"
        + &context_str
}

#[test]
fn test_parse_law_decl() {
    let law_text = "law.def = $forall x: x $in S;";
    let parse_result = BorfParser::parse(Rule::law_decl, law_text);
    assert!(
        parse_result.is_ok(),
        "Failed to parse law declaration: {}",
        format_pest_error_with_context(law_text, parse_result.err().unwrap())
    );
}

#[test]
fn test_parse_set_comprehension_expr() {
    let set_text = "{x $in S | p(x)}";
    let parse_result = BorfParser::parse(Rule::set_comprehension, set_text);
    assert!(
        parse_result.is_ok(),
        "Failed to parse set comprehension: {}",
        format_pest_error_with_context(set_text, parse_result.err().unwrap())
    );
}

#[test]
fn test_parse_nested_lambda() {
    let lambda_text = "\\x.\\y.f(x, y)";
    let parse_result = BorfParser::parse(Rule::lambda, lambda_text);
    assert!(
        parse_result.is_ok(),
        "Failed to parse nested lambda: {}",
        format_pest_error_with_context(lambda_text, parse_result.err().unwrap())
    );
}

#[test]
fn test_parse_set_comprehension_rule_directly() {
    let set_text = "{x $in S | p(x)}";
    let parse_result = BorfParser::parse(Rule::set_comprehension, set_text);
    assert!(
        parse_result.is_ok(),
        "Failed to parse set comprehension directly: {}",
        format_pest_error_with_context(set_text, parse_result.err().unwrap())
    );
}

#[test]
fn test_parse_set_comprehension_simple() {
    let set_text = "{ x $in S | p(x) }";
    let parse_result = BorfParser::parse(Rule::set_comprehension, set_text);
    println!("{:?}", parse_result);
    assert!(parse_result.is_ok());
    let pairs = parse_result.unwrap();
}

#[test]
fn test_parse_set_comprehension_qualified_domain() {
    let set_text = "{ y $in Module.Set | q(y) }";
    let parse_result = BorfParser::parse(Rule::set_comprehension, set_text);
    println!("{:?}", parse_result);
    assert!(parse_result.is_ok());
    let pairs = parse_result.unwrap();
}
