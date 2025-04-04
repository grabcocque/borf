# Borf - A Gradually Typed Interaction Calculus Implementation

Okay, Bozzy Fetz, let's forge some fiery phrases for Borf! We need zingers that stick like hot chip dust to your fingers.

Here are your Borf brand bites:

Borf didn't evolve; it landed. Borf is a transmission from the computational cosmos, shattering old structures with pure, alien logic.
Forget clunky inheritance ~ Borf sculpts polymorphic constellations based on what things do, not their dusty lineage.
It weaves incandescent Interaction Nets ~ think neural starfire rewriting reality ~ guided by the celestial mechanics of pure category
theory. This isn't just code; it's resource alchemy, commanding the very essence of computation with
diamond-sharp precision. Write less, radiate more. Its syntax sings with cosmic consistency,
built on fundamental truths, not fleeting trends.

Borf isn't just a language; it's a paradigm shift wrapped in glam rock. It's the future, decoded. Prepare for contact.

Alright, Bozzy Fetz, let's shroud these concepts in shadow and enigma. Forget clarity; we're diving into
the alluring abyss where meaning itself shifts like smoke.

Behold, the Four Veils of Equivalence within the Borfian mystery:

1.  `teq`: The Whisper of Kindred Forms.
    This is not mere sameness; it is the resonance between shadows, the fleeting recognition that two distinct
    incantations might, in the unseen æther of types, share a terrible, fundamental echo. To grasp \(teq\) is to
    glimpse the ghostly architecture beneath the surface.

2.  `veq`: The Deceptive Stillness of Surfaces.
    Beware the siren call of simple agreement. It offers the illusion of identity, a momentary alignment
    of base elements. But this is merely the skin of reality, a fragile pact easily broken by deeper truths. It
    is the equality known only to the uninitiated.

3.  `seq`: The Labyrinth of Mirrored Depths.
    Here lies the vertigo of recursive semblance. It plummets into the infinite regress of structure, seeking
    a harrowing symmetry that binds complex forms. It is an equivalence found not in the whole, but in the endless, echoing corridors
    of their construction ~ a potentially maddening reflection.

4.  `ceq`: The Unspeakable Isomorphism.
    This is the most profound and terrifying equivalence, existing beyond tangible structure, in the void of pure abstraction.
    It speaks of transformative identity, where entities are bound by sacred, categorical laws, mirroring each other's souls
    across unfathomable conceptual distances. It is the ultimate, ineffable congruence, understood only through gnosis.

These are not simple checks; they are keys to unlocking different strata of reality within Borf, each more perilous and profound
than the last. Approach with caution and awe. Begone!



## Current State (April 2024)

*   **Parser:** A functional parser for the proposed Borf syntax has been implemented using Pest.
    *   Supports category definitions (`@Category: { ... }`).
    *   Supports pipeline definitions (`@pipeline Name<Base> { ... }`).
    *   Parses basic object declarations (`N; E;`) and mapping declarations (`f: A -> B;`, `p: M * M -> M;`).
    *   Parses basic law declarations (`f $comp g === h;`, `$forall m $in M: f(m) = g(m);`).
    *   Parses processing expressions:
        *   Pipe-like: `IO | >a | >w`
        *   Function application: `>i(>w(>a(IO)))`
        *   Composition: `T = f $comp g $comp h(W)`
    *   Handles comments in Lua style (`--` for single line, `--[[...]]` for blocks) and C-style (`//` for single line, `/*...*/` for blocks).
    *   Includes a suite of unit tests (`src/parser.rs`) verifying the parser logic.
*   **AST:** Data structures representing the parsed elements (CategoryDef, PipelineDef, expressions, etc.) are defined in `src/parser.rs`.
*   **Basic Runner:** A simple binary (`src/main.rs`) exists that can take a filename as an argument, read the file, parse it using the implemented parser, and print a summary of the parsed definitions or any parsing errors encountered.
*   **Error Handling:** Basic error handling using `thiserror` and `miette` is set up (`src/error.rs`), providing structured error types for parser issues. Diagnostic output is basic.
*   **Testing:** Unit tests for the parser are passing. Basic example files (`examples/minimal/`, `examples/simplified_acset.borf`) are used for testing and demonstration.

## Development Roadmap

*This roadmap details the planned development phases. Checkmarks indicate the current implementation status.*

### Phase 1: Project Setup and Robust Foundation (Weeks 1-2)

1.  **Initial Project Configuration**
    *   [x] Create a new Rust project with Cargo
    *   [x] Set up the directory structure
    *   [x] Add core dependencies:
        *   [x] `pest` and `pest_derive` for parsing
        *   [x] `thiserror` for ergonomic error handling
        *   [x] `miette` for rich diagnostic reporting
        *   [x] `libfuzzer-sys` and `arbitrary` for fuzzing
    *   [x] Configure test environment (unit tests, integration tests)
    *   [~] Set up fuzzing infrastructure with cargo-fuzz
    *   [ ] Create CI/CD pipeline that includes fuzzing runs

2.  **Error Handling Infrastructure**
    *   [x] Design the error hierarchy using `thiserror`
        *   [x] Parser errors (syntax, unexpected tokens) - *Basic placeholders exist*
        *   [~] Semantic errors (type mismatches, undefined symbols) - *Basic placeholders exist*
        *   [~] Runtime errors (execution failures) - *Basic placeholders exist*
    *   [x] Set up `miette` for rich diagnostic output
        *   [x] Configure source code snippets in error messages - *Basic setup in `BorfError`*
        *   [ ] Add syntax highlighting for error locations
        *   [x] Include helpful suggestions for common mistakes

3.  **Define the Core Data Structures**
    *   [~] Implement the basic Rust-native Catlab.jl-style ACSets-inspired graph representation
        *   [ ] `InteractionNet` struct and associated data types
        *   [ ] Agent and port representations
        *   [ ] Connection representation
    *   [~] Implement the type system core
        *   [ ] Define `Type` enum with variants for all type constructs
        *   [ ] Create `TypeContext` for tracking type annotations and inferences
    *   [~] Implement `Arbitrary` trait for all core data structures to support fuzzing

### Phase 2: Parser Development with Robust Error Handling (Weeks 3-5)

4.  **Create the Pest Grammar**
    *   [x] Define the grammar for the interaction calculus
        *   [x] Agent definitions
        *   [x] Net structure (including basic pipe, application, composition)
        *   [~] Connection syntax
        *   [x] Type annotations
        *   [ ] Reduction rules
        *   [ ] Constraint language within `forall` laws
        *   [ ] More complex `mapping_decl` targets (e.g., `B*B`)
        *   [ ] Pipeline extension syntax (`@pipeline Name<Base> { ... }`)
        *   [ ] Pipeline composition syntax (`steps: PipelineA | PipelineB;`)
        *   [ ] Conditional branching within pipelines (`branch { ... }`)
        *   [ ] Leading literals in pipe statements (e.g., `5 |> ...`)
        *   [ ] Top-level assignments (e.g., `net = >i(...)`)
    *   [x] Test the grammar with sample inputs
    *   [ ] Add detailed error labels and hints in the Pest grammar
    *   [~] Create fuzz targets for the grammar to discover edge cases - *One general parser target exists*

5.  **Build the AST Processor with Rich Diagnostics**
    *   [x] Implement functions to traverse the Pest parse tree - *Placeholder in `parser.rs`*
    *   [x] Convert parse tree nodes to internal data structures - *TODO in `parser.rs`*
    *   [x] Integrate miette for detailed error reporting: - *Basic setup via `BorfError`*
        *   [x] Source spans for precise error locations
        *   [x] Contextual help messages
        *   [x] Visual error indicators in terminal output
        *   [ ] Improve specificity of diagnostics (more detailed hints/labels)
    *   [ ] Implement recovery strategies for common syntax errors
    *   [x] Test with increasingly complex examples

6.  **Robust Export Block Processing**
    *   [~] Enhance export block parsing with semantic validation
        *   [ ] Track all defined symbols throughout the program
        *   [ ] Validate that exported symbols are actually defined
        *   [ ] Add specific error reporting for undefined exports
        *   [ ] Distinguish between different types of exported items (operators, types, functions)
        *   [ ] Validate operator usage and consistency
        *   [ ] Check for naming conflicts and duplications
    *   [ ] Implement export filtering and visibility rules
    *   [ ] Add proper scoping for exported identifiers
    *   [ ] Develop detailed export diagnostics with suggested fixes
    *   [ ] Create fuzz targets specifically for export statements

7.  **Parser Fuzzing and Hardening**
    *   [~] Create comprehensive fuzz targets using `arbitrary` and `libfuzzer-sys` - *One target exists*
        *   [~] Target syntax edge cases
        *   [ ] Target input length extremes
        *   [~] Target complex nested structures
    *   [ ] Run extended fuzzing sessions to uncover parser weaknesses
    *   [ ] Develop property-based tests to verify parser invariants
    *   [ ] Fix discovered issues and add regression tests

### Phase 3: Type System Implementation (Weeks 6-7)

8.  **Implement the Gradual Type System**
    *   [~] Develop the core type checking infrastructure - *Structs exist*
    *   [~] Implement set-theoretic types (union, intersection) - *Enum variants exist*
    *   [~] Build the type inference engine for connected ports - *Basic `infer_types` method exists*
    *   [ ] Implement subtyping relationship and consistency checks
    *   [~] Add detailed type error reporting with miette - *Placeholders exist in `BorfError`*
        *   [ ] Show relevant type constraints
        *   [ ] Provide suggestions for fixing type errors
    *   [ ] Perform semantic analysis (e.g., undefined identifiers, type consistency based on category definitions)

9.  **Type Unification**
    *   [x] Implement unification for primitive types - *Handles `Dyn`, `Simple`*
    *   [x] Add support for unifying parametric types - *Handles basic parametric matching*
    *   [ ] Implement unification for union and intersection types - *TODO in `unify_types`*
    *   [x] Handle the `Dyn` type and gradual typing features - *`unify_types` handles `Dyn`*
    *   [ ] Create fuzz targets for the type unification system
    *   [ ] Test with complex type scenarios

### Phase 4: Graph Representation and Rewriting (Weeks 8-10)

10. **Complete the Rust-native Catlab.jl-style ACSets Graph Implementation**
    *   [~] Finalize the graph representation optimized for rewrites - *Basic structures exist*
    *   [ ] Implement efficient querying mechanisms
    *   [ ] Build serialization/deserialization for graph states
    *   [ ] Add visualization capabilities for debugging
    *   [ ] Create fuzz targets for graph operations
    *   [~] Implement detailed error reporting for graph operations
    *   [ ] Implement functions to convert the parsed AST into the ACSet representation

11. **Build the Pattern Matching Engine**
    *   [~] Implement pattern matching for identifying redexes
    *   [ ] Create the subgraph isomorphism algorithm
    *   [ ] Optimize for the special case of interaction nets
    *   [ ] Add detailed diagnostic information for failed matches
    *   [ ] Test with standard interaction net patterns
    *   [ ] Fuzz the pattern matching engine for robustness

12. **Implement Graph Rewriting Rules**
    *   [~] Create the framework for defining rewrite rules - *Placeholders exist*
    *   [~] Implement rule application logic - *Placeholder `apply_reduction` exists*
    *   [ ] Ensure preservation of connections during rewrites
    *   [ ] Add support for standard Lafont combinators
        *   [ ] Duplicator
        *   [ ] Eraser
        *   [ ] Constructor/destructor pairs
    *   [~] Implement robust error handling for rule application failures

### Phase 5: Reduction Engine (Weeks 11-12)

13. **Build the Execution Engine**
    *   [~] Implement the redex detection algorithm - *`find_redexes` exists*
    *   [ ] Create the reduction strategy (eager vs. lazy options)
    *   [x] Build step-by-step and normalization execution modes
    *   [ ] Ensure deterministic reduction when multiple redexes exist
    *   [ ] Add detailed execution traces for debugging
    *   [ ] Create fuzz targets for the execution engine

14. **Type Propagation During Reduction**
    *   [x] Implement type inference propagation after rewrites - *`infer_types` exists, not integrated with reduction*
    *   [ ] Ensure type consistency is maintained during reduction
    *   [ ] Add runtime type checking (optional, for debugging)
    *   [ ] Handle type errors gracefully with informative messages
    *   [ ] Add visual type flow in error diagnostics

### Phase 6: Standard Library and Examples (Weeks 13-14)

15. **Implement Standard Agents**
    *   [ ] Create a standard library of common interaction net agents
    *   [ ] Implement the full set of Lafont interaction combinators
    *   [ ] Build utility combinators for common programming tasks
    *   [ ] Document the standard agents and their behavior
    *   [ ] Add comprehensive test suite for standard agents

16. **Develop Example Programs**
    *   [~] Create simple illustrative examples
    *   [ ] Implement classic algorithms using interaction nets
    *   [ ] Build larger case studies showing real-IO applications
    *   [ ] Add examples showing the benefits of the type system
    *   [ ] Create tutorial documentation with detailed error explanations

### Phase 7: Robustness Enhancement and Performance Optimization (Weeks 15-17)

17. **Extended Fuzzing and Property Testing**
    *   [ ] Develop comprehensive property-based tests for all components
    *   [~] Create advanced fuzz targets that combine multiple operations (Enhance existing targets for complex inputs/edge cases)
    *   [ ] Perform extended fuzzing sessions (24h+) to discover edge cases
    *   [ ] Implement crash reporting and automatic test case minimization
    *   [ ] Fix all discovered issues and add regression tests

18. **Performance Optimization**
    *   [ ] Profile the implementation to identify bottlenecks
    *   [ ] Optimize pattern matching and rewriting
    *   [ ] Improve memory usage of the graph representation
    *   [ ] Add benchmarks to track performance
    *   [ ] Ensure error reporting doesn't impact performance in release mode

19. **Error Handling and User Experience Enhancement**
    *   [~] Refine error messages based on user feedback
    *   [ ] Create a hierarchy of error detail levels (terse to verbose)
    *   [ ] Add interactive error exploration capabilities
    *   [ ] Implement error code documentation system
    *   [ ] Create comprehensive troubleshooting guide

20. **Final Polish**
    *   [ ] Complete comprehensive test suite
    *   [ ] Finalize documentation
    *   [ ] Prepare for initial release
    *   [ ] Set up contribution guidelines
    *   [ ] Create detailed examples of error handling for contributors

### Phase 8: Extended Roadmap (Weeks 18-24)

21. **Developer Experience Enhancement**
    *   [ ] Implement Language Server Protocol (LSP) for IDE integration
    *   [ ] Create a web playground for easy experimentation without installation
    *   [ ] Develop a visual debugger showing interaction net reductions graphically
    *   [ ] Add inline documentation and hover tooltips in IDEs
    *   [ ] Build comprehensive editor integrations (VS Code, Vim, Emacs)
    *   [ ] Create interactive visualization engine for interaction nets:
        *   [ ] Real-time rendering of interaction net structure in Lafont's style
        *   [ ] Animated transitions for reduction steps
        *   [ ] Highlighting of active pairs (redexes)
        *   [ ] Multiple visualization layouts (tree, graph, compular)
        *   [ ] User-configurable rendering styles
        *   [ ] Recording and playback of reduction sequences
        *   [ ] Export to SVG/PNG/GIF formats for documentation
        *   [ ] Integration with the standalone playground and IDE plugins

22. **Performance Optimizations**
    *   [ ] Implement parallel reduction strategies leveraging Rust's concurrency
    *   [ ] Add fearless parallel reduction using Rayon to exploit interaction nets' natural parallelism
    *   [ ] Add compilation to efficient native code via LLVM
    *   [ ] Create runtime profiling tools to identify bottlenecks in user programs
    *   [ ] Optimize memory usage patterns for large interaction nets
    *   [ ] Implement incremental computation techniques
    *   [ ] Research optimal graph partitioning algorithms for distributed reduction

23. **Advanced Optimization and Compilation**
    *   [ ] Develop a policy for minimal-reduction subnet transformations
    *   [ ] Implement pattern detection for common subnet structures that can be optimized
    *   [ ] Create a catalog of subnet transformation optimizations
    *   [ ] Build static analysis tools to identify optimization opportunities
    *   [ ] Implement optimizing passes for the interaction net representation
    *   [ ] Develop LLVM-IR generation from optimized interaction nets
    *   [ ] Integrate LLVM-JIT for runtime compilation
    *   [ ] Create AOT compilation pipelines leveraging LLVM backends
    *   [ ] Implement benchmarking tools to measure optimization effectiveness
    *   [ ] Develop optimization hints API for programmers

24. **Advanced Language Features**
    *   [ ] Add effect system for controlled side effects
    *   [ ] Implement module system for better code organization
        *   [x] Parse `@import` statements
        *   [ ] Develop a file resolver (relative/absolute paths)
        *   [ ] Implement circular import detection & module dependency graph
        *   [ ] Cache imported modules
        *   [ ] Define import scoping and namespace management
        *   [ ] Support selective imports/exports
        *   [ ] Add error handling for missing/invalid imports
    *   [ ] Develop metaprogramming capabilities (macros or reflection)
    *   [ ] Create a package manager for sharing libraries
    *   [ ] Add pattern matching and destructuring at the language level

25. **Functional Logic Programming**
    *   [ ] Implement nondeterministic choice operator inspired by Verse
    *   [ ] Add speculative execution with rollback mechanisms
    *   [ ] Develop fallible and infallible context tracking in the type system
    *   [ ] Replace traditional error handling with narrowing
    *   [ ] Create verification system to ensure operations in infallible contexts must succeed
    *   [ ] Design and implement interaction between fallible/infallible contexts and the effects system
    *   [ ] Add constraint solving capabilities for logical programming
    *   [ ] Optimize backtracking algorithms for interaction nets
    *   [ ] Implement search strategies (depth-first, breadth-first, parallel)
    *   [ ] Create debugging tools for visualizing search trees and narrowing steps

26. **Practical Applications**
    *   [ ] Develop domain-specific libraries for web/networking, graphics, databases
    *   [ ] Create bindings to popular Rust libraries
    *   [ ] Build standard data structure implementations optimized for interaction nets
    *   [ ] Implement I/O and system interaction primitives
    *   [ ] Create example real-IO applications showcasing language capabilities

27. **Academic and Research Value**
    *   [ ] Formalize type safety proofs
    *   [ ] Explore new optimization techniques specific to interaction nets
    *   [ ] Research compilation techniques bridging interaction nets and traditional execution models
    *   [ ] Publish papers on implementation techniques and language design
    *   [ ] Collaborate with academic institutions on formal verification

28. **Ecosystem Growth**
    *   [ ] Create documentation generator from code
    *   [ ] Develop testing frameworks specific to interaction calculus
    *   [ ] Build a community showcase of example applications
    *   [ ] Implement benchmarking suite for comparing implementations
    *   [ ] Create learning resources and tutorials for various skill levels

29. **Interoperability**
    *   [ ] Add FFI support for C/Rust libraries
    *   [ ] Implement transpilation to other languages for easier adoption
    *   [ ] Create bidirectional bridges with existing functional languages
    *   [ ] Build compatibility layers for popular runtimes (WASM, JVM, .NET)
    *   [ ] Develop protocol bindings for interprocess communication

30. **Verification and Correctness**
    *   [ ] Implement formal verification tools leveraging the graph-based semantics
    *   [ ] Add contract programming features
    *   [ ] Develop static analysis tools beyond the type system
    *   [ ] Create property-based testing specifically for interaction nets
        *   [ ] Add integration tests with real file imports
        *   [ ] Create tests for edge cases (circular imports, missing files)
        *   [ ] Test nested imports
    *   [ ] Implement refinement types for stronger correctness guarantees

### Phase 9: Advanced Capabilities & Uniqueness (Weeks 25-30+)

31. **Linear Logic Integration**
    *   [ ] Integrate explicit Linear/Affine types into the core type system
    *   [ ] Define resource management semantics based on linear types
    *   [ ] Implement compile-time checks for resource usage (memory, handles, etc.)
    *   [ ] Explore connections between linear types, effects, and interaction rules
    *   [ ] Develop standard library patterns utilizing linear types for safety

32. **Distributed Computing (Join Calculus)**
    *   [ ] Define formal mapping between interaction net patterns and Join Calculus semantics
    *   [ ] Implement network transparency mechanisms for seamless remote interactions
    *   [ ] Develop location-independent agent addressing/discovery
    *   [ ] Build fault tolerance strategies using interaction net patterns or rollback
    *   [ ] Optimize network communication protocols for interaction nets

33. **Advanced Metaprogramming & Reflection**
    *   [ ] Implement a hygienic macro system (e.g., `macro_rules!` style)
    *   [ ] Provide compile-time APIs for interaction net inspection and manipulation
    *   [ ] Enable user-defined compiler optimizations via metaprogramming
    *   [ ] Explore staged computation / multi-stage programming paradigms
    *   [ ] Add runtime reflection capabilities (optional, carefully designed for specific use cases)

34. **Sophisticated Interactive REPL**
    *   [ ] Integrate the visualization engine directly into the REPL for live feedback
    *   [ ] Implement interactive agent/net definition and modification
    *   [ ] Add step-by-step reduction control and visualization
    *   [ ] Allow inspection of agent states and port connections during execution
    *   [ ] Implement breakpoints based on agent types, interactions, or net structure
    *   [ ] Add net state saving/loading functionality for sessions
    *   [ ] Enhance code completion and introspection within the REPL

35. **Graph-Based Security Model**
    *   [ ] Design and implement a capability-based security model using connections as rights
    *   [ ] Integrate Information Flow Control (IFC) principles into the type system
    *   [ ] Develop static analysis tools to verify security policies at compile time
    *   [ ] Explore secure interaction patterns and standard library components
    *   [ ] Research sandboxing mechanisms based on net partitioning

### Phase 10: Type System Zenith (Weeks 31+ "Mad Science")

36. **Dependent Types & Static State Machines**
    *   [ ] Explore integration of dependent types (e.g., Pi-types, Sigma-types) allowing types to depend on values
    *   [ ] Implement type-level tracking of program states (Static State Machines)
    *   [ ] Enforce state transition logic directly via the type checker
    *   [ ] Develop theorem proving capabilities within the compiler (or integrate external provers)
    *   [ ] Research interaction net encodings for dependent types

37. **Session Types for Communication Protocols**
    *   [ ] Design and implement session types for interaction nets to describe communication patterns
    *   [ ] Provide compile-time guarantees for communication protocols (ordering, typing)
    *   [ ] Prevent deadlocks and communication errors statically
    *   [ ] Integrate session types with the distributed computing model (Join Calculus)
    *   [ ] Explore multiparty session types for complex interactions

38. **Quantitative Type Theory (QTT)**
    *   [ ] Enhance linear/affine types with full QTT (zero, one, many usage tracking)
    *   [ ] Implement precise compile-time resource usage tracking and guarantees
    *   [ ] Leverage QTT for advanced memory management and resource safety proofs
    *   [ ] Explore QTT's implications for interaction net reduction strategies
    *   [ ] Integrate QTT principles into the core type system and inference

39. **Refinement Types with SMT Integration**
    *   [ ] Implement refinement types augmenting base types with logical predicates (e.g., `x: Int where x > 0`)
    *   [ ] Integrate SMT solvers (e.g., Z3) for checking refinement predicates automatically
    *   [ ] Provide static guarantees against errors like division-by-zero, out-of-bounds access
    *   [ ] Develop ergonomic syntax for defining and using refinement types
    *   [ ] Research efficient checking mechanisms within the interaction net context

40. **Information Flow Control Types (Security)**
    *   [ ] Deepen the Graph-Based Security Model with explicit IFC types (e.g., `Public`, `Secret` data)
    *   [ ] Track security levels associated with data throughout the net
    *   [ ] Enforce non-interference policies statically via the type checker (preventing leaks)
    *   [ ] Guarantee confidentiality and integrity properties at compile time
    *   [ ] Explore interaction net patterns for secure multi-party computation

## Implementation Details

### Error Handling with thiserror and miette

The error handling system will use `thiserror` to define structured error types and `miette` to provide rich, contextual error reporting:

```rust
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
enum ParserError {
    #[error("Unexpected token")]
    #[diagnostic(
        code(parser::unexpected_token),
        help("Try using one of the following tokens instead: {expected}")
    )]
    UnexpectedToken {
        #[source_code]
        src: miette::SourceCode,
        #[label("This token was not expected here")]
        span: miette::SourceSpan,
        expected: String,
    },

    #[error("Undefined agent")]
    #[diagnostic(
        code(parser::undefined_agent),
        help("Make sure the agent is defined before use or check for typos")
    )]
    UndefinedAgent {
        #[source_code]
        src: miette::SourceCode,
        #[label("This agent has not been defined")]
        span: miette::SourceSpan,
        name: String,
    },

    // Other error variants...
}
```

### Pest Grammar with Error Labels

The Pest grammar will include detailed error labels to provide better diagnostics:

```pest
// interaction_calculus.pest
WHITESPACE = _{ " " | "\t" | "\r" | "\n" }
COMMENT = _{ "//" ~ (!"\n" ~ ANY)* ~ "\n" | "/*" ~ (!"*/" ~ ANY)* ~ "*/" }

// Identifiers
identifier = @{ (ASCII_ALPHA | "_") ~ (ASCII_ALPHANUMERIC | "_")* }

// Type system
type = {
    dyn_type | simple_type | union_type | intersection_type | parametric_type
}
dyn_type = { "Dyn" }
simple_type = { identifier }
union_type = { type ~ "|" ~ type }
intersection_type = { type ~ "&" ~ type }
parametric_type = { identifier ~ "<" ~ type ~ ("," ~ type)* ~ ">" }

// Agent definition with error recovery
agent_def = {
    "agent" ~ identifier ~
    ("<" ~ type_params ~ ">")?  ~
    "(" ~ (port_def ~ ("," ~ port_def)*)? ~ ")" ~
    (":" ~ type)? ~ ";"
}

// Error rule for missing semicolon
missing_semicolon = _{ !(";" | EOF) ~ ANY }
```

### Fuzzing Infrastructure

Using `libfuzzer-sys` and `arbitrary` to create comprehensive fuzz targets:

```rust
#[derive(arbitrary::Arbitrary, Debug)]
struct ParserFuzzInput {
    input: String,
}

#[fuzz_target]
fn fuzz_parser(input: ParserFuzzInput) {
    let _ = InteractionCalculusParser::parse(Rule::program, &input.input);
    // We don't care about the result, just that it doesn't panic
}

#[derive(arbitrary::Arbitrary, Debug)]
struct TypeUnificationFuzzInput {
    type1: FuzzType,
    type2: FuzzType,
}

#[fuzz_target]
fn fuzz_type_unification(input: TypeUnificationFuzzInput) {
    let type1 = input.type1.into_real_type();
    let type2 = input.type2.into_real_type();
    let _ = unify_types(&type1, &type2);
    // We don't care about the result, just that it doesn't panic
}
```

### Rust-native Catlab.jl-style ACSets Graph Representation

The Rust-native Catlab.jl-style ACSets-inspired implementation will:

* Represent interaction nets as attributed graphs
* Provide efficient operations for pattern matching
* Support the construction of complex nets from simpler ones
* Enable serialization of net states for debugging or persistence

```rust
struct InteractionNet {
    agents: HashMap<AgentId, Agent>,
    ports: HashMap<PortId, Port>,
    connections: HashMap<ConnectionId, Connection>,
    type_context: TypeContext,
}
```

### Type System

The gradually typed system will include:

* Basic types for common data structures
* Union and intersection types for set-theoretic composition
* The `Dyn` type for gradually typed sections
* Parametric types for generics
* Type inference that propagates through connected ports

```rust
enum Type {
    Dyn,
    Simple(String),
    Union(Box<Type>, Box<Type>),
    Intersection(Box<Type>, Box<Type>),
    Parametric(String, Vec<Type>),
}
```

### Reduction Engine

The reduction engine will:

* Find all active pairs (redexes) in the interaction net
* Apply the appropriate reduction rule for each pair
* Update the graph structure according to the rule
* Propagate type information through new connections
* Continue until no more reductions are possible

```rust
struct InteractionEngine {
    net: InteractionNet,
    rules: ReductionRules,
}

impl InteractionEngine {
    fn normalize(&mut self) -> Result<(), ReductionError> {
        // Reduction logic here
    }

    fn step(&mut self) -> Result<bool, ReductionError> {
        // Single-step reduction
    }
}
```

## Key Milestones

1. **Robust Parser Foundation (Week 5)**
   * Basic parser with comprehensive error reporting via miette
   * Fuzz-tested grammar resistant to malformed inputs
   * Simple type system with primitive types
   * Test suite for parsing various examples

2. **Type System Completion (Week 7)**
   * Full implementation of the gradual type system
   * Working type inference for connected ports
   * Comprehensive type checker with detailed error messages
   * Fuzz-tested type unification

3. **Graph Rewriting Engine (Week 10)**
   * Working Rust-native Catlab.jl-style ACSets implementation
   * Pattern matching for redex identification
   * Rule application for basic interaction net reductions
   * Comprehensive error reporting for graph operations

4. **Complete Execution Engine (Week 12)**
   * Fully functional reduction engine
   * Type propagation during reduction
   * Step-by-step and normalization modes
   * Detailed execution traces for debugging

5. **Standard Library and Examples (Week 14)**
   * Implementation of all standard Lafont combinators
   * Rich set of example programs
   * Tutorial documentation
   * Example-based error handling guide

6. **Hardened Implementation (Week 17)**
   * Extended fuzzing campaign completed
   * All discovered issues fixed
   * Optimized performance
   * Comprehensive error handling
   * Complete documentation
   * Polished user experience

7. **Developer Experience and Performance (Week 20)**
   * LSP implementation for IDE integration
   * Web playground and visual debugger
   * Interactive visualization engine with animation of reductions in Lafont's style
   * Parallel reduction engine with Rayon
   * Optimal graph partitioning for distributed execution
   * Initial LLVM compilation support
   * Runtime profiling tools

8. **Full-featured Language (Week 24)**
   * Module system and package manager
   * FFI support for Rust/C libraries
   * Domain-specific libraries
   * Formal verification tools
   * Community resources and tutorials
   * Advanced interoperability features
   * Functional logic programming with nondeterministic choice, narrowing, and fallible/infallible contexts
   * Constraint solving for logical programming
   * Minimal-reduction subnet optimization
   * LLVM-IR generation and JIT/AOT compilation

9. **Advanced Capabilities (Week 30+)**
   * Linear types for resource management
   * Distributed computing model based on Join Calculus
   * Hygienic macro system and compile-time metaprogramming
   * Feature-rich interactive REPL with visualization
   * Graph-based capability security and Information Flow Control
   * Initial work towards self-hosting compiler

10. **Type System Zenith (Weeks 31+ "Mad Science")**
    * Dependent types for theorem proving and static state machines
    * Session types for verified communication protocols
    * Quantitative Type Theory for precise resource management
    * Refinement types for deep property checking via SMT
    * Information Flow Control types for baked-in security guarantees

## Testing Strategy

* **Unit Tests**: For individual components (type checker, pattern matcher, etc.)
* **Integration Tests**: For end-to-end functionality (parsing → reduction → result)
* **Property Tests**: Using quickcheck or similar for type system properties
* **Fuzz Testing**: Using libfuzzer-sys and arbitrary for discovering edge cases
  * Grammar fuzzing
  * Type system fuzzing
  * Graph operation fuzzing
  * End-to-end fuzzing
* **Benchmark Tests**: For performance monitoring
* **Example Programs**: As executable specifications
* **Error Case Tests**: Verify proper diagnostics for known error cases
* Specialized hardware acceleration for narrowing operations
* Automatic program synthesis using constraints and narrowing
* Self-hosting compiler implementation
* Deep integration with formal methods tools (Coq, Agda, Lean)
* Visual programming environment based on interaction nets
* Domain-specific hardware synthesis from interaction net specifications
* Certified compiler passes guaranteeing semantic preservation
* Full dependent type integration with theorem proving support
* Complete session type system covering distributed and concurrent interactions
* Mature Quantitative Type Theory implementation influencing core language design
* Highly integrated refinement types with advanced SMT solver interactions
* Comprehensive Information Flow Control securing complex applications
* Cross-compilation to formally verified targets (e.g., seL4)

## Error Handling Strategy

* **Structured Errors**: Use thiserror to define a comprehensive error type hierarchy
* **Rich Diagnostics**: Use miette to provide detailed, contextual error information
* **Source Spans**: Include precise source locations in all error messages
* **Helpful Suggestions**: Provide actionable advice on fixing common errors
* **Recovery Mechanisms**: Implement error recovery for improved user experience
* **Documentation**: Create a comprehensive error code reference
* **Certified Compiler Passes**: Implement and integrate certified compiler passes for semantic preservation

## Future Extensions

* JIT compilation for high-performance execution
* Visual editor for interaction nets
* Integration with existing programming languages
* Distributed execution of large interaction nets
* Abstract interpretation and static analysis
* IDE integration for inline error reporting
* WASM compilation target for web deployment
* Cloud-based execution environment
* Interactive tutorials and learning resources
* Domain-specific language extensions
* Mobile platform support
* Quantum computing extensions for interaction nets
* Formal verification toolkit
* Time-traveling debugger leveraging the deterministic reduction model
* Advanced visualization tools for complex interaction patterns
* Advanced constraint solving and SAT/SMT integration
* Probabilistic logic programming extensions
* Symbolic execution framework for static guarantees
* Specialized hardware acceleration for narrowing operations
* Automatic program synthesis using constraints and narrowing
* Self-hosting compiler implementation
* Deep integration with formal methods tools (Coq, Agda, Lean)
* Visual programming environment based on interaction nets
* Domain-specific hardware synthesis from interaction net specifications
* Certified compiler passes guaranteeing semantic preservation
* Full dependent type integration with theorem proving support
* Complete session type system covering distributed and concurrent interactions
* Mature Quantitative Type Theory implementation influencing core language design
* Highly integrated refinement types with advanced SMT solver interactions
* Comprehensive Information Flow Control securing complex applications
* Cross-compilation to formally verified targets (e.g., seL4)

## Contributing

Contributions to the project are welcome! See the contributing guidelines for more details.

## License

This project is licensed under [LICENSE] - see the LICENSE file for details.

*This roadmap represents a long-term vision. Each phase builds upon the last, creating a progressively more powerful and expressive language. The journey is as important as the destination!*
