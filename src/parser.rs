use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "borf.pest"]
struct InteractionCalculusParser;

fn parse_program(input: &str) -> Result<InteractionNet, Box<dyn std::error::Error>> {
    let parsed = InteractionCalculusParser::parse(Rule::program, input)?
        .next()
        .unwrap();

    let mut net = InteractionNet::new();
    let mut rules = ReductionRules { rules: Vec::new() };

    // Process the parsed AST to build our interaction net
    // This would involve walking the parse tree and converting
    // each element into our internal representation

    // Implementation details would go here

    Ok(net)
}
