# Borf Style Guide

This document outlines the idiomatic style for writing Borf code. Borf is a concatenative functional logic programming language that executes code by reducing interaction nets.

## Core Principles

1. Concatenative style: Values come before operations (e.g., `x y f` instead of `f(x, y)`)
2. Quotations for functions: Use `[a, b -> ...]` syntax for function definitions 
3. Named parameters in quotations: Use the assignment arrow `->` to bind parameter names
4. Pipeline operator: Use `|>` for chaining operations and improving readability

## Syntax Conventions

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
// Stream/generator processing with pipeline
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