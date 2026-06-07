# functional-graph

**Functional graph iteration, cycle detection, tree decomposition, and period analysis for x → f(x) mappings.**

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

## Overview

A **functional graph** is a directed graph where every node has exactly one outgoing edge. Formally, for a function `f: V → V`, each node `x` has exactly one successor `f(x)`. These structures appear naturally in:

- **Iterated function systems** — repeatedly applying a function and studying orbits
- **Pseudo-random number generators** — each state maps to exactly one next state
- **Pollard's rho algorithm** — cycle detection in factoring and discrete log
- **Number theory** — iterated maps like `x → σ(x)` (sum of divisors), `x → φ(x)` (Euler totient)
- **Combinatorics** — mapping patterns, permutation cycles with fixed-point structures

Every connected component of a functional graph has a unique structure: **exactly one directed cycle**, with directed trees feeding into the cycle nodes. This crate provides efficient algorithms for detecting cycles, computing periods, finding transient lengths, and decomposing the graph into its canonical form.

## Features

- **`FunctionalGraph`** — Store and iterate x → f(x) mappings with bounds checking
- **`CycleDetection`** — Floyd's tortoise and hare algorithm: O(μ + λ) time, O(1) space
- **`TreeDecomposition`** — Decompose each component into cycle + feeding trees
- **`PeriodFinder`** — Find cycle length (period/lambda) for any starting node
- **`TransientLength`** — Compute steps before entering a cycle (rho length/mu)

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
functional-graph = "0.1.0"
```

## Quick Start

```rust
use functional_graph::*;

// Create a functional graph: 0 → 1 → 2 → 3 → 4 → 2 (rho shape)
// Tail: 0, 1 | Cycle: 2, 3, 4
let graph = FunctionalGraph::new(vec![1, 2, 3, 4, 2]);

// Cycle detection using Floyd's algorithm
let detector = CycleDetection::new(&graph);
let (mu, lambda) = detector.detect(0);
assert_eq!(mu, 2);       // 2 steps before cycle
assert_eq!(lambda, 3);   // cycle length is 3

// Find the cycle entry point
let entry = detector.cycle_entry(0);
assert_eq!(entry, 2);

// Get all cycle nodes
let cycle = detector.cycle_nodes(0);
assert_eq!(cycle, vec![2, 3, 4]);

// Iterate the function
let path = graph.iterate(0, 8);
assert_eq!(path, vec![0, 1, 2, 3, 4, 2, 3, 4, 2]);

// Period and transient analysis
let period = PeriodFinder::new(&graph).period(0);
assert_eq!(period, 3);

let transient = TransientLength::new(&graph).transient(0);
assert_eq!(transient, 2);

// Tree decomposition
let td = TreeDecomposition::decompose(&graph);
assert_eq!(td.component_count(), 1);
assert_eq!(td.depth[0], 2);  // node 0 is 2 steps from cycle
assert_eq!(td.depth[2], 0);  // node 2 is on the cycle
```

## Algorithm Details

### Floyd's Tortoise and Hare

The cycle detection uses Floyd's algorithm in three phases:

1. **Phase 1 — Meeting point**: The tortoise moves one step at a time, the hare moves two. They meet inside the cycle.
2. **Phase 2 — Cycle start (μ)**: Reset the tortoise to the start. Both move one step at a time. They meet at the cycle entry.
3. **Phase 3 — Cycle length (λ)**: From the cycle entry, count steps until returning.

Time complexity: **O(μ + λ)** — linear in the sum of transient and cycle lengths.
Space complexity: **O(1)** — only a few pointers, no additional data structures.

### Tree Decomposition

The decomposition algorithm walks from each unvisited node until reaching a previously visited node or completing a new cycle. It assigns:

- **`depth[i]`**: Distance from node `i` to its cycle (0 for cycle nodes)
- **`root[i]`**: The cycle node that node `i` eventually reaches
- **`trees[c]`**: All non-cycle nodes that feed into cycle node `c`
- **`cycles[k]`**: The cycle nodes of the k-th connected component

## Examples

### Pseudo-Random Number Generator Analysis

```rust
use functional_graph::*;

// Linear congruential generator: x → (a*x + c) mod m
let m = 16;
let a = 5;
let c = 3;
let succ: Vec<usize> = (0..m).map(|x| (a * x + c) % m).collect();
let graph = FunctionalGraph::new(succ);

let detector = CycleDetection::new(&graph);
for start in 0..m {
    let (mu, lambda) = detector.detect(start);
    println!("Start {}: transient={}, period={}", start, mu, lambda);
}
```

### Multiple Components

```rust
use functional_graph::*;

// Two separate components:
// Component 1: 0 → 1 → 0 (cycle of length 2)
// Component 2: 2 → 3 → 2 (cycle of length 2)
let graph = FunctionalGraph::new(vec![1, 0, 3, 2]);
let td = TreeDecomposition::decompose(&graph);
assert_eq!(td.component_count(), 2);
```

## API Reference

| Type | Description |
|------|-------------|
| `FunctionalGraph` | Core graph representation with successor function |
| `CycleDetection` | Floyd's algorithm for (μ, λ) detection |
| `TreeDecomposition` | Component decomposition into cycles + trees |
| `PeriodFinder` | Cycle length computation |
| `TransientLength` | Pre-cycle steps computation |

## Performance

All algorithms run in **O(n)** time where `n` is the number of nodes. The tree decomposition uses O(n) additional space for tracking visited nodes and component membership.

## License

MIT License. See [LICENSE](LICENSE) for details.
