# Lojban Neuro-Symbolic (NeSy) Engine

A high-performance, future-ready pipeline designed to parse, semantically compile, and logically reason about Lojban text using a zero-copy WebAssembly (Wasm) architecture.

## 🚀 Architectural Overview

The engine utilizes the **WASI Preview 2 Component Model** to facilitate deterministic execution and polyglot interoperability. By composing discrete Wasm modules into a single, fused binary, the system achieves near-native performance through shared linear memory.

### The Zero-Copy Pipeline

1. **Lexical Analysis (`logos`)**: Rapidly tokenizes raw Lojban strings into morphological classes without intermediate string allocations.
2. **Preprocessing**: Resolves metalinguistic erasures (`si`, `sa`, `su`) and quotations (`zo`, `zoi`) at the token level.
3. **Recursive Descent Parser**: A high-speed,  parser that consumes the token stream to build an arena-allocated Abstract Syntax Tree (AST).
4. **Semantic Compiler**: Maps the AST to First-Order Logic (FOL) predicates. It utilizes a **Perfect Hash Function (PHF)** dictionary baked into the binary for  predicate lookups.
5. **Reasoning Core (`egglog`)**: Performs equality saturation and logical entailment checking within an E-Graph database.

## 🛠️ Tech Stack

* 
**Language**: Rust (Latest Stable).


* **Wasm Runtime**: Wasmtime 41.0.3.
* 
**Package Manager**: Nix / NixOS (for hermetic dev environments).


* **Task Runner**: Just.
* **Logic Engine**: Egglog (Equality Saturation).
* **Lexing**: Logos.

---

## 💻 Development Setup

### Prerequisites

* 
[Nix](https://nixos.org/download.html) (Recommended for a reproducible environment).


* 
[Just](https://github.com/casey/just).



### Loading the Environment

```bash
# Load the Nix shell with all necessary WASI and Rust tooling
nix flake update
nix develop

```

### Build & Run

The project uses a structured `Justfile` to automate the complex build and composition process.

```bash
# Build components, fuse them into a single engine, and launch the REPL
just run

```

---

## 🧩 Component Breakdown

| Component | Responsibility | Key Optimization |
| --- | --- | --- |
| **Parser** | Morphological/Structural Parsing | Single-pass, zero-copy recursive descent. |
| **Semantics** | Syntax-to-Logic Mapping | Static PHF dictionary lookup (). |
| **Reasoning** | Logical Entailment | E-Graph equality saturation via `egglog`. |
| **Orchestrator** | Pipeline Coordination | Cross-component calls within a single Wasm sandbox. |

---

## 📜 World Interface (WIT)

The system boundaries are defined by `world.wit`, enforcing strict I/O typing for the engine pipeline .

```rust
world engine-pipeline {
    import parser;
    import semantics;
    import reasoning;
    
    export execute: func(input: string) -> bool;
}

```

## 🧪 Future Roadmap

* **SIMD Gismu Matching**: Accelerating the morphological scan using AVX-512 instructions.
* **Lujvo Decomposition**: Algorithmically breaking down compound words into their base meanings during semantic compilation.
* **BAI Modal Tags**: Extending the reasoning core to support Lojban's extensive case-tagging system.
