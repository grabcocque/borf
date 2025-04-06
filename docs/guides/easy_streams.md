# Simple Data Processing with Streams

## What are Streams?

Streams are a super-easy way to work with data sequences in Borf. They let you:

- Process data one piece at a time (saving memory)
- Create clear data processing pipelines
- Deal with infinite data sources
- Write cleaner, more maintainable code

## Choose Your Style: Basic vs Fluent

Borf offers two ways to work with streams, depending on your style preference.

### Basic Style (Generator Module)
For those who like a straightforward, functional approach:

```borf
import { Generator } from "@borf/prelude"

// Create a range from 1 to 100
let numbers = Generator.range(1, 101, 1)

// Process the numbers
let result = numbers
  |> [g -> Generator.filter(g, [n -> n % 2 == 0])]
  |> [g -> Generator.map(g, [n -> n * 3])]
  |> [g -> Generator.take(g, 5)]
  |> Generator.collect

// result = [6, 12, 18, 24, 30]
```

### Fluent Style (Stream Module)
For those who prefer a more chainable, method-style syntax:

```borf
import { Stream } from "@borf/prelude"

// Create a range from 1 to 100
let numbers = Stream.from_range(1, 101, 1)

// Process the numbers
let result = numbers
  |> [s -> Stream.filter(s, [n -> n % 2 == 0])]
  |> [s -> Stream.map(s, [n -> n * 3])]
  |> [s -> Stream.take(s, 5)]
  |> Stream.collect

// result = [6, 12, 18, 24, 30]
```

Both approaches get the same job done! Pick the style that feels most natural to you.

## Real-World Examples

### Processing User Data

```borf
let process_users = [user_ids ->
  Stream.from_array(user_ids)
    |> [s -> Stream.map(s, fetch_user_data)]       // Get user details
    |> [s -> Stream.filter(s, is_active_account)]  // Only process active accounts
    |> [s -> Stream.map(s, calculate_metrics)]     // Calculate user metrics
    |> [s -> Stream.for_each(s, update_dashboard)] // Update the dashboard
]

// Process users in batches without memory concerns
process_users(get_all_user_ids())
```

### Reading Large Log Files

```borf
let analyze_logs = [log_file ->
  let lines = file_to_stream(log_file)
  
  // Extract error messages from logs
  let errors = lines
    |> [s -> Stream.filter(s, contains_error)]
    |> [s -> Stream.map(s, parse_error_message)]
    |> [s -> Stream.map(s, add_timestamp)]
  
  // Count error types
  let error_counts = errors
    |> [s -> Stream.map(s, [err -> err.type])]
    |> [s -> Stream.reduce(s, [counts type ->
         counts[type] = (counts[type] || 0) + 1
         counts
       ], {})]
  
  error_counts
]

// Works with log files of any size!
let error_report = analyze_logs("system.log")
```

### Data Transformation Pipeline

```borf
let transform_data = [raw_data ->
  Stream.from_array(raw_data)
    |> [s -> Stream.map(s, normalize)]      // Normalize values
    |> [s -> Stream.filter(s, validate)]    // Remove invalid entries
    |> [s -> Stream.map(s, enrich)]         // Add computed properties
    |> [s -> Stream.map(s, format)]         // Format for output
    |> Stream.collect                       // Gather results
]

// The resulting code looks exactly like the steps in your process!
```

## When to Use Streams

Streams are perfect when you need to:

✅ Process large datasets efficiently  
✅ Create clear data transformation pipelines  
✅ Deal with potentially infinite data sources  
✅ Improve code readability  
✅ Avoid loading everything into memory  

## Quick Stream Cheat Sheet

### Creating Streams

```borf
// From arrays
let items = Stream.from_array([1, 2, 3, 4, 5])

// From ranges
let numbers = Stream.from_range(1, 100, 1)

// Repeating values
let repeated = Stream.repeat("hello", 5)

// Custom generator
let timestamps = Stream.from_function([_ -> now()], null)

// Cycling through values
let cycle = Stream.cycle(["red", "green", "blue"])
```

### Transforming Streams

```borf
// Map (transform values)
|> [s -> Stream.map(s, [x -> x * 2])]

// Filter (keep some values)
|> [s -> Stream.filter(s, [x -> x > 10])]

// Limit results
|> [s -> Stream.take(s, 5)]

// Skip some values
|> [s -> Stream.drop(s, 3)]

// Combine streams
|> [s -> Stream.zip(s, other_stream)]

// Chain streams
|> [s -> Stream.concat(s, next_stream)]
```

### Terminal Operations

```borf
// Get all values
|> Stream.collect

// Reduce to a single value
|> [s -> Stream.reduce(s, [acc x -> acc + x], 0)]

// Process each value
|> [s -> Stream.for_each(s, print)]

// Find specific values
|> [s -> Stream.find(s, [x -> x.id == "abc123"])]

// Check if any match
|> [s -> Stream.any(s, [x -> x < 0])]

// Check if all match
|> [s -> Stream.all(s, [x -> x > 0])]
```

## Get Started!

```borf
import { Stream } from "@borf/prelude"

// Your streamlined data processing starts here
let example = Stream.from_range(1, 10, 1)
  |> [s -> Stream.map(s, [n -> n * n])]
  |> Stream.collect

// Easy, efficient, and clean!
``` 