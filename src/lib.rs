//! # functional-graph
//!
//! Functional graph iteration, cycle detection, tree decomposition, and period analysis
//! for directed graphs where each node has exactly one outgoing edge (x → f(x)).
//!
//! ## Overview
//!
//! A **functional graph** is a directed graph where every node has exactly one outgoing
//! edge, i.e., each node `x` maps to exactly one successor `f(x)`. These graphs arise
//! naturally in iterated function systems, pseudo-random number generators, Pollard's rho
//! algorithm, and various combinatorial structures.
//!
//! Every connected component of a functional graph consists of exactly one directed cycle,
//! with trees (directed inward) feeding into the cycle nodes.
//!
//! ## Core Types
//!
//! - [`FunctionalGraph`] — representation and iteration of x → f(x) mappings
//! - [`CycleDetection`] — Floyd's tortoise and hare algorithm for cycle finding
//! - [`TreeDecomposition`] — decompose each component into cycle + feeding trees
//! - [`PeriodFinder`] — find the cycle length (period) for any starting node
//! - [`TransientLength`] — compute steps before entering a cycle (rho length)

/// A functional graph where each node has exactly one outgoing edge.
///
/// Nodes are 0-indexed. The graph stores a successor function `f: usize -> usize`.
#[derive(Clone, Debug)]
pub struct FunctionalGraph {
    /// Successor function: succ[i] = f(i)
    succ: Vec<usize>,
}

impl FunctionalGraph {
    /// Create a new functional graph from a successor function.
    ///
    /// `succ[i]` must be a valid node index (< len) for all i.
    ///
    /// # Panics
    ///
    /// Panics if any successor is out of bounds.
    pub fn new(succ: Vec<usize>) -> Self {
        let n = succ.len();
        for &s in &succ {
            assert!(s < n, "Successor {} out of bounds for graph of size {}", s, n);
        }
        Self { succ }
    }

    /// Number of nodes in the graph.
    pub fn len(&self) -> usize {
        self.succ.len()
    }

    /// Returns true if the graph has zero nodes.
    pub fn is_empty(&self) -> bool {
        self.succ.is_empty()
    }

    /// Get the successor of node `i`, i.e., f(i).
    ///
    /// # Panics
    ///
    /// Panics if `i` is out of bounds.
    pub fn successor(&self, i: usize) -> usize {
        self.succ[i]
    }

    /// Iterate from node `start` for `steps` steps, returning all visited nodes
    /// (including the start node).
    pub fn iterate(&self, start: usize, steps: usize) -> Vec<usize> {
        let mut path = Vec::with_capacity(steps + 1);
        let mut current = start;
        path.push(current);
        for _ in 0..steps {
            current = self.succ[current];
            path.push(current);
        }
        path
    }

    /// Get the raw successor vector.
    pub fn successors(&self) -> &[usize] {
        &self.succ
    }
}

/// Cycle detection using Floyd's tortoise and hare algorithm.
///
/// Given a functional graph and a starting node, finds the first node
/// in the cycle (the "meeting point") and the cycle length.
pub struct CycleDetection<'a> {
    graph: &'a FunctionalGraph,
}

impl<'a> CycleDetection<'a> {
    /// Create a new cycle detector for the given graph.
    pub fn new(graph: &'a FunctionalGraph) -> Self {
        Self { graph }
    }

    /// Detect the cycle reachable from `start`.
    ///
    /// Returns `(mu, lambda)` where:
    /// - `mu` is the number of steps before entering the cycle (transient length)
    /// - `lambda` is the length of the cycle
    pub fn detect(&self, start: usize) -> (usize, usize) {
        // Phase 1: Find meeting point using tortoise and hare
        let mut tortoise = self.graph.successor(start);
        let mut hare = self.graph.successor(self.graph.successor(start));

        while tortoise != hare {
            tortoise = self.graph.successor(tortoise);
            hare = self.graph.successor(self.graph.successor(hare));
        }

        // Phase 2: Find mu (start of cycle)
        let mut mu = 0;
        tortoise = start;
        while tortoise != hare {
            tortoise = self.graph.successor(tortoise);
            hare = self.graph.successor(hare);
            mu += 1;
        }

        // Phase 3: Find lambda (cycle length)
        let mut lambda = 1;
        hare = self.graph.successor(tortoise);
        while tortoise != hare {
            hare = self.graph.successor(hare);
            lambda += 1;
        }

        (mu, lambda)
    }

    /// Returns the first node in the cycle reachable from `start`.
    pub fn cycle_entry(&self, start: usize) -> usize {
        let (mu, _) = self.detect(start);
        let mut node = start;
        for _ in 0..mu {
            node = self.graph.successor(node);
        }
        node
    }

    /// Returns the cycle nodes reachable from `start`, in order.
    pub fn cycle_nodes(&self, start: usize) -> Vec<usize> {
        let entry = self.cycle_entry(start);
        let mut nodes = vec![entry];
        let mut current = self.graph.successor(entry);
        while current != entry {
            nodes.push(current);
            current = self.graph.successor(current);
        }
        nodes
    }
}

/// Tree decomposition of a functional graph.
///
/// Each connected component of a functional graph has exactly one directed cycle,
/// with trees feeding into the cycle. This struct decomposes the graph into
/// these components.
#[derive(Clone, Debug)]
pub struct TreeDecomposition {
    /// For each node, its distance to the cycle (0 if on cycle itself).
    pub depth: Vec<usize>,
    /// For each node, the cycle node it eventually reaches.
    pub root: Vec<usize>,
    /// For each cycle node, the list of nodes in its tree (by depth order).
    pub trees: Vec<Vec<usize>>,
    /// The cycle nodes for each component.
    pub cycles: Vec<Vec<usize>>,
}

impl TreeDecomposition {
    /// Decompose the functional graph into cycle + feeding trees.
    pub fn decompose(graph: &FunctionalGraph) -> Self {
        let n = graph.len();
        let mut depth = vec![0; n];
        let mut root = vec![0; n];
        let mut visited = vec![false; n];
        let mut cycles: Vec<Vec<usize>> = Vec::new();
        let mut trees: Vec<Vec<usize>> = Vec::new();

        for start in 0..n {
            if visited[start] {
                continue;
            }

            // Walk until we hit a visited node
            let mut path = Vec::new();
            let mut current = start;
            let mut path_set = vec![false; n];

            while !visited[current] && !path_set[current] {
                path_set[current] = true;
                path.push(current);
                current = graph.successor(current);
            }

            if visited[current] {
                // This path feeds into an already-processed component
                let root_node = root[current];
                let base_depth = depth[current] + 1;
                for (i, &node) in path.iter().enumerate() {
                    visited[node] = true;
                    root[node] = root_node;
                    depth[node] = base_depth + (path.len() - 1 - i);
                }
            } else {
                // Found a new cycle
                let cycle_start_idx = path.iter().position(|&x| x == current).unwrap();
                let cycle: Vec<usize> = path[cycle_start_idx..].to_vec();
                let cycle_set: Vec<bool> = {
                    let mut s = vec![false; n];
                    for &c in &cycle {
                        s[c] = true;
                    }
                    s
                };

                // Mark cycle nodes
                for &c in &cycle {
                    visited[c] = true;
                    root[c] = c;
                    depth[c] = 0;
                }

                // Mark tree nodes (those before cycle in path)
                for (i, &node) in path[..cycle_start_idx].iter().enumerate() {
                    let dist_to_cycle = cycle_start_idx - i;
                    visited[node] = true;
                    depth[node] = dist_to_cycle;
                    // Walk forward to find root
                    let mut r = node;
                    for _ in 0..dist_to_cycle {
                        r = graph.successor(r);
                    }
                    root[node] = r;
                }

                let cycle_idx = cycles.len();
                cycles.push(cycle.clone());
                trees.push(Vec::new()); // placeholder, filled below
            }
        }

        // Build tree lists
        trees = vec![Vec::new(); cycles.len()];
        let mut cycle_index = vec![0usize; n]; // maps cycle node -> index in cycles vec
        for (ci, cycle) in cycles.iter().enumerate() {
            for &c in cycle {
                cycle_index[c] = ci;
            }
        }

        for node in 0..n {
            if depth[node] > 0 {
                let ci = cycle_index[root[node]];
                trees[ci].push(node);
            }
        }

        // Sort trees by depth
        for tree in &mut trees {
            tree.sort_by_key(|&node| depth[node]);
        }

        TreeDecomposition {
            depth,
            root,
            trees,
            cycles,
        }
    }

    /// Returns the number of connected components.
    pub fn component_count(&self) -> usize {
        self.cycles.len()
    }
}

/// Finds the period (cycle length) for any starting node.
pub struct PeriodFinder<'a> {
    detector: CycleDetection<'a>,
}

impl<'a> PeriodFinder<'a> {
    /// Create a new period finder.
    pub fn new(graph: &'a FunctionalGraph) -> Self {
        Self {
            detector: CycleDetection::new(graph),
        }
    }

    /// Find the period (cycle length) reachable from `start`.
    pub fn period(&self, start: usize) -> usize {
        let (_, lambda) = self.detector.detect(start);
        lambda
    }
}

/// Computes the transient length (steps before entering a cycle).
pub struct TransientLength<'a> {
    detector: CycleDetection<'a>,
}

impl<'a> TransientLength<'a> {
    /// Create a new transient length computer.
    pub fn new(graph: &'a FunctionalGraph) -> Self {
        Self {
            detector: CycleDetection::new(graph),
        }
    }

    /// Compute the transient length from `start`.
    pub fn transient(&self, start: usize) -> usize {
        let (mu, _) = self.detector.detect(start);
        mu
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_self_loop() {
        // 0 -> 0 (self-loop)
        let g = FunctionalGraph::new(vec![0]);
        assert_eq!(g.successor(0), 0);
        let (mu, lambda) = CycleDetection::new(&g).detect(0);
        assert_eq!(mu, 0);
        assert_eq!(lambda, 1);
    }

    #[test]
    fn test_two_node_cycle() {
        // 0 -> 1 -> 0
        let g = FunctionalGraph::new(vec![1, 0]);
        let (mu, lambda) = CycleDetection::new(&g).detect(0);
        assert_eq!(mu, 0);
        assert_eq!(lambda, 2);
    }

    #[test]
    fn test_rho_shape() {
        // 0 -> 1 -> 2 -> 3 -> 4 -> 2  (cycle: 2-3-4, tail: 0-1)
        let g = FunctionalGraph::new(vec![1, 2, 3, 4, 2]);
        let (mu, lambda) = CycleDetection::new(&g).detect(0);
        assert_eq!(mu, 2);
        assert_eq!(lambda, 3);
    }

    #[test]
    fn test_cycle_entry_rho() {
        // Same rho shape
        let g = FunctionalGraph::new(vec![1, 2, 3, 4, 2]);
        let entry = CycleDetection::new(&g).cycle_entry(0);
        assert_eq!(entry, 2);
    }

    #[test]
    fn test_cycle_nodes() {
        let g = FunctionalGraph::new(vec![1, 2, 3, 4, 2]);
        let nodes = CycleDetection::new(&g).cycle_nodes(0);
        assert_eq!(nodes, vec![2, 3, 4]);
    }

    #[test]
    fn test_iterate() {
        let g = FunctionalGraph::new(vec![1, 2, 0]);
        let path = g.iterate(0, 4);
        assert_eq!(path, vec![0, 1, 2, 0, 1]);
    }

    #[test]
    fn test_period_finder() {
        let g = FunctionalGraph::new(vec![1, 2, 3, 4, 2]);
        let period = PeriodFinder::new(&g).period(0);
        assert_eq!(period, 3);
    }

    #[test]
    fn test_transient_length() {
        let g = FunctionalGraph::new(vec![1, 2, 3, 4, 2]);
        let t = TransientLength::new(&g).transient(0);
        assert_eq!(t, 2);
    }

    #[test]
    fn test_transient_from_cycle_node() {
        let g = FunctionalGraph::new(vec![1, 2, 3, 4, 2]);
        let t = TransientLength::new(&g).transient(2);
        assert_eq!(t, 0);
    }

    #[test]
    fn test_tree_decomposition_single_cycle() {
        // 0 -> 1 -> 2 -> 0 (pure cycle)
        let g = FunctionalGraph::new(vec![1, 2, 0]);
        let td = TreeDecomposition::decompose(&g);
        assert_eq!(td.component_count(), 1);
        assert_eq!(td.depth, vec![0, 0, 0]);
        assert!(td.trees[0].is_empty());
    }

    #[test]
    fn test_tree_decomposition_with_tail() {
        // 0 -> 1 -> 2 -> 3 -> 2 (cycle: 2-3, tail: 0-1)
        let g = FunctionalGraph::new(vec![1, 2, 3, 2]);
        let td = TreeDecomposition::decompose(&g);
        assert_eq!(td.depth[0], 2);
        assert_eq!(td.depth[1], 1);
        assert_eq!(td.depth[2], 0);
        assert_eq!(td.depth[3], 0);
        assert_eq!(td.root[0], 2);
        assert_eq!(td.root[1], 2);
    }

    #[test]
    fn test_tree_decomposition_two_components() {
        // 0 -> 1 -> 0, 2 -> 3 -> 2
        let g = FunctionalGraph::new(vec![1, 0, 3, 2]);
        let td = TreeDecomposition::decompose(&g);
        assert_eq!(td.component_count(), 2);
    }

    #[test]
    #[should_panic]
    fn test_invalid_successor() {
        FunctionalGraph::new(vec![5]); // 5 is out of bounds
    }

    #[test]
    fn test_large_cycle() {
        // 0 -> 1 -> 2 -> ... -> 9 -> 0
        let succ: Vec<usize> = (1..=9).chain(std::iter::once(0)).collect();
        let g = FunctionalGraph::new(succ);
        let (mu, lambda) = CycleDetection::new(&g).detect(0);
        assert_eq!(mu, 0);
        assert_eq!(lambda, 10);
    }
}
