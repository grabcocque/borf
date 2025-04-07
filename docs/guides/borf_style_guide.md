# Borf Style Guide

This document outlines the idiomatic style for writing Borf code. Borf is a concatenative functional logic programming language that executes code by reducing interaction nets.

## Core Principles

1. Concatenative style: Values come before operations (e.g., `x y f` instead of `f(x, y)`)
2. Quotations for functions: Use `[a, b -> ...]` syntax for function definitions 
3. Named parameters in quotations: Use the assignment arrow `->` to bind parameter names
4. Pipeline operator: Use `|>` for chaining operations and improving readability

## Syntax Conventions

### Comments

Borf uses Lua-style comments:

```borf
-- This is a single-line comment

--[[
  This is a multi-line comment
  that spans multiple lines
]]--
```

**Important**: Never use `//` or `/* */` style comments from other languages.

### Logical Operators

Borf uses English words for logical operators, not symbols:

```borf
-- Correct
condition1 and condition2   -- Logical AND
condition1 or condition2    -- Logical OR
not condition              -- Logical NOT
condition1 iff condition2  -- Logical IF AND ONLY IF

-- Incorrect (don't use)
condition1 && condition2   -- Don't use && for AND
condition1 || condition2   -- Don't use || for OR
!condition                 -- Don't use ! for NOT
```

### Equality and Comparison Operators

Borf uses specific equality operators for different types of comparisons:

```borf
a veq b      -- Value equality (for comparing values and function results)
a teq b      -- Type equality (for comparing types)
a ceq b      -- Category equality (for comparing objects in categories)
a seq b      -- Structural equality (for comparing structures)

a <:: b      -- Subtyping relation (a is a subtype of b)
```

Never use `==` or `===` for equality comparisons in Borf code.

### Pipeline Operator (`|>`)

The pipeline operator is a fundamental part of Borf syntax that passes the result of an expression to the next function:

```borf
value |> function |> result
```

Using the pipeline operator makes the flow of data explicit and improves readability, especially in longer chains:

```borf
initial_value |>
  first_operation |> 
  second_operation |> 
  third_operation
```

**ESSENTIAL RULE:** Always use the pipeline operator when chaining multiple operations or creating data processing pipelines. This makes your code more readable and maintainable.

**Good example (clear data flow):**
```borf
input_data |>
  [data -> data first_transform] |>
  [data -> data second_transform] |>
  [data -> data third_transform] |>
  final_operation
```

**Bad example (unclear flow):**
```borf
input_data
  first_transform
  second_transform
  third_transform
  final_operation
```

### Assignment Arrow (`->`)

The assignment arrow is used for variable binding and in quotation parameter lists:

```borf
value -> variable_name    -- Binds a value to a variable
[x, y -> x y +]           -- Defines a quotation with parameters x and y
```

Be careful not to confuse the pipeline operator `|>` with the assignment arrow `->`.

### Commas in Parameter Lists

Use commas to separate parameters in quotation parameter lists:

```borf
[a, b, c -> ...]    -- Correct: Parameters separated by commas
[a b c -> ...]      -- Incorrect: Missing commas between parameters
```

Commas are also used in literals like maps, tuples, and lists:

```borf
{key1: value1, key2: value2}    -- Map literal
(a, b, c)                       -- Tuple literal
[1, 2, 3, 4]                    -- List literal
```

### Match Expressions

Use match expressions for pattern matching:

```borf
value {
  | Pattern1 -> result1
  | Pattern2 -> result2
  | _ -> default_result
} match
```

### Function Application

Follow the concatenative style for function application:

```borf
x y function       -- Apply function to arguments x and y (concatenative style)
function(x, y)     -- Avoid this applicative style
```

## Style Examples

### Concatenative Style (Preferred)

```borf
5 10 +                  -- Addition
"Hello" " " "World" + + -- String concatenation

-- Function application
a b c my_function

-- Chained operations
value 
  first_operation |> intermediate 
  second_operation
```

### Module and Namespace Access

```borf
-- Importing modules (use @ symbol)
deps: {
  (@CurrentModule, @OtherModule, true)
}

-- Accessing functions from modules (no @ symbol)
value OtherModule.function

-- Creating local aliases for imported functions
ne: Prim.ne  -- Create local alias for Prim.ne
value ne     -- Now you can use ne directly
```

### Applicative Style (Avoid)

```borf
+(5, 10)                        -- Avoid this style for function application
concat("Hello", " ", "World")   -- Avoid this style for function application 
my_function(a, b, c)            -- Avoid this style for function application
```

## Formatting

1. Use 2-space indentation
2. Use spaces around operators: `a + b` not `a+b`
3. Use line breaks for readability in complex expressions
4. For pipeline chains, use consistent indentation

## Best Practices

1. Use pipeline operators to improve readability of data flow
2. Use descriptive parameter names in quotations
3. Write short, focused functions
4. Utilize pattern matching to handle different cases clearly
5. Use comments to explain complex operations or algorithms

## Documentation and Comments

Effective documentation helps users understand both how to use your code and the principles behind it.

### Module Documentation

Each module should begin with a multi-line comment block that:

```borf
--[[
  Module Name - One-line description of primary purpose

  This module provides tools for [functionality], letting you:

  - Capability 1 with concrete benefit
  - Capability 2 with concrete benefit
  - Capability 3 with concrete benefit
  - Capability 4 with concrete benefit

  Additional context about when and how to use this module.
]]--
```

### Function Documentation

Each significant function should include a comment block that:

```borf
--[[
  function_name - Brief description of what it does

  Explains the practical use case for this function.
  Focus on when and why a user would choose this function.

  Example: argument1 argument2 function_name
]]--
```

### Comment Style

1. **Practical Focus**: Explain what the code accomplishes for the user rather than implementation theory
2. **Examples**: Include code examples demonstrating proper usage
3. **Consistent Format**: Use Lua-style comments (`--` for single line, `--[[ ]]--` for multi-line)
4. **Contextual Comments**: Add inline comments at key decision points or complex operations
5. **Sectioning**: Use comment separators to create logical sections in longer files:

```borf
--------------------------------------------------
-- SECTION NAME
--------------------------------------------------
```

### Documentation Patterns

The most effective modules follow this pattern:

1. **Introduction**: Clear explanation of purpose and capabilities
2. **Type/Function Reference**: Well-organized declarations with descriptive comments
3. **Guarantees/Principles**: Explanation of key design principles (if applicable)
4. **Best Practices**: Guidance on proper usage patterns
5. **Practical Examples**: Real-world usage examples showing common patterns

Examples should:
- Demonstrate complete workflows, not just isolated function calls
- Include proper error handling
- Show composition of multiple operations
- Use the pipeline operator to make data flow clear
- Include explanatory comments for each step

### Explaining Concepts in Practical Terms

When documenting abstract or theoretical concepts (like algebraic laws, category theory principles, or mathematical properties), always provide both the formal mathematical definition AND a practical, real-world explanation of benefits:

```borf
structure: {
  -- Maps to Category with additional monoidal structure
  obj: !Pipeline
  hom: !Transformer
  id: [x -> x]
  comp: >>>
  tensor: |||
  
  -- LAWS
  -- Each law explained with practical benefits:
  
  associativity: (f >>> g) >>> h veq f >>> (g >>> h)
    -- "You can group pipeline steps however you want without changing the result"
    -- This means you can refactor complex pipelines into logical groups without breaking functionality
    -- Example: (parse >>> validate) >>> transform is the same as parse >>> (validate >>> transform)
  
  left_identity: id >>> f veq f
    -- "A 'do-nothing' transformation at the start won't affect your pipeline"
    -- This lets you conditionally add processing steps without changing pipeline structure
    -- Example: identity >>> transform is the same as transform
}
```

This "best of both worlds" approach serves two audiences:

1. **For mathematical/theoretical users**: Preserves formal rigor with precise definitions, laws, and notation
2. **For practical users**: Provides immediate understanding of real-world benefits without requiring mathematical background

For each abstract concept, provide:

1. **Formal Definition**: State the mathematical law or property precisely
2. **Plain Language Explanation**: Describe what the property means in everyday terms
3. **Practical Benefits**: Explain why this property makes the programmer's life easier
4. **Concrete Example**: Show a code example demonstrating the practical application

**IMPORTANT: Character Restrictions**

- Use only 7-bit ASCII characters in actual code (outside of comments and strings)
- For mathematical formulas in code definitions, use ASCII alternatives:
  - Use `.` instead of `∘` for composition
  - Use `=>` instead of `⇒` for implications
  - Use `*` instead of `×` for products
  - Use `a`, `b`, `c` instead of Greek letters like `α`, `β`, `γ`
- You may use extended Unicode characters within comments or documentation blocks
- When expressing mathematical concepts in comments, consider providing both:
  ```borf
  -- Mathematical: α_B ∘ F(f) = G(f) ∘ α_A  (naturality)
  -- ASCII version: a_B . F(f) = G(f) . a_A
  ```

### Documenting Laws/Guarantees

When documenting mathematical laws that function as guarantees or invariants, use the following approach:

1. Keep formal laws in a `guarantees` or `laws` section of the structure
2. Name laws clearly (e.g., associativity, identity, commutativity)
3. Include both the formal definition AND practical guidance:

```borf
guarantees: {
  -- Associativity: (f >>> g) >>> h veq f >>> (g >>> h)
  associativity: forall [f g h ->
    (f >>> g) >>> h veq f >>> (g >>> h)
  ]
    -- "Function composition order doesn't matter, only the sequence"
    -- Benefit: Lets you refactor complex function chains without changing behavior
    -- To ensure: Keep functions pure (no side effects) and avoid depending on evaluation order
  ,

  -- Identity: id >>> f veq f and f >>> id veq f
  identity: forall [f ->
    id >>> f veq f and f >>> id veq f
  ]
    -- "Identity functions don't change your data"
    -- Benefit: Makes conditional application of functions safe
    -- To ensure: Make sure identity functions truly return their input unchanged
}
```

For each law/guarantee, include:

1. **Formal Definition**: Keep the precise mathematical formulation
2. **Plain Language Translation**: Explain what the law means in everyday terms
3. **Practical Benefit**: Articulate clearly why this property is useful to programmers
4. **Usage Requirements**: If the user must enforce a constraint or invariant to get this benefit, explain how

This approach helps both:
- Mathematical users who need formal specifications
- Practical users who need to understand concrete benefits and requirements

### Providing Applied Examples

For theoretical modules, include a comprehensive examples section that shows the practical application of the theoretical concepts:

```borf
example: {
  --[[
    Example 1: Real-World Use Case Title
    
    One-line description of what this example demonstrates
  ]]--
  real_world_example: InputType -> OutputType [input ->
    -- 1. Setup and initialization
    -- Show how to initialize the theoretical structures in a real context
    
    -- 2. Practical application
    -- Demonstrate how the mathematical properties enable practical benefits
    
    -- 3. Results and benefits
    -- Show concrete outcomes that users care about
  ],
  
  --[[
    Example 2: Another Applied Example
    
    Shows a different use case for the same concepts
  ]]--
  another_example: OtherInput -> OtherOutput [data ->
    -- ...implementation...
  ]
}
```

When creating examples for theoretical modules:

1. **Choose Relatable Domains**: Use examples from familiar domains like data processing, configuration management, or text handling
2. **Highlight Specific Benefits**: Each example should demonstrate a specific practical benefit of the theoretical concept
3. **Show Real-World Problems**: Address problems users actually encounter, not artificial academic examples
4. **Connect Theory to Practice**: Explicitly point out where theoretical properties are enabling practical benefits
5. **Include Multiple Examples**: Provide at least 2-3 different examples to show versatility

Good examples demonstrate how these properties make code:
- More reliable (fewer bugs)
- More maintainable (easier to change)
- More reusable (more general purpose)
- More efficient (better performance)

This approach makes abstract concepts accessible to all developers while preserving mathematical rigor. Users don't need to understand the mathematics to get the benefits, but those who do understand the math can see how the theoretical foundations inform the design.

## Common Patterns

### Data Transformation Pipeline

Data transformation pipelines are a core pattern in Borf, especially for generators, streams, and collections. Always use the pipeline operator to make these flows explicit:

```borf
input_data |>
  [data -> data first_transformation] |>
  [data -> data second_transformation] |>
  [data -> data third_transformation] |>
  final_operation
```

For generator operations (map, filter, reduce, etc.), always chain with pipelines:

```borf
-- Stream/generator processing with pipeline
source |>
  [s -> s [item -> item condition] filter] |>
  [s -> s [item -> item transform] map] |>
  [s -> s count limit] |>
  collect
```

When processing needs intermediate variables for clarity:

```borf
source |>
  first_operation |> [intermediate ->
    intermediate |>
      next_operation |>
      final_operation
  ]
```

### Function Definition with Parameters

```borf
my_function: a * b -> c [x, y ->
  x y + -- Function body
]
```

### Conditionals 

```borf
-- Basic conditional with if/else branches
then_expression iff condition or_else else_expression

-- Conditional with blocks
{
  -- Multiple expressions in then branch
  first_expression
  second_expression
} iff condition or_else {
  -- Multiple expressions in else branch
  alternative_expression
}
```

### Pattern Matching

```borf
value {
  | {type: "Success", data: data} -> data process_data
  | {type: "Error", message: msg} -> msg log_error
} match
```

### Recursion

```borf
-- Named recursive function
[args -> 
  -- Base case
  { base_case_result } iff is_base_case args or_else {
    -- Recursive case
    args transform recursive_fn
  }
] -> recursive_fn

-- Alternative with explicit recursion
[i, max, should_continue -> 
  {
    i 1 + -> i
    i max should_continue recursive_func
  } iff should_continue or_else i
] -> recursive_func
```