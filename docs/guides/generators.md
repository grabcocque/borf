# Process Infinite Data Streams Without Crashing Your App

## The Problem

Ever tried to:
- Process a massive file without loading it all into memory?
- Work with streams of data that never end (like user events or sensor readings)?
- Generate sequences that would be inefficient to calculate all at once?
- Make your data processing code more readable and maintainable?

Traditional solutions either use too much memory, perform poorly, or make your code a mess of callback functions.

## The Borf Solution: Generators

Generators are **on-demand** data producers that only calculate values when you need them. They're perfect for:

```borf
// Create an infinite stream of numbers
let numbers = Generator.range(1, null, 1)  // Will never run out of memory!

// Process only what you need
let first_ten_squares = numbers
  |> [g -> Generator.map(g, [n -> n * n])]
  |> [g -> Generator.take(g, 10)]
  |> Generator.collect

// first_ten_squares = [1, 4, 9, 16, 25, 36, 49, 64, 81, 100]
```

## Real-World Examples

### 1. Efficient CSV Processing

```borf
// Process a huge CSV file with minimal memory usage
let process_big_csv = [filename ->
  // Create a line-by-line generator from the file
  let lines = file_to_lines(filename)

  // Skip the header
  let data = Generator.drop(lines, 1)

  // Parse and transform each line
  let results = data
    |> [g -> Generator.map(g, parse_csv_line)]
    |> [g -> Generator.filter(g, is_valid_record)]
    |> [g -> Generator.map(g, transform_record)]

  // Process each record one at a time
  Generator.for_each(results, save_to_database)
]

// The entire file is never loaded into memory at once!
process_big_csv("massive_data.csv")  // Works with terabytes of data
```

### 2. Realtime Data Dashboard

```borf
// Create a realtime sensor data dashboard
let sensor_dashboard = [->
  // Create an infinite generator of sensor readings
  let temperature_readings = sensor_generator("temperature")

  // Calculate rolling average
  let rolling_avg = rolling_average_generator(temperature_readings, 10)

  // Set up UI updates
  Generator.for_each(
    rolling_avg,
    [avg ->
      update_chart(avg)
      update_gauge(avg)
      check_alerts(avg)
    ]
  )
]

// Never blocks the UI, processes data as it arrives
start_dashboard()
```

### 3. Fibonacci Sequence Generator

```borf
// Calculate Fibonacci numbers on demand
let fibonacci = Generator.generator(
  [state ->
    {
      _type: "yielded",
      value: state.current,
      next_state: {
        current: state.next,
        next: state.current + state.next
      }
    }
  ],
  { current: 0, next: 1 }
)

// Get the 100th Fibonacci number efficiently
let hundredth = Generator.drop(fibonacci, 99)
  |> [g -> Generator.take(g, 1)]
  |> Generator.collect
  |> [arr -> arr[0]]

// No need to calculate all previous numbers in advance
```

## Why Generators Are Better

✅ **Memory Efficient** - Process infinite streams with constant memory
✅ **Lazy Evaluation** - Only calculate what you actually use
✅ **Composable** - Build complex data pipelines with simple parts
✅ **Readable** - Makes data transformation logic clear and linear
✅ **Flexible** - Works with any data source or transformation

## Quick Reference

### Creating Generators

```borf
// From a range
let numbers = Generator.range(1, 100, 1)

// From an array
let items = Generator.from_array(["a", "b", "c"])

// Custom generator
let random_numbers = Generator.generator(
  [_ -> {
    _type: "yielded",
    value: Prim.random_int(1, 100),
    next_state: null
  }],
  null
)

// Repeating values
let repeated = Generator.repeat("hello", 5)

// Cycling through values
let cycles = Generator.cycle([1, 2, 3])
```

### Transforming Generators

```borf
// Map values
let doubled = Generator.map(numbers, [n -> n * 2])

// Filter values
let evens = Generator.filter(numbers, [n -> n % 2 == 0])

// Limit values
let first_five = Generator.take(numbers, 5)

// Skip values
let after_ten = Generator.drop(numbers, 10)

// Combine generators
let pairs = Generator.zip(letters, numbers)
```

### Consuming Generators

```borf
// Get all values (for finite generators)
let all_values = Generator.collect(limited_gen)

// Process each value
Generator.for_each(numbers, print)

// Reduce to a single value
let sum = Generator.reduce(numbers, [acc n -> acc + n], 0)

// Find a specific value
let first_big = Generator.find(numbers, [n -> n > 100])

// Check conditions
let has_even = Generator.any(numbers, [n -> n % 2 == 0])
let all_positive = Generator.all(numbers, [n -> n > 0])
```

## Try it today!

Ready to make your data processing code more efficient and maintainable? Start using generators in your next Borf project!

```borf
import { Generator } from "@borf/prelude"

// Your efficient data processing starts here
```
