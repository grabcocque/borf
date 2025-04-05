use borf::errors::{create_report, BorfError};
use miette::GraphicalReportHandler;
use std::path::Path;

fn main() {
    // Configure Miette to use pretty graphical output with unicode symbols
    miette::set_hook(Box::new(|| {
        let mut handler = GraphicalReportHandler::new();
        handler.with_cause_chain(true);
        Box::new(handler)
    }))
    .unwrap();

    // Sample invalid Borf code snippets that will trigger different errors
    let examples = vec![
        ("Missing colon in entity declaration", "entity x Type"),
        ("Missing closing brace", "module: { fn: { a b }"),
        ("Unexpected token", "type: {a b c} + wrong"),
        ("Using 'fun' instead of 'fn'", "fun: {a b c}"),
        ("Module without @ prefix", "Module: { }"),
    ];

    for (description, code) in examples {
        println!("\n\n--- Example: {} ---", description);
        println!("Code: {}", code);

        match borf::parse_str(code) {
            Ok(_) => println!("Surprisingly, this parsed successfully!"),
            Err(err) => {
                // Convert to a BorfError and create a report
                let report = create_report(err);
                println!("\nError:");
                eprintln!("{:?}", report);
            }
        }
    }

    // Example with file parsing (will fail if the file doesn't exist)
    println!("\n\n--- Example: File parsing error ---");
    let path = Path::new("nonexistent_file.borf");
    match borf::parse_file(path) {
        Ok(_) => println!("Surprisingly, this parsed successfully!"),
        Err(err) => {
            let report = create_report(err);
            println!("\nError:");
            eprintln!("{:?}", report);
        }
    }
}
