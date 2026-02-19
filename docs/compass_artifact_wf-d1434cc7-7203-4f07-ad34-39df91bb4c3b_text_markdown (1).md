# Neuro-symbolic reasoning for Lojban in Rust: a feasibility deep-dive

A Rust-based neuro-symbolic reasoning engine for Lojban is feasible today, with mature crates covering every layer of the proposed six-layer architecture. **The critical path runs through four decisions**: pest.rs v2.8.6 for PEG parsing (the only Rust parser supporting the stack operations Lojban's ZOI quotes require), egglog v1.0.0 or ascent v0.8.0 for the inference core, Scallop's provenance semiring architecture as the neuro-symbolic bridge pattern, and Oxigraph v0.5.4 with neo-Davidsonian event reification for the knowledge graph. The Lojban ecosystem provides workable but aging upstream resources — ilmentufa's PEG grammars are the most complete reference, but no Rust-native parser achieves full coverage, meaning the grammar must be ported manually. Below is a layer-by-layer analysis of every component, with exact version numbers, known pitfalls, and architectural recommendations current through early 2026.

---

## Layer 1: parsing Lojban's 600-rule PEG grammar in Rust

**pest.rs v2.8.6** (released February 5, 2026, MSRV 1.83.0) is the strongest choice for implementing Lojban's PEG parser. It natively supports PEG semantics with ordered choice, and critically provides **PUSH/POP/PEEK stack operations** required for matching ZOI quote delimiters (`zoi X. arbitrary text .X`). A Lojban grammar runs roughly 600+ rules spanning morphology and syntax — at this scale, pest exhibits **non-linear compile-time growth** (a documented ~700-line grammar compiles in ~30 seconds, generating 20K+ lines of Rust), but this is a one-time build cost, not a runtime concern. The bigger runtime risk is that **pest lacks packrat parsing (memoization)**, meaning certain deeply recursive patterns can trigger exponential backtracking. Careful rule ordering and pest's `set_call_limit()` function mitigate this.

**faster-pest v0.1.4** claims 7× speedup over standard pest on JSON benchmarks but is a **non-starter for Lojban** — it lacks stack operations (PUSH/POP/PEEK), positive lookahead, and case-insensitive matching. **logos v0.16.1** (released January 30, 2026) excels as a **lexing pre-pass** at ~1200 MB/s throughput, classifying Lojban's morphological word classes — gismu (CVCCV/CCVCV patterns), cmavo (CV/CVV), lujvo, fu'ivla, and cmevla — via regex patterns before feeding tokens to the PEG parser. The two-phase architecture (logos lexer → pest parser) would substantially reduce the PEG grammar's complexity.

The upstream grammar situation requires careful navigation. **ilmentufa** (github.com/lojban/ilmentufa, 55 stars, last updated January 2025) contains **five PEG grammar variants**: `camxes.peg` (standard CLL grammar), `camxes-beta.peg` (backward-compatible extensions), and three experimental variants adding the Cmevla-Brivla Merger and Ce-Ki-Tau reforms. The standard `camxes.peg` should be the porting target. **camxes-rs** (github.com/lojban/camxes-rs, 4 stars, last updated April 2025) exists but has near-zero community adoption and unknown completeness. No Rust-native parser achieves full Lojban coverage — the grammar must be manually ported from ilmentufa's PEG.js format to pest syntax, a substantial but tractable effort.

Handling **elidable terminators** (ku, kei, vau — roughly 30 optional closing markers) is straightforward in PEG: make each terminator optional with `KU_elidible = { KU_clause | "" }`. PEG's ordered alternation provides longest-match disambiguation naturally, which is exactly how the original camxes grammar handles elision. Robin Lee Powell's original insight — that "elidable terminators can merely be made optional if longest-match disambiguation is used" — was the key breakthrough enabling PEG parsing of Lojban. **ZOI quotes** require pest's stack: `PUSH(delimiter) ~ (!PEEK ~ ANY)* ~ POP` captures arbitrary text between matching delimiter words.

For parser benchmarks: nom outperforms pest when both produce typed ASTs (~74ms vs ~82ms on canada.json), but pest's declarative grammar format is far more maintainable for a 600-rule grammar. tree-sitter uses GLR (not PEG) and would require grammar reformulation. **pest is the only practical choice** balancing PEG compatibility, maintenance ergonomics, and the stack operations Lojban demands.

---

## Layer 2: the semantic compiler and formal logic IR

The semantic compiler must translate pest parse trees into a formal logic intermediate representation. This is the layer where Lojban's design advantages over natural language become decisive: Lojban's grammar is already compositional, scope is syntactically explicit, and predicate place structures are formally defined.

**Montague-style compositional semantics** maps directly onto Lojban's architecture. Each gismu carries a typed predicate signature (e.g., `klama`: x1 goes to x2 from x3 via x4 by means x5), and cmavo function as higher-order semantic operators. The semantic compiler walks the parse tree bottom-up, assigning each node a typed lambda term and composing via β-reduction. Variable binding should use **locally nameless representation** — the `unbound` crate (released October 2025) provides `Alpha` and `Subst` derivable traits, handling Lojban's da/de/di prenex variables and poi/noi relative clauses cleanly. The `moniker` crate is an older alternative with similar capabilities.

Lojban's **12 abstraction types** (selma'o NU) require distinct IR representations because they create fundamentally different semantic objects. The critical distinctions: **nu** creates a spatiotemporal event (reifiable, causable), **du'u** creates a proposition (truth-bearing, not spacetime-bound), **ka** creates a property via λ-abstraction using ce'u as the bound variable, **ni** returns a quantity on a measurement scale, and **jei** returns a truth value in [0,1] — which is Lojban's built-in fuzzy logic mechanism. A well-designed IR needs a typed enum with all 12 variants, where ka-abstractions carry their ce'u binding positions and jei-abstractions integrate with the system's fuzzy truth-value infrastructure.

The **xorlo reform** (approved 2003, universally adopted) introduces a subtle but far-reaching complication: `lo broda` is now a **plural constant** rather than an existentially quantified variable. This shifts Lojban's formal semantics from singular first-order logic toward **plural logic**. Standard FOL theorem provers assume singular quantification; xorlo means predicates like `simxu` (reciprocal) take plural referents. The bound variables da/de/di remain singular, creating an expressibility gap — experimental plural quantifiers `su'oi` and `ro'oi` exist but are unofficial. The IR should represent `lo`-phrases as **discourse referents** (closer to Discourse Representation Theory than classical FOL), with explicit distributivity annotations when determinable.

The **tersmu** Haskell semantic parser (v0.2.2, effectively dormant since 2014) provides the only existing reference implementation of Lojban-to-logic translation. Its ~450-line core in `Lojban.hs` handles quantifier export, connective scope, and anaphora resolution, but the author acknowledges the output "doesn't have an obvious standard semantics" — it copies some Lojbanic constructs directly rather than fully interpreting them. Key lessons from tersmu: treat indicators (attitudinals) and frees as metadata annotations rather than logical content, handle Skolem functions for `lo mamta be da` patterns, and export quantifiers to the closest prenex in order.

---

## Layer 3: knowledge graph storage with n-ary predicate support

**Oxigraph v0.5.4** (MSRV 1.87, RocksDB 10.5.1 backend) provides **full SPARQL 1.1 compliance** with query, update, and federated query support. It now supports **RDF 1.2 / SPARQL 1.2** behind the `rdf-12` feature flag (replacing the old `rdf-star` flag), which is critical for annotating triples with provenance and confidence scores. The crate compiles to **WebAssembly** (automatically disabling RocksDB and falling back to in-memory storage) and offers parallel N-Triples/N-Quads parsing via bulk loader. Query evaluation is "not yet optimized" per the maintainers but sufficient for the scale of a Lojban reasoning system. The modular crate ecosystem (`oxrdf`, `spargebra`, `spareval`) allows importing only needed components.

Lojban's **n-ary predicates** (up to 5-place relations) require **neo-Davidsonian event reification** in RDF. Each bridi introduces an event node, with place arguments connected via labeled edges:

```turtle
_:e1 a lojban:klama .
_:e1 lojban:x1 :goer .       # agent
_:e1 lojban:x2 :destination . # goal
_:e1 lojban:x3 :origin .      # source
_:e1 lojban:x4 :route .       # path
_:e1 lojban:x5 :means .       # vehicle
```

This maps perfectly to Lojban's bridi structure and is SPARQL-queryable. RDF-star (RDF 1.2) complements this by enabling **statement-level annotations** — attaching confidence scores, evidential markers, and temporal bounds to individual role assertions without additional reification overhead. The W3C's "Defining N-ary Relations on the Semantic Web" working group note validates this exact pattern.

For storage: **sled is effectively abandoned** (no releases since 2021, the promised 1.0 never materialized, the author moved on). **redb v3.0.2** is the leading pure-Rust alternative — ACID-compliant, LMDB-inspired, actively maintained. However, for a full knowledge graph, Oxigraph's integrated RocksDB backend is the better choice, with redb serving as a lighter option for auxiliary indices. **petgraph v0.8.2** provides fast in-memory graph algorithms but **lacks hypergraph support** — use it as a computational layer for graph traversals, not as the primary store.

---

## Layer 4: the hybrid inference engine

Three Rust reasoning engines stand out for different aspects of the hybrid inference layer.

**egglog v1.0.0** (released ~January 2026, 651 stars) unifies **Datalog with equality saturation**, making it ideal for logical normalization. It stores functions and relations in e-graph-backed tables, computes fixpoints where both relational facts and term equivalences stabilize, and extracts optimal representations via user-defined cost functions. The PLDI 2023 paper established its theoretical foundations. Reaching 1.0.0 signals API maturity. Parallel execution support (via rayon) is available but recent. For the reasoning engine, egglog would handle **equivalence-preserving normalization** of Lojban logical forms — collapsing semantically equivalent expressions before inference.

**ascent v0.8.0** (~October 2025, 153K downloads) embeds Datalog directly in Rust via procedural macros, with **lattice semantics** and **parallel execution** (`ascent_par!`). Its BYODS (Bring Your Own Data Structures) feature allows relation backing by custom structures — critically, union-find-backed relations (`trrel_uf`) dramatically accelerate transitive closure computations common in knowledge graph reasoning. Ascent is the strongest choice for the **core Datalog evaluation engine** due to its native Rust ergonomics and lattice support (essential for representing truth-value bounds).

**Scryer Prolog v0.10.0** (September 2025, ~2,370 stars) implements the full WAM with SLG resolution (tabling), CLP(ℤ), CLP(B), attributed variables, and DCGs. It can be used as a Rust library (the embedding API is improving, explicitly listed as a 2025 priority) for **backward chaining with backtracking**. The WAM-style explicit choice-point stack is strongly preferred over recursive approaches in Rust — it avoids stack overflow risks (Rust's default 8MB stack), supports cut, tabling, and last-call optimization. For clause selection, **first-argument hashing** (the standard WAM `switch_on_term`/`switch_on_constant` pattern) provides O(1) dispatch to matching clauses.

**Nemo** (TU Dresden, 151 stars, KR 2024 paper) handles **10⁵–10⁸ facts** on a laptop using column-based hierarchically-sorted in-memory tables, matching or exceeding Soufflé and RDFox performance. It's written entirely in Rust but not yet published on crates.io as a library — it requires nightly Rust and is self-described as unstable. For standalone Datalog evaluation of large fact bases, Nemo would be the scaling option.

The **ena crate v0.14.3** provides union-find with snapshot-based backtracking, extracted from the Rust compiler itself. Its `snapshot()`/`rollback_to()`/`commit()` API enables efficient backtracking search for unification, and extending it with semiring annotations per binding gives fuzzy unification "for free."

The **Chalk architecture pattern** (now sunset, but architecturally influential) demonstrates a clean three-layer separation applicable to Lojban: (1) a domain layer providing Lojban parse trees and gismu definitions, (2) a lowering layer converting Lojban constructs to Horn clauses with thematic roles, and (3) a generic SLG solver that knows nothing about Lojban. This separation ensures the logic solver is reusable and the Lojban-specific complexity is contained in the lowering layer.

---

## Layer 5: the neuro-symbolic bridge

**Scallop** (github.com/scallop-lang/scallop, 424 stars, PLDI 2023) is the most directly relevant framework — its core runtime is **implemented in Rust** (nightly required), it provides Python bindings (scallopy) for PyTorch integration, and its **provenance semiring architecture** is the theoretical key to the entire neuro-symbolic bridge. Scallop supports **18+ built-in provenance types**: unit (standard Datalog), proofs (full derivation tracking), minmaxprob (simple probabilistic), difftopkproofs/DTKP (the flagship differentiable provenance — top-k proofs with weighted model counting), and all standard fuzzy t-norm operators. The provenance framework generalizes the Green-Karvounarakis-Tannen (PODS 2007) semiring theory: a tuple **(T, 0, 1, ⊕, ⊗, ⊖, ⃝=)** where ⊗ tracks conjunctive steps and ⊕ tracks alternative derivations.

For implementing provenance semirings in the custom engine, the trait interface is straightforward:

- **⊗ (multiply)** composes confidence through inference steps — if premise A has confidence 0.9 and premise B has 0.8, their conjunction under the product semiring has confidence 0.72
- **⊕ (add)** combines alternative derivation paths — multiple ways to derive the same fact increase overall confidence
- **Viterbi semiring** ([0,1], max, ×) tracks the single most probable derivation
- **Tropical semiring** (ℝ≥0∪{∞}, min, +) handles shortest-path/minimum-cost reasoning

The universal provenance ℕ[X] (polynomials over tuple IDs) subsumes all commutative semirings — any specific semiring factors through it via a unique homomorphism. For Datalog with recursion, the ω-continuous formal power series ℕ∞⟦X⟧ provides the universal construction.

**IBM's Logical Neural Networks** offer the model for **bidirectional inference**. Each logical formula maintains truth-value bounds [L, U] ∈ [0,1]². The upward pass computes formula truth from operand bounds (standard evaluation); the downward pass **tightens operand bounds from formula constraints** (inverse evaluation). For implication `x → y`: if we know `x → y` is true and `y` is false, the downward pass deduces ¬x (modus tollens). Convergence is guaranteed because all operations are monotonic (lower bounds only increase, upper bounds only decrease). The open-source implementation (github.com/IBM/LNN, 293 stars, Apache 2.0) provides a Python reference using Łukasiewicz activation functions. Porting the upward/downward pass pattern to Rust is straightforward — the core is ~200 lines of bound-tightening arithmetic.

**AlphaGeometry** (Nature 2024, solving 25/30 IMO geometry problems) validates the **loose coupling** pattern: a symbolic deduction engine exhaustively applies rules, and when stuck, a neural language model suggests auxiliary constructions. The symbolic engine verifies all suggestions — the neural component never proves, only suggests. This architecture maps directly to the Lojban system: the symbolic engine handles logical inference, and when tanru disambiguation or commonsense reasoning is needed, a neural embedding model scores candidate interpretations that the symbolic engine then verifies.

For **tanru disambiguation** — the hardest Lojban-specific challenge — neural embeddings over gismu definitions and place structures can score candidate relationships between seltau (modifier) and tertau (head). The existing lujvo dictionary (~thousands of entries with fixed tanru→relationship mappings) provides training data. A lightweight embedding model (BERT-sized, loaded via candle or ort) generates vectors for each gismu's definition and place structure; cosine similarity between place-structure embeddings identifies the most likely overlap pattern.

---

## Layer 6: explanation generation and evidential tracking

Every derived fact should carry a **provenance chain** recording the inference rules applied, premises consumed, confidence at each step, and evidential type. Lojban's attitudinal/evidential system maps naturally to metadata categories: **se'o** (personal experience → DirectObservation), **ti'e** (hearsay → Hearsay with source and depth), **ka'u** (cultural knowledge → Assumption), **pe'i** (opinion → SubjectiveAssessment). Cyc's **microtheory** architecture provides the organizational model — partition assertions by context so that contradictory beliefs coexist in separate microtheories, with inheritance hierarchies connecting them. A Lojban knowledge base might have microtheories for `sanskeMt` (science), `lijdaMt` (religion), and `se'oMt` (personal experience), each internally consistent but potentially contradicting each other.

For explanation generation, LNN-style bound tightening produces a natural proof trace: each downward-pass tightening records which formula constraint forced which operand's bounds to change. Combined with the provenance semiring's derivation trees, the system can generate explanations in Lojban itself (following tersmu's approach of outputting "stilted forethoughtful Lojban") or in structured logical notation.

---

## Rust ML crates for the neural components

The neural components (embedding models for tanru disambiguation, language model for creative suggestions) have four viable Rust options:

**candle v0.9.1** (Hugging Face, ~16K stars) provides the lightest-weight path — a ~22MB static binary supporting BERT, sentence transformers, and LLaMA inference with CUDA, Metal, and WGPU backends. It loads Safetensors and GGUF (quantized) models, compiles to WebAssembly, and starts in milliseconds. For embedding generation (the primary neural workload), candle is ideal.

**burn v0.20.0** (Tracel AI, ~9K stars, January 2026) offers the most complete Rust-native training+inference framework with **7 backends** (WGPU, CUDA, HIP/ROCm, Metal, NdArray, LibTorch, Candle). Its CubeCL compute language targets "peak performance on diverse hardware." If the system needs to fine-tune embedding models, burn provides native Rust autodiff.

**ort v2.0.0-rc.11** (pyke, ~1,949 stars) wraps ONNX Runtime for deploying pre-trained models with hardware acceleration across CUDA, TensorRT, OpenVINO, CoreML, DirectML, and QNN. For production deployment of a fixed embedding model, ort provides the broadest hardware coverage.

**tch-rs v0.23.0** (January 2026) tracks PyTorch v2.10.0 with quarterly releases. It brings the full PyTorch ecosystem (~2GB+ binary) but ensures feature parity. Use only if specific PyTorch-only models are needed.

**rust-bert v0.23.0** provides ready-to-use NLP pipelines but shows **reduced maintenance activity** — issues accumulate without response since mid-2024, and it still requires libtorch v2.4 (not tracking latest PyTorch). For new projects, prefer candle or ort.

---

## Memory management patterns for symbolic computation

The **rustc pattern** (the Rust compiler's own approach) is the gold standard for arena allocation + string interning in symbolic systems: a `DroplessArena` allocates string bytes, an `IndexSet` deduplicates and maps strings to dense u32 IDs, and `Symbol(u32)` is a 4-byte Copy type enabling O(1) comparison. The recommended stack for a Lojban reasoning engine:

- **String interning**: `lasso v0.7.3` Rodeo for runtime interning (single-threaded) or ThreadedRodeo (concurrent via DashMap sharding). For Lojban's ~1,350 gismu and ~400 cmavo — a finite, known vocabulary — `string_cache v0.9` from Servo enables **compile-time atom generation** where known symbols become zero-cost pointer comparisons.
- **Term storage**: `id-arena v2.2.1` returns opaque `Id<Term>` indices instead of references, completely eliminating borrow checker friction for cyclic term graphs. Indices are Copy, Clone, Send, Sync — storeable anywhere without lifetime parameters.
- **Arena allocation**: `bumpalo v3.19+` for heterogeneous AST nodes (bump allocation is essentially a pointer increment), or `typed-arena v2.0.2` for single-type arenas that run Drop implementations and support cyclic references.

The combined pattern yields a `Term` struct with **4-byte string references** (lasso Spur) and **4-byte term references** (id-arena Id), O(1) comparison on both, no lifetime parameters, and excellent cache locality. Memory reduction of **40–60%** versus naive pointer-heavy representations is typical, with allocation counts reduced by orders of magnitude.

---

## Lojban ecosystem resources and their limitations

The Lojban computational linguistics ecosystem is **functional but dormant academically**. No peer-reviewed papers on Lojban computational linguistics have appeared since 2020. Key resources:

**jbovlaste** (now read-only, replacement at lensisku.lojban.org) provides XML exports encoding all ~1,350 gismu with place structures in `x1`–`xN` notation, parenthetical semantic type annotations, and English glosses with place-numbered keywords. A Haskell library (`Language.Lojban.Jbovlaste`) parses this XML. The place structures constitute an informal but comprehensive ontological schema — Hintz (2014, TU Darmstadt) demonstrated automated alignment of gismu place structures to FrameNet frames, validating the structural analogy.

**SUMO mapping** has never been attempted directly, but a transitive path exists: gismu → English glosses → WordNet synsets → SUMO concepts (SUMO is the only formal ontology mapped to all 117K+ WordNet synsets). SUMO's process ontology with Agent/Patient/Instrument roles directly parallels Lojban's place structure system. This mapping would provide formal axiomatization of gismu relationships, enabling theorem-prover-backed reasoning.

**Test corpora** include camxes-py's test sentences (JSON format with GOOD/BAD/UNKNOWN annotations), the Tatoeba parallel corpus (10,000+ Lojban sentences), the BPFK text corpus (dozens of translated literary works with entries through January 2026), and the La-Lojban preprocessed parallel corpus (community-improved translations).

**Ben Goertzel's Lojban-for-AGI** proposals (AGI-13 conference) proposed Lojban++ as an AGI interlingua, but the initiative has been **superseded by MeTTa** — Hyperon's purpose-built formal language for AGI programming. Hyperon itself is actively developed (alpha released April 2024, v0.2.6+, with a fast MeTTa compiler in Rust via the MORK kernel), but contains no continued Lojban work.

---

## Performance expectations and scaling characteristics

**Rust vs Python**: For CPU-bound symbolic reasoning, Rust delivers **25–100× speedup** over Python (binary trees: 28×, n-body: 150×). For GPU-bound neural inference, the advantage narrows since both execute optimized kernels, but Rust eliminates GIL overhead, Python runtime memory, and startup latency (candle starts in milliseconds vs seconds for PyTorch import). **Rust vs C++**: within 5–10% on most benchmarks, with Rust occasionally winning; Discord's Read States service measured Rust within 2–3% of optimized C++.

**Datalog scaling**: Nemo (Rust) handles 10⁵–10⁸ facts on a laptop, matching or exceeding Soufflé (C++) on ChaseBench and SNOMED ontology benchmarks. Ascent provides competitive performance with rayon-based parallelism. No published head-to-head benchmark of all Rust Datalog engines exists, but each has been independently benchmarked against established non-Rust systems and shown competitive.

**Memory for knowledge bases**: Nemo's column-based layout achieves roughly **80–160 bytes per fact** including indices at 10⁸-fact scale. String interning plus arena allocation reduces symbolic computation memory by 40–60%. For a Lojban reasoning system, a working knowledge base of millions of facts would comfortably fit in single-digit gigabytes.

---

## Architectural synthesis and recommended approach

The **Attempto Controlled English (ACE)** pipeline provides the closest architectural precedent: a DCG parser produces Discourse Representation Structures (DRS) that handle anaphora, plurals, and generalized quantifiers before translation to FOL/OWL. Lojban's design makes this pipeline simpler — syntactic ambiguity is eliminated at parse time, and scope is explicitly marked — but discourse-level phenomena (anaphoric ri/ra/ru, discourse connective .i) still require a DRS-like intermediate layer.

The recommended architecture crystallizes as:

- **logos** lexer → **pest** PEG parser → parse tree
- **Montague-style compositional walker** using `unbound` for variable binding → neo-Davidsonian logical IR with 12 abstraction types
- **Oxigraph** with RDF 1.2 for persistent knowledge graph, neo-Davidsonian reification for n-ary predicates
- **ascent** for embedded bottom-up Datalog evaluation with lattice-valued truth bounds, **egglog** for equivalence normalization, **ena** for unification with snapshot backtracking
- **Scallop-inspired provenance semiring** architecture for the neuro-symbolic bridge, with **candle** loading a BERT/sentence-transformer model for embedding-based tanru disambiguation
- **LNN-style bidirectional inference** (upward/downward bound tightening) for propagating truth-value constraints, with provenance chains carrying Lojban evidential markers (se'o, ti'e, ka'u, pe'i)
- **Chalk-style three-layer separation**: Lojban domain IR → lowering to Horn clauses → generic solver

## Conclusion

The feasibility bottleneck is not the Rust ecosystem — which provides mature, high-performance crates for every layer — but the **Lojban-to-logic lowering rules**. tersmu's 450-line Haskell core is the only reference implementation, it is a decade old, and its author described the output as lacking "obvious standard semantics." The xorlo reform's introduction of plural constants without plural variables creates formal gaps that no existing system resolves. The practical recommendation is to start with a restricted Lojban subset (simple bridi with explicit terminators, no tanru, no abstractions beyond nu and du'u) and incrementally extend coverage, using the lujvo dictionary and Tatoeba parallel corpus as test oracles. The AlphaGeometry lesson applies directly: build the symbolic deduction engine first to be correct on its restricted domain, then add neural components to expand coverage into ambiguous territory. Lojban's jei abstraction — returning a truth value in [0,1] — is the language's own built-in fuzzy logic mechanism, providing a native integration point for the LNN-style truth-value bounds that would otherwise feel bolted on.