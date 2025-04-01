use crate::evaluator::{InteractionNet, ReductionRules};
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "borf.pest"]
pub struct InteractionCalculusParser;

pub fn parse_program(input: &str) -> Result<InteractionNet, Box<dyn std::error::Error>> {
    let _parsed = InteractionCalculusParser::parse(Rule::program, input)?
        .next()
        .unwrap();

    let net = InteractionNet::new();
    let _rules = ReductionRules::new();

    // Process the parsed AST to build our interaction net
    // This would involve walking the parse tree and converting
    // each element into our internal representation

    // Implementation details would go here

    Ok(net)
}

// Helper functions to parse various parts of the syntax could go here
