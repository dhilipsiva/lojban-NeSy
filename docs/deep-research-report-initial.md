# Building a High‑Performance Lojban Neuro‑Symbolic Engine in Rust

## Executive summary

This report synthesises the provided design documents with primary references (especially *The Complete Lojban Language* reference grammar) and the current Rust + neuro‑symbolic research ecosystem, to propose a rigorous, performance‑oriented blueprint for a **Lojban‑native neuro‑symbolic reasoning engine** implemented in **Rust**.

Lojban is a strong substrate for verifiable reasoning because its reference grammar explicitly positions the language as **predicate‑logic‑based**, **unambiguous in grammar**, and engineered to **unambiguously resolve speech into words**. citeturn3view0 However, *semantic* completeness remains non‑trivial: Lojban’s **tanru (predicate compounds)** are intentionally flexible and can be **semantically ambiguous**, which the reference grammar notes can make some “logical manipulations impossible”. citeturn3view2 This makes Lojban an unusually good fit for **neuro‑symbolic** approaches: keep the *symbolic* core exact where Lojban is exact (syntax, quantifier scoping, logical connectives), and use *neural* components where the language is intentionally underspecified (tanru interpretation, lexical smoothing, commonsense bridging).

The proposed system is built around a deterministic pipeline: **Lojban text → typed AST → typed logical IR → symbolic reasoning (Datalog/Prolog/e‑graphs) → (optional) differentiable scoring + provenance → proofs/explanations**. Critical architectural choices are driven by primary sources:
- Lojban’s **cmavo interactions** (e.g., `zo`, `si`, `SA`, `ZOI`, `zei`) and its explicit catalogue of **elidable terminators** demand a parser design that supports early preprocessing and robust handling of elision. citeturn31view0  
- Lojban’s logical connectives encode a **systematic truth‑table design**, explicitly grounded in the reference grammar’s discussion of the **16 possible truth functions** and the vowel‑encoded basis (A/E/O/U). citeturn25view1  
- The language’s quantifier prefix (`prenex`) separated by `zo'u` has direct logical significance, and the reference grammar provides explicit prenex examples aligned with first‑order logic interpretations. citeturn5view1  
- For the Rust parser, **pest** is uniquely valuable because its grammar language supports a **stack (PUSH/POP/PEEK)**, and Lojban’s ZOI quoting is explicitly acknowledged (in the PEG ecosystem) as requiring more than “pure PEG” because the delimiter must be remembered. citeturn9search0turn21search2  

On the reasoning side, the report recommends a **hybrid symbolic toolbox**:  
- **egglog** for combined Datalog + equality saturation normalisation and rewrite‑driven canonicalisation. citeturn1search2  
- **Ascent** for ergonomic embedded Datalog with parallel execution and aggregators where forward‑chaining is needed. citeturn10search0turn10search12  
- **Scryer Prolog** for backward‑chaining queries and Prolog‑style interaction where that is advantageous. citeturn10search5turn10search1  
- **Oxigraph** (or an RDF‑compatible store) for persistence and SPARQL interoperability, with event reification for Lojban’s n‑ary predicates. citeturn1search3turn1search7  

For the “neuro” bridge, three research lines are especially relevant:
- **Scallop**: neurosymbolic Datalog with differentiable reasoning built on **provenance semirings**. citeturn0search2turn15search0  
- **Logical Neural Networks (LNN)**: differentiable logic with interpretable structure and open‑world truth bounds. citeturn0search11turn0search3  
- **Logic Tensor Networks (LTN)**: “Real Logic” for many‑valued differentiable first‑order reasoning. citeturn15search3turn15search15  

Unspecified assumptions that materially affect engineering trade‑offs (and therefore remain explicitly open in this report) include: **target hardware (CPU vs GPU vs edge)**, **latency vs throughput targets**, expected **knowledge base scale**, and the precise **scope of supported logic** (Datalog only vs richer FOL fragments vs modal/temporal extensions). These are consolidated at the end.

## Background and literature review

The provided documents converge on a central insight: current LLM‑centric systems are powerful but opaque, and Lojban can act as a stable, human‑speakable intermediate representation whose syntax aligns with logic. This aligns with the reference grammar’s own positioning: Lojban’s grammar is described as based on predicate logic and engineered for unambiguous structure and word segmentation. citeturn3view0

### Lojban linguistic features that matter for an engine

Lojban’s computational advantages are not merely “controlled language” restrictions; they are embodied in the grammar and morphology.

The reference grammar describes **three basic word classes** (cmavo, brivla, cmene) with uniquely identifiable properties that support mechanical recognition in parsing. citeturn8view0 Brivla morphology is further constrained: brivla end in a vowel, contain a consonant pair early, and are penult‑stressed—properties intended to make word‑class identification systematic. citeturn29view0turn8view1 Names (cmene) have their own identifying properties, notably ending in consonants and requiring pauses, which is relevant to lexing and to robust token boundary detection. citeturn8view2

On the semantics‑mapping side, three features are “load‑bearing” for the engine:

Lojban’s logical connectives are explicitly systematised. The reference grammar explains that there are 16 possible truth functions and that Lojban treats four as fundamental, encoded by vowels (A/E/O/U), from which the rest are composed using negation and swapping. citeturn25view1turn25view0 This makes Lojban unusually well‑suited for compiling into canonical logical forms and for carrying out equivalence‑preserving rewrites.

Quantifiers and scope are explicitly represented via the **prenex** with `zo'u`; the reference grammar provides concrete examples that map cleanly to the quantifier prefix discipline in logic. citeturn5view1 This is a major advantage over English semantic parsing: the parser does not need to guess scope; it needs to preserve it.

Lojban also has deliberate semantic underspecification: **tanru**. The reference grammar explicitly shows that tanru grouping can lead to distinct meanings and remarks that tanru semantic ambiguity can prevent certain logical manipulations. citeturn3view2 This is a key reason the “neuro” component is not optional if the system aims to be broadly usable beyond a pedagogical FOL toy.

Finally, there is a practical implementation constraint: Lojban has special cmavo interactions and “front‑end processing” demands. The reference grammar catalogues early‑stage interactions (`zo` quoting the next word, `si/sa/su` erasure, `ZOI` delimiter quoting, `zei` compounding) and enumerates **elidable terminators** (e.g., `ku`, `kei`, `vau`, `li'u`, `ku'o`) that affect parse boundaries. citeturn31view0 This implies an architecture with (at least) a preprocessing/lexing stage before full syntactic parsing; a naive single‑pass parser will be brittle.

### Parser and grammar ecosystem

Modern Lojban parsers commonly rely on PEG grammars (camxes family). The ilmentufa project explicitly distributes multiple `.peg` grammar variants (standard and extensions). citeturn0search1turn21search3

A crucial edge case is ZOI quoting: the PEG grammar ecosystem notes that “pure PEG” cannot handle ZOI quotes without remembering the delimiter, because the closing delimiter must match the opener. citeturn21search2 This immediately biases the Rust syntax layer toward parser tooling that offers a delimiter stack.

In Rust, **pest** is a strong fit because it supports stack operations (PUSH/POP/PEEK) directly in the grammar. citeturn9search0turn9search8

### Neuro‑symbolic architectures most relevant here

Neuro‑symbolic integration is not a single design. Several primary approaches map well to Lojban’s needs:

Provenance‑semiring‑based reasoning is a key theoretical tool. The classic provenance semiring paper shows how relational computations for different semantics (e.g., probabilistic databases, bag semantics, why‑provenance) can be unified with semiring annotations. citeturn15search0 This provides a principled foundation for “soft unification” and “confidence‑carrying proofs”.

Scallop operationalises this idea for neurosymbolic programming by combining a Datalog‑style language and recursive reasoning with differentiable reasoning grounded in provenance semirings. citeturn0search2turn0search10 This is particularly close to what the provided documents propose (symbolic core + fuzzy neural layer), and it is relevant even if Scallop is not used directly, because the semiring interface becomes an internal contract.

Logical Neural Networks (LNN) frame “neural = symbolic” by making formula structure explicit and supporting differentiable inference while maintaining interpretable semantics. citeturn0search11turn0search3 Lojban’s connectives and quantifier discipline make it natural to compile parsed text into such formula graphs.

Logic Tensor Networks (LTN) introduce “Real Logic”, a many‑valued differentiable first‑order logic that can express learning and constraints uniformly via fuzzy truth values in [0, 1]. citeturn15search3turn15search15 This is a good model for handling vague predicates and tanru semantics where crisp proofs are not appropriate.

For completeness, adjacent designs such as DeepProbLog (neural predicates inside probabilistic logic programming) and NeurASP (neural outputs treated as probabilistic facts in ASP) illustrate alternative coupling patterns. citeturn15search5turn15search6

### Rust ecosystem foundations for high‑performance symbolic systems

Rust’s core value proposition here is not just speed but **memory safety**, explicit ownership, and strong concurrency primitives grounded in the borrowing model. citeturn9search6 Modern engines will typically leverage both async and parallel computation; Tokio provides a standard async runtime with futures‑based scheduling relevant to model serving and I/O pipelines. citeturn9search10 Rayon provides data‑parallelism with a guarantee of data‑race freedom, useful for batch parsing, grounding, and some inference workloads. citeturn11search2

For performance‑critical symbolic IRs, arenas and interning matter:
- **bumpalo** documents bump allocation as a fast arena allocation strategy (pointer bump + bounds check) appropriate when bulk deallocation is acceptable. citeturn11search3  
- **id‑arena** provides id‑based arenas suitable for mutable graph‑like term structures without borrow checker entanglement. citeturn12search13  
- **lasso** provides string interning strategies, including a concurrent interner and post‑intern “resolver” modes for reduced contention. citeturn12search0turn12search8  
- **ena** provides union‑find with snapshot/rollback capabilities, which is directly relevant to unification + backtracking. citeturn12search2turn12search18  

For neural components, the ecosystem has diversified:
- **Candle** is a performance‑oriented Rust ML framework with multiple backends, including CUDA and WASM support. citeturn10search3turn10search7  
- **Burn** targets portability and performance across backends, positioning itself explicitly as a Rust deep learning framework for training and inference. citeturn10search2turn10search10  
- **ort** provides Rust bindings for ONNX Runtime for hardware‑accelerated inference and broad deployment portability. citeturn11search0turn11search4  

## Proposed architecture and system design

This section collapses the requested “proposed architecture”, “system design”, “APIs/dataflow/persistence/model serving/interoperability”, and the core engineering trade‑offs into a cohesive blueprint. The structure intentionally aligns with the provided documents’ “parser → semantic compiler → symbolic inference → neural bridge → explanations” concept, while tightening boundaries and upgrading persistence/serving choices.

### System component architecture

```mermaid
flowchart LR
  UI[Client Interfaces<br/>CLI / REPL / HTTP / gRPC / WASM] --> NORM[Normalizer<br/>Unicode+orthography<br/>token boundary & pause rules]
  NORM --> PRE[Preprocessor<br/>zo/si/sa/su<br/>ZOI/LOhU/LU<br/>zei absorption<br/>elidable terminators hints]
  PRE --> LEX[Lexer<br/>morphology & selma'o hints]
  LEX --> PARSE[Parser<br/>PEG grammar + stack<br/>(pest)]
  PARSE --> AST[Typed AST<br/>bridi/sumti/selbri<br/>connectives/quantifiers]
  AST --> SEM[Semantic Compiler<br/>type + scope + discourse]
  SEM --> LIR[Logical IR<br/>typed terms + formulas<br/>events/propositions/properties]
  LIR --> NORM2[Canonicaliser<br/>EqSat rewrites / NNF/CNF<br/>place filling & se-permutation]
  NORM2 --> REASON[Reasoning Core<br/>Datalog / Prolog / SMT hooks]
  REASON <--> STORE[(Knowledge Store<br/>RDF/SPARQL + indices<br/>Oxigraph + side K/V)]
  REASON <--> NEURO[Neuro Layer<br/>embeddings + scorers<br/>tanru disambiguation<br/>fuzzy unification]
  REASON --> EXPL[Explanation + Proof Objects<br/>proof tree + provenance<br/>Lojban regeneration]
  EXPL --> UI
```

**Why this separation is necessary (not stylistic):**
- Lojban’s early parser stages depend on **word‑level operators** such as `zo`, erasure operators `si/sa/su`, ZOI delimiters, and `zei` compounding. citeturn31view0 Treating these as “just grammar rules” often leads to brittle parsing; an explicit preprocessing stage is the safer boundary.
- ZOI quoting requires delimiter memory; PEG tooling without a stack becomes awkward or non‑compositional. citeturn21search2turn9search0
- Semantic correctness depends on strict scope and connective structure; Lojban’s truth‑functional machinery is systematic and should be preserved in IR rather than flattened early. citeturn25view1

### Dataflow for ingestion, serving, and training

```mermaid
flowchart TD
  subgraph Ingestion
    C1[Lojban corpora<br/>(BPFK / Tatoeba / texts)] --> P1[Parse + compile to IR]
    D1[Dictionary/ontology<br/>jbovlaste XML] --> S1[Load predicate schemas<br/>(place structures)]
    P1 --> KB1[Insert facts/rules + metadata]
    S1 --> P1
  end

  subgraph Serving
    Q1[User query in Lojban] --> P2[Parse + compile query IR]
    P2 --> R1[Reasoner]
    KB1[(Persistent store)] <--> R1
    R1 --> A1[Answers + proofs]
    A1 --> R3[Surface realiser<br/>Lojban output]
  end

  subgraph Training
    T1[Synthetic generator<br/>grammar + schema] --> B1[Batch parse/compile]
    B1 --> L1[Learning tasks<br/>tanru scores / embedding constraints]
    L1 --> M1[Model training<br/>Burn/Candle/ONNX]
    M1 --> NEU[Deployed scorer]
  end

  NEU --> R1
```

### APIs and interoperability

A practical design is to expose the engine through multiple stable “front doors” and one stable “core protocol”:

**Core protocol (recommended):**
- **Logical IR serialisation** via Serde (JSON for dev/interop; optional binary such as MessagePack/CBOR later). Serde is explicitly designed for efficient generic serialisation/deserialisation. citeturn11search1turn19search14
- RDF/SPARQL interoperability via Oxigraph for persistence and query portability. citeturn1search3turn1search7

**Service APIs (recommended design):**
- A synchronous CLI and REPL for research/dev.
- A server mode (HTTP via axum, or gRPC via tonic if needed). Axum’s modularity and extractor model are designed for building Rust web services. citeturn18search3turn18search7
- Optional WASM build for interactive demos and embedded use; the `wasm32-unknown-unknown` target is explicitly defined as a “minimal” wasm target suitable for web/JS environments. citeturn18search0

### Persistence and the n‑ary predicate problem

Lojban bridi are naturally n‑ary (place structures can have multiple places). Mapping them to triple stores requires reification, typically by introducing an event/proposition node and then linking role edges.

Oxigraph implements SPARQL 1.1 and RDF ingestion/serialisation formats, making it suitable as a standards‑based persistence layer. citeturn1search3turn1search7 The provided documents’ idea of using an in‑memory graph (e.g., petgraph) is still valuable for local graph algorithms, but for interoperability and persistence, a SPARQL store becomes the stronger default.

## Lojban linguistic‑to‑logical mapping and ambiguity handling

This section provides the requested mapping of Lojban constructs to symbolic representations, including examples, and covers ambiguity resolution (syntactic vs semantic).

### Core mapping: bridi, selbri, sumti → logical atoms

The reference grammar introduces bridi as a core concept in its “quick tour”, emphasising that Lojban sentences share a structural uniformity where English would split roles across nouns/verbs/adjectives. citeturn3view1

A conservative, implementation‑friendly mapping is:

- **Selbri** → predicate symbol (interned id)
- **Sumti** → term (constant, variable, description, abstraction)
- **Bridi** → atomic formula (or a structured formula if connected/negated/quantified)

Example (place‑structure padding is a compiler normalisation step, not a linguistic claim):

- Input: `mi dunda ti do`  
- IR (schematic): `dunda(mi, ti, do)`  
- If the dictionary schema says `dunda` is 3‑place, compile directly; if not all places filled, compile missing places as “unspecified” (see `zo'e` below). The concept of indefinite pro‑sumti like `zo'e` is explicitly treated as “unspecified”, and the grammar notes its quantification is context‑dependent. citeturn22view0turn23view1

### Quantifiers and scope: prenex and `zo'u`

The reference grammar describes the prenex as a quantifier‑bearing prefix separated by `zo'u` and provides examples equivalent to existential and universal quantification. citeturn5view1

Example:
- Input: `ro da zo'u da viska mi`  
- Logical reading (typical FOL): ∀x (viska(x, mi)) citeturn4view1turn5view1

Implementation implications:
- The semantic compiler should represent prenex variables using a **bound variable representation** (e.g., de Bruijn indices or “locally nameless”) to ensure alpha‑equivalence and avoid capture bugs.
- Quantifier order must be preserved exactly; do not “optimise” prenex order unless it is proven equivalence‑preserving.

### Negation: `na` vs `naku`

The reference grammar explicitly contrasts internal bridi negation `na` (before the selbri) with external negation `naku` in the prenex, and discusses transformations between them. citeturn4view0turn5view3

Example:
- Input: `mi na klama le zarci` → ¬klama(mi, le_zarci, …) citeturn4view0  
- Prenex form uses `naku` as an explicit negation boundary. citeturn4view0turn5view3

Implementation implication:
- Your IR should represent negation with explicit scope nodes; avoid string‑level rewriting.
- Support “negation boundary movement” as a controlled rewrite set (especially if you use egg/egglog normalisation), but keep it gated based on logic fragment (see open questions).

### Logical connectives: systematic truth functions

Lojban’s connective system is explicitly presented as systematic, with a basis of four vowels and compositional derivation of the remaining truth functions. citeturn25view1turn25view0

Implementation implication:
- Compile connectives into a canonical operator set (AND/OR/IFF/IMPLIES/etc.) with explicit truth‑table semantics.
- Preserve the **connective type** (sumti connective vs bridi connective vs tanru connective), because the grammar notes it is necessary for unambiguity and correct interpretation. citeturn25view1

### Discourse and anaphora: `ri` and `go'i`

Lojban’s “anaphoric pro‑sumti/pro‑bridi” mechanisms must be handled as **discourse state**, not as pure logic.

The reference grammar gives explicit rules and examples:
- `ri` repeats the last complete sumti (and avoids self‑reference pitfalls). citeturn23view1turn22view1  
- `go'i` repeats the last bridi, carrying tense and enabling “yes” answers and controlled repetition. citeturn23view2turn22view2

Implementation implications:
- Maintain a discourse stack storing: last bridi IR, last sumti IRs, named referents created via `goi`, and scope boundaries.
- During semantic compilation, resolve `ri/go'i` into explicit IR references (or into placeholders that the reasoner can resolve with a context object). The choice depends on whether you want context dependence to be visible as a first‑class “world state”.

### The hard semantic cases: tanru and xorlo

**Tanru**: The reference grammar itself highlights that tanru can shift meaning with grouping and that its semantic ambiguity is not always logically tractable. citeturn3view2 This strongly suggests:
- Represent tanru as a structured term (binary tree or n‑ary list) in the AST.
- Defer interpretation to a dedicated “tanru semantics” module that can output:
  - a ranked set of candidate logical expansions, and/or
  - a fuzzy predicate grounding used by the neuro layer.

**Xorlo**: The community’s accepted “xorlo” reforms affect the semantics of articles/gadri and thus quantificational behaviour, and documentation explicitly notes that the CLL predates acceptance and readers should consult xorlo materials. citeturn1search5turn1search1  
Implementation implication:
- Treat “xorlo compliance” as a semantic mode with explicit tests; do not silently mix CLL and xorlo semantics.

## Neuro‑symbolic integration patterns and neural components

This section addresses the requested neuro‑symbolic architectures, differentiable components, and hybrid integration patterns, and connects them to Lojban‑specific needs.

### Recommended integration strategy

A robust approach for this project is a **three‑tier neuro‑symbolic design**:

Tier A: **Exact symbolic semantics** where Lojban is exact  
This includes parsing, prenex scoping, connective compilation, terminator elision handling, and discourse‑reference mechanics (ri/go’i). The reference grammar provides the necessary formal behaviour for these mechanisms. citeturn31view0turn5view1turn23view1turn25view1

Tier B: **Probabilistic / semiring‑weighted symbolic inference** to represent uncertainty  
Use provenance semirings to attach weights to facts and derivations. citeturn15search0 This enables “fuzzy unification” and “soft proof ranking” without corrupting the symbolic proof objects.

Tier C: **Neural scorers/grounders** for tanru interpretation and lexical smoothing  
The neural layer should not be responsible for producing final truth; it should produce *scores and candidate structures* that the symbolic layer can verify or incorporate with weights.

This tiering is consistent with Scallop’s model (symbolic Datalog with differentiable provenance) citeturn0search2turn0search10 and with LNN/LTN’s idea that logical structure should remain explicit while truth can be many‑valued and learnable. citeturn0search11turn15search3

### Candidate neuro‑symbolic “bridge” mechanisms

**Provenance semiring bridge (Scallop‑style):**  
- Use a semiring where ⊗ composes confidence through conjunction and ⊕ aggregates alternative proofs. The semiring provenance foundation provides the formal justification for propagating such annotations through relational reasoning. citeturn15search0  
- This can support both non‑differentiable (e.g., Viterbi “max”) and differentiable relaxations (e.g., smooth max / log‑sum‑exp).

**Bounded truth propagation (LNN‑style):**  
- Maintain truth as bounds rather than single scalars to support open‑world reasoning. LNN explicitly targets resilience to incomplete knowledge and differentiable reasoning with interpretable formula‑structured neurons. citeturn0search11turn0search3

**Many‑valued grounding (LTN‑style):**  
- Treat predicates as neural groundings mapping tuples to truth scores in [0,1], and enforce constraints as differentiable losses. LTN frames this as Real Logic supporting end‑to‑end differentiable first‑order reasoning. citeturn15search3turn15search15

### How Lojban constructs map to neural vs symbolic responsibilities

A pragmatic separation (aligned with the provided documents’ intent) is:

- **Symbolic‑first (hard constraints):**  
  - prenex and quantifier order (`zo'u`) citeturn5view1  
  - truth‑functional connectives citeturn25view1  
  - bridi negation transformations (`na`/`naku`) citeturn5view3  
  - discourse references (`ri`, `go'i`) citeturn23view1turn23view2  
  - cmavo interactions and elidable terminators citeturn31view0

- **Neural‑assisted (soft constraints / scoring):**  
  - tanru semantic composition and candidate expansion scoring (because the grammar explicitly warns about semantic ambiguity). citeturn3view2  
  - synonymy/relatedness across selbri for fuzzy matching (commonsense bridging)  
  - disambiguation defaults under xorlo mode (since semantics can differ from CLL). citeturn1search5turn1search1  

### Neural runtime choices in Rust

A practical, production‑minded stance is:
- Prefer **ONNX Runtime via ort** when you want stable deployment across CPU/GPU/edge with a fixed model format. citeturn11search0turn11search4  
- Prefer **Candle** for lightweight Rust‑native inference/training with multi‑backend support including WASM. citeturn10search7turn10search3  
- Prefer **Burn** when you expect to implement custom differentiable operators and want a Rust‑native training story with portable backends. citeturn10search10turn10search2  

## Data, training, and evaluation plan

This section combines the requested dataset plan (including synthetic generation), training/evaluation strategies, and test suites.

### Datasets and corpora

Because Lojban corpora are modest compared to major natural languages, the key strategy is to use Lojban primarily as a *meaning representation* and rely on a blend of:
- **native Lojban corpora** for parsing/coverage and some lexical statistics,
- **parallel corpora** for semantics alignment,
- **synthetic data** for systematic coverage of grammar + reasoning tasks.

Recommended sources (with licensing considerations noted):

The BPFK text corpus provides an accessible corpus of Lojban documents and is actively updated. citeturn13search3turn14search1

Tatoeba provides downloadable sentence/translation pairs, with the project’s default licensing for text as **CC‑BY 2.0 FR**, which requires attribution. citeturn14search0turn13search0

The camxes‑py repository provides a large set of test sentences packaged in JSON and labelled GOOD/BAD/UNKNOWN, useful as a parser conformance and robustness dataset. citeturn13search1

The jbovlaste dictionary provides structured lexical/definition data and an XML export pathway; importantly, jbovlaste explicitly states its content is public domain. citeturn14search2turn13search2 This is pivotal for building a predicate schema store (place structures, glosses, role hints).

### Synthetic data generation methods

Synthetic generation is crucial for two reasons: (a) coverage of rare grammatical forms and (b) controlled reasoning benchmarks.

Recommended generators:

Grammar‑guided generator: Use the PEG grammar to generate valid syntax trees, then render surface strings. This provides test cases for elidable terminators, connective nesting, and ZOI/quotes. The generator must respect the reference grammar’s cmavo interaction rules and terminator catalogue, otherwise it will generate strings that are “valid‑looking” but violate preprocessing assumptions. citeturn31view0

Schema‑guided bridi generator: Sample predicates from jbovlaste; for each predicate, generate place‑filled bridi by sampling entity constants and optionally inserting `zo'e` for unspecified places (which the reference grammar treats as indefinite/unspecified). citeturn23view1turn14search2

Reasoning task generator: Generate paired datasets of (facts, rules, query, expected answers) in restricted fragments (Horn/Datalog). Constrain generated rules to stratified negation for tractability in Datalog engines. This supports evaluation of reasoning soundness independent of real‑world semantics.

Tanru ambiguity generator: Create tanru trees and generate multiple candidate expansions; label them using heuristics (dictionary definitions similarity, role compatibility) or via human annotation. This becomes a supervised or weakly supervised dataset for the tanru neural component.

### Evaluation metrics and test suites

A rigorous evaluation needs **separate metrics for parsing, semantic compilation, symbolic inference, and neuro‑symbolic behaviour**.

Parsing and preprocessing:
- Coverage: percentage of corpus sentences producing a parse.
- Conformance: differential testing vs reference implementations (camxes‑py / ilmentufa) on shared test suites. camxes‑py’s labelled test sentences are directly usable for this. citeturn13search1turn0search1
- Robustness: stress tests exercising interactions like `zo`, `si/sa/su`, `ZOI`, and `zei`, as enumerated in the reference grammar. citeturn31view0

Semantic compilation:
- Logical‑form exact match (where a canonical form exists, e.g., prenex + connective compilation).
- Alpha‑equivalence match for bound variables (to avoid spurious failures).
- Scope tests: prenex ordering and `na/naku` transformations should match the reference grammar’s intended interpretations. citeturn5view1turn5view3

Symbolic reasoning:
- Query answer correctness (precision/recall or exact set match).
- Proof validity: each proof object must type‑check against rule schemas.
- Performance: query latency distribution and throughput at fixed KB sizes.

Neuro‑symbolic behaviour:
- Calibration of confidence scores vs empirical correctness (e.g., expected calibration error for weighted answers).
- Tanru disambiguation: top‑k accuracy against curated labels; and downstream impact: reasoning success rate improvement when tanru disambiguation is enabled.
- Ablation tests: symbolic‑only vs neuro‑symbolic; measure both accuracy and proof stability.

## Implementation roadmap, performance engineering, and developer experience

This section merges the roadmap (with milestones/effort/risk), performance plan, profiling strategy, and developer tooling recommendations.

### Roadmap with milestones, effort, and risk analysis

Assumptions for estimates: a small core team familiar with Rust and parsing, working full time; no extraordinary external dependencies; and **scope limited initially to a decidable reasoning fragment** (Horn/Datalog + bounded extras). Target hardware and latency SLOs are unspecified, so effort is sized for CPU‑first correctness and profiling‑driven optimisation.

| Milestone | Key deliverables | Est. effort | Main risks | Mitigations |
|---|---|---:|---|---|
| Repository foundation | Workspace layout, CI skeleton, baseline CLI/REPL loop | 1–2 person‑weeks | Tooling churn | Use stable Rust, minimal deps at start |
| Preprocessing + lexing | Implement `zo/si/sa/su`, `ZOI`, `zei`, terminator scaffolding per CLL interaction list | 3–5 pw | Mis‑handling early interactions breaks all downstream parsing | Derive test cases directly from the reference grammar’s interaction list citeturn31view0 |
| PEG parsing | Port camxes PEG (or subset) to pest; add ZOI delimiter stack via pest stack | 4–8 pw | ZOI and elision corner cases | pest stack ops are first‑class citeturn9search0turn21search2; differential tests vs camxes‑py citeturn13search1 |
| Typed AST + IR | Stable enums for bridi/sumti/selbri/connectives; interning + arenas | 4–6 pw | IR redesign churn | Lock IR contracts early; use lasso + id‑arena patterns citeturn12search0turn12search13 |
| Semantic compiler v1 | Prenex (`zo'u`), connectives, `na/naku`, pro‑sumti (`ri`, `go'i`) | 6–10 pw | Semantics underspecification, xorlo divergence | Treat xorlo as explicit mode citeturn1search5; scope tests from CLL examples citeturn5view1turn23view1 |
| Symbolic reasoning core | Choice of Datalog engine; backward chaining; proof objects | 6–12 pw | Scalability + correctness | Start with Ascent or egglog citeturn10search0turn1search2; use Scryer Prolog for goal‑directed queries citeturn10search5 |
| Persistence + interoperability | Oxigraph store; event reification; import/export formats | 4–8 pw | n‑ary mapping complexity | Adopt event reification + SPARQL storage; Oxigraph supports SPARQL 1.1 citeturn1search3turn1search7 |
| Neuro layer v1 | Embedding service (Candle/ort); tanru scoring; fuzzy matching | 6–10 pw | Data scarcity for training | Lean on synthetic tanru datasets + weak supervision; use ONNX for portable inference citeturn11search0turn10search7 |
| Differentiable provenance (optional phase) | Semiring‑weighted inference; differentiable scoring | 6–12 pw | Complexity blow‑up | Start non‑differentiable; borrow theory from provenance semirings citeturn15search0 / Scallop citeturn0search2 |
| Hardening + scale | Benchmarks, profiling, concurrency, memory optimisation | 6–12 pw | Performance regressions, concurrency bugs | Criterion benchmarks citeturn9search3 + flamegraphs citeturn12search3turn12search15; structured tracing citeturn19search0 |

### Performance engineering plan

A high‑performance Rust engine needs **measurement‑first** optimisation. The plan is to establish stable benchmark suites early and use profiling tools continuously.

Benchmarks (recommended minimum set):
- **Lexer/preprocessor throughput** (MB/s, tokens/s) on corpus and synthetic inputs.
- **Parser throughput + peak memory** for representative sentences, including heavy constructs (nested connectives, many elidable terminators, ZOI quotes).
- **Semantic compilation time** per sentence and allocations per sentence.
- **Inference latency**: p50/p95/p99 query latency for fixed KB sizes; throughput under concurrency.
- **Neuro scoring latency** per call and batch performance for embedding/tanru scoring (CPU vs GPU if available).

Tooling:
- Micro‑benchmarks with Criterion for statistically robust regression detection. citeturn9search3turn9search11
- CPU profiling with `cargo flamegraph` and flamegraph‑driven iteration. citeturn12search3turn12search7
- Methodical profiling guidance (including cache analysis tools like Cachegrind/Callgrind) is consolidated in the Rust Performance Book. citeturn12search15

Memory/CPU engineering tactics (high‑confidence, Rust‑idiomatic):
- Use **bump allocation** (bumpalo) for short‑lived AST/IR objects; its design explicitly targets fast allocation via pointer bumping. citeturn11search3
- Use **string interning** (lasso) for predicate ids / cmavo / gismu to avoid repeated allocation and speed comparisons; lasso provides both single‑threaded and concurrent modes. citeturn12search0turn12search8
- Use **id‑based arenas** (id‑arena) for graph‑like term storage without lifetime coupling. citeturn12search13
- Use **snapshot‑rollback union‑find** (ena) to make unification and backtracking efficient and allocation‑light. citeturn12search2turn12search18
- Use **Rayon** for CPU parallelism in batch tasks; it guarantees race‑free execution and supports parallel iterators. citeturn11search2

SIMD considerations:
- Rust offers vendor intrinsics via `std::arch` (portable SIMD remains nightly/experimental). citeturn17search3turn17search0  
Given this, SIMD should be treated as an optional, later optimisation for hotspots such as vector similarity computation; prefer BLAS‑like kernels inside ML runtimes when possible.

### Developer experience and tooling

A research‑grade engine benefits from first‑class introspection:
- Structured logging/instrumentation with `tracing`, which is explicitly designed for async systems where traditional logs interleave. citeturn19search0turn19search8
- A REPL using rustyline (readline‑like editing) for interactive experiments. citeturn19search5turn19search13
- Visualisation: emit proof graphs and ASTs as DOT/GraphViz and optionally Mermaid for documentation, plus JSON dumps via Serde. citeturn11search1turn19search14

## Deployment, CI/CD, legal/ethical considerations, and open questions

This final section consolidates the requested deployment guidance, CI/CD recommendations, legal/ethical issues, and the list of unspecified assumptions/open questions.

### Deployment options

Containers:
- Use multi‑stage builds to minimise runtime images and separate build dependencies from the final artefact; Docker documents multi‑stage builds as a best practice for reducing final image size and producing cleaner outputs. citeturn18search10  
- For model artefacts, either bake models into the image (simpler) or mount them at runtime (more flexible for updates).

WASM:
- The `wasm32-unknown-unknown` target is a “minimal” WebAssembly target intended for web/JS environments. citeturn18search0
- For edge ML inference, WASI‑NN is a standardisation track for ML inference from WASM modules. citeturn18search1  
- The ort documentation notes the relationship between ONNX inference and WASI‑NN runtimes (e.g., Wasmtime). citeturn11search8  
Practical recommendation: **WASM builds should prioritise parsing + symbolic reasoning first**, and treat neural inference as optional (or use WASI‑NN where available).

Cloud GPUs:
- If GPU inference is needed, prefer ONNX Runtime (via ort) for broad accelerator support and deployment flexibility. citeturn11search0turn11search4  
- Alternatively, Candle provides GPU backends and supports model execution in multiple environments, including WASM. citeturn10search7

### CI/CD recommendations

A minimal but rigorous CI/CD pipeline:
- Formatting + linting (`rustfmt`, `clippy`) on every PR.
- Unit tests for lexer/preprocessing, parser, semantic compiler, reasoning, and corpus regression suites.
- Differential tests vs camxes‑py sentence suite where feasible. citeturn13search1
- Criterion benchmarks as a gated (or nightly) job to detect performance regressions. citeturn9search3
- Optional: build matrix for native + wasm targets (where features permit). citeturn18search0

### Legal and ethical considerations

Licensing of resources:
- jbovlaste explicitly states its content is public domain, which simplifies schema/dictionary integration. citeturn14search2  
- Tatoeba’s text sentences are under the default CC‑BY 2.0 FR licence (attribution required), and audio has independent licensing constraints. citeturn14search0turn14search14  
- The BPFK corpus aggregates many texts; each document may have its own provenance and licensing expectations. citeturn13search3  
The system should track per‑document provenance and include attribution tooling for exported datasets.

AI ethics:
- The system’s symbolic core supports explainability via proof objects; this can reduce “hallucinated reasoning” risks compared to pure neural systems, but only if the pipeline clearly separates **proved** facts from **scored/heuristic** suggestions (tanru interpretation, fuzzy matching).
- If deployed in user‑facing systems, store and expose evidential/provenance metadata (source corpora, confidence semiring, or proof trace) alongside answers.

### Open questions and unspecified assumptions

The following are critical design inputs that are **unspecified by the prompt and documents** (and must be decided for an implementation to converge):

Target hardware for production runs is unspecified (CPU‑only, GPU, edge devices, WASM/browser).

Latency and throughput constraints are unspecified (interactive REPL vs high‑QPS API serving; expected KB size; concurrent users).

Exact “logic scope” is unspecified:
- Will the engine be limited to Datalog/Horn clauses (decidable, scalable)?
- Will it support richer FOL fragments (risking undecidability/timeouts)?
- Will it integrate modal/temporal reasoning (Lojban supports tense and other modalities, but full modelling scope is a separate research project)?

Xorlo compliance level is unspecified; xorlo materially changes gadri semantics and must be treated as an explicit mode with tests. citeturn1search5turn1search1

Tanru semantics policy is unspecified:
- Which default interpretation strategies are chosen?
- What is the neural disambiguation target (select one reading vs maintain a distribution)?
The reference grammar itself warns about tanru semantic ambiguity, making this a first‑class research and product decision. citeturn3view2

Knowledge representation choice is partially open:
- RDF/SPARQL via Oxigraph is recommended for interoperability, but whether to also maintain an internal Datalog store (Ascent/egglog) as the “fast path” is a strategic choice. citeturn1search3turn10search0turn1search2

Differentiability requirement is unspecified:
- If end‑to‑end differentiability is required, Scallop/LTN/LNN‑style designs become more central. citeturn0search2turn15search3turn0search11  
- If not, a semiring‑weighted but non‑differentiable pipeline may be simpler and more robust in Rust.

If these open items are clarified, the architecture can be narrowed to an implementable “v1 target” with sharper performance budgets and a bounded semantics contract.