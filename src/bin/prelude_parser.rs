//! Parses the Borf prelude files and outputs their AST structure.

use borf::parse_file;
use clap::{Arg, Command};
use std::error::Error;
use std::fs;
use std::process;

fn main() {
    let matches = Command::new("prelude_parser")
        .version("0.1.0")
        .author("Borf Development Team")
        .about("Parser for Borf prelude files")
        .arg(
            Arg::new("dir")
                .help("The directory containing prelude files")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Print detailed parsing information")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    if let Some(dir_path) = matches.get_one::<String>("dir") {
        if let Err(e) = run(dir_path, matches.get_flag("verbose")) {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}

fn run(dir_path: &str, verbose: bool) -> Result<(), Box<dyn Error>> {
    println!("Processing prelude directory: {}", dir_path);

    let dir_entries = fs::read_dir(dir_path)?;

    for entry in dir_entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().is_some_and(|ext| ext == "borf") {
            println!("Parsing prelude file: {}", path.display());

            // Parse the prelude file
            match parse_file(&path) {
                Ok(module) => {
                    println!("  ✓ Successfully parsed module: {}", module.name);

                    if verbose {
                        println!("  - Declarations: {}", module.declarations.len());

                        // Count declarations by type
                        let mut type_count = 0;
                        let mut op_count = 0;
                        let mut fn_count = 0;

                        for decl in &module.declarations {
                            match decl {
                                borf::parser::ast::Declaration::TypeDecl(type_decl) => {
                                    type_count += type_decl.names.len();
                                    if verbose {
                                        for name in &type_decl.names {
                                            println!("    * Type: {}", name);
                                        }
                                    }
                                }
                                borf::parser::ast::Declaration::OpDecl(op_decl) => {
                                    op_count += op_decl.names.len();
                                    if verbose {
                                        for name in &op_decl.names {
                                            println!("    * Operator: {}", name);
                                        }
                                    }
                                }
                                borf::parser::ast::Declaration::FnDecl(fn_decl) => {
                                    fn_count += 1;
                                    if verbose {
                                        println!("    * Function: {}", fn_decl.name);
                                    }
                                }
                                borf::parser::ast::Declaration::FnImpl(fn_impl) => {
                                    fn_count += 1;
                                    if verbose {
                                        println!("    * Function: {}", fn_impl.name);
                                    }
                                }
                                _ => {}
                            }
                        }

                        println!("  - Types: {}", type_count);
                        println!("  - Operators: {}", op_count);
                        println!("  - Functions: {}", fn_count);
                    }
                }
                Err(e) => {
                    eprintln!("  ✗ Failed to parse {}: {}", path.display(), e);
                }
            }
        }
    }

    Ok(())
}
