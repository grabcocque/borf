# Transformation Summary to Idiomatic Borf

This document summarizes the transformations made to convert non-idiomatic code in the prelude files to more idiomatic Borf syntax, focusing particularly on the proper use of pipeline operators (`|>`) versus assignment arrows (`->`).

## Key Transformations

### 1. Function Application Style

Changed applicative to concatenative style:

**Before:**
```borf
f(x, y)
unify(x, y)
Map.lookup(m, k)
```

**After:**
```borf
x y f
x y unify
m k Map.lookup
```

### 2. Lambda/Quotation Parameters

Changed Haskell-like lambdas to Borf quotations with named parameters:

**Before:**
```borf
[x y -> x + y]
[f -> process(f(input))]
```

**After:**
```borf
[x, y -> x y +]
[f -> input f process]
```

### 3. Assignment and Variable Binding

Changed let-in expressions to pipeline operators and arrow assignments:

**Before:**
```borf
let x = f(y) in
  g(x)
```

**After:**
```borf
y f |> x  // Use pipeline operator for data flow
x g
```

Important note: We use `|>` (pipeline operator) for passing data through a transformation, not `->` (assignment arrow).

### 4. Function Composition

Made function composition more concatenative:

**Before:**
```borf
xs |> List.fold [acc (k, v) -> 
  Map.insert(acc, k, v)
] m1
```

**After:**
```borf
xs |> List.fold [acc, (k, v) -> 
  acc k v Map.insert
] m1
```

### 5. Conditionals

Transformed conditional statements:

**Before:**
```borf
condition ? then_val : else_val
```

**After:**
```borf
then_val iff condition or_else else_val
```

### 6. Eliminated Chained Method Calls

Replaced object-oriented style chaining with concatenative flow:

**Before:**
```borf
s1_stream.kind.match({...})
```

**After:**
```borf
s1_stream kind {...} match
```

### 7. Enhanced Pipeline Usage

Better use of the pipeline operator for improved readability:

**Before:**
```borf
f(g(h(x)))
```

**After:**
```borf
x |> h |> g |> f
```

Or:
```borf
x h g f
```

## Specific Examples from Prelude Files

### Example 1: Concatenative Style in Function Definition

**Before:**
```borf
firsto: List * a -> !Goal [xs, x -> caro(xs, x)]
```

**After:**
```borf
firsto: List * a -> !Goal [xs, x -> xs x caro]
```

### Example 2: Function Application with Pipeline Operator

**Before:**
```borf
make_kleisli_cat: T.U -> !KleisliCategory [_ ->
  let state_type = @T.type_of({_type: "State", substitution: {}, var_counter: 0}) in
  let empty_goals = Set.empty() in
  ...
]
```

**After:**
```borf
make_kleisli_cat: T.U -> !KleisliCategory [_ ->
  {_type: "State", substitution: {}, var_counter: 0} |> state_type
  @T state_type type_of |> state_type
  Set.empty |> empty_goals
  ...
]
```

### Example 3: Pipeline Operator in Data Transformations

**Before:**
```borf
run: [s ->
  s g1 run -> s1_stream
  s1_stream kind {...} match
]
```

**After:**
```borf
run: [s ->
  s g1 run |> s1_stream
  s1_stream kind {...} match
]
```

### Example 4: Simplification of Helper Function Definitions

**Before:**
```borf
[x, sorted, res ->
  sorted nullo [x] res unify conj
  [h, t, res1 ->
    /* ... */
  ] fresh
  disj
] -> insert_sorted
```

**After:**
```borf
[x, sorted, res ->
  sorted nullo [x] res unify conj
  [h, t, res1 ->
    /* ... */
  ] fresh
  disj
] |> insert_sorted
```

## Benefits of the Transformation

1. **Consistency**: Code follows a single paradigm throughout
2. **Readability**: Concatenative style is easier to read with the pipeline operator
3. **Execution Model**: Better matches Borf's interaction net reduction model
4. **Paradigm Purity**: Avoids mixing applicative and concatenative styles
5. **Semantic Clarity**: Clear distinction between data flow (`|>`) and variable binding (`->`)
6. **Maintenance**: Easier to maintain code that follows consistent conventions

## Files Modified

The following files were updated to follow idiomatic Borf style:

1. **minikanren.borf** - Logic programming implementation
2. **computation.borf** - Computation structures with effects
3. **dagger.borf** - Dagger categories implementation
4. **profunctor.borf** - Bidirectional programming abstractions
5. **verify.borf** - Verification functions for categorical transformations
6. **unrestricted.borf** - Unrestricted type system implementation

A detailed Borf style guide has been created at `docs/guides/borf_style_guide.md` to document these conventions for future development.