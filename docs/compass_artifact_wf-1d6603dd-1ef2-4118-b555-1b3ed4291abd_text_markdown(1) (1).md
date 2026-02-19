# Building a Lojban-native symbolic reasoning engine in Rust

**Lojban is the only human-speakable language with a formally unambiguous, machine-parseable grammar that maps directly to predicate logic — making it uniquely suited as the native language of a symbolic reasoning engine.** A Rust implementation can leverage Lojban's existing PEG grammar (`camxes`), a rich ecosystem of parsing and logic crates (`pest`, `egg`, `egglog`, `ascent`), and prior work from projects like `tersmu` and OpenCog. The critical gap is not parsing — that problem is solved — but semantic interpretation: translating parse trees into well-defined logical forms that a reasoning engine can operate on. No complete formal semantics for Lojban exists today, and building this engine would require creating one.

---

## Lojban's formal grammar is unambiguous by design

Lojban achieves what no natural language can: **every grammatically valid sentence produces exactly one parse tree**. This property was machine-verified using YACC during the language's original specification in the 1990s and has since been re-expressed as a Parsing Expression Grammar (PEG) in the `camxes` project.

The grammar distinguishes three morphologically distinct word classes identifiable by phonological form alone. **Brivla** (predicate words) always contain a consonant cluster and end in a vowel — the ~1,350 root predicates (gismu) follow strict CVCCV or CCVCV patterns like `klama` ("x1 goes to x2 from x3 via x4 using x5"). **Cmavo** (structure words) are short, cluster-free particles grouped into ~140 grammatical categories (selma'o) that function as articles, connectives, quantifiers, and terminators. **Cmene** (names) end in consonants and are bounded by pauses. This morphological separation eliminates lexical ambiguity entirely — a listener can always identify word boundaries and syntactic roles from sound alone.

The canonical PEG grammar lives at `github.com/lojban/camxes` with ~600 rules covering both morphology and syntax. The `ilmentufa` project (`github.com/lojban/ilmentufa`) maintains five PEG grammar variants including the standard, beta, and experimental grammars. Multiple parser implementations exist across languages: Java (original camxes using Rats!), JavaScript (ilmentufa, the de facto web parser), Python (camxes-py using parsimonious), Haskell (tersmu using Pappy), C (legacy jbofihe using YACC), and critically, **Rust** (`github.com/lojban/camxes-rs`, last updated April 2025). The grammar uses bracket-like elidable terminators (`ku`, `kei`, `vau`, etc.) and explicit scope markers (`cu` separates arguments from predicates, `.i` separates sentences) to maintain unambiguity while allowing concise expression.

---

## Predicate logic is built into the language's DNA

Lojban's semantics were shaped by John Parks-Clifford ("pc"), a student of Richard Montague himself, who documented the mapping in his "Logic Language Draft 1.1." The core correspondence is direct: a **bridi** (sentence) maps to a well-formed formula, a **selbri** (predicate word) maps to a predicate symbol, and **sumti** (arguments) map to terms. Each gismu defines a fixed-arity predicate — `klama` is a 5-place relation where x1 is the goer, x2 the destination, x3 the origin, x4 the route, and x5 the means. This place-structure system is essentially a built-in frame semantics, directly analogous to FrameNet frames or PropBank argument structures, as demonstrated in Gerold Hintz's 2014 Hamburg master's thesis.

The quantifier system maps cleanly to first-order logic: `ro` = ∀, `su'o` = ∃, `pa` = ∃!, `no` = ¬∃, plus numerical quantifiers (`re`, `ci`, etc.). Lojban variables `da`, `de`, `di` function as bound variables in a **prenex** — a construct directly equivalent to the quantifier prefix in prenex normal form. The sentence `ro da poi prenu zo'u da morsi` translates to ∀x(prenu(x) → morsi(x)), "for all x that are persons, x is mortal."

All **16 binary truth functions** are expressible through a compositional vowel system: A = inclusive or (∨), E = and (∧), O = biconditional (↔), U = whether-or-not. Negation prefixes (`na-`, `-nai`) and operand swapping (`se`) generate the remaining functions. Connective forms vary by what they connect — `ijek` for sentences, `ek` for arguments, `jek` for predicates, `gihek` for predicate tails — maintaining unambiguity about scope. DeMorgan's Law transformations are explicitly supported: distributing `naku` over connectives follows standard logical rules, documented in CLL Chapter 16.

The **xorlo reform** (adopted ~2003 by 11-0 BPFK vote) fundamentally changed quantification. Previously, `lo broda` had an implicit existential quantifier. Under xorlo, bare `lo broda` has **no default quantifier** — it is a "plural constant" that generically refers to broda-satisfying things. This moved Lojban from singular first-order logic toward **plural logic**, creating both opportunities (more natural expression) and challenges (less quantificational precision) for formal reasoning.

---

## The semantic gap is the real engineering challenge

Parsing Lojban is a solved problem. The hard part — and the core contribution a reasoning engine must make — is **semantic interpretation**: translating parse trees into well-defined logical forms. The Lojban wiki itself states bluntly: "A complete formal semantics is far beyond the state of the art for Lojban."

**tersmu** (`gitlab.com/zugz/tersmu`), a Haskell semantic parser by Martin Bays, is the closest any project has come to full Lojban-to-logic translation. It implements CLL+xorlo baseline rules, handling quantifiers, all logical connective positions, modal operators, anaphora, and relative clauses. But its author acknowledges significant limitations: it treats `na` identically to `naku` (contradicting CLL's account), ignores attitudinals entirely, copies tanru semantics verbatim without resolution, and notes that the output "doesn't even have an obvious semantics." The ~450-line core algorithm documents every edge case where CLL is underspecified.

Six specific hard problems define the frontier:

- **Tanru semantics**: Compound predicates like `tsani blanu` (sky-blue) have an "arbitrarily generic or specific" relationship between modifier and head. No formal specification exists. OpenCog proposed modeling these via IntensionalInheritanceLink and ContextLink structures, but this remains experimental.
- **Abstraction typing**: Lojban's 12 abstractors (`nu` for events, `du'u` for propositions, `ka` for properties with λ-variable `ce'u`, `ni` for quantities, etc.) create higher-order logical structures that exceed first-order logic. The `ka`/`ce'u` system is genuinely lambda abstraction: `lo ka ce'u xunre` = λx.red(x).
- **Negation scope**: The `na` vs `naku` distinction and interactions with tenses and modals remain contested in the community. Moving `naku` past a quantifier inverts it (standard FOL), but bare `na` before the selbri has disputed scope behavior.
- **Implicit arguments**: Elided places (filled by `zo'e`, "something unspecified") are semantically loaded but computationally invisible — a reasoning engine must decide how to handle them.
- **Attitudinals**: The rich system of emotional and evidential markers (`.ui` = happy, `pe'i` = in my opinion, `ka'u` = cultural knowledge) is explicitly designed to be outside truth-functional logic but carries important epistemic information.
- **Tense as modal logic**: Lojban's tense system (`pu` = past, `ca` = present, `ba` = future, plus spatial tenses) is far richer than simple temporal operators and requires sophisticated temporal or modal logic modeling.

---

## The Rust ecosystem provides every building block needed

A Lojban reasoning engine in Rust can be assembled from mature, well-maintained crates covering every architectural layer. The most important selection decisions involve the parser, the reasoning core, and the knowledge store.

**For parsing**, `pest` (v2.8.4, 5,226 GitHub stars) is the natural choice — it is a PEG parser generator that accepts grammar files in exactly the format Lojban's `camxes.peg` uses. The existing `camxes-rs` project (`github.com/lojban/camxes-rs`) already implements a Rust Lojban parser. For morphological tokenization, `logos` generates optimized DFA lexers at compile time and can efficiently recognize Lojban's phonologically distinct word classes. For interactive use with error recovery, `chumsky` (v0.11.1) provides robust parser combinators.

**For the reasoning core**, the most promising approach combines multiple engines:

- **egglog** (v1.0.0) unifies Datalog relational queries with equality saturation — ideal because Lojban knowledge bases are naturally relational (predicate-argument structures as Datalog facts) while logical equivalences (DeMorgan's, double negation, quantifier transformations) are naturally term-rewriting rules. This is likely **the single most important crate** for the project.
- **egg** (v0.10.0, POPL 2021 Distinguished Paper) provides the underlying e-graph equality saturation library with `define_language!` and `rewrite!` macros for defining Lojban's logical language and transformation rules.
- **ascent** (v0.8.0) offers full Datalog with lattices, parallel execution via rayon, and aggregation — useful for type inference and fixed-point computations over the knowledge base.
- **z3 Rust bindings** (v0.13.x, ~64K downloads/month) provide SMT solving for consistency checking and constraint satisfaction.
- **Scryer Prolog** (v0.10.0) implements a full ISO Prolog in Rust using the Warren Abstract Machine, usable as a library for backward-chaining reasoning.

**For infrastructure**, `salsa` (v0.24.0, used by rust-analyzer) provides incremental computation — when the knowledge base changes, only affected derivations are recomputed. `lasso` handles string interning, critical for Lojban's fixed vocabulary (~1,300 gismu, ~600 cmavo) where `klama` should be a single integer key everywhere. `id-arena` implements the arena-plus-ID pattern that Rust compiler projects use for term graphs, avoiding borrow checker friction with shared logical terms. `oxigraph` provides a full SPARQL 1.1 graph database in Rust for Semantic Web integration, and `redb` offers pure-Rust embedded key-value storage for persistence.

---

## Prior art provides a blueprint, not a solution

The most instructive precedent is **Attempto Controlled English (ACE)**, a controlled subset of English that translates unambiguously to first-order logic via the APE parser → Discourse Representation Structures pipeline. ACE's companion reasoner **RACE** proves logical consequences and generates natural-language proof explanations. This is precisely the architecture a Lojban engine should target — but Lojban starts with far stronger formal properties than ACE, since its grammar is inherently unambiguous rather than artificially restricted.

**OpenCog's Lojban initiative** (documented by Ben Goertzel in 2016) proposed replacing hand-coded English-to-logic rules with Lojban-mediated translation. Roman Treutlein built a Lojban-to-Atomese framework mapping place structures to `EvaluationLink`/`PredicateNode` atoms. Goertzel argued that "if we could make an OpenCog capable of fluent, sensible conversation in Lojban, we'd be 90% of the way to one capable of fluent, sensible conversation in English." The work demonstrated the mapping's feasibility but was never completed to production quality. OpenCog has since evolved into Hyperon with the MeTTa language.

The **Jorne project** (Brian Eubanks, 2005) mapped Lojban predicates to RDF triples, published in "Wicked Cool Java." Each gismu's place structure maps naturally to RDF properties. **Nick Nicholas's Prolog analyzer** used DCG grammar to translate Lojban to Prolog facts, identifying the key insight that "a Lojban-to-Prolog semantic analyser would be addressing many of the current issues in NLP knowledge representation." The **lojysamban** project implemented Prolog with Lojban syntax in Haskell. None reached production maturity, but collectively they prove the concept and document the pitfalls.

The **jbovlaste dictionary** (`jbovlaste.lojban.org`, XML-exportable) constitutes a rudimentary ontology of ~1,300 root predicates with typed place structures — essentially a predefined schema for the knowledge base. Testing corpora include **22,000+ sentences** in camxes-py's test suite, **10,000+ parallel Lojban-English sentences** on Tatoeba, and the BPFK text corpus containing translated literature (Alice in Wonderland, The Little Prince, etc.).

---

## Recommended architecture layers Lojban's strengths onto proven patterns

The engine should follow a layered pipeline inspired by ACE/RACE and chalk's architecture (which cleanly separates domain IR, logic rules, and proof engine):

```
Lojban Text
    │
    ▼
┌──────────────────────────────────┐
│  Morphology (logos) + Parsing    │  ← pest with camxes.peg
│  Output: Concrete Syntax Tree    │
└──────────────┬───────────────────┘
               ▼
┌──────────────────────────────────┐
│  Semantic Construction           │  ← The hard part: CST → Logical IR
│  Handles: quantifier scope,      │    Locally nameless representation
│  connectives, abstraction,       │    for bound/free variables
│  negation, place structures      │
└──────────────┬───────────────────┘
               ▼
┌──────────────────────────────────┐
│  Logical IR (id-arena + lasso)   │  ← Events as first-class objects
│  Typed: Propositions, Properties,│    (Davidsonian event semantics)
│  Events, Quantities              │
└──────────────┬───────────────────┘
               ▼
┌──────────────────────────────────┐
│  Knowledge Base                  │  ← oxigraph (RDF/SPARQL) +
│  Indexed: discrimination trees,  │    redb (persistent KV store)
│  feature vectors                 │    salsa (incremental updates)
└──────────────┬───────────────────┘
               ▼
┌──────────────────────────────────┐
│  Reasoning Layer                 │
│  • egglog: Datalog + equality    │  ← Primary engine
│    saturation for normalization  │
│  • ascent: parallel Datalog      │  ← Fixed-point computation
│    with lattices                 │
│  • z3: SMT for consistency       │  ← Constraint solving
│  • Scryer Prolog: backward       │  ← Optional query interface
│    chaining queries              │
└──────────────┬───────────────────┘
               ▼
┌──────────────────────────────────┐
│  Output: Proofs, Answers, Models │
│  • Back-translation to Lojban    │  ← Natural-language explanations
│  • Formal proof objects          │
│  • SPARQL query results          │
└──────────────────────────────────┘
```

For the **Logical IR**, use the locally nameless representation — De Bruijn indices for bound variables (handling Lojban's quantifiers `ro`, `su'o` cleanly) with named free variables (for sumti placeholders like `zo'e`). Apply **Davidsonian event semantics** so that `mi klama le zarci` becomes ∃e.klama(e, mi, le_zarci, _, _), making events first-class objects that abstractors (`nu`, `pu'u`, `za'i`) can reference. Model Lojban's `ka`/`ce'u` system as genuine lambda abstraction. For decidability, use a **layered approach**: Horn clauses (Datalog) for common queries, Description Logic fragments (OWL 2 EL/QL profiles) for ontological reasoning, and full FOL with timeouts for complex proofs.

The **semantic construction** layer — translating parse trees into logical IR — is where most engineering effort must concentrate. Start by porting tersmu's ~450-line core algorithm from Haskell to Rust, addressing its documented limitations. Define explicit scope rules for `na` vs `naku`, implement xorlo-compliant plural reference, and handle abstraction typing. For tanru, adopt a pragmatic default (intersective modification: `tsani blanu` = sky ∧ blue) with an extensibility point for intensional rules.

## Conclusion

Building a Lojban-native symbolic reasoning engine in Rust is architecturally tractable because the hardest problem in NLP — ambiguous parsing — simply doesn't exist for Lojban. The PEG grammar is complete, multiple parser implementations exist (including one in Rust), and the Rust crate ecosystem provides production-grade tools for every layer from lexing to equality saturation to SPARQL storage. The genuine research frontier is **semantic construction** — the translation from unambiguous parse trees to well-typed logical forms — where tersmu's Haskell implementation provides both a starting point and a catalog of every unresolved edge case. The recommended path forward: build the parser on `pest` + `camxes.peg`, define the logical IR using `id-arena` + `lasso` with locally nameless representation, implement semantic construction by porting and extending tersmu, run reasoning through `egglog` as the primary engine with `ascent` and `z3` as complements, and store knowledge in `oxigraph` for Semantic Web interoperability. The ~1,300 gismu place structures in jbovlaste provide a ready-made ontological schema, and 22,000+ test sentences provide immediate validation data.