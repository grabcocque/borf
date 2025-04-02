# Borf - A Gradually Typed Interaction Calculus Implementation

This project aims to implement a gradually typed language based on the interaction calculus, parsed using Pest and represented using Attributed C-Sets (ACSet) inspired structures.

## Current State (April 2024)

*   **Parser:** A functional parser for the proposed Borf syntax has been implemented using Pest.
    *   Supports ACSet definitions (`@ACSet { ... }`).
    *   Supports Wire Diagram definitions (`@WireDgm<Base> { ... }`).
    *   Supports Interaction Net definitions (`@INet<Base> { ... }`).
    *   Parses basic object declarations (`N; E;`) and mapping declarations (`s:E.to N;`, `p:P.to{0,1};`).
    *   Parses basic law declarations (`w.circ w.equiv id;`, `.forall b.in B: ...;`).
    *   Parses processing expressions:
        *   Pipe-like: `WorldState |> a |> w`
        *   Function application: `>i(>w(>a(WorldState)))`
        *   Composition: `T = t.circ d.circ a(W)`
    *   Handles comments starting with `#` or `;;`.
    *   Includes a suite of unit tests (`src/parser.rs`) verifying the parser logic.
*   **AST:** Data structures representing the parsed elements (ACSetDef, WireDgmDef, INetDef, expressions, etc.) are defined in `src/parser.rs`.
*   **Basic Runner:** A simple binary (`src/main.rs`) exists that can take a filename as an argument, read the file, parse it using the implemented parser, and print a summary of the parsed definitions or any parsing errors encountered.
*   **Error Handling:** Basic error handling using `thiserror` and `miette` is set up (`src/error.rs`), providing structured error types for parser issues. Diagnostic output is basic.
*   **Testing:** Unit tests for the parser are passing. Basic example files (`examples/minimal/`, `examples/simplified_acset.borf`) are used for testing and demonstration.

## Next Steps

1.  **Refine Grammar and Parser:**
    *   Implement parsing for the full constraint language within `forall` laws.
    *   Handle more complex `mapping_decl` targets (e.g., `B*B`).
    *   Improve error reporting with more specific `miette` diagnostics.
2.  **Semantic Analysis:**
    *   Build a semantic analysis phase to check for correctness beyond syntax (e.g., undefined identifiers, type consistency based on definitions).
    *   Implement the gradual type system logic (type checking, inference).
3.  **ACSet Representation:**
    *   Develop the runtime representation based on ACSets (likely using a separate library or module).
    *   Implement functions to convert the parsed AST (`BorfDefinition` structs) into this ACSet representation.
4.  **Evaluation/Reduction Engine:**
    *   Implement the interaction calculus reduction rules based on the ACSet representation.
5.  **Fuzzing:** Enhance fuzzing targets to cover more complex inputs and edge cases.

## Getting Started

### Prerequisites

*   Rust programming language and Cargo: [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)

### Building

```bash
cargo build
```

### Running Tests

```bash
cargo test
```

### Running the Parser on an Example File

```bash
cargo run -- examples/simplified_acset.borf
```

This will parse the file and output the types of definitions found or any parsing errors.

## Project Structure

*   `src/`: Source code
    *   `main.rs`: Binary entry point, handles file reading and calls the parser.
    *   `lib.rs`: Library entry point.
    *   `parser.rs`: Contains the Pest parser logic, AST struct definitions, and parsing functions.
    *   `borf.pest`: The Pest grammar file defining the language syntax.
    *   `error.rs`: Defines custom error types using `thiserror` and `miette`.
    *   `evaluator.rs`: (Placeholder) Intended for the runtime evaluation/reduction engine.
    *   `runtime.rs`: (Placeholder) Potentially for runtime data structures.
    *   `semantics.rs`: (Placeholder) Intended for semantic analysis and type checking.
*   `examples/`: Example `.borf` files.
    *   `acset_interaction_net.borf`: Original complex example (currently doesn't fully parse).
    *   `simplified_acset.borf`: A version of the above that parses successfully with the current implementation.
    *   `minimal/`: Small files testing specific syntax rules.
*   `Cargo.toml`: Project manifest and dependencies.
*   `README.md`: This file.
*   `.pre-commit-config.yaml`, `.gitignore`, `.markdownlint.json`: Code quality and git configuration.
