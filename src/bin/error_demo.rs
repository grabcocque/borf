use borf::errors::{create_report, BorfError};
use miette::{GraphicalReportHandler, GraphicalTheme};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up miette hook for pretty printing
    miette::set_hook(Box::new(|_| {
        // Use GraphicalTheme::unicode() for the default unicode theme
        let handler = GraphicalReportHandler::new_themed(GraphicalTheme::unicode())
            .with_cause_chain()
            .with_context_lines(3);
        Box::new(handler)
    }))?;

    println!("Demonstrating Borf Error Handling with Miette:\n");

    // Sample invalid Borf code snippets that will trigger different errors
    let invalid_inputs = vec![
        ("missing_token", "let x = ", "Expected: missing token error"),
        (
            "unexpected_token",
            "let x = 10 + ;",
            "Expected: unexpected token error",
        ),
        // Add more invalid inputs here...
    ];

    for (name, code, _desc) in invalid_inputs {
        println!("-- Testing invalid input: {} --", name);
        // Use the correct parsing function
        match borf::error_reporting::parse_string_to_module_with_enhanced_errors(
            code,
            Some(name.to_string()),
        ) {
            Ok(_) => {
                println!("  Unexpected success (parsing should have failed)");
            }
            Err(err) => {
                // Can convert EnhancedError back into BorfError for uniform handling
                let borf_error = BorfError::Parse(Box::new(borf::errors::ParseError::Unexpected(
                    format!("Enhanced Error Wrapper: {}", err),
                )));
                let report = create_report(borf_error, Some(name.to_string())); // Pass None for source_name
                eprintln!("{:?}", report); // Use debug print for the report
            }
        }
        println!("-----------------------------\n");
    }

    // Example of handling a potential evaluation error (if REPL evaluated)
    let eval_code = "1 / 0";
    println!("-- Testing evaluation error: {} --", eval_code);
    // This part needs the evaluator setup, skipping for now as it's complex
    // match borf::parse_and_evaluate_string(eval_code) { // Hypothetical function
    //     Ok(value) => println!("  Unexpected success: {:?}", value),
    //     Err(err) => {
    //         let report = create_report(err, None); // Pass None for source_name
    //         eprintln!("{:?}", report);
    //     }
    // }
    println!("-----------------------------\n");

    // Example with file parsing (will fail if the file doesn't exist)
    let file_path = Path::new("non_existent_file.borf");
    println!("-- Testing file parsing error: {} --", file_path.display());
    match borf::parse_file(file_path) {
        // Assuming parse_file returns BorfError
        Ok(_) => println!("  Unexpected success (file parsing should fail)"),
        Err(err) => {
            let report = create_report(err, Some(file_path.to_string_lossy().into_owned())); // Pass source name
            eprintln!("{:?}", report);
        }
    }
    println!("-----------------------------\n");

    Ok(())
}
