# Rewriting to Idiomatic Borf

This guide helps maintain consistent, idiomatic Borf code throughout the prelude and other modules.

## Core Principles of Idiomatic Borf

1. **Concatenative Style**: Values come before operations (`x y f` not `f(x, y)`)
2. **Named Parameters**: Use `[x, y -> ...]` with commas for quotations with named parameters
3. **Pipeline Operator**: Use `|>` for explicit data flow and readability when chaining operations
4. **Stack-Based Thinking**: Consider data flow in terms of stack operations
5. **Explicit Data Flow**: Make transformations clear and follow the data through your program

## Common Transformations

### 1. Function Application

**Non-idiomatic (applicative):**
```
f(x, y)
Map.insert(m, k, v)
```

**Idiomatic (concatenative):**
```
x y f
m k v Map.insert
```

### 2. Lambda/Quotation Expressions

**Non-idiomatic (applicative lambda):**
```
[x y -> x + y]
[f g -> [x -> g(f(x))]]
```

**Idiomatic (concatenative quotation):**
```
[x, y -> x y +]
[f, g -> [x -> x f g]]
```

### 3. Let Expressions

**Non-idiomatic (let-in binding):**
```
let x = f(y) in
x + z
```

**Idiomatic (stack manipulation):**
```
y f -> x
x z +
```

Or using pipeline:
```
y f |> [x -> x z +]
```

### 4. Conditionals

**Non-idiomatic (applicative if):**
```
cond ? then_val : else_val
x iff cond or_else y
```

**Idiomatic (concatenative if):**
```
then_val else_val cond [t, e, c -> c t e if]
then_val else_val cond if
```

### 5. Complex Expressions

**Non-idiomatic:**
```
f(g(x), h(y))
```

**Idiomatic:**
```
x g y h f
```

Or with pipeline for clarity:
```
x |> [x -> 
  y |> [y -> 
    x y f
  ]
]
```

## Using the Pipeline Operator

The pipeline operator (`|>`) is essential for maintaining readability in Borf code. It makes data flow explicit by showing how values move through transformations.

**Non-idiomatic:**
```
h(g(f(x)))
```

**Idiomatic:**
```
x |> f |> g |> h
```

This is especially valuable for complex transformations:

```
input_data |> 
  [data -> data normalize] |> 
  [data -> data [item -> item is_valid] filter] |> 
  [filtered -> filtered process]
```

### Pipeline Operator Benefits

1. **Explicit Data Flow**: Makes it obvious how data flows through transformations
2. **Readability**: Creates a clear top-to-bottom or left-to-right reading pattern
3. **Modularity**: Allows easy addition or removal of processing steps
4. **Debuggability**: Makes it easier to isolate and test individual transformation steps

### When to Use Pipeline Operators

**ALWAYS use pipeline operators when:**
- Chaining multiple operations
- Creating data processing pipelines
- Working with generator/stream transformations
- Building sequences of data transformations
- Creating complex processing workflows

## Error Handling

**Non-idiomatic:**
```
try {
  risky_operation()
} catch (e) {
  handle_error(e)
}
```

**Idiomatic:**
```
[-> risky_operation] 
[e -> e handle_error] 
try_catch
```

## Examples

### Example 1: Map Operation

**Non-idiomatic:**
```
map([1, 2, 3], [x -> x * 2])
```

**Idiomatic:**
```
[1, 2, 3] [x -> x 2 *] map
```

### Example 2: Filter Operation

**Non-idiomatic:**
```
filter([1, 2, 3, 4], [x -> x % 2 == 0])
```

**Idiomatic:**
```
[1, 2, 3, 4] [x -> x 2 % 0 ==] filter
```

### Example 3: Complex Transformation

**Non-idiomatic:**
```
let data = get_data() in
let filtered = filter(data, [x -> is_valid(x)]) in
let transformed = map(filtered, [x -> transform(x)]) in
process_result(transformed)
```

**Idiomatic with intermediate variables:**
```
get_data |> [data ->
  data [x -> x is_valid] filter |> [filtered ->
    filtered [x -> x transform] map |> [result ->
      result process_result
    ]
  ]
]
```

**Idiomatic with pipeline flow:**
```
get_data |>
  [data -> data [x -> x is_valid] filter] |>
  [filtered -> filtered [x -> x transform] map] |>
  [result -> result process_result]
```

**Most idiomatic (clean pipeline):**
```
get_data |>
  [data -> data [x -> x is_valid] filter] |>
  [data -> data [x -> x transform] map] |>
  process_result
```

## Common Idioms and Best Practices

1. **Data Flow**: Think of operations as transformations on a data stack
2. **Use Pipeline Operators**: Always use `|>` when chaining operations for explicit data flow
3. **Parameter Naming**: Use descriptive parameter names in quotations with commas between them
4. **Consistent Indentation**: Align pipeline stages for readability
5. **Intermediate Variables**: Use named variables when it improves clarity
6. **Pipeline Stages**: Each pipeline stage should do one clear transformation
7. **Quotation Parameters**: Include commas between parameters: `[x, y -> ...]` not `[x y -> ...]`
8. **Terminal Operations**: End pipelines with a clear terminal operation that produces output
9. **Point-Free Style**: When appropriate, compose functions without naming intermediate results
10. **Generator Pipelines**: Always use pipelines when working with generators/streams