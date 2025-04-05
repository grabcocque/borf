use crossbeam_utils::sync::ShardedLock;
use petgraph::dot::{Config, Dot};
use petgraph::graph::{DiGraph, NodeIndex};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{debug, info, span, trace, warn, Level};

static NEXT_PARSE_ID: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone, Serialize)]
pub struct RuleStats {
    pub calls: usize,
    pub successes: usize,
    pub failures: usize,
    pub total_time: Duration,
    pub avg_time: Duration,
    pub max_time: Duration,
    pub min_time: Duration,
}

#[derive(Debug, Clone, Serialize)]
pub struct ParseTreeNode {
    pub rule: String,
    pub span: (usize, usize),
    pub text: String,
    pub children: Vec<ParseTreeNode>,
    pub duration: Duration,
    pub success: bool,
}

#[derive(Debug)]
pub struct ParserObserver {
    parse_id: usize,
    source_name: String,
    source_text: String,
    rule_stats: ShardedLock<HashMap<String, RuleStats>>,
    parse_tree: Mutex<Option<ParseTreeNode>>,
    #[allow(dead_code)]
    active_nodes: ShardedLock<HashMap<usize, Vec<ParseTreeNode>>>, // depth -> nodes at that depth
    parent_stack: ShardedLock<Vec<(usize, String)>>, // stack of (depth, rule_name)
    node_stack: Mutex<Vec<ParseTreeNode>>,           // Stack of nodes being built
    start_time: Instant,
    current_depth: AtomicUsize,
    max_depth: AtomicUsize,
}

impl ParserObserver {
    pub fn new(source_name: &str, source_text: &str) -> Arc<Self> {
        let parse_id = NEXT_PARSE_ID.fetch_add(1, Ordering::SeqCst);

        let observer = Arc::new(Self {
            parse_id,
            source_name: source_name.to_string(),
            source_text: source_text.to_string(),
            rule_stats: ShardedLock::new(HashMap::new()),
            parse_tree: Mutex::new(None),
            active_nodes: ShardedLock::new(HashMap::new()),
            parent_stack: ShardedLock::new(Vec::new()),
            node_stack: Mutex::new(Vec::new()),
            start_time: Instant::now(),
            current_depth: AtomicUsize::new(0),
            max_depth: AtomicUsize::new(0),
        });

        info!(
            parse_id = parse_id,
            source = source_name,
            "Starting parsing operation"
        );

        observer
    }

    pub fn enter_rule(&self, rule_name: &str, input_pos: usize) -> RuleTracer {
        let current = self.current_depth.fetch_add(1, Ordering::SeqCst) + 1;
        let mut max = self.max_depth.load(Ordering::SeqCst);

        // Update max depth if needed
        while current > max {
            match self
                .max_depth
                .compare_exchange(max, current, Ordering::SeqCst, Ordering::SeqCst)
            {
                Ok(_) => break,
                Err(actual) => max = actual,
            }
        }

        // Push this rule onto the parent stack
        if let Ok(mut parent_stack) = self.parent_stack.write() {
            parent_stack.push((current, rule_name.to_string()));
        }

        // Create a placeholder node in the node stack
        if let Ok(mut node_stack) = self.node_stack.lock() {
            // Create placeholder text - will be updated when we know the end position
            let placeholder_node = ParseTreeNode {
                rule: rule_name.to_string(),
                span: (input_pos, input_pos), // Will be updated on success
                text: String::new(),          // Will be updated on success
                children: Vec::new(),
                duration: Duration::default(), // Will be updated on success/failure
                success: false,                // Will be updated on success
            };
            node_stack.push(placeholder_node);
        }

        let span = span!(
            Level::TRACE,
            "rule",
            name = rule_name,
            pos = input_pos,
            depth = current
        );
        let _guard = span.enter();

        trace!(
            rule = rule_name,
            pos = input_pos,
            depth = current,
            "Entering rule"
        );

        // Note: self is already an Arc<ParserObserver> so we use a little unsafe to
        // create a new Arc pointing to the same object (increasing refcount)
        let self_ptr = self as *const ParserObserver;
        let arc_self = unsafe { Arc::from_raw(self_ptr) };
        let observer_clone = Arc::clone(&arc_self);
        // Don't drop this Arc since we don't really own its allocation
        std::mem::forget(arc_self);

        RuleTracer {
            observer: observer_clone,
            rule_name: rule_name.to_string(),
            start_pos: input_pos,
            start_time: Instant::now(),
            depth: current,
        }
    }

    pub fn generate_report(&self) -> String {
        let elapsed = self.start_time.elapsed();
        let stats_result = self.rule_stats.read();
        // Handle potential poison error gracefully
        let stats = match stats_result {
            Ok(s) => s,
            Err(poisoned) => {
                warn!("Rule stats lock poisoned! Using recovered data.");
                poisoned.into_inner()
            }
        };

        let mut report = String::new();
        report.push_str(&format!(
            "Parse #{} of '{}' completed in {:?}\n",
            self.parse_id, self.source_name, elapsed
        ));
        report.push_str(&format!(
            "Max recursion depth: {}\n\n",
            self.max_depth.load(Ordering::SeqCst)
        ));

        report.push_str("Rule Statistics:\n");

        let mut stats_vec: Vec<_> = stats.iter().collect();
        stats_vec.sort_by(|a, b| b.1.calls.cmp(&a.1.calls));

        for (rule, stat) in stats_vec {
            report.push_str(&format!(
                "  {}: {} calls ({} successful, {} failed)\n",
                rule, stat.calls, stat.successes, stat.failures
            ));
            report.push_str(&format!(
                "    Avg time: {:?}, Max: {:?}, Min: {:?}\n",
                stat.avg_time, stat.max_time, stat.min_time
            ));
        }

        // Add parse tree if available
        if let Ok(tree_guard) = self.parse_tree.lock() {
            if let Some(tree) = tree_guard.as_ref() {
                report.push_str("\nParse Tree:\n");
                self.append_tree(&mut report, tree, 0);
            }
        }

        report
    }

    #[allow(clippy::only_used_in_recursion)] // Recursion is inherent here
    fn append_tree(&self, report: &mut String, node: &ParseTreeNode, indent: usize) {
        let status = if node.success { "✓" } else { "✗" };
        let prefix = "  ".repeat(indent);

        report.push_str(&format!(
            "{}{} {} at {:?}: '{}' ({:?})\n",
            prefix,
            status,
            node.rule,
            node.span,
            node.text.replace('\n', "\\n"),
            node.duration
        ));

        for child in &node.children {
            self.append_tree(report, child, indent + 1);
        }
    }

    pub fn export_parse_tree_dot(&self) -> Option<String> {
        let tree_guard = self.parse_tree.lock().ok()?;
        let root = tree_guard.as_ref()?;

        let mut graph = DiGraph::new();
        let mut node_map = HashMap::new();

        // Helper function to build the graph recursively
        fn build_graph(
            graph: &mut DiGraph<String, ()>,
            node_map: &mut HashMap<String, NodeIndex>,
            node: &ParseTreeNode,
            parent_idx: Option<NodeIndex>,
            id_counter: &mut usize,
        ) -> NodeIndex {
            let node_id = format!("{}_{}", node.rule, *id_counter);
            *id_counter += 1;

            // Create a nice-looking label with rule name, span info, and abbreviated text
            // Show success/failure status and make text more readable
            let status = if node.success { "✓" } else { "✗" };
            let truncated_text = node
                .text
                .replace('\n', "\\n")
                .chars()
                .take(30)
                .collect::<String>();
            let text_display = if truncated_text.len() < node.text.len() {
                format!("{}...", truncated_text)
            } else {
                truncated_text
            };

            let label = format!(
                "{} {}\n{:?}\n'{}'",
                status, node.rule, node.span, text_display
            );

            let idx = graph.add_node(label);
            node_map.insert(node_id.clone(), idx);

            if let Some(parent) = parent_idx {
                graph.add_edge(parent, idx, ());
            }

            for child in &node.children {
                build_graph(graph, node_map, child, Some(idx), id_counter);
            }

            idx
        }

        let mut counter = 0;
        build_graph(&mut graph, &mut node_map, root, None, &mut counter);

        // Configure the DOT output for better visualization
        let dot_config = &[Config::EdgeNoLabel];

        // Set different colors based on success/failure directly in the DOT output
        let mut dot_output = "digraph ParseTree {\n    \
            rankdir=TB;\n    \
            node [shape=box, style=filled, fontname=\"Courier New\"];\n    \
            edge [color=darkblue, penwidth=1.5];\n"
            .to_string();

        // Add node styles for success/failure
        for node_idx in graph.node_indices() {
            let label = &graph[node_idx];
            let fillcolor = if label.starts_with("✓") {
                "lightgreen"
            } else {
                "lightpink"
            };

            // Add node attributes individually
            dot_output.push_str(&format!(
                "    {} [fillcolor=\"{}\"];\n",
                node_idx.index(),
                fillcolor
            ));
        }

        // Add the graph content
        dot_output.push_str(&format!(
            "    {}\n}}",
            format!("{:?}", Dot::with_config(&graph, dot_config))
                .replace("\\\\n", "\\n") // Fix double escaping
                .replace("digraph {", "")
                .replace("}", "")
                .trim()
        ));

        Some(dot_output)
    }

    pub fn register_rule_result(
        &self,
        rule_name: &str,
        start_pos: usize,
        end_pos: Option<usize>,
        duration: Duration,
        depth: usize,
    ) {
        let success = end_pos.is_some();
        let end = end_pos.unwrap_or(start_pos);

        // Update rule stats
        {
            // Use write().ok() or handle poisoned lock
            if let Ok(mut stats) = self.rule_stats.write() {
                let rule_stat = stats.entry(rule_name.to_string()).or_insert(RuleStats {
                    calls: 0,
                    successes: 0,
                    failures: 0,
                    total_time: Duration::default(),
                    avg_time: Duration::default(),
                    max_time: Duration::default(),
                    min_time: Duration::from_secs(999_999),
                });

                rule_stat.calls += 1;
                if success {
                    rule_stat.successes += 1;
                } else {
                    rule_stat.failures += 1;
                }

                rule_stat.total_time += duration;
                // Prevent division by zero if calls somehow becomes 0 before increments
                if rule_stat.calls > 0 {
                    rule_stat.avg_time = rule_stat.total_time / rule_stat.calls as u32;
                } else {
                    rule_stat.avg_time = Duration::default();
                }

                if duration > rule_stat.max_time {
                    rule_stat.max_time = duration;
                }

                // Ensure min_time is only updated if it's not the default large value or duration is smaller
                if duration < rule_stat.min_time {
                    rule_stat.min_time = duration;
                }
            } else {
                warn!(
                    rule = rule_name,
                    "Failed to acquire write lock for rule stats (poisoned?)"
                );
            }
        }

        // Pop the rule from the parent stack
        if let Ok(mut parent_stack) = self.parent_stack.write() {
            if let Some(popped) = parent_stack.pop() {
                if popped.0 != depth || popped.1 != rule_name {
                    // Mismatched pop, something's wrong with our tracking
                    warn!(
                        "Mismatched rule on parent stack: expected ({}, {}), got ({}, {})",
                        depth, rule_name, popped.0, popped.1
                    );
                    // Re-push the item we just popped
                    parent_stack.push(popped);
                }
            }
        }

        // Pop the node from the node stack and update it
        if let Ok(mut node_stack) = self.node_stack.lock() {
            if let Some(mut node) = node_stack.pop() {
                // Create the text for this rule
                let text = if start_pos < self.source_text.len() && end <= self.source_text.len() {
                    self.source_text[start_pos..end].to_string()
                } else {
                    "[invalid range]".to_string()
                };

                // Update the node with the actual values
                node.span = (start_pos, end);
                node.text = text;
                node.duration = duration;
                node.success = success;

                // If this is the top-level rule, store it as the root
                if node_stack.is_empty() {
                    if let Ok(mut tree) = self.parse_tree.lock() {
                        *tree = Some(node);
                    }
                } else if success {
                    // Add this node as a child to its parent
                    let parent_idx = node_stack.len() - 1;
                    node_stack[parent_idx].children.push(node);
                }
            } else {
                warn!("Node stack underflow when processing rule: {}", rule_name);
            }
        }

        // Decrement depth counter
        // Make sure depth doesn't underflow (though fetch_sub wraps on unsigned integers)
        if self.current_depth.load(Ordering::SeqCst) > 0 {
            self.current_depth.fetch_sub(1, Ordering::SeqCst);
        }
    }

    /// Returns a clone of the current rule statistics.
    pub fn get_stats(&self) -> HashMap<String, RuleStats> {
        // Handle potential poison error
        self.rule_stats.read().map_or_else(
            |poisoned| {
                warn!("Rule stats lock poisoned on read! Using recovered data.");
                poisoned.into_inner().clone()
            },
            |guard| guard.clone(),
        )
    }

    /// Creates a sample parse tree for demonstration purposes
    pub fn create_demo_parse_tree(&self, source_text: &str) -> Option<String> {
        // Create a simple demo parse tree
        let mut root = ParseTreeNode {
            rule: "file".to_string(),
            span: (0, source_text.len()),
            text: source_text.to_string(),
            children: Vec::new(),
            duration: Duration::from_millis(100),
            success: true,
        };

        // Module declaration node
        let mut module_decl = ParseTreeNode {
            rule: "module_decl".to_string(),
            span: (0, source_text.len()),
            text: source_text.to_string(),
            children: Vec::new(),
            duration: Duration::from_millis(50),
            success: true,
        };

        // Add some children to the module declaration
        let module_name = ParseTreeNode {
            rule: "identifier".to_string(),
            span: (1, 11), // Approximate position for "TestModule"
            text: "TestModule".to_string(),
            children: Vec::new(),
            duration: Duration::from_millis(5),
            success: true,
        };

        let mut module_body = ParseTreeNode {
            rule: "module_body".to_string(),
            span: (14, source_text.len() - 1), // Inside the braces
            text: source_text[14..source_text.len() - 1].to_string(),
            children: Vec::new(),
            duration: Duration::from_millis(40),
            success: true,
        };

        // Add a declaration to the module body
        let entity_decl = ParseTreeNode {
            rule: "entity_decl".to_string(),
            span: (20, 35), // Approximate position for a declaration
            text: "val: Int = 42".to_string(),
            children: Vec::new(),
            duration: Duration::from_millis(10),
            success: true,
        };

        // Build the tree structure
        module_body.children.push(entity_decl);
        module_decl.children.push(module_name);
        module_decl.children.push(module_body);
        root.children.push(module_decl);

        // Store as the parse tree
        if let Ok(mut tree) = self.parse_tree.lock() {
            *tree = Some(root);
        }

        // Export to DOT format
        self.export_parse_tree_dot()
    }
}

/// Raw tracer struct for rule parsing operations
pub struct RuleTracer {
    observer: Arc<ParserObserver>,
    rule_name: String,
    start_pos: usize,
    start_time: Instant,
    depth: usize,
}

impl RuleTracer {
    pub fn success(self, end_pos: usize) {
        let duration = self.start_time.elapsed();
        trace!(
            rule = self.rule_name.as_str(),
            start_pos = self.start_pos,
            end_pos = end_pos,
            duration_us = duration.as_micros() as u64,
            "Rule matched successfully"
        );

        // Check if observer is still valid before calling
        // Use a local clone for registration to avoid dropping self early
        let obs = &self.observer;
        obs.register_rule_result(
            &self.rule_name,
            self.start_pos,
            Some(end_pos),
            duration,
            self.depth,
        );
        // Prevent drop from running by forgetting self
        std::mem::forget(self);
    }
}

impl Drop for RuleTracer {
    fn drop(&mut self) {
        // No need to check start_time elapsed, drop only runs if not forgotten
        // Check strong_count to avoid panic during unwind if observer is gone
        if Arc::strong_count(&self.observer) > 0 {
            let duration = self.start_time.elapsed();
            trace!(
                rule = self.rule_name.as_str(),
                start_pos = self.start_pos,
                duration_us = duration.as_micros() as u64,
                "Rule failed to match"
            );

            self.observer.register_rule_result(
                &self.rule_name,
                self.start_pos,
                None,
                duration,
                self.depth,
            );
        } else {
            // Log if observer is gone during drop?
            // This might happen during panics/unwinding
            // Use debug or trace level to avoid noise
            debug!(
                "Observer already dropped when RuleTracer failed for rule: {}",
                self.rule_name
            );
        }
    }
}
