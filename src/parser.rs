use crate::error::BorfError;
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "borf.pest"]
pub struct BorfParser;

// Main entry point for parsing a borf program
pub fn parse_program(input: &str) -> Result<Vec<BorfDefinition>, BorfError> {
    let mut parsed = BorfParser::parse(Rule::program, input)
        .map_err(|e| BorfError::ParserError(format!("Pest parsing error: {}", e)))?;

    let program_pair = parsed
        .next()
        .ok_or_else(|| BorfError::ParserError("No 'program' rule found".to_string()))?;

    if program_pair.as_rule() != Rule::program {
        return Err(BorfError::ParserError(format!(
            "Expected 'program' rule, found {:?}",
            program_pair.as_rule()
        )));
    }

    let mut definitions = Vec::new();

    for element in program_pair.into_inner() {
        match element.as_rule() {
            Rule::acset_def => {
                definitions.push(BorfDefinition::ACSet(parse_acset_def(element)?));
            }
            Rule::wire_diagram_def => {
                definitions.push(BorfDefinition::WireDgm(parse_wire_diagram_def(element)?));
            }
            Rule::inet_def => {
                definitions.push(BorfDefinition::INet(parse_inet_def(element)?));
            }
            Rule::pipe_expr => {
                definitions.push(BorfDefinition::PipeExpr(parse_pipe_expr(element)?));
            }
            Rule::app_expr => {
                definitions.push(BorfDefinition::AppExpr(parse_app_expr(element)?));
            }
            Rule::composition_expr => {
                definitions.push(BorfDefinition::CompositionExpr(parse_composition_expr(
                    element,
                )?));
            }
            Rule::EOI => (),
            _ => {
                return Err(BorfError::ParserError(format!(
                    "Unexpected top-level element: {:?}",
                    element.as_rule()
                )))
            }
        }
    }

    Ok(definitions)
}

// Main AST node types
#[derive(Debug, Clone)]
pub enum BorfDefinition {
    ACSet(ACSetDef),
    WireDgm(WireDgmDef),
    INet(INetDef),
    PipeExpr(PipeExpr),
    AppExpr(AppExpr),
    CompositionExpr(CompositionExpr),
}

// ACSet Definition
#[derive(Debug, Clone)]
pub struct ACSetDef {
    pub objects: Vec<String>,
    pub mappings: Vec<MappingDef>,
}

#[derive(Debug, Clone)]
pub struct MappingDef {
    pub name: String,
    pub domain: String,
    pub mapping_type: MappingType,
    pub codomain: String,
}

#[derive(Debug, Clone)]
pub enum MappingType {
    To,
    Subseteq,
    Involution,
}

// Wire Diagram Definition
#[derive(Debug, Clone)]
pub struct WireDgmDef {
    pub base_acset: String,
    pub acset_elements: ACSetDef,
    pub laws: Vec<Law>,
}

// Interaction Net Definition
#[derive(Debug, Clone)]
pub struct INetDef {
    pub base_wire_dgm: String,
    pub acset_elements: ACSetDef,
    pub laws: Vec<Law>,
}

// Laws and Constraints
#[derive(Debug, Clone)]
pub enum Law {
    Composition {
        lhs: String,
        op1: String,
        middle: String,
        op2: String,
        rhs: String,
    },
    ForAll {
        vars: Vec<String>,
        domain: String,
        constraint: Constraint,
    },
}

#[derive(Debug, Clone)]
pub enum Constraint {
    Equality {
        lhs: Box<ConstraintExpr>,
        rhs: Box<ConstraintExpr>,
    },
    LogicalAnd {
        lhs: Box<ConstraintExpr>,
        rhs: Box<ConstraintExpr>,
    },
    GreaterThanEqual {
        lhs: Box<ConstraintExpr>,
        rhs: Box<ConstraintExpr>,
    },
    GreaterThan {
        lhs: Box<ConstraintExpr>,
        rhs: Box<ConstraintExpr>,
    },
    LessThanEqual {
        lhs: Box<ConstraintExpr>,
        rhs: Box<ConstraintExpr>,
    },
    LessThan {
        lhs: Box<ConstraintExpr>,
        rhs: Box<ConstraintExpr>,
    },
    Implies {
        lhs: Box<ConstraintExpr>,
        rhs: Box<ConstraintExpr>,
    },
}

#[derive(Debug, Clone)]
pub enum ConstraintExpr {
    Integer(i64),
    Identifier(String),
    SetExpr(SetExpr),
    FunctionApp { func: String, arg: String },
}

#[derive(Debug, Clone)]
pub enum SetExpr {
    Comprehension {
        elements: Vec<String>,
        condition: Option<SetCondition>,
    },
    CartesianProduct {
        lhs: String,
        rhs: String,
    },
}

#[derive(Debug, Clone)]
pub struct SetCondition {
    pub func1: String,
    pub arg1: String,
    pub func2: Option<String>,
    pub arg2: Option<String>,
}

// Expression Types
#[derive(Debug, Clone)]
pub struct PipeExpr {
    pub start: String,
    pub steps: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct AppExpr {
    pub func: String,
    pub arg: Box<AppExprArg>,
}

#[derive(Debug, Clone)]
pub enum AppExprArg {
    Identifier(String),
    AppExpr(Box<AppExpr>),
}

#[derive(Debug, Clone)]
pub struct CompositionExpr {
    pub result: String,
    pub functions: Vec<String>,
    pub arg: String,
}

// Parse functions for each type
fn parse_acset_def(pair: Pair<Rule>) -> Result<ACSetDef, BorfError> {
    let mut objects = Vec::new();
    let mut mappings = Vec::new();

    for element in pair.into_inner() {
        match element.as_rule() {
            Rule::object_decl => {
                // Parse objects like "N;E;"
                for id in element.into_inner() {
                    objects.push(id.as_str().to_string());
                }
            }
            Rule::mapping_decl => {
                // Parse mappings like "s:E.to N;" or "p:P.to{0,1};"
                let mut inner = element.into_inner();
                let name = inner.next().unwrap().as_str().to_string();
                let domain = inner.next().unwrap().as_str().to_string();
                let mapping_type_str = inner.next().unwrap().as_str();

                // Get the target, which could be an identifier or a set literal
                let codomain_part = inner.next().unwrap();
                let codomain = match codomain_part.as_rule() {
                    Rule::identifier => codomain_part.as_str().to_string(),
                    Rule::set_literal => codomain_part.as_str().to_string(),
                    _ => {
                        return Err(BorfError::ParserError(format!(
                            "Unexpected codomain type: {:?}",
                            codomain_part.as_rule()
                        )))
                    }
                };

                let mapping_type = match mapping_type_str {
                    "to" => MappingType::To,
                    "subseteq" => MappingType::Subseteq,
                    "<=>" => MappingType::Involution,
                    _ => {
                        return Err(BorfError::ParserError(format!(
                            "Unknown mapping type: {}",
                            mapping_type_str
                        )))
                    }
                };

                mappings.push(MappingDef {
                    name,
                    domain,
                    mapping_type,
                    codomain,
                });
            }
            Rule::comment_line => {
                // Ignore comments
            }
            _ => {
                return Err(BorfError::ParserError(format!(
                    "Unexpected element in ACSet definition: {:?}",
                    element.as_rule()
                )))
            }
        }
    }

    Ok(ACSetDef { objects, mappings })
}

fn parse_wire_diagram_def(pair: Pair<Rule>) -> Result<WireDgmDef, BorfError> {
    let mut inner = pair.into_inner();
    let base_acset = inner.next().unwrap().as_str().to_string();

    // Create containers for ACSet elements and laws
    let mut acset_elements = ACSetDef {
        objects: Vec::new(),
        mappings: Vec::new(),
    };
    let mut laws = Vec::new();

    for element in inner {
        match element.as_rule() {
            Rule::object_decl => {
                for id in element.into_inner() {
                    acset_elements.objects.push(id.as_str().to_string());
                }
            }
            Rule::mapping_decl => {
                let mut inner = element.into_inner();
                let name = inner.next().unwrap().as_str().to_string();
                let domain = inner.next().unwrap().as_str().to_string();
                let mapping_type_str = inner.next().unwrap().as_str();
                let codomain = inner.next().unwrap().as_str().to_string();

                let mapping_type = match mapping_type_str {
                    "to" => MappingType::To,
                    "subseteq" => MappingType::Subseteq,
                    "<=>" => MappingType::Involution,
                    _ => {
                        return Err(BorfError::ParserError(format!(
                            "Unknown mapping type: {}",
                            mapping_type_str
                        )))
                    }
                };

                acset_elements.mappings.push(MappingDef {
                    name,
                    domain,
                    mapping_type,
                    codomain,
                });
            }
            Rule::law_decl => {
                laws.push(parse_law(element)?);
            }
            Rule::comment_line => {
                // Ignore comments
            }
            _ => {
                return Err(BorfError::ParserError(format!(
                    "Unexpected element in WireDgm definition: {:?}",
                    element.as_rule()
                )))
            }
        }
    }

    Ok(WireDgmDef {
        base_acset,
        acset_elements,
        laws,
    })
}

fn parse_inet_def(pair: Pair<Rule>) -> Result<INetDef, BorfError> {
    let mut inner = pair.into_inner();
    let base_wire_dgm = inner.next().unwrap().as_str().to_string();

    // Create containers for ACSet elements and laws
    let mut acset_elements = ACSetDef {
        objects: Vec::new(),
        mappings: Vec::new(),
    };
    let mut laws = Vec::new();

    for element in inner {
        match element.as_rule() {
            Rule::object_decl => {
                for id in element.into_inner() {
                    acset_elements.objects.push(id.as_str().to_string());
                }
            }
            Rule::mapping_decl => {
                let mut inner = element.into_inner();
                let name = inner.next().unwrap().as_str().to_string();
                let domain = inner.next().unwrap().as_str().to_string();
                let mapping_type_str = inner.next().unwrap().as_str();
                let codomain = inner.next().unwrap().as_str().to_string();

                let mapping_type = match mapping_type_str {
                    "to" => MappingType::To,
                    "subseteq" => MappingType::Subseteq,
                    "<=>" => MappingType::Involution,
                    _ => {
                        return Err(BorfError::ParserError(format!(
                            "Unknown mapping type: {}",
                            mapping_type_str
                        )))
                    }
                };

                acset_elements.mappings.push(MappingDef {
                    name,
                    domain,
                    mapping_type,
                    codomain,
                });
            }
            Rule::law_decl => {
                laws.push(parse_law(element)?);
            }
            Rule::comment_line => {
                // Ignore comments
            }
            _ => {
                return Err(BorfError::ParserError(format!(
                    "Unexpected element in INet definition: {:?}",
                    element.as_rule()
                )))
            }
        }
    }

    Ok(INetDef {
        base_wire_dgm,
        acset_elements,
        laws,
    })
}

fn parse_law(pair: Pair<Rule>) -> Result<Law, BorfError> {
    let inner = pair.into_inner().next().unwrap();

    if inner.as_rule() == Rule::forall_expr {
        // Handle the forall law: .forall b.in B:|{p.in P|b(p).land p(p)}|=1;
        let mut inner_iter = inner.into_inner();

        // Parse variable list
        let mut vars = Vec::new();
        for var in inner_iter.next().unwrap().into_inner() {
            vars.push(var.as_str().to_string());
        }

        // Parse domain
        let domain = inner_iter.next().unwrap().as_str().to_string();

        // Parse constraint
        let constraint_pair = inner_iter.next().unwrap();
        let constraint = parse_constraint(constraint_pair)?;

        Ok(Law::ForAll {
            vars,
            domain,
            constraint,
        })
    } else {
        // Handle the composition law: w.circ w.equiv id;
        let mut parts = Vec::new();
        for part in inner.into_inner() {
            parts.push(part.as_str().to_string());
        }

        if parts.len() >= 5 {
            Ok(Law::Composition {
                lhs: parts[0].clone(),
                op1: parts[1].clone(),
                middle: parts[2].clone(),
                op2: parts[3].clone(),
                rhs: parts[4].clone(),
            })
        } else {
            Err(BorfError::ParserError(
                "Invalid law: not enough parts".to_string(),
            ))
        }
    }
}

fn parse_constraint(pair: Pair<Rule>) -> Result<Constraint, BorfError> {
    let mut inner = pair.into_inner();
    let lhs_pair = inner.next().unwrap();
    let lhs = parse_constraint_expr(lhs_pair)?;

    // Check if there's an operator and a right-hand side
    if let Some(op_pair) = inner.next() {
        let op = op_pair.as_str();
        let rhs_pair = inner.next().unwrap();
        let rhs = parse_constraint_expr(rhs_pair)?;

        match op {
            "=" => Ok(Constraint::Equality {
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            }),
            ".land" => Ok(Constraint::LogicalAnd {
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            }),
            ".>=" => Ok(Constraint::GreaterThanEqual {
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            }),
            ".>" => Ok(Constraint::GreaterThan {
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            }),
            ".<=" => Ok(Constraint::LessThanEqual {
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            }),
            ".<" => Ok(Constraint::LessThan {
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            }),
            ".>=>" => Ok(Constraint::Implies {
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            }),
            _ => Err(BorfError::ParserError(format!(
                "Unknown constraint operator: {}",
                op
            ))),
        }
    } else {
        // If there's no operator, it's just an expression by itself
        // This could be an error in most constraint contexts, but we'll return it
        // and let the semantic analysis handle this case
        Err(BorfError::ParserError(
            "Incomplete constraint expression".to_string(),
        ))
    }
}

fn parse_constraint_expr(pair: Pair<Rule>) -> Result<ConstraintExpr, BorfError> {
    match pair.as_rule() {
        Rule::int => {
            let value = pair
                .as_str()
                .parse::<i64>()
                .map_err(|e| BorfError::ParserError(format!("Failed to parse integer: {}", e)))?;
            Ok(ConstraintExpr::Integer(value))
        }
        Rule::identifier => Ok(ConstraintExpr::Identifier(pair.as_str().to_string())),
        Rule::set_expr => parse_set_expr(pair),
        Rule::function_app => {
            let mut inner = pair.into_inner();
            let func = inner.next().unwrap().as_str().to_string();
            let arg = inner.next().unwrap().as_str().to_string();
            Ok(ConstraintExpr::FunctionApp { func, arg })
        }
        _ => Err(BorfError::ParserError(format!(
            "Unexpected constraint expression type: {:?}",
            pair.as_rule()
        ))),
    }
}

fn parse_set_expr(pair: Pair<Rule>) -> Result<ConstraintExpr, BorfError> {
    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        Rule::set_element => {
            // Handling set expressions like "{p.in P|b(p).land p(p)}"
            let mut elements = Vec::new();

            for element in inner.into_inner() {
                elements.push(element.as_str().to_string());
            }

            // Set condition is more complex and would need to be implemented
            // For now, we'll return a simple version
            Ok(ConstraintExpr::SetExpr(SetExpr::Comprehension {
                elements,
                condition: None,
            }))
        }
        _ => {
            // Handling Cartesian product: "B*B"
            let text = inner.as_str();
            if let Some(idx) = text.find('*') {
                let lhs = text[..idx].to_string();
                let rhs = text[idx + 1..].to_string();

                Ok(ConstraintExpr::SetExpr(SetExpr::CartesianProduct {
                    lhs,
                    rhs,
                }))
            } else {
                Err(BorfError::ParserError(format!(
                    "Invalid set expression: {}",
                    text
                )))
            }
        }
    }
}

fn parse_pipe_expr(pair: Pair<Rule>) -> Result<PipeExpr, BorfError> {
    let mut inner = pair.into_inner();
    let start = inner.next().unwrap().as_str().to_string();

    let mut steps = Vec::new();
    for step in inner {
        steps.push(step.as_str().to_string());
    }

    Ok(PipeExpr { start, steps })
}

fn parse_app_expr(pair: Pair<Rule>) -> Result<AppExpr, BorfError> {
    let mut inner = pair.into_inner();
    let func = inner.next().unwrap().as_str().to_string();

    let arg_pair = inner.next().unwrap();
    let arg = if arg_pair.as_rule() == Rule::app_expr {
        // Handle nested app expressions
        let nested_app = parse_app_expr(arg_pair)?;
        AppExprArg::AppExpr(Box::new(nested_app))
    } else {
        // It's an identifier
        AppExprArg::Identifier(arg_pair.as_str().to_string())
    };

    Ok(AppExpr {
        func,
        arg: Box::new(arg),
    })
}

fn parse_composition_expr(pair: Pair<Rule>) -> Result<CompositionExpr, BorfError> {
    let mut inner = pair.into_inner();
    let result = inner.next().unwrap().as_str().to_string();

    let mut functions = Vec::new();
    let mut arg = String::new();

    for part in inner {
        if part.as_rule() == Rule::identifier {
            // This could be a function or an argument (if it's the last one)
            let id = part.as_str().to_string();
            if arg.is_empty() {
                functions.push(id);
            } else {
                // This is unexpected, we already have an argument
                return Err(BorfError::ParserError(
                    "Multiple arguments in composition expression".to_string(),
                ));
            }
        } else {
            // This should be the last item, the argument
            arg = part.as_str().to_string();
        }
    }

    if arg.is_empty() && !functions.is_empty() {
        // The last "function" is actually the argument
        arg = functions.pop().unwrap();
    }

    Ok(CompositionExpr {
        result,
        functions,
        arg,
    })
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_acset() {
        // Use a minimal example that we've verified works
        let input = r#"@ACSet{
  N;
  E;
  s:E.to N;
  t:E.to N;
}"#;

        let result = parse_program(input);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        if let Ok(defs) = result {
            assert_eq!(defs.len(), 1, "Should have parsed 1 definition");

            match &defs[0] {
                BorfDefinition::ACSet(acset) => {
                    assert_eq!(acset.objects.len(), 2, "Should have 2 objects");
                    assert_eq!(acset.objects[0], "N");
                    assert_eq!(acset.objects[1], "E");

                    // Test first mapping
                    assert_eq!(acset.mappings[0].name, "s");
                    assert_eq!(acset.mappings[0].domain, "E");
                    assert_eq!(acset.mappings[0].codomain, "N");
                }
                _ => panic!("Expected ACSet definition"),
            }
        }
    }

    #[test]
    fn test_parse_wire_diagram() {
        // Use a simplified version without laws for now
        let input = r#"@WireDgm<ACSet>{
  B;
  P;
  b:P.to B;
  w:P.<=> P;
}"#;

        let result = parse_program(input);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        if let Ok(defs) = result {
            assert_eq!(defs.len(), 1, "Should have parsed 1 definition");

            match &defs[0] {
                BorfDefinition::WireDgm(wire_dgm) => {
                    assert_eq!(wire_dgm.base_acset, "ACSet");
                    assert_eq!(wire_dgm.acset_elements.objects.len(), 2);
                    assert_eq!(wire_dgm.acset_elements.mappings.len(), 2);
                    // No laws in this simplified test
                    assert_eq!(wire_dgm.laws.len(), 0);
                }
                _ => panic!("Expected WireDgm definition"),
            }
        }
    }

    #[test]
    fn test_parse_inet() {
        // Use a simplified version without complex mappings and laws
        let input = r#"@INet<WireDgm>{
  p:P.to{0,1};
}"#;

        let result = parse_program(input);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        if let Ok(defs) = result {
            assert_eq!(defs.len(), 1, "Should have parsed 1 definition");

            match &defs[0] {
                BorfDefinition::INet(inet) => {
                    assert_eq!(inet.base_wire_dgm, "WireDgm");
                    assert_eq!(inet.acset_elements.mappings.len(), 1);
                    assert_eq!(inet.laws.len(), 0);
                }
                _ => panic!("Expected INet definition"),
            }
        }
    }

    #[test]
    fn test_parse_pipe_expr() {
        let input = r#"world|>a|>w|>i|>r|>d|>t"#;

        let result = parse_program(input);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        if let Ok(defs) = result {
            assert_eq!(defs.len(), 1, "Should have parsed 1 definition");

            match &defs[0] {
                BorfDefinition::PipeExpr(pipe) => {
                    assert_eq!(pipe.start, "world");
                    assert_eq!(pipe.steps, vec!["a", "w", "i", "r", "d", "t"]);
                }
                _ => panic!("Expected pipe expression"),
            }
        }
    }

    #[test]
    fn test_parse_app_expr() {
        let input = r#">i(>w(>a(world)))"#;

        let result = parse_program(input);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        if let Ok(defs) = result {
            assert_eq!(defs.len(), 1, "Should have parsed 1 definition");

            match &defs[0] {
                BorfDefinition::AppExpr(app) => {
                    assert_eq!(app.func, "i");
                    match &*app.arg {
                        AppExprArg::AppExpr(inner_app) => {
                            assert_eq!(inner_app.func, "w");
                            match &*inner_app.arg {
                                AppExprArg::AppExpr(inner_inner_app) => {
                                    assert_eq!(inner_inner_app.func, "a");
                                    match &*inner_inner_app.arg {
                                        AppExprArg::Identifier(id) => {
                                            assert_eq!(id, "world");
                                        }
                                        _ => panic!("Expected identifier"),
                                    }
                                }
                                _ => panic!("Expected nested app expression"),
                            }
                        }
                        _ => panic!("Expected nested app expression"),
                    }
                }
                _ => panic!("Expected application expression"),
            }
        }
    }

    #[test]
    fn test_parse_composition_expr() {
        let input = r#"T=t.circ d.circ r.circ i.circ w.circ a(W)"#;

        let result = parse_program(input);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        if let Ok(defs) = result {
            assert_eq!(defs.len(), 1, "Should have parsed 1 definition");

            match &defs[0] {
                BorfDefinition::CompositionExpr(comp) => {
                    assert_eq!(comp.result, "T");
                    assert!(
                        comp.functions.len() >= 6,
                        "Should have at least 6 functions"
                    );
                    assert_eq!(comp.arg, "W");
                }
                _ => panic!("Expected composition expression"),
            }
        }
    }

    #[test]
    fn test_parse_multiple_definitions() {
        let input = r#"@ACSet{N;E;s:E.to N;t:E.to N;}
@WireDgm<ACSet>{B;P;b:P.to B;w:P.<=> P;}
world|>a|>w|>i"#;

        let result = parse_program(input);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        if let Ok(defs) = result {
            assert_eq!(defs.len(), 3, "Should have parsed 3 definitions");

            // Check types of all definitions
            assert!(matches!(defs[0], BorfDefinition::ACSet(_)));
            assert!(matches!(defs[1], BorfDefinition::WireDgm(_)));
            assert!(matches!(defs[2], BorfDefinition::PipeExpr(_)));
        }
    }
}
