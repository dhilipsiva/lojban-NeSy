# A Lojban-native neuro-symbolic reasoning engine in Rust is feasible — and timely

**Building a general-purpose neuro-symbolic reasoning engine on Lojban in Rust is not only feasible but architecturally compelling.** Lojban's grammar is formally isomorphic to predicate logic — every sentence (bridi) is a predicate-argument frame, every connective encodes one of the 16 truth functions, and quantifiers, modality, and tense map directly to their logical counterparts. The Rust ecosystem now provides strong individual building blocks (Datalog engines, PEG parsers, knowledge graph stores, ML frameworks), though the critical neuro-symbolic integration layer must be built from scratch. This gap is precisely the research opportunity: no existing system combines a human-speakable logical language with differentiable symbolic reasoning in a systems language. The closest precedent — OpenCog's partial Lojban-to-Atomese pipeline and the Rust-based Scallop neurosymbolic language — validates the approach without preempting it.

---

## Lojban's grammar is predicate logic in disguise

Lojban's core structure maps to formal logic with almost no translation loss. A **bridi** (sentence) is a proposition. A **selbri** (predicate word) is a relation. **Sumti** (arguments) fill typed argument slots. The sentence `mi tavla do la .lojban.` parses directly to `tavla(speaker, listener, topic)` — a predicate with three arguments, immediately representable as a knowledge graph triple or n-ary relation.

This is not a loose analogy. Lojban implements **all 16 binary truth functions** through a systematic vowel-coding scheme: `je` (AND), `ja` (OR), `jonai` (XOR), `jo` (IFF), `ju` (whether-or-not). Negation affixes `na-` and `-nai` modify truth tables compositionally, so `na.a.nai` yields material implication. The language provides distinct connective forms for joining sumti (`.e`, `.a`), selbri (`je`, `ja`), bridi-tails (`gi'e`, `gi'a`), and sentences (`.ije`, `.ija`) — eliminating the syntactic ambiguity that plagues natural-language logic extraction. Quantifiers follow first-order logic exactly: `ro` (∀), `su'o` (∃), numeric quantifiers (`pa`, `re`, `ci` for exactly 1, 2, 3), plus generalized quantifiers (`so'e` for "most"). The **prenex** construction `ro da su'o de zo'u da prami de` maps directly to ∀x ∃y: loves(x,y), with `zo'u` explicitly separating quantifier scope from the proposition body.

Three features make Lojban especially powerful as a reasoning substrate beyond basic FOL. **Temporal operators** (`pu`, `ca`, `ba` for past/present/future; `pu'o`, `ca'o`, `ba'o` for event phases) map to temporal logic. **Evidentials** (`za'a` for direct observation, `pe'i` for belief, `ti'e` for hearsay, `ja'o` for inference) encode epistemic provenance — critical for explainable AI. **Modal operators** (`ka'e` for inherent capability, `ei` for deontic obligation) extend into modal logic. A Lojban statement can simultaneously encode what is claimed, when it holds, how the speaker knows it, and with what certainty — metadata that symbolic reasoners can exploit for trust propagation and explanation generation.

The **~1,300 root words (gismu)** each define a typed relation template with 2-5 argument places, forming a ready-made ontology. The word `klama` encodes `goes(agent, destination, origin, route, vehicle)` — a 5-ary relation decomposable into knowledge graph edges. Compound words (lujvo) extend this compositionally. Abstractions (`nu` for events, `ka` for properties, `du'u` for propositions, `si'o` for concepts) enable reification — treating relations as arguments to other relations — which is essential for higher-order knowledge representation and meta-reasoning.

---

## Parsers exist and a Rust port is already underway

The Lojban community maintains multiple machine-parseable grammar specifications. The canonical format is **PEG (Parsing Expression Grammar)**, with the reference file `lojban.peg` in the camxes repository. The **ilmentufa** parser (55 GitHub stars, actively maintained through 2025) provides five PEG grammar variants compiled to JavaScript, covering the official BPFK grammar plus experimental extensions. The older **jbofi'e** parser used YACC/Bison based on the official 3rd Baseline LALR grammar from the CLL specification.

Critically, **a Rust parser already exists**: `camxes-rs` (github.com/lojban/camxes-rs, last updated April 2025) implements the Lojban PEG grammar in Rust. While small (4 stars), it provides a working foundation. The PEG grammar format maps directly to **pest**, the most popular Rust PEG parser generator (~4,500 stars, production-ready). Translating the ilmentufa PEG grammars to pest `.pest` files is a mechanical process — the grammar formalisms are nearly identical. This is the recommended path:

- **Parse layer**: pest-based PEG parser translating existing Lojban grammar files
- **AST layer**: Custom Rust types representing bridi, selbri, sumti, connectives, quantifiers, tense, and evidentials
- **Logic layer**: AST-to-formal-logic compiler producing first-order logic with modal/temporal/epistemic extensions
- **Reference**: camxes-rs as starting point; ilmentufa's `camxes.peg` as grammar source

One challenge is **grammar fragmentation**. The official 3rd Baseline (1997) is stable but aging. The community uses experimental extensions (ce-ki-tau connectives, cmevla-brivla merger) with different parser variants. The Relojban fork attempts cleaner codification. A pragmatic approach: implement the standard BPFK grammar first, design the parser to be grammar-swappable via pest grammar files.

---

## The neuro-symbolic landscape reveals clear architectural patterns

Six major systems define the current state of the art, each demonstrating a different integration strategy between neural and symbolic reasoning.

**IBM's Logical Neural Networks (LNN)** achieve the strongest interpretability: every neuron corresponds to a logical proposition, with truth values expressed as bounds [L, U] ⊆ [0,1] supporting open-world reasoning under uncertainty. Bidirectional inference (both modus ponens and modus tollens) propagates constraints through the formula graph. **Scallop** (PLDI 2023, University of Pennsylvania) is particularly relevant — it is **written in Rust** with Python bindings and implements neurosymbolic Datalog with configurable provenance semirings that balance accuracy against scalability. **DeepProbLog** extends probabilistic logic programming with neural predicates, compiling ground programs into Sentential Decision Diagrams for efficient differentiable inference. **Logic Tensor Networks** ground first-order logic in tensor spaces using fuzzy t-norms, making all reasoning differentiable. **NeurASP** connects neural network outputs to Answer Set Programming solvers, leveraging ASP's non-monotonic reasoning and constraint language. **AlphaGeometry** (DeepMind, Nature 2024) demonstrates the power of loose coupling: a symbolic deduction engine handles rigorous proof while a transformer suggests auxiliary constructions when the symbolic engine gets stuck.

The key architectural lesson across these systems is that **the integration layer is the hard part and the research contribution**. Three dominant patterns emerge:

- **Probabilistic bridging**: Neural outputs become probability distributions over symbolic atoms (DeepProbLog, NeurASP, Scallop). Gradients flow through probabilistic reasoning back to neural parameters.
- **Fuzzy/continuous relaxation**: Logical operators are replaced with differentiable approximations — t-norms for conjunction, t-conorms for disjunction, generalized means for quantifiers (LTN, LNN). This preserves gradient flow but introduces approximation error.
- **Loose coupling with iterative handoff**: Symbolic and neural components operate independently, calling each other when stuck (AlphaGeometry). Simpler to implement, easier to debug, but no end-to-end gradient flow.

For a Lojban-based system, **LNN's architecture is the most natural fit** because Lojban sentences already have a 1-to-1 correspondence with logical formulae. Each parsed bridi maps to an LNN-style neuron with clear symbolic semantics. Scallop's Rust implementation and provenance semiring approach provide the most direct engineering reference.

---

## Rust provides strong building blocks but no integrated framework

The Rust ecosystem assessment reveals **mature individual components** with a **critical integration gap** — exactly the space this project would fill.

For the **symbolic reasoning core**, three options stand out. **ascent** (the most actively developed Datalog crate) supports lattices, parallel execution via rayon, and custom backing data structures — published at OOPSLA. **egglog** (PLDI 2023) uniquely combines Datalog with equality saturation, enabling both relational querying and equational term rewriting in a single framework. **Nemo** (TU Dresden, KR 2024) provides a scalable in-memory rule engine with native RDF/SPARQL I/O, handling **10⁵–10⁸ facts** on commodity hardware. For full first-order logic with backtracking, **scryer-prolog** (~2,371 stars) implements ISO Prolog on the Warren Abstract Machine in Rust.

For **knowledge graph storage**, **Oxigraph** implements SPARQL 1.1 with RocksDB persistence, Turtle/N-Triples/RDF-XML support, and WASM compilation capability. Combined with Nemo's reasoning, this provides a complete knowledge graph stack.

For the **neural component**, the recommended layered approach uses **candle** (Hugging Face, ~16K stars) for native Rust inference of transformer and embedding models, **ort** for deploying pre-trained ONNX models with hardware acceleration, and **burn** (~13.2K stars) for any Rust-native training with its multi-backend architecture (WGPU, CUDA, Metal, ROCm).

| Component | Recommended Crate | Maturity | Notes |
|---|---|---|---|
| Lojban parsing | pest + camxes-rs reference | ★★★★★ | Direct PEG grammar translation |
| Datalog reasoning | ascent or egglog | ★★★★☆ | egglog adds equality saturation |
| Prolog/unification | scryer-prolog | ★★★☆☆ | Full WAM, active development |
| Knowledge graphs | Oxigraph + Nemo | ★★★★☆ | SPARQL 1.1, scalable reasoning |
| Neural inference | candle + ort | ★★★★☆ | Transformers, ONNX support |
| Neural training | burn | ★★★★☆ | Multi-backend, autodiff |
| Neuro-symbolic bridge | **Must build** | ☆☆☆☆☆ | **The core research contribution** |

---

## Proposed architecture for the Lojban reasoning engine

The system architecture follows a six-layer design where Lojban serves as both the input language and the internal representation language — a "Lojban-native" approach where the logical structure is preserved end-to-end rather than translated away.

**Layer 1 — Lojban Parser** (pest PEG). Ingests Lojban text and produces a typed AST preserving all logical structure: bridi, selbri with place structures, sumti with quantifiers, connectives with truth-function types, tense/evidential/modal annotations. Grammar files are swappable. The existing `camxes-rs` and ilmentufa PEG grammars provide the starting spec.

**Layer 2 — Semantic Compiler**. Transforms Lojban AST into a formal logic intermediate representation. Each bridi becomes a predicate application. Quantifier prenexes become FOL quantifier blocks. Connectives become logical operators. Tenses become temporal logic operators. Evidentials become epistemic annotations attached to propositions. This layer produces a representation analogous to LNN's formula graphs or Scallop's relational facts — but richer, preserving Lojban's modality and evidentiality.

**Layer 3 — Knowledge Graph Store** (Oxigraph + custom layer). Stores grounded facts as RDF triples (or n-ary extensions). The ~1,300 gismu place structures define the relation schema. Supports SPARQL queries for retrieval. Nemo provides rule-based reasoning over the graph.

**Layer 4 — Inference Engine** (hybrid: ascent/egglog + scryer-prolog). Datalog handles forward-chaining rule evaluation with stratified negation. Prolog handles backward-chaining goal-directed queries with unification and backtracking. Egglog's equality saturation handles term simplification and optimization. The inference engine operates on the formal logic IR from Layer 2.

**Layer 5 — Neuro-Symbolic Bridge** (custom, the core research contribution). This layer implements the differentiable interface between neural and symbolic components. Drawing from LNN and Scallop:

- **Neural predicates**: Neural networks (candle/ort) output probability distributions that ground symbolic atoms — e.g., an image classifier populates `pixra(x, y)` ("x is a picture of y") with confidence scores
- **Truth-value bounds**: Following LNN, maintain [L, U] intervals on propositions rather than point estimates, supporting open-world reasoning
- **Provenance tracking**: Following Scallop, tag derived facts with provenance information enabling both gradient flow for training and explanation generation for users
- **Bidirectional inference**: LNN-style upward/downward message passing on the formula graph

**Layer 6 — Explanation Generator**. Extracts inference traces from Layers 4-5 and renders them as human-readable Lojban or natural language. Because the internal representation preserves Lojban's structure, explanations can be generated as valid Lojban sentences — the system can literally "explain its reasoning in Lojban." Evidential markers (`ja'o` for inference, `za'a` for observation) annotate the provenance of each claim.

```
┌─────────────────────────────────────────────────────────┐
│  Lojban Text Input / Conversational Interface           │
├─────────────────────────────────────────────────────────┤
│  Layer 1: Lojban PEG Parser (pest)                      │
│  → Typed AST: bridi, selbri, sumti, connectives, tense  │
├─────────────────────────────────────────────────────────┤
│  Layer 2: Semantic Compiler                             │
│  → Formal Logic IR (FOL + temporal + epistemic + modal)  │
├───────────────────────┬─────────────────────────────────┤
│  Layer 3: Knowledge   │  Layer 5: Neuro-Symbolic Bridge │
│  Graph Store          │  • Neural predicates (candle)   │
│  (Oxigraph + Nemo)    │  • Truth-value bounds [L, U]    │
│                       │  • Provenance semirings          │
├───────────────────────┤  • Bidirectional inference       │
│  Layer 4: Inference   │                                 │
│  Engine               │                                 │
│  (ascent/egglog +     │                                 │
│   scryer-prolog)      │                                 │
├───────────────────────┴─────────────────────────────────┤
│  Layer 6: Explanation Generator                         │
│  → Lojban reasoning traces with evidential markers      │
└─────────────────────────────────────────────────────────┘
```

---

## Prior art validates the approach without preempting it

The most significant precedent is **OpenCog's Lojban framework** (2013–2016). Ben Goertzel proposed "Lojban++" as an interlingua for AGI communication, and Roman Treutlein built a partial Lojban-to-Atomese converter. The pipeline parsed Lojban via PEG, converted parse trees to OpenCog's Atomese knowledge representation (EvaluationLinks, PredicateNodes, ConceptNodes), and used Probabilistic Logic Networks for inference. Goertzel's 2016 proposal to replace OpenCog's entire NLP pipeline (RelEx2Logic) with a Lojban-mediated approach demonstrated the language's suitability — but the implementation remained partial, and OpenCog has since shifted to its Hyperon rewrite.

Gerold Hintz's 2014 TU Darmstadt thesis on "Semantic Parsing Using Lojban" demonstrated extracting predicate-argument structures from parallel Lojban-English corpora for frame-semantic parsing, validating Lojban as a source of machine-readable semantics. Speer and Havasi's 2004 MIT work explored computational NLP on Lojban. Multiple researchers have proposed Lojban as a machine translation interlingua.

**No existing project combines Lojban with neuro-symbolic reasoning in Rust.** The OpenCog work used Scheme/C++. Academic neuro-symbolic systems (DeepProbLog, LTN, NeurASP, LNN) use Python/PyTorch. Scallop is Rust-based but uses its own Datalog-like language, not a human-speakable logical language. This project would occupy a unique position: a **human-readable logical language** as the native representation in a **high-performance systems language** with **differentiable neuro-symbolic reasoning**.

---

## Key risks, mitigations, and research contributions

**Risk 1: Lojban's corpus is tiny.** With ~20 active speakers and limited text, training neural components on Lojban data is impractical. **Mitigation**: The neural components process non-Lojban inputs (images, embeddings, sensor data) and ground their outputs into Lojban-structured symbolic atoms. Lojban is the internal representation, not the training data. LLMs can generate Lojban knowledge bases from English specifications.

**Risk 2: Grammar fragmentation.** Multiple incompatible grammar variants exist. **Mitigation**: Target the standard BPFK grammar (ilmentufa's `camxes.peg`) first. Design the parser layer to be grammar-swappable via pest grammar files. Avoid experimental extensions until the core is stable.

**Risk 3: Scalability of symbolic reasoning.** Grounding, SDD compilation, and answer-set enumeration are known bottlenecks. **Mitigation**: Use Scallop's provenance semiring approach to select accuracy-scalability tradeoffs at runtime. Leverage ascent's parallel execution and Nemo's demonstrated scalability to 10⁸ facts. LNN's iterative convergence avoids full grounding.

**Risk 4: Tanru ambiguity.** Lojban compound words (tanru) are intentionally semantically vague — `sutra bajra` could mean "quickly-running" or "quick-type-of-runner." **Mitigation**: Treat tanru ambiguity as probabilistic — the neural component assigns probability distributions over possible interpretations, which the symbolic reasoner propagates. This is actually a feature for neuro-symbolic integration, not a bug.

The system's **primary research contributions** would be: (1) demonstrating that a human-speakable logical language can serve as a native knowledge representation that is simultaneously human-readable and machine-processable; (2) implementing LNN-style differentiable reasoning with Lojban's richer-than-FOL logic (temporal, epistemic, modal); (3) showing that Lojban's evidential system enables a natural explainability mechanism where the system annotates its reasoning with how it knows each fact; and (4) proving that Rust's ecosystem is sufficient for building production-quality neuro-symbolic systems.

## Conclusion

This project is feasible as a research proof-of-concept with a **6–12 month timeline for a minimal viable system** (parser + semantic compiler + Datalog reasoning + basic neural grounding + explanation output). The Lojban parser is the most straightforward component given existing PEG grammars and the camxes-rs reference. The neuro-symbolic bridge is the hardest and most novel component — where LNN's bidirectional inference and Scallop's provenance semirings provide the theoretical foundation but the Lojban-specific implementation must be original. The killer insight is that **Lojban's evidential system gives you explainability for free**: when the system says `ja'o lo remna cu morta` ("I conclude [by inference] that humans are mortal"), the evidential marker `ja'o` is not a post-hoc explanation bolted on — it is the native way Lojban encodes epistemic provenance. No other neuro-symbolic system has access to a reasoning language that treats explainability as first-class grammar.