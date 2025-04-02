use borf::error::BorfError;
use borf::parser::{BorfParser, Rule};
use pest::Parser;

fn main() -> Result<(), BorfError> {
    println!("Testing grammar parsing...");

    // Test constraint expressions
    test_parse("a = b", Rule::constraint_expr, "constraint_expr")?;
    test_parse("x > 10", Rule::constraint_expr, "constraint_expr")?;
    test_parse("x $land y", Rule::constraint_expr, "constraint_expr")?;
    test_parse("x => y", Rule::constraint_expr, "constraint_expr")?;

    // Test forall expressions - removed "$forall" prefix as it's part of law_decl
    test_parse("b $in B: b = 1", Rule::forall_expr, "forall_expr")?;
    test_parse("b $in B: b > 0", Rule::forall_expr, "forall_expr")?;
    test_parse("p, q $in P: p = q", Rule::forall_expr, "forall_expr")?;

    // Test law declarations with complete syntax
    test_parse("$forall b $in B: b = 1", Rule::law_decl, "law_decl")?;
    test_parse("w $circ w $equiv id", Rule::law_decl, "law_decl")?;

    println!("All tests passed!");
    Ok(())
}

fn test_parse(input: &str, rule: Rule, rule_name: &str) -> Result<(), BorfError> {
    match BorfParser::parse(rule, input) {
        Ok(pairs) => {
            println!("✅ Successfully parsed as {}: {}", rule_name, input);
            // Print the parse tree
            for pair in pairs {
                print_pair(pair, 0);
            }
            Ok(())
        }
        Err(e) => {
            println!("❌ Failed to parse as {}: {}", rule_name, input);
            println!("   Error: {}", e);
            Err(BorfError::ParserError(format!(
                "Failed to parse {} rule: {}",
                rule_name, e
            )))
        }
    }
}

fn print_pair(pair: pest::iterators::Pair<Rule>, indent: usize) {
    let indent_str = " ".repeat(indent);
    println!(
        "{}Rule: {:?}, Span: {:?}, Text: {}",
        indent_str,
        pair.as_rule(),
        pair.as_span(),
        pair.as_str()
    );

    // Recursively print inner pairs
    for inner_pair in pair.into_inner() {
        print_pair(inner_pair, indent + 2);
    }
}
