use crate::errors::ParseError;
use crate::observer::ParserObserver;
use crate::parser::Rule;
use crate::traceable_parser::TraceableParser;
use crossbeam_utils::sync::WaitGroup;
use dashmap::DashMap;
use pest::iterators::Pairs;
use pest::Parser;
use rayon::prelude::*;
use std::fmt::Debug;
use std::path::Path;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use std::thread;
use tracing::info;

/// Results are stored in insertion order to maintain determinism
#[derive(Debug)]
pub struct DeterministicParseResults<T: Debug + Send + Sync> {
    results: DashMap<usize, T>,
    #[allow(dead_code)] // Allow dead code for now, might be used later
    next_index: AtomicUsize,
}

impl<T: Debug + Send + Sync> DeterministicParseResults<T> {
    pub fn new() -> Self {
        Self {
            results: DashMap::new(),
            next_index: AtomicUsize::new(0),
        }
    }

    pub fn insert(&self, index: usize, result: T) {
        self.results.insert(index, result);
    }

    pub fn get_ordered_results(self) -> Vec<T> {
        // Find the maximum index present in the map
        let max_index = self
            .results
            .iter()
            .map(|entry| *entry.key())
            .max()
            .unwrap_or(0);
        let mut ordered = Vec::with_capacity(max_index + 1);

        // Iterate from 0 up to max_index and retrieve results
        for i in 0..=max_index {
            if let Some((_, value)) = self.results.remove(&i) {
                ordered.push(value);
            } else {
                // Handle missing index - this shouldn't happen if all jobs complete successfully
                // You might want to log a warning or error here
                eprintln!("Warning: Missing result for index {}", i);
            }
        }
        ordered
    }
}

impl<T: Debug + Send + Sync> Default for DeterministicParseResults<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub fn parse_files_deterministic<P, F, T, PF>(
    paths: &[impl AsRef<Path>],
    parser_factory: F,
    process_pairs: PF,
) -> Vec<Result<T, ParseError>>
where
    P: Parser<Rule>,
    F: Fn() -> P + Sync,
    T: Send + Sync + std::fmt::Debug + 'static,
    PF: Fn(Pairs<Rule>, &str, &str) -> Result<T, ParseError> + Send + Sync + Copy,
{
    let wg = WaitGroup::new();
    let results = Arc::new(DeterministicParseResults::new());

    // Use filter_map directly as suggested by clippy
    let work: Vec<_> = paths
        .iter()
        .enumerate()
        .filter_map(|(idx, path)| {
            let path = path.as_ref().to_path_buf();
            let file_content = match std::fs::read_to_string(&path) {
                Ok(content) => content,
                Err(err) => {
                    let results_clone = Arc::clone(&results);
                    // Store the IO error associated with its index
                    results_clone.insert(idx, Err(ParseError::Io(err)));
                    return None; // Skip this file
                }
            };
            // If read succeeds, return the data for processing
            Some((idx, path, file_content))
        })
        .collect();

    // Second phase: parse in parallel but maintain deterministic output order
    work.into_par_iter().for_each(|(idx, path, content)| {
        let wg_clone = wg.clone();
        let results_clone = Arc::clone(&results);

        let _guard = wg_clone;
        let filename = path.to_string_lossy().to_string();

        // Create observer for this parse operation
        let observer = ParserObserver::new(&filename, &content);

        // Create parser with tracer
        let parser = TraceableParser::with_observer(parser_factory(), observer.clone());

        // Perform the parse
        let result = match parser.traceable_parse(Rule::file, &content) {
            Ok(pairs) => {
                // Process pairs into result type T using the provided function
                process_pairs(pairs, &content, &filename)
            }
            Err(err) => {
                // Convert pest error to your error type
                Err(ParseError::from_pest(
                    *err,
                    &content,
                    Some(filename.clone()),
                ))
            }
        };

        // Log completion with timing stats
        info!(
            file = filename,
            "Parse completed, stats: {}",
            observer.generate_report()
        );

        // Store result in order using the correct index
        results_clone.insert(idx, result);
    });

    wg.wait();
    Arc::try_unwrap(results).unwrap().get_ordered_results()
}

// Example usage with a placeholder parser function
pub fn parse_files_deterministic_result<P, R, T>(
    files: Vec<(usize, String)>,
    parser_fn: P,
) -> DeterministicParseResults<Result<T, R>>
where
    P: Fn(&str) -> Result<T, R> + Sync + Send,
    T: Debug + Send + Sync,
    R: Debug + Send + Sync,
{
    let results = DeterministicParseResults::new();

    files.into_par_iter().for_each(|(index, content)| {
        let parsed = parser_fn(&content);
        results.insert(index, parsed);
    });

    results
}

// Example usage (requires modification to integrate with the actual parser)
#[allow(dead_code)]
fn example_concurrent_parse(file_contents: Vec<String>) -> Vec<Result<String, String>> {
    let parser_fn = |content: &str| -> Result<String, String> {
        // Replace with your actual parsing logic
        // This example just returns the first 10 chars or an error
        if content.len() > 5 {
            Ok(content.chars().take(10).collect())
        } else {
            Err("Input too short".to_string())
        }
    };

    let indexed_files: Vec<(usize, String)> = file_contents.into_iter().enumerate().collect();

    let parse_results = parse_files_deterministic_result(indexed_files, parser_fn);

    // Get the results in the original order
    let ordered_results: Vec<Result<String, String>> = parse_results.get_ordered_results();

    ordered_results
}

// A simple concurrent worker pool (optional, Rayon handles this)
#[allow(dead_code)]
fn run_workers(
    num_workers: usize,
    tasks: Vec<Box<dyn FnOnce() + Send>>,
) -> Result<(), &'static str> {
    if tasks.is_empty() {
        return Ok(());
    }

    let mut handles = vec![];
    let task_queue = crossbeam_channel::unbounded();

    // Send tasks to the queue
    for task in tasks {
        task_queue.0.send(task).map_err(|_| "Failed to send task")?;
    }

    // Drop the sender to signal no more tasks
    drop(task_queue.0);

    for _ in 0..num_workers {
        let receiver = task_queue.1.clone();
        let handle = thread::spawn(move || {
            while let Ok(task) = receiver.recv() {
                task();
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().map_err(|_| "Worker thread panicked")?;
    }

    Ok(())
}
