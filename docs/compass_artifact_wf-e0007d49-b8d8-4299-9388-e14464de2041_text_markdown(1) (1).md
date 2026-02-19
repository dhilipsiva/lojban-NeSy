# Building a Lojban-native symbolic inference engine in Rust

**Lojban is uniquely suited to serve as the input language for a symbolic reasoning engine because its grammar was designed from first principles as applied first-order predicate logic (AFOPL)**. Every Lojban sentence (bridi) is already an atomic well-formed formula; its predicate words (selbri) have fixed arity with defined place structures; and its connectives implement all 16 binary truth functions. No other language — natural or constructed — offers this direct a mapping from surface syntax to inference-ready logical forms. The critical gap, identified by Ben Goertzel in 2013 and still unfilled, is that **no software performs formal semantic interpretation of Lojban expressions** — parsing exists, but the parse-tree-to-logic bridge does not. This report lays out the architecture, tooling, and specific Rust implementation strategies to build that bridge and the inference engine behind it.

---

## The Lojban grammar is PEG-native and machine-parseable at every layer

The official Lojban grammar exists in three formalizations, each with distinct strengths. The **3rd Baseline YACC grammar** (grammar.300, dated January 1997) is the authoritative specification: approximately **600+ production rules** across the main grammar (rules 1–899) and lexer rules (900–1099), with ~100 terminal categories called *selma'o*. The **EBNF grammar** in CLL Chapter 21 restates these rules in human-readable form. The **PEG grammar**, created by Robin Lee Powell for the camxes parser, is now the de facto standard used by all modern parsers and represents the most complete formalization, uniquely integrating morphology and syntax into a single grammar.

A key architectural fact: Lojban parsing is not single-pass. The official pipeline requires **four preprocessing steps** before structural parsing — lexical filtering (handling quote-words ZO, ZOI, LOhU), erasure processing (SI, SA, SU), absorption (merging ZEI compounds, absorbing indicators), and lexer-lexeme insertion (collapsing token strings whose grammar role depends on following context). The YACC grammar explicitly notes that **Lojban is not inherently LALR(1)**, which is precisely why the PEG approach works better — PEG's ordered choice and unlimited lookahead handle elidable terminators and context-dependent constructs that break CFG parsers. All grammar files are explicitly **public domain**.

The parser ecosystem spans seven implementations across five languages. **ilmentufa** (55 stars, last updated January 2025) is the most actively maintained, providing five PEG grammar variants and powering the community's online parser. **camxes-rs** (github.com/lojban/camxes-rs, updated April 2025) is the only Rust implementation but appears minimal. The MarkMcCaskey/zantufa Rust experiment explicitly notes difficulty: "It seems I either need to write my own PEG library or improve one of the existing ones." **camxes-py** is well-tested against 22,000+ sentences and provides a good reference for expected parse behavior. The original camxes (Java/Rats!) and the C-based jbofi'e (which outputs Prolog terms directly) round out the ecosystem.

## pest.rs is the right parser generator, but expect a substantial AST layer

**pest.rs is the most natural fit** for this project because the reference Lojban grammar is already in PEG format, enabling near-direct translation to `.pest` files. pest's scannerless parsing matches Lojban's unified morphology-syntax design. Its external grammar file approach parallels the existing `.peg` files. And since the camxes PEG grammar already avoids left recursion, pest's fundamental PEG limitation is irrelevant here.

However, pest has three significant limitations for a grammar this large. First, it generates a **flat `enum Rule`** with all rules as variants — you get untyped `Pairs<Rule>` iterators, not a typed AST. For a grammar with hundreds of rules, the manual match-arm boilerplate to construct typed AST nodes is enormous. Second, **compile times scale with grammar size** because the proc macro processes the entire grammar during compilation; expect multi-second builds for a 500+ rule grammar. Third, pest has **no built-in error recovery** equivalent to YACC's `error` token — elidable terminator insertion will require custom logic layered on top.

The recommended mitigation is a **two-phase tokenizer-parser architecture**: use **logos** (an extremely fast lexer generator) to classify Lojban words into selma'o categories and handle preprocessing (magic words, absorption), then feed the token stream into pest for structural parsing. This separates the messy preprocessing from the clean grammar. An alternative worth benchmarking is **faster-pest**, a drop-in replacement claiming 7× speedup over standard pest for equivalent grammars.

No existing pest-based Lojban grammar exists anywhere. The closest Rust precedent (camxes-rs) uses unknown internals, and the zantufa Rust experiment stalled on PEG library limitations. **You will be building the first production-quality Rust Lojban parser.**

| Parser Tool | Grammar Type | Typed Output | External Grammar | Best For |
|---|---|---|---|---|
| **pest** | PEG | ❌ Flat enum | ✅ `.pest` file | Direct PEG translation |
| **nom** | Combinators | ✅ Custom types | ❌ Code-based | Performance-critical paths |
| **rust-peg** | PEG | ✅ Custom types | ❌ Inline macro | Smaller, typed grammars |
| **lalrpop** | LR(1) | ✅ Custom types | ✅ `.lalrpop` file | Left-recursive grammars |
| **tree-sitter** | GLR | ❌ Generic tree | ✅ External | Incremental/editor parsing |

## Lojban maps to first-order logic with lambda extensions

The mapping from Lojban to formal logic is remarkably direct — arguably the cleanest natural-language-to-logic mapping that exists. At the core level, `ko'a broda ko'e` maps to `broda(ko'a, ko'e)`. Each of Lojban's **~1,300 root words (gismu)** is a predicate with fixed arity (1–5 places), and the dictionary defines each place's semantic role. Unfilled places are implicitly existentially quantified: `mi klama` means `∃x₂∃x₃∃x₄∃x₅. klama(mi, x₂, x₃, x₄, x₅)`.

**Quantification** uses an explicit prenex separated by `zo'u`, directly mirroring FOL prenex normal form. The variables `da`, `de`, `di` serve as logical variables (x, y, z), and quantifiers bind them: `ro da su'o de zo'u da prami de` unambiguously means `∀x∃y. prami(x, y)`. Scope follows strict left-to-right order in the prenex — eliminating the scope ambiguities that plague English ("Every student read a book"). Numeric quantifiers (`pa` = exactly 1, `re` = exactly 2) extend beyond standard FOL into counting quantifiers. Vague quantifiers (`so'e` = most, `so'i` = many) require fuzzy or probabilistic extensions.

**Connectives** implement all **16 binary truth functions** through a combinatorial system. Four base vowels encode four connectives: **A** = OR (∨), **E** = AND (∧), **O** = IFF (↔), **U** = regardless-of. Prefixing `na` negates the first operand; suffixing `nai` negates the second; `se` swaps operands. This generates all 14 non-trivial truth functions (TTTT and FFFF excluded as trivially true/false). Six grammatical variants handle different syntactic positions (ek for sumti, jek for tanru, gihek for bridi-tails, etc.), but all share the same logical semantics. This system is **complete** — any propositional logic expression is expressible.

**Negation** operates at three distinct levels. Bridi negation (`na`) is propositional NOT, negating the entire claim. Scope-sensitive negation (`naku`) can be placed at any position and interacts with quantifiers via De Morgan's laws: `naku ro da` ↔ `su'o da naku` (¬∀x ↔ ∃x¬). Scalar negation (`na'e`, `to'e`) is metalinguistic, meaning "other than" or "opposite of" — this does *not* map to logical NOT and should be handled differently in the inference engine (perhaps as a constraint on the predicate's value space).

**Abstraction** pushes Lojban beyond pure FOL into higher-order territory. The abstractor `ka` combined with the placeholder `ce'u` implements **lambda abstraction**: `lo ka ce'u xunre` = `λx.red(x)` = "the property of being red." Multiple `ce'u` create multi-argument functions. The abstractor `du'u` creates propositional objects (reified propositions that can be arguments to predicates like `djuno` = "know"). The abstractor `nu` creates event objects (Davidson-style event semantics). The abstractor `jei` yields **fuzzy truth values in [0,1]**, providing native support for graded reasoning. These abstractions require the logical form representation to support at minimum lambda terms and reified propositions.

## The AST must bridge three representations

The transformation pipeline from raw text to inference-ready clauses passes through three distinct intermediate representations, each serving a different purpose:

**Stage 1: Concrete Syntax Tree (CST)** — preserves every grammatical detail including elidable terminators (`ku`, `vau`, `kei`), parenthetical markers, and indicator attachment. This is what pest produces directly.

**Stage 2: Typed Abstract Syntax Tree (AST)** — strips grammatical machinery, retaining only semantically meaningful structure. In Rust, this should use algebraic data types:

```rust
enum Bridi {
    Simple { selbri: Selbri, sumti: Vec<(PlaceTag, Sumti)>, modal: Vec<Modal> },
    Connected { connective: LogicalConnective, left: Box<Bridi>, right: Box<Bridi> },
    Negated(Box<Bridi>),
    Quantified { quantifier: Quantifier, variable: Variable, scope: Box<Bridi> },
}

enum Selbri {
    Gismu(Symbol),                              // root predicate (interned)
    Tanru(Box<Selbri>, Box<Selbri>),            // modifier-head compound
    Se(u8, Box<Selbri>),                        // place permutation
    Abstraction { kind: AbstractionKind, inner: Box<Bridi> },
}

enum Sumti {
    Constant(Symbol),                           // named entities (la .djon.)
    Variable(VarId),                            // da, de, di
    Description { gadri: Gadri, inner: Box<Selbri>, quantifier: Option<Quantifier> },
    LambdaVar,                                  // ce'u placeholder
}
```

**Stage 3: Logical Form** — a representation suitable for inference, with explicit quantifiers, connectives as logical operators, and abstractions as lambda terms:

```rust
enum LogicalForm {
    Predicate { name: Symbol, args: Vec<Term> },
    And(Box<LogicalForm>, Box<LogicalForm>),
    Or(Box<LogicalForm>, Box<LogicalForm>),
    Not(Box<LogicalForm>),
    Implies(Box<LogicalForm>, Box<LogicalForm>),
    ForAll(VarId, Box<LogicalForm>),
    Exists(VarId, Box<LogicalForm>),
    Lambda(VarId, Box<LogicalForm>),
    Apply(Box<LogicalForm>, Term),
    Equals(Term, Term),
}
```

The AST→LogicalForm transformation is where **Montague-style compositional semantics** applies: each syntactic rule is paired with a semantic rule that builds the logical form bottom-up. Lojban's transparent syntax-semantics interface makes this dramatically simpler than for English — there are no syntactic ambiguities to resolve, and place structures are explicit predicates with known arity.

For inference-ready clauses, the logical form undergoes standard normalization: eliminate implications → push negation inward (NNF) → Skolemize existentials → distribute disjunction over conjunction (CNF) → separate into individual clauses. The **egg** crate (equality saturation via e-graphs) is ideal for implementing logical equivalences (De Morgan's laws, double negation, quantifier transformations) as rewrite rules, letting the engine find canonical forms automatically.

## The inference engine should follow Chalk's three-layer architecture

The most actionable architectural pattern comes from **Chalk**, the Rust trait solver. Chalk separates three concerns: (1) a **domain layer** that thinks in domain terms (for us, Lojban predicates and bridi), (2) a **lowering layer** that converts domain concepts into logic clauses, and (3) a **generic solver** that knows nothing about the domain. This pattern is directly applicable:

- **Layer 1 (Lojban Domain)**: Parses Lojban text, maintains the dictionary of selbri place structures, handles tanru interpretation, manages the gismu/lujvo vocabulary.
- **Layer 2 (Lowering)**: Transforms Lojban AST nodes into logical clauses. Each bridi becomes one or more Horn clauses (or FOHH clauses if using Chalk-style extended logic). Abstractions are lowered to lambda terms. Evidentials become metadata annotations on clauses.
- **Layer 3 (Generic Solver)**: Performs unification (via `ena`-style union-find), SLD resolution with backtracking, clause indexing, and proof search. This layer is entirely reusable across domains.

For the solver itself, the key implementation decisions are:

**Unification** should use the `ena` crate's union-find algorithm, extracted from the Rust compiler itself. Variables are represented as newtyped integer keys (cheap `Copy` types), stored in a `UnificationTable` that supports **snapshotting for backtracking** — when a choice point fails, roll back the snapshot to undo all bindings. This avoids the ownership/borrowing conflicts that arise with pointer-based term representations.

**Backtracking** is best implemented with an explicit choice-point stack rather than recursive calls or continuations. Each `ChoicePoint` stores: the current goal list, a unification table snapshot, and the remaining untried clauses. On failure, pop the stack and restore the snapshot. This is the most Rust-idiomatic approach and mirrors the Warren Abstract Machine's (WAM) architecture.

**Clause indexing** via first-argument hashing is essential for performance — store a `HashMap<Symbol, Vec<ClauseIndex>>` that maps the principal functor of each clause's head to matching clauses. This turns linear clause scanning into O(1) lookup for the common case.

## Five Rust engines provide reusable components and architectural lessons

**Scryer Prolog** (github.com/mthom/scryer-prolog, 2,400+ stars) is the most complete WAM implementation in Rust — a full ISO Prolog with SLG resolution, CLP(ℤ), and a packed UTF-8 string representation that yields **24× memory reduction** for string processing. Its architecture is the gold standard reference for WAM-in-Rust, and its DCG (Definite Clause Grammar) support maps naturally to Lojban's grammar. However, embedding Scryer directly may be overkill; studying its source for WAM patterns is more valuable than depending on it.

**Ascent** (s-arash.github.io/ascent/) stands out as the most capable Datalog engine for this use case, offering **lattice support** (critical for modeling Lojban's graded predicates and fuzzy truth values via `jei`), **parallel execution** via Rayon, stratified negation, and built-in aggregation. It's backed by academic publications at CC and OOPSLA conferences. For the materialized knowledge base layer — pre-computing forward-chained inferences — Ascent is the strongest candidate.

**egg/egglog** (egraphs-good.github.io/) provides equality saturation, a technique where rewrite rules are applied exhaustively to find all equivalent forms of an expression. For Lojban, this means defining logical equivalences as rewrite rules and letting egg discover canonical forms. egglog extends this with Datalog-style rules, creating a system that can express both equality reasoning and horn-clause inference. Published at PLDI 2023, this is production-quality infrastructure.

**Datafrog** (3.6M+ crate downloads) is the lightweight option — a zero-dependency Datalog engine using leapfrog triejoin, originally built for the Rust borrow checker (Polonius). It excels at join-heavy fixed-point computations but lacks top-down reasoning, negation, and aggregation.

**Oxigraph** (github.com/oxigraph/oxigraph) is a Rust-native SPARQL-compatible graph database with RocksDB storage, ACID transactions, and full SPARQL 1.1 support. For persistent knowledge storage — storing Lojban assertions as RDF-like quads with SPARQL query support — Oxigraph provides a ready-made solution. Lojban's n-ary predicates (up to 5 places) require reification or n-ary relation patterns rather than simple triples, but this is a solved problem in RDF modeling.

## Evidentials create a built-in epistemic provenance system

Lojban's **11 evidential markers** are architecturally significant because they provide a native, grammatically integrated system for tracking *how* knowledge was acquired — a feature that large-scale reasoning systems like Cyc have had to engineer separately. The evidentials map naturally to epistemic logic operators and can drive confidence-weighted inference:

`ca'e` (I define) functions as an **axiom marker** — assertions tagged with it should have confidence 1.0 and serve as definitional truths. `za'a` (I observe) marks **empirical observations** with high but not absolute confidence. `ja'o` (I conclude) marks **deductive inferences** whose confidence inherits from premises. `su'a` (I generalize) marks **inductive generalizations** with moderate confidence. `ti'e` (I hear/hearsay) marks **testimonial evidence** with lower confidence. `ru'a` (I postulate) marks **assumptions** that can be retracted.

The inference engine should track evidentials as **metadata on assertions**, forming a provenance graph that answers "why does the system believe X?" Each fact in the knowledge base carries its evidential source, and confidence propagates through inference chains. The `ju'o` certainty modifier (certain / uncertain / impossible) provides a native three-value epistemic scale that can map directly to probability ranges.

Crucially, Lojban's grammar **formally separates** propositional content (bridi) from non-logical annotations (attitudinals and evidentials). Attitudinals (emotional markers like `.ui` = happiness) are parsed but routed to a separate pragmatic layer. This clean architectural boundary — already present in the grammar — means the inference engine never confuses logical content with metalinguistic commentary.

## Lessons from SUMO, Cyc, and ACE inform the knowledge base design

Three systems provide directly applicable architectural lessons. **Cyc** (25+ million axioms, 40,000+ predicates) demonstrates that a single monolithic knowledge base is unmanageable — its solution is **microtheories**, named contexts that must be internally consistent but can disagree with each other. A Lojban engine should partition assertions into contexts from the start. Cyc also uses **1,100+ specialized inference modules** rather than a single algorithm, suggesting that different Lojban constructs (tanru interpretation, abstraction reduction, quantifier reasoning) should have dedicated inference strategies.

**SUMO** (Suggested Upper Merged Ontology) demonstrates the value of mapping between a natural language lexicon and formal predicates. SUMO manually maps all ~100,000 WordNet synsets to ontological terms. For Lojban, the gismu dictionary with its defined place structures is already a mini-ontology — **each gismu entry is effectively an axiom schema** defining a predicate's arity and semantic roles. A working Lojban engine should load the entire gismu dictionary as its base ontology.

**Attempto Controlled English (ACE)** proves that a controlled-language-to-logic pipeline works in practice, converting restricted English to Discourse Representation Structures and then to Prolog/OWL. Lojban has a massive structural advantage: where ACE must fight English's ambiguity by restricting what speakers can say, **Lojban was designed to be unambiguous from the start**. The entire language is already "controlled" in the CNL sense. ACE's architecture (DCG parser → DRS → logic) maps cleanly to the proposed Lojban pipeline (PEG parser → AST → logical form → clauses).

A seminal but little-known document — a Lojban-to-Prolog semantic analyser described at lojban.org/files/papers/lojban_parser_paper — demonstrates a working (if limited) Lojban→Prolog conversion. It converts bridi to Prolog terms (`denpa(mi, X, Y)` for "I wait"), maps `da`/`de`/`di` to Prolog variables, and performs question answering through formal manipulation. Additionally, the **OpenCog project** explored using Lojban as an intermediate representation for mapping English parse trees to logical forms (Atomese), noting that Lojban's predicate structure makes the NL→logic mapping "far more transparent than with English."

## Memory management demands arenas and interning

Symbolic computation in Rust requires specific memory management patterns to avoid death by a thousand allocations. **Arena allocation** using `bumpalo` or `typed-arena` is essential — AST nodes, logical terms, and proof objects are created in bursts and discarded together. Arena-based allocation with 100K unique strings requires **18 allocations** compared to ~100,000 allocations for naive `Arc<str>` — a 5,000× reduction with corresponding performance gains.

**String interning** is equally critical. Lojban has ~1,300 gismu and ~600 cmavo — a finite, known vocabulary. The **lasso** crate provides fast concurrent string interning, returning `u32` keys (`Spur` type) for O(1) comparison. The **string_cache** crate from Servo supports compile-time atom generation for known strings, ideal for pre-interning the entire Lojban vocabulary at compile time. All predicate comparisons in the inference engine then become integer comparisons rather than string comparisons.

The recommended crate stack for the complete system:

- **logos** → fast lexer for Lojban word classification and preprocessing
- **pest** (or **faster-pest**) → PEG parser translating the camxes grammar
- **lasso** or **string_cache** → symbol interning for all Lojban vocabulary
- **bumpalo** → arena allocation for parse trees and inference terms
- **ena** → union-find for unification with snapshot-based backtracking
- **egg/egglog** → equality saturation for logical normalization and term rewriting
- **ascent** → Datalog with lattices for materialized forward-chaining inference
- **petgraph** → in-memory knowledge graph with DOT export for visualization
- **oxigraph** → persistent SPARQL-compatible knowledge store
- **serde** + **bincode** → fast serialization of the knowledge base
- **tracing** → structured logging for inference step traces and proof trees

## Conclusion

The proposed architecture decomposes into five distinct subsystems: a **two-phase parser** (logos lexer → pest PEG parser), a **typed AST with compositional semantic analysis** (Montague-style lowering to logical forms), an **egg-powered normalizer** (equality saturation for canonical form discovery), a **hybrid inference engine** (WAM-style backward chaining for queries + Ascent-powered forward chaining for materialization), and an **evidential knowledge base** (Oxigraph-backed persistent storage with microtheory partitioning and provenance tracking).

The single most important insight is that **Lojban's grammar already does 80% of the work** that NLP systems spend enormous effort on. There is no word sense disambiguation (each word has one meaning in context). There is no syntactic ambiguity (every sentence has exactly one parse). There is no scope ambiguity (prenex linearizes all quantifiers). The remaining 20% — tanru interpretation (compound predicate semantics), implicit quantifier handling for unfilled places, and pragmatic inference — are well-defined problems with known solution approaches. What has been missing is not theory but engineering: actually building the semantic mapping pipeline. The Rust ecosystem now provides every component needed to do so.