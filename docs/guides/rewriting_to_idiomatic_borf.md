# Rewriting to Idiomatic Borf

This guide helps maintain consistent, idiomatic Borf code throughout the prelude and other modules.

## Core Principles of Idiomatic Borf

1. **Concatenative Style**: Values come before operations
2. **Named Parameters**: Use `[x, y -> ...]` for quotations with named parameters
3. **Pipeline Operator**: Use `|>` for readability when composing operations
4. **Stack-Based Thinking**: Consider data flow in terms of stack operations

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

The pipeline operator (`|>`) is particularly useful for maintaining readability in concatenative code:

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
data 
|> [data -> data transform1] 
|> [result -> result transform2]
```

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

**Idiomatic:**
```
get_data |> [data ->
  data [x -> x is_valid] filter |> [filtered ->
    filtered [x -> x transform] map |> [result ->
      result process_result
    ]
  ]
]
```

Or more concisely:
```
get_data
[x -> x is_valid] filter
[x -> x transform] map
process_result
```

## Common Idioms

1. **Data Flow**: Think of operations as transformations on a data stack
2. **Point-Free Style**: When possible, compose functions without naming parameters
3. **Parameter Naming**: Use descriptive parameter names in quotations
4. **Pipeline for Clarity**: Use `|>` when direct concatenation becomes unclear