use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::parser::{ast::*, parse_program, BorfParser, Rule};
use pest::Parser;

// Helper function to count modules in the parsed prelude
fn count_modules(items: &[TopLevelItem]) -> usize {
    items
        .iter()
        .filter(|item| matches!(item, TopLevelItem::Module(_)))
        .count()
}

// Helper function to count declarations in a module
fn count_declarations(module: &ModuleDef) -> usize {
    module
        .elements
        .iter()
        .filter_map(|elem| match elem {
            ModuleElement::Declaration(_) => Some(()),
        })
        .count()
}

// Helper to visualize the structure of a module
fn visualize_module(module: &ModuleDef) -> String {
    let mut result = format!("@{}", module.name);

    if let Some(param) = &module.type_param {
        result.push_str(&format!("<{}>", param));
    }

    result.push_str(": {\n");

    // Count declaration types
    let mut object_count = 0;
    let mut mapping_count = 0;
    let mut law_count = 0;

    for elem in &module.elements {
        match elem {
            ModuleElement::Declaration(decl) => match decl {
                Declaration::ObjectDecl { .. } => object_count += 1,
                Declaration::MappingDecl { .. } => mapping_count += 1,
                Declaration::LawDecl { .. } => law_count += 1,
            },
        }
    }

    result.push_str(&format!("  Objects: {}\n", object_count));
    result.push_str(&format!("  Mappings: {}\n", mapping_count));
    result.push_str(&format!("  Laws: {}\n", law_count));
    result.push_str("}\n");

    result
}

// Extract module names from parsed items
fn get_module_names(items: &[TopLevelItem]) -> Vec<String> {
    items
        .iter()
        .filter_map(|item| match item {
            TopLevelItem::Module(module) => Some(module.name.clone()),
            _ => None,
        })
        .collect()
}

// Check if Core module exports required components
fn validate_core_exports(items: &[TopLevelItem]) -> bool {
    if let Some(TopLevelItem::Module(core_module)) = items
        .iter()
        .find(|item| matches!(item, TopLevelItem::Module(m) if m.name == "Core"))
    {
        let mut found_typ = false;
        let mut found_op = false;
        let mut found_fn = false;

        for elem in &core_module.elements {
            if let ModuleElement::Declaration(Declaration::ObjectDecl { name, .. }) = elem {
                if name == "typ" {
                    found_typ = true;
                } else if name == "op" {
                    found_op = true;
                } else if name == "fn" {
                    found_fn = true;
                }
            }
        }

        found_typ && found_op && found_fn
    } else {
        false
    }
}

// Find all law declarations across modules
fn find_all_laws(items: &[TopLevelItem]) -> HashMap<String, Vec<String>> {
    let mut laws_by_module = HashMap::new();

    for item in items {
        if let TopLevelItem::Module(module) = item {
            let mut module_laws = Vec::new();

            for elem in &module.elements {
                if let ModuleElement::Declaration(Declaration::LawDecl { name, .. }) = elem {
                    module_laws.push(name.clone());
                }
            }

            if !module_laws.is_empty() {
                laws_by_module.insert(module.name.clone(), module_laws);
            }
        }
    }

    laws_by_module
}

// Check for specific elements in the prelude
fn verify_prelude_structure(items: &[TopLevelItem]) -> Vec<String> {
    let mut missing = Vec::new();

    // Check for required modules
    let expected_modules = [
        "Primitives",
        "T",
        "R",
        "Cat",
        "Mod",
        "Grph",
        "Wire",
        "Net",
        "IO",
        "Rewrite",
        "Red",
        "Term",
        "Xform",
        "Core",
    ];

    let module_names = get_module_names(items);

    for &expected in &expected_modules {
        if !module_names.contains(&expected.to_string()) {
            missing.push(format!("Missing module: {}", expected));
        }
    }

    // Check for export statement
    let has_export = items
        .iter()
        .any(|item| matches!(item, TopLevelItem::Export(name) if name == "Core"));

    if !has_export {
        missing.push("Missing 'export Core;' statement".to_string());
    }

    missing
}

// Comprehensive test for parsing the prelude
#[test]
fn test_parse_prelude_complete() {
    // Get the prelude file path
    let prelude_path = Path::new("src/prelude/mod.borf");

    // Read the prelude file
    let prelude_content =
        fs::read_to_string(prelude_path).expect("Should be able to read the prelude file");

    // Parse the prelude using our AST parser
    let parse_result = parse_program(&prelude_content);

    // Verify parsing succeeded
    assert!(
        parse_result.is_ok(),
        "Failed to parse prelude with AST parser"
    );

    let items = parse_result.unwrap();

    // Verify basic structure
    println!(
        "Prelude parsed successfully with {} top-level items",
        items.len()
    );

    // Count module declarations
    let module_count = count_modules(&items);
    println!("Found {} module declarations", module_count);

    // Print module names
    let module_names = get_module_names(&items);
    println!("Module names: {:?}", module_names);

    // Verify all expected modules are present
    let missing_elements = verify_prelude_structure(&items);
    assert!(
        missing_elements.is_empty(),
        "Missing elements in prelude: {:?}",
        missing_elements
    );

    // Verify the Core module has the expected exports
    assert!(
        validate_core_exports(&items),
        "Core module missing expected exports"
    );

    // Find all law declarations
    let laws = find_all_laws(&items);
    println!("Found laws in {} modules", laws.len());
    for (module, laws) in &laws {
        println!("  Module {} has {} laws", module, laws.len());
    }

    // --- Insta Snapshot Testing ---
    // Find specific modules for snapshot testing
    let primitives_module = items.iter().find_map(|item| {
        if let TopLevelItem::Module(m) = item {
            if m.name == "Primitives" {
                return Some(m);
            }
        }
        None
    });
    let t_module = items.iter().find_map(|item| {
        if let TopLevelItem::Module(m) = item {
            if m.name == "T" {
                return Some(m);
            }
        }
        None
    });

    // Snapshot the 'Primitives' module if found
    if let Some(module) = primitives_module {
        insta::assert_debug_snapshot!("prelude_primitives_module", module);
    } else {
        panic!("Primitives module not found for snapshotting");
    }

    // Snapshot the 'T' module if found
    if let Some(module) = t_module {
        insta::assert_debug_snapshot!("prelude_t_module", module);
    } else {
        panic!("T module not found for snapshotting");
    }
    // --- End Insta Snapshot Testing ---

    // Visualize the structure of each module
    println!("\nModule Structure Summary:");
    for item in &items {
        if let TopLevelItem::Module(module) = item {
            println!("{}", visualize_module(module));
        }
    }

    // Check the export statement
    let export_statements: Vec<_> = items
        .iter()
        .filter_map(|item| match item {
            TopLevelItem::Export(name) => Some(name),
            _ => None,
        })
        .collect();

    assert_eq!(
        export_statements.len(),
        1,
        "Expected exactly one export statement, found {}",
        export_statements.len()
    );

    assert_eq!(
        export_statements[0], "Core",
        "Expected export of 'Core', found export of '{}'",
        export_statements[0]
    );
}

// Test that validates individual module structures
#[test]
fn test_prelude_module_structures() {
    // Get the prelude file
    let prelude_path = Path::new("src/prelude/mod.borf");
    let prelude_content =
        fs::read_to_string(prelude_path).expect("Should be able to read the prelude file");

    // Parse the prelude
    let parse_result = parse_program(&prelude_content);
    assert!(
        parse_result.is_ok(),
        "Failed to parse prelude with AST parser"
    );
    let items = parse_result.unwrap();

    // Verify structure of each module
    let modules: Vec<_> = items
        .iter()
        .filter_map(|item| match item {
            TopLevelItem::Module(module) => Some(module),
            _ => None,
        })
        .collect();

    // Module-specific assertions

    // 1. Primitives module
    let primitives = modules
        .iter()
        .find(|m| m.name == "Primitives")
        .expect("Primitives module not found");
    assert!(
        primitives.type_param.is_none(),
        "Primitives should not have a type parameter"
    );

    // Count mapping declarations in Primitives
    let primitives_mappings = primitives
        .elements
        .iter()
        .filter(|e| {
            matches!(
                e,
                ModuleElement::Declaration(Declaration::MappingDecl { .. })
            )
        })
        .count();
    assert!(
        primitives_mappings >= 5,
        "Expected at least 5 mapping declarations in Primitives, found {}",
        primitives_mappings
    );

    // 2. T module (Type system)
    let t_module = modules
        .iter()
        .find(|m| m.name == "T")
        .expect("T module not found");

    // Check for important type declarations
    let has_required_types = t_module.elements.iter().any(|e| {
        if let ModuleElement::Declaration(Declaration::ObjectDecl { name, .. }) = e {
            name == "U" || name == "B" || name == "N" || name == "Z" || name == "S" || name == "Sym"
        } else {
            false
        }
    });
    assert!(
        has_required_types,
        "T module is missing required basic type declarations"
    );

    // 3. Core module
    let core = modules
        .iter()
        .find(|m| m.name == "Core")
        .expect("Core module not found");

    // Check that Core has enough declarations to be complete
    let core_decl_count = count_declarations(core);
    assert!(
        core_decl_count >= 5,
        "Core module should have at least 5 declarations, found {}",
        core_decl_count
    );

    // Check for dependencies definition in Core
    let has_deps = core.elements.iter().any(|e| {
        if let ModuleElement::Declaration(Declaration::ObjectDecl { name, .. }) = e {
            name == "deps"
        } else {
            false
        }
    });
    assert!(has_deps, "Core module is missing deps declaration");
}

// Test to extract and validate parse tree structure
#[test]
fn test_prelude_parse_tree_structure() {
    // Parse the prelude directly with pest to examine the parse tree
    let prelude_path = Path::new("src/prelude/mod.borf");
    let prelude_content =
        fs::read_to_string(prelude_path).expect("Should be able to read the prelude file");

    let parse_result = BorfParser::parse(Rule::program, &prelude_content);
    assert!(
        parse_result.is_ok(),
        "Failed to parse prelude with pest grammar"
    );

    let pairs = parse_result.unwrap();

    // Count the total number of nodes in the parse tree
    fn count_nodes(pairs: pest::iterators::Pairs<Rule>) -> usize {
        let mut count = 0;
        for pair in pairs {
            count += 1;
            count += count_nodes(pair.into_inner());
        }
        count
    }

    let node_count = count_nodes(pairs.clone());
    println!("Parse tree contains {} nodes", node_count);

    // Depth of the parse tree
    fn tree_depth(pairs: pest::iterators::Pairs<Rule>) -> usize {
        let mut max_depth = 0;
        for pair in pairs {
            let inner_depth = tree_depth(pair.into_inner());
            max_depth = max_depth.max(inner_depth + 1);
        }
        max_depth
    }

    let depth = tree_depth(pairs.clone());
    println!("Parse tree has a depth of {}", depth);

    // The parse tree should be substantial, indicating complete parsing
    assert!(
        node_count > 1000,
        "Parse tree should have more than 1000 nodes, found {}",
        node_count
    );
    assert!(
        depth > 5,
        "Parse tree should have depth greater than 5, found {}",
        depth
    );

    // Ensure the program rule is at the top level
    let program_rule_count = pairs.filter(|p| p.as_rule() == Rule::program).count();
    assert_eq!(
        program_rule_count, 1,
        "Expected exactly one program rule at the top level"
    );
}

// Simple test just for the insta snapshots
#[test]
fn test_insta_snapshots() {
    // Create a simple ModuleDef for testing snapshots
    let primitives_module = ModuleDef {
        name: "TestPrimitives".to_string(),
        type_param: None,
        elements: vec![
            ModuleElement::Declaration(Declaration::ObjectDecl {
                name: "test_obj".to_string(),
                type_constraint: None,
            }),
            ModuleElement::Declaration(Declaration::MappingDecl {
                name: "test_map".to_string(),
                type_constraint: None,
                value: None,
                constraint: None,
            }),
        ],
    };

    // Take a snapshot of the test module
    insta::assert_debug_snapshot!("test_primitives_module", primitives_module);
}

// Test to capture snapshots of all top-level syntactic forms
#[test]
fn test_prelude_syntax_snapshots() {
    // Get the prelude file path
    let prelude_path = Path::new("src/prelude/mod.borf");

    // Read the prelude file
    let prelude_content =
        fs::read_to_string(prelude_path).expect("Should be able to read the prelude file");

    // Parse the prelude using our AST parser
    let parse_result = parse_program(&prelude_content);

    // Verify parsing succeeded
    assert!(
        parse_result.is_ok(),
        "Failed to parse prelude with AST parser"
    );

    let items = parse_result.unwrap();

    // 1. Snapshot the complete list of top-level items
    insta::assert_debug_snapshot!("prelude_all_top_level_items", &items);

    // 2. Snapshot examples of each top-level item type

    // Find a module declaration
    if let Some(module) = items.iter().find_map(|item| match item {
        TopLevelItem::Module(m) => Some(m),
        _ => None,
    }) {
        insta::assert_debug_snapshot!("prelude_module_declaration", module);
    }

    // Find an export statement
    if let Some(export_name) = items.iter().find_map(|item| match item {
        TopLevelItem::Export(name) => Some(name),
        _ => None,
    }) {
        insta::assert_debug_snapshot!("prelude_export_statement", export_name);
    }

    // Find an import statement (if any)
    if let Some(import_path) = items.iter().find_map(|item| match item {
        TopLevelItem::Import(path) => Some(path),
        _ => None,
    }) {
        insta::assert_debug_snapshot!("prelude_import_statement", import_path);
    }

    // 3. Snapshot examples of different declaration types within modules

    // Object declaration
    let object_decl = items.iter().find_map(|item| {
        if let TopLevelItem::Module(m) = item {
            m.elements.iter().find_map(|elem| {
                if let ModuleElement::Declaration(Declaration::ObjectDecl { .. }) = elem {
                    Some(elem)
                } else {
                    None
                }
            })
        } else {
            None
        }
    });

    if let Some(decl) = object_decl {
        insta::assert_debug_snapshot!("prelude_object_declaration", decl);
    }

    // Mapping declaration
    let mapping_decl = items.iter().find_map(|item| {
        if let TopLevelItem::Module(m) = item {
            m.elements.iter().find_map(|elem| {
                if let ModuleElement::Declaration(Declaration::MappingDecl { .. }) = elem {
                    Some(elem)
                } else {
                    None
                }
            })
        } else {
            None
        }
    });

    if let Some(decl) = mapping_decl {
        insta::assert_debug_snapshot!("prelude_mapping_declaration", decl);
    }

    // Law declaration
    let law_decl = items.iter().find_map(|item| {
        if let TopLevelItem::Module(m) = item {
            m.elements.iter().find_map(|elem| {
                if let ModuleElement::Declaration(Declaration::LawDecl { .. }) = elem {
                    Some(elem)
                } else {
                    None
                }
            })
        } else {
            None
        }
    });

    if let Some(decl) = law_decl {
        insta::assert_debug_snapshot!("prelude_law_declaration", decl);
    }

    // 4. Snapshot examples of different syntactic forms within declarations

    // Type expressions
    let type_expr = items.iter().find_map(|item| {
        if let TopLevelItem::Module(m) = item {
            m.elements.iter().find_map(|elem| {
                if let ModuleElement::Declaration(Declaration::MappingDecl {
                    type_constraint: Some(type_expr),
                    ..
                }) = elem
                {
                    Some(type_expr)
                } else {
                    None
                }
            })
        } else {
            None
        }
    });

    if let Some(expr) = type_expr {
        insta::assert_debug_snapshot!("prelude_type_expression", expr);
    }

    // Value expressions
    let value_expr = items.iter().find_map(|item| {
        if let TopLevelItem::Module(m) = item {
            m.elements.iter().find_map(|elem| {
                if let ModuleElement::Declaration(Declaration::MappingDecl {
                    value: Some(expr),
                    ..
                }) = elem
                {
                    Some(expr)
                } else {
                    None
                }
            })
        } else {
            None
        }
    });

    if let Some(expr) = value_expr {
        insta::assert_debug_snapshot!("prelude_value_expression", expr);
    }
}

// Test to capture snapshots of the raw Pest parse tree structure
#[test]
fn test_prelude_pest_parse_tree_snapshots() {
    // Get the prelude file path
    let prelude_path = Path::new("src/prelude/mod.borf");

    // Read the prelude file
    let prelude_content =
        fs::read_to_string(prelude_path).expect("Should be able to read the prelude file");

    // Parse with the Pest parser directly
    let parse_result = BorfParser::parse(Rule::program, &prelude_content);
    assert!(
        parse_result.is_ok(),
        "Failed to parse prelude with Pest grammar"
    );

    let pairs = parse_result.unwrap();

    // Helper function to convert a Pest Pair into a more snapshot-friendly format
    fn simplify_pair(pair: pest::iterators::Pair<Rule>) -> SimplifiedPair {
        SimplifiedPair {
            rule: format!("{:?}", pair.as_rule()),
            text: if pair.as_str().len() <= 100 {
                pair.as_str().to_string()
            } else {
                format!(
                    "{}... (truncated, {} chars total)",
                    &pair.as_str()[..97],
                    pair.as_str().len()
                )
            },
            span: format!("{:?}", pair.as_span()),
            children: pair.into_inner().map(simplify_pair).collect(),
        }
    }

    // Simplified structure for snapshotting Pest pairs
    #[derive(Debug)]
    struct SimplifiedPair {
        rule: String,
        text: String,
        span: String,
        children: Vec<SimplifiedPair>,
    }

    // Find the first statement pair as a representation
    let first_program_pair = pairs.clone().next().unwrap();

    // Take a snapshot of the program structure
    insta::assert_debug_snapshot!(
        "prelude_pest_program_structure",
        simplify_pair(first_program_pair.clone())
    );

    // Get the first few statements to snapshot
    let statements = first_program_pair
        .into_inner()
        .filter(|p| p.as_rule() == Rule::statement)
        .take(3)
        .collect::<Vec<_>>();

    if !statements.is_empty() {
        // Snapshot the first statement (typically a comment or module)
        insta::assert_debug_snapshot!(
            "prelude_pest_first_statement",
            simplify_pair(statements[0].clone())
        );

        // If we have more statements, snapshot them too
        if statements.len() > 1 {
            insta::assert_debug_snapshot!(
                "prelude_pest_second_statement",
                simplify_pair(statements[1].clone())
            );
        }

        if statements.len() > 2 {
            insta::assert_debug_snapshot!(
                "prelude_pest_third_statement",
                simplify_pair(statements[2].clone())
            );
        }
    }
}

// Test to snapshot specific declaration patterns from key modules
#[test]
fn test_prelude_declaration_pattern_snapshots() {
    // Get the prelude file path
    let prelude_path = Path::new("src/prelude/mod.borf");

    // Read the prelude file
    let prelude_content =
        fs::read_to_string(prelude_path).expect("Should be able to read the prelude file");

    // Parse the prelude using our AST parser
    let parse_result = parse_program(&prelude_content);

    // Verify parsing succeeded
    assert!(
        parse_result.is_ok(),
        "Failed to parse prelude with AST parser"
    );

    let items = parse_result.unwrap();

    // Helper function to find a module by name
    fn find_module<'a>(items: &'a [TopLevelItem], name: &str) -> Option<&'a ModuleDef> {
        items.iter().find_map(|item| {
            if let TopLevelItem::Module(m) = item {
                if m.name == name {
                    return Some(m);
                }
            }
            None
        })
    }

    // 1. Snapshots from the T module (type system foundation)
    if let Some(t_module) = find_module(&items, "T") {
        // Look for the subtyping relation <::
        let subtyping_decl = t_module.elements.iter().find_map(|elem| {
            if let ModuleElement::Declaration(Declaration::MappingDecl { name, .. }) = elem {
                if name == "<::" {
                    return Some(elem);
                }
            }
            None
        });

        if let Some(decl) = subtyping_decl {
            insta::assert_debug_snapshot!("prelude_t_subtyping_relation", decl);
        }

        // Look for a law declaration
        let refl_law = t_module.elements.iter().find_map(|elem| {
            if let ModuleElement::Declaration(Declaration::LawDecl { name, .. }) = elem {
                if name.contains("refl") {
                    return Some(elem);
                }
            }
            None
        });

        if let Some(law) = refl_law {
            insta::assert_debug_snapshot!("prelude_t_reflexivity_law", law);
        }
    }

    // 2. Snapshots from the Cat module (categorical structure)
    if let Some(cat_module) = find_module(&items, "Cat") {
        // Look for the composition operator
        let composition_op = cat_module.elements.iter().find_map(|elem| {
            if let ModuleElement::Declaration(Declaration::MappingDecl { name, .. }) = elem {
                if name == "." {
                    return Some(elem);
                }
            }
            None
        });

        if let Some(op) = composition_op {
            insta::assert_debug_snapshot!("prelude_cat_composition_operator", op);
        }

        // Look for a utility function
        let utility_fn = cat_module.elements.iter().find_map(|elem| {
            if let ModuleElement::Declaration(Declaration::MappingDecl {
                name,
                value: Some(_),
                ..
            }) = elem
            {
                if name != "." && !name.starts_with("law.") {
                    return Some(elem);
                }
            }
            None
        });

        if let Some(func) = utility_fn {
            insta::assert_debug_snapshot!("prelude_cat_utility_function", func);
        }
    }

    // 3. Snapshots from the Wire module (wiring diagrams)
    if let Some(wire_module) = find_module(&items, "Wire") {
        // Look for compatibility with another module
        let compatibility_decl = wire_module.elements.iter().find_map(|elem| {
            if let ModuleElement::Declaration(Declaration::ObjectDecl { name, .. }) = elem {
                if name == "N" || name == "E" {
                    return Some(elem);
                }
            }
            None
        });

        if let Some(decl) = compatibility_decl {
            insta::assert_debug_snapshot!("prelude_wire_compatibility_declaration", decl);
        }
    }

    // 4. Snapshots from the Core module (final exports)
    if let Some(core_module) = find_module(&items, "Core") {
        // The deps declaration that defines module dependencies
        let deps_decl = core_module.elements.iter().find_map(|elem| {
            if let ModuleElement::Declaration(Declaration::ObjectDecl { name, .. }) = elem {
                if name == "deps" {
                    return Some(elem);
                }
            }
            None
        });

        if let Some(deps) = deps_decl {
            insta::assert_debug_snapshot!("prelude_core_dependencies", deps);
        }
    }
}

#[test]
fn test_block_expressions() {
    let input = "let f = \\x. { 
        let y = x + 1; 
        y * 2
    }";
    assert_parses(input);
}

#[test]
fn test_linear_arrow() {
    let input = "map: a -o b";
    assert_parses(input);
}

#[test]
fn test_fallible_operations() {
    let input = "let result = $seq(r1, \\n. {
        if n > 0
        then !Success(n * 2)
        else !Failure(\"Value too small\")
    })";
    assert_parses(input);
}

#[test]
fn test_empty_sequence() {
    let input = "let empty_seq = <>";
    assert_parses(input);
}

#[test]
fn test_type_constructors() {
    let input = "!Result: Type -> Type";
    assert_parses(input);
}

#[test]
fn test_void_type() {
    let input = "law.void_bottom = $forall t $in T: Void <:: t";
    assert_parses(input);
}
