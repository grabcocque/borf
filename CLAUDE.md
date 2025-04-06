# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build/Test Commands
- Build: `cargo build`
- Run REPL: `cargo run`
- Run all tests: `cargo test`
- Run single test: `cargo test test_name` (e.g., `cargo test test_basic_literals`)
- Run tests with output: `cargo test -- --nocapture`
- Generate snapshot tests: `cargo insta test --review`

## Code Style Guidelines
- **Error Handling**: Use thiserror/miette for rich error reporting with spans and suggestions
- **Errors**: Create specific error variants with detailed diagnostic information
- **Tests**: Snapshot-based testing via insta library with descriptive test names
- **Documentation**: Doc comments (///) for public APIs and functions
- **Imports**: Group by external/internal, explicit imports
- **Naming**: snake_case for variables/functions, CamelCase for types
- **Pattern Matching**: Prefer for error handling and AST processing
- **Modules**: Maintain clear separation between parser, evaluator, and error modules