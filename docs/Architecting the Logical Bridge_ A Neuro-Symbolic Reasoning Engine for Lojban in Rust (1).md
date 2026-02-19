# **Architecting the Logical Bridge: A Neuro-Symbolic Reasoning Engine for Lojban in Rust**

## **1\. The Convergence of Constructed Languages and Artificial Intelligence**

### **1.1 The "Black Box" Problem and the Linguistic Bottleneck**

The contemporary landscape of Artificial Intelligence is dominated by connectionist models, particularly Large Language Models (LLMs) based on transformer architectures. These systems exhibit remarkable proficiency in generating human-like text and performing heuristic reasoning. However, they suffer from a fundamental opacity often described as the "Black Box" problem. The internal decision-making processes—distributed across billions of parameters—are inherently difficult to trace, verify, or correct. Furthermore, these models operate on natural languages (English, Chinese, Spanish), which are evolutionarily optimized for social signaling and ambiguity rather than logical precision.  
This "Linguistic Bottleneck" impedes the development of verifiable reasoning systems. When a user queries an AI in English, the system must perform an implicit, probabilistic translation from the ambiguous surface form to an internal representation of meaning. In the sentence "I saw the man with the telescope," the prepositional phrase "with the telescope" can syntactically attach to "saw" (instrument of seeing) or "the man" (possession). An LLM resolves this based on statistical likelihood derived from training data, which is indistinguishable from hallucination in edge cases.

### **1.2 Lojban: A Substrate for Verifiable Reasoning**

The solution to the linguistic bottleneck lies in the utilization of a language engineered specifically for syntactic unambiguity and semantic isomorphism with formal logic: **Lojban**.1 Developed by the Logical Language Group (LLG), Lojban is a constructed language (conlang) derived from Loglan. Its design criteria—unambiguous grammar, audiovisual isomorphism, and cultural neutrality—make it an ideal intermediate representation (IR) for neuro-symbolic AI.  
In Lojban, the surface structure of a sentence maps deterministically to a unique parse tree. This tree, in turn, maps directly to a predicate logic formula. The sentence *mi viska lo nanmu poi ponse lo kance* (I see the man who possesses the telescope) is syntactically distinct from *mi viska lo nanmu sepi'o lo kance* (I see-via-instrument-of-telescope the man). This property allows a reasoning engine to treat user input effectively as executable code, eliminating the "guessing phase" inherent in natural language processing (NLP).

### **1.3 The Neuro-Symbolic Proposition**

This report proposes the architecture for a standalone **Neuro-Symbolic Reasoning Engine** built in **Rust**. This system is not merely a chatbot but a rigorous inference machine that combines the explainability of **Symbolic AI** (GOFAI) with the robustness of **Neural AI**.

* **Symbolic Component:** A Prolog-style backward-chaining inference engine that performs logical unification on Lojban text. It guarantees correctness and provides perfect explainability (proof trees).  
* **Neural Component:** A vector-embedding layer that provides "common sense" fuzziness. It allows the rigid logical engine to understand that *viska* (see) and *ganse* (sense) are semantically related, enabling soft matching when exact logical proofs fail.  
* **Infrastructure:** Built in Rust for memory safety and high performance, utilizing pest.rs for parsing and petgraph for knowledge graph representation.2

## ---

**2\. Computational Linguistics of Lojban: A Formal Analysis**

To architect a reasoning engine, one must first deconstruct the linguistic data structures it will manipulate. Lojban is not composed of nouns and verbs in the Indo-European sense, but of *predicates* (*selbri*) and *arguments* (*sumti*).

### **2.1 The Bridi: Atomic Unit of Logic**

The fundamental unit of Lojban is the *bridi* (predication). A *bridi* expresses a relationship between arguments.3 This maps directly to the atomic formulas of First-Order Logic (FOL).

| Lojban Construct | Logical Equivalent | Computational Type |
| :---- | :---- | :---- |
| **Selbri** | Predicate Symbol ($P$) | Function Identifier |
| **Sumti** | Argument / Term ($t$) | Variable / Constant |
| **Bridi** | Atomic Formula ($P(t\_1, t\_2,...)$) | Struct { head: Id, args: Vec\<Term\> } |
| **Cu** | Separator | Token (Parser Delimiter) |

Consider the Lojban sentence: *mi dunda ti do* (I give this to you).

* **Selbri:** *dunda* (give).  
* **Place Structure (Terbri):** The definition of *dunda* dictates three arguments: $x\_1$ (donor), $x\_2$ (gift), $x\_3$ (recipient).4  
* **Sumti:** *mi* ($x\_1$), *ti* ($x\_2$), *do* ($x\_3$).  
* **Logical Form:** $dunda(mi, ti, do)$.

This fixed arity of *gismu* (root words) is computationally advantageous. Unlike English verbs, which have variable valency ("I run" vs "I run a company"), Lojban *gismu* function like typed functions in a programming language API. The dictionary (jbovlaste) provides the function signature for all 1,342 root words, which the reasoning engine can load as a schema.

### **2.2 Morphology and Tokenization**

Lojban morphology is algorithmic. The word type can be determined solely by its phonological structure (CV pattern), allowing for highly efficient lexing without dictionary lookups.5

* **Cmavo:** Structure words. Form: V, CV, V'V. (e.g., *.i*, *lo*, *cu*). These are the operators and delimiters of the language.  
* **Gismu:** Root predicates. Form: CVCCV or CCVCV. (e.g., *klama*, *prenu*). These are the primitives of the knowledge base.  
* **Lujvo:** Compound predicates. Formed by concatenating *rafsi* (affixes) of *gismu*. (e.g., *lujvo* \= *logji* \+ *valsi*). These represent complex concepts derived from primitives.  
* **Cmevla:** Names. Anything ending in a consonant.

A Rust-based lexer can define regex-like rules (or PEG atomic rules) to categorize these tokens with $O(n)$ complexity.

### **2.3 The Syntax of Logic: Quantifiers and Scope**

Lojban is one of the few languages with a formal system for quantifier scope (Prenex Normal Form).6

* **Prenex:** The clause before the *selbri* where variables are bound.  
* **Sentence:** *ro prenu cu prami pa gerku*  
* **Ambiguity:** Does everyone love one specific dog, or does everyone love a dog (potentially different ones)?  
* **Lojban Resolution:** The order of quantifiers in the prenex (or appearing left-to-right) determines scope.  
  * *ro prenu... pa gerku*: $\\forall x \\in People, \\exists y \\in Dogs : Love(x, y)$.  
  * *pa gerku zo'u ro prenu...*: $\\exists y \\in Dogs, \\forall x \\in People : Love(x, y)$.

The reasoning engine must implement a **Prenex Expansion** pass during parsing to hoist inline quantifiers to the front of the logical formula, ensuring the symbolic solver respects the strict logical scope defined by the grammar.7

## ---

**3\. State of the Art in Lojban Parsing: A Critical Review**

Before designing a new parser, it is essential to analyze existing solutions to identify their limitations for a high-performance, embedded reasoning context.

### **3.1 Camxes (Java/JavaScript)**

*Camxes* is the reference implementation of the formal Lojban grammar.8 Written originally for the Rats\! parser generator in Java, it was later ported to JavaScript (camxes.js) and Python.

* **Pros:** It is the "gold standard" for correctness. If camxes parses it, it is valid Lojban.  
* **Cons:**  
  * **Performance:** The JavaScript port, widely used in web tools, suffers from the performance overhead of dynamic typing.  
  * **Integration:** Integrating a Java or Node.js process into a Rust-based system introduces significant FFI (Foreign Function Interface) friction and latency.  
  * **Output:** The AST produced is often verbose, containing extensive syntactic sugar that obscures the logical structure.

### **3.2 Tersmu (Haskell)**

*Tersmu* is a semantic parser implemented in Haskell that translates Lojban text directly into First Order Logic (FOL).7

* **Significance:** It serves as the primary "prior art" for this research. *Tersmu* tackles the complex task of expanding anaphora (back-references like *ri*) and logical connectives.7  
* **Limitation:** Being written in Haskell makes it difficult to embed directly into a Rust neuro-symbolic ecosystem without substantial overhead.

### **3.3 Camxes-rs**

*Camxes-rs* is an existing Rust crate attempting to port the grammar.10

* **Status:** While a noble effort, many direct ports of PEG parsers struggle with the intricate "state" required for certain Lojban constructs (like *zoi* quotes) if not architected with Rust's ownership model in mind. Furthermore, a reasoning engine requires a specific *kind* of AST—one optimized for logic translation, not just distinct code reproduction.

### **3.4 The Need for a Pest.rs Implementation**

To build a "standalone" and "research-grade" engine, we require a parser that:

1. **Zero-Copy:** Uses Rust lifetimes to reference the input string directly in AST nodes, minimizing memory allocation.  
2. **Type Safety:** Uses Rust enums to make invalid tree states unrepresentable.  
3. **Extensibility:** Allows modular addition of "neuro" features (like attaching embeddings to nodes) directly in the parse tree.  
   Pest.rs is the ideal candidate. It uses a separate grammar file (.pest), compiles to highly optimized Rust code, and provides a "Pairs" iterator that facilitates complex AST construction.

## ---

**4\. Designing the Syntactic Layer with Pest.rs**

### **4.1 Architecture of the Grammar**

The transition from standard PEG (used by camxes) to pest requires adapting to pest's specific syntax and capabilities.

#### **4.1.1 The Grammar File (lojban.pest)**

The grammar is defined in a separate file. We structure it hierarchically: **Text** \-\> **Paragraph** \-\> **Sentence** \-\> **Bridi** \-\> **Sumti/Selbri**.

Code snippet

// Top Level  
text \= { SOI \~ (paragraph | sentence | fragment)\* \~ EOI }  
WHITESPACE \= \_{ " " | "\\t" | "\\r" | "\\n" | "." }  
COMMENT \= \_{ "\#" \~ (\!NEWLINE \~ ANY)\* }

// Morphology (Atomic Rules)  
// @ ensures no implicit whitespace inside the word   
gismu \= @{ (consonant \~ vowel \~ consonant \~ consonant \~ vowel) | (consonant \~ consonant \~ vowel \~ consonant \~ vowel) }  
cmavo \= @{\!cmevla \~ (consonant? \~ vowel)+ }  
cmevla \= @{ (letter)+ \~ consonant \~ "."? }

// Syntax  
sentence \= { (term)\* \~ bridi\_tail }  
bridi\_tail \= { selbri \~ (sumti | term)\* \~ "vau"? }

// Predicate Structure  
selbri \= { tag? \~ (tanru | simple\_selbri) }  
simple\_selbri \= { gismu | lujvo | fuivla }  
tanru \= { simple\_selbri \~ (simple\_selbri)+ }

// Argument Structure  
sumti \= { lo\_clause | la\_clause | pron\_clause | quote\_clause }  
lo\_clause \= { "lo" \~ selbri \~ "ku"? }  
pron\_clause \= { "mi" | "do" | "ko'a" } // Simplified list

#### **4.1.2 Handling Elidable Terminators**

One of the most complex features of Lojban is **Terminator Elision**. The word *ku* terminates a *lo* description, but it can be omitted if no ambiguity arises.

* Standard PEG is greedy. sumti+ will consume everything that looks like a sumti.  
* **The Problem:** In lo broda cu brode, lo broda is the sumti. cu marks the main verb. If ku is missing, the parser must know that cu cannot be part of the lo clause.  
* **The Pest Solution:** We utilize **Ordered Choice** and **Negative Lookahead** predicates (\!).

Code snippet

// A sumti continues ONLY if it is not followed immediately by a separator  
// that belongs to the parent bridi.  
sumti\_tail \= { selbri \~\!separator }   
separator \= \_{ "cu" | "vau" | "i" | "ni'o" }

This logic dictates that the inner construct must yield when it encounters a token reserved for the higher-level structure.

#### **4.1.3 The ZOI Quote Challenge**

Lojban allows quoting non-Lojban text using zoi X. text.X, where X can be any word, and the quote ends only when that specific X appears again.

* **Challenge:** Standard PEG is context-free and stateless; it cannot remember "what X was" to match it later.  
* **Pest Solution:** Pest supports a **Stack** (PUSH, POP) for exactly this scenario.11  
* **Implementation:**  
  Code snippet  
  // Push the delimiter onto the stack, consume content until matching delimiter   
  zoi\_quote \= { "zoi" \~ PUSH(word) \~ (\!PEEK \~ ANY)\* \~ POP }

  This feature is crucial for correctly parsing Lojban quotes without complex post-processing.

### **4.2 AST Design in Rust**

The output of pest is a stream of tokens (Pairs). We must transform this into a strongly typed Abstract Syntax Tree (AST).

Rust

\#  
pub enum LojbanNode {  
    Text(Vec\<LojbanNode\>),  
    Bridi {  
        selbri: Box\<SelbriNode\>,  
        sumti: Vec\<SumtiNode\>,  
        terms: Vec\<TermNode\>, // Tenses, modals  
        is\_negated: bool,     // "na" detection  
    },  
    Sumti {  
        content: SumtiContent,  
        quantifier: Option\<Quantifier\>,  
        relative\_clauses: Vec\<LojbanNode\>, // nested bridi  
    },  
}

\#  
pub enum SelbriNode {  
    Gismu(String),  
    Lujvo { parts: Vec\<String\> },  
    Tanru { components: Vec\<Box\<SelbriNode\>\> },  
}

This AST abstracts away the "syntactic sugar" of the language. Whether the user typed lo broda ku or lo broda, the AST node is identical: Sumti::Description(Pred("broda")). This normalization is critical for the logical engine.

## ---

**5\. The Semantic Translation Layer**

The parser gives us a tree of words; reasoning requires a graph of logic. This layer translates the AST into **First-Order Logic (FOL)**. This process mirrors the functionality of the Haskell project tersmu 7, but integrated directly into the Rust pipeline.

### **5.1 The Semantic Walker**

We implement a SemanticWalker trait that traverses the AST. This walker maintains a **Discourse Context** stack.

Rust

struct DiscourseContext {  
    assignments: HashMap\<String, Term\>, // ko'a \-\> "la.alis."  
    last\_sumti: Vec\<Term\>,              // For "ri" resolution  
    last\_bridi: Option\<Formula\>,        // For "go'i" resolution  
}

### **5.2 Anaphora Resolution**

Lojban relies heavily on back-references.

* *ri*: Refers to the last sumti.  
* *go'i*: Refers to the last bridi.  
* *ko'a*: Refers to an explicitly assigned variable (goi).

The Semantic Walker resolves these *during translation*. When it encounters ri in the AST, it peeks at ctx.last\_sumti and substitutes the referent immediately.7 This ensures that the logical engine sees Love(Alice, Alice) instead of Love(Alice, Referent\_to\_last).

### **5.3 Prenex Expansion and Quantifiers**

The walker identifies quantifiers attached to sumti.

* Input AST: Bridi { selbri: "viska", sumti: \[ "mi", "ro gerku" \] }  
* Transformation: The walker detects ro (all) on argument 2\.  
* Logic Output: ForAll(y, Implies(Dog(y), See(mi, y))).

This handles the "Standard Logic" of Lojban. The system must also handle **Relative Clauses** (poi and noi).

* *lo nanmu poi prenu*: A man such that he is a person (Restrictive). This adds a conjunction to the logic: $\\exists x : Man(x) \\land Person(x)$.  
* *lo nanmu noi prenu*: A man (incidentally, he is a person). This adds a separate parenthetical assertion but does not restrict the domain of $x$.

### **5.4 Normalizing to Terbri (Place Structures)**

The walker must query the Schema (loaded from jbovlaste).

* Query: mi klama (I go).  
* Schema: klama(x1, x2, x3, x4, x5).4  
* Normalization: The AST sumti are mapped to slots. Missing slots are filled with Term::Zo'e (Unspecified).  
* Result: klama(mi, Zo'e, Zo'e, Zo'e, Zo'e).

This normalization is vital for unification. The Prolog engine relies on matching argument positions (arity). A standard Prolog fact klama(a,b,c,d,e) would not match klama(a) without this padding.

## ---

**6\. The Symbolic Inference Core**

The heart of the system is a custom inference engine inspired by Prolog but tailored for Lojban's logical primitives.

### **6.1 Data Structures**

We define the atomic elements of the inference engine.

Rust

\#  
pub enum Term {  
    Atom(String),         // "la.alis."  
    Variable(usize),      // Logic variable?0,?1  
    Anon(usize),          // Existential "da", "de"  
    Wildcard,             // "zo'e"  
    Compound(String, Vec\<Term\>), // sub-terms  
}

\#  
pub struct Fact {  
    pub head: Term, // Typically a Compound term: predicate(args...)  
}

\#  
pub struct Rule {  
    pub head: Term,  
    pub body: Vec\<Term\>, // Conjunction of goals  
}

### **6.2 The Unification Algorithm**

We leverage Rust libraries like symbolic-mgu for robust unification.12 **Function unify(t1, t2, env):**

1. **Dereference:** Follow variable chains in env. If ?x is bound to ?y and ?y is bound to A, then ?x is A.  
2. **Identity:** If t1 \== t2, return Success.  
3. **Wildcard Handling:**  
   * If t1 is Wildcard (*zo'e*), it unifies with t2 *without binding*. It effectively ignores the mismatch. This allows klama(mi, zo'e) to match klama(mi, le zarci).  
4. **Variable Binding:**  
   * If t1 is a Variable, check for **Occurs Check** (does t1 appear inside t2?). If not, bind t1 \-\> t2 in env.  
5. **Compound Matching:**  
   * If t1 and t2 are Compounds (Predicates), check:  
     * Are names identical? (*klama* \== *klama*).  
     * Is arity identical?  
     * Recursively unify all arguments.

### **6.3 Backward Chaining (SLD Resolution)**

The solver processes queries against the Knowledge Base (KB). Alternatives like crepe (Datalog macro) exist for Rust 14, but a custom backward chainer allows better control over Lojban-specific features (like Tense logic).  
**Function solve(goals, env):**

1. If goals is empty, yield env (Solution found).  
2. Pop first goal $G$.  
3. **Fact Search:** Iterate through all Facts in KB.  
   * Try unify(G, Fact). If success, recursively solve(rest, new\_env).  
4. **Rule Search:** Iterate through all Rules in KB.  
   * Try unify(G, Rule.head).  
   * If success, replace $G$ with Rule.body (sub-goals) and solve.

## ---

**7\. The Neuro-Symbolic Integration**

Standard symbolic logic is brittle. If the user asks about *viska* (seeing) but the database knows about *ganse* (sensing), a strict logical engine returns "False". A human would answer "Yes, loosely". This section details the integration of vector embeddings and **Provenance Semirings** to bridge this semantic gap, inspired by the Scallop framework.

### **7.1 Vector Space Models for Lojban**

Lojban's *gismu* are well-defined semantic primitives. We create a vector space where each *gismu* is a point.

* **Embeddings:** We use **FastText** (pre-trained on Wikipedia and fine-tuned on the Lojban corpus) to generate 300-dimensional vectors for all 1,342 *gismu*.  
* **Metric:** We use **Cosine Similarity** to measure semantic distance.1  
  * $Sim(A, B) \= \\frac{A \\cdot B}{||A|| ||B||}$

### **7.2 Fuzzy Unification with Provenance Semirings**

Instead of boolean True/False, we use a **Provenance Semiring** framework (similar to Scallop) to track the "confidence" or "proof path" of a deduction.

* **Structure:** A semiring $(K, \\oplus, \\otimes, 0, 1)$.  
  * $K$: The set of confidence scores (e.g., $\[0.0, 1.0\]$).  
  * $\\oplus$ (Aggregation): $max(a, b)$ (Choose the best proof).  
  * $\\otimes$ (Combination): $a \\times b$ (A chain is the product of its links).

**Soft Unification:**  
When exact unification fails, the engine computes $Sim(Pred\_1, Pred\_2)$.

* If $Sim \> Threshold$, unification "succeeds" with a provenance score $P \= Sim$.  
* The reasoning engine propagates this score: $Score\_{total} \= Score\_{step1} \\times Score\_{step2}$.

**Example:**

* Query: catlu(?x, le dadysli) (Who looks at the pendulum?).  
* Fact: viska(la.alis., le dadysli) (Alice sees the pendulum).  
* Logic: catlu\!= viska. Unification fails.  
* Neuro: Sim(catlu, viska) \= 0.85. Unification succeeds with $P=0.85$.  
* Result: ?x \= la.alis. (Confidence: 85%).

### **7.3 Handling Lujvo (Compounds)**

The system effectively handles unknown words (*lujvo*) using vector composition.

* Unknown input: *zbasai* (make-meal / cook).  
* Analysis: zbasu (make) \+ sanmi (meal).  
* Vector: $\\vec{v}\_{zbasai} \= \\alpha \\vec{v}\_{zbasu} \+ \\beta \\vec{v}\_{sanmi}$.  
* Matching: This composite vector will likely be close to *jukpa* (cook), allowing the system to reason about words it has never explicitly seen defined, purely based on their morphological roots.

## ---

**8\. Knowledge Graph Architecture**

To support "General Purpose Reasoning" and "Knowledge Graphs" as requested, the transient logic definitions must be grounded in a persistent store.

### **8.1 Graph Representation with Petgraph**

We utilize the petgraph crate to represent the static knowledge of the world.2

* **Nodes:** Entities (Constants/Sumti) and Event Abstractions (Nu).  
* **Edges:** Roles (Sumti Places).

Since Lojban predicates are n-ary, we cannot use simple subject-predicate-object triples. We use **Event Reification** or hypergraphs.

* **Fact:** *mi dunda ti do* (I give this to you).  
* **Graph Nodes:**  
  * Event\_101 (Type: *dunda*).  
  * Entity\_Mi.  
  * Entity\_Ti.  
  * Entity\_Do.  
* **Graph Edges (Labeled):**  
  * Event\_101 \--(x1)--\> Entity\_Mi.  
  * Event\_101 \--(x2)--\> Entity\_Ti.  
  * Event\_101 \--(x3)--\> Entity\_Do.

### **8.2 Persistent Storage**

For a standalone research PoC, we use an embedded database. **Sled** is a high-performance, embedded, pure Rust database that maps well to this structure.

* **Storage Scheme:**  
  * Key: PredicateHash \+ ArgsHash  
  * Value: Serialized Fact Struct  
    This allows the inference engine to load the graph into memory (Petgraph) on startup or query it lazily from disk.

## ---

**9\. Implementation Strategy and Applications**

### **9.1 Crate Ecosystem**

The project relies on a carefully selected stack of Rust crates:

* **Parser:** pest (v2.0+), pest\_derive.  
* **Logic:** symbolic-mgu (Unification) 13, petgraph (Graph structures).  
* **Neuro:** ndarray (Matrix math), rust-bert or fasttext bindings.  
* **Interface:** rustyline (Readline implementation for REPL), clap (CLI arguments).  
* **Serialization:** serde, serde\_json.

### **9.2 Application: Explainable AI (XAI)**

The most significant application of this engine is **Self-Explanation**. Because every deduction is generated via a symbolic proof tree (even if fuzzy steps were used), the system can reconstruct its "Chain of Thought" in Lojban.  
**Scenario:**

* **User:** *krinu ma lo nu do djuno* (Why do you know \[that Alice is here\]?).  
* **System Internal:** Proof { Rule: Seeing-\>Knowing, Fact: Viska(Alice, Here), Confidence: 0.9 }.  
* **System Output:** *ni'i lo nu mi viska la.alis.* (Because I see Alice).

This transparency is the defining feature of the architecture, satisfying the requirement for "Explainable/Conversational AI."

### **9.3 Future Outlook: The Conversational Loop**

The standalone engine functions as a REPL (Read-Eval-Print Loop).

1. **Read:** rustyline captures input.  
2. **Parse:** pest generates AST (using Stack for quotes).  
3. **Translate:** SemanticWalker generates FOL goals (resolving ri and go'i).  
4. **Solve:** InferenceCore searches Graph \+ Vectors for solutions.  
5. **Gen:** Result is converted back to Lojban text.  
6. **Print:** Output to console.

This architecture provides a complete, closed-loop system for researching Logical AI, offering a stark alternative to the hallucination-prone models of today. By grounding AI in the unambiguous soil of Lojban, we build a bridge between the statistical power of neural networks and the rigorous truth of formal logic.

#### **Works cited**

1. Semantic parsing using Lojban – On the middle ground between semantic ontology and language, accessed February 11, 2026, [https://www.inf.uni-hamburg.de/en/inst/ab/lt/teaching/theses/completed-theses/2014-ma-hinz.pdf](https://www.inf.uni-hamburg.de/en/inst/ab/lt/teaching/theses/completed-theses/2014-ma-hinz.pdf)  
2. petgraph \- Rust \- Docs.rs, accessed February 11, 2026, [https://docs.rs/petgraph/](https://docs.rs/petgraph/)  
3. The Complete Lojban Language (2016)/Chapter 2 \- Wikisource, the free online library, accessed February 11, 2026, [https://en.wikisource.org/wiki/The\_Complete\_Lojban\_Language\_(2016)/Chapter\_2](https://en.wikisource.org/wiki/The_Complete_Lojban_Language_\(2016\)/Chapter_2)  
4. from Wikibooks: Lojban/Place structure, accessed February 11, 2026, [https://mw.lojban.org/papri/from\_Wikibooks:\_Lojban/Place\_structure](https://mw.lojban.org/papri/from_Wikibooks:_Lojban/Place_structure)  
5. Lojban grammar \- Wikipedia, accessed February 11, 2026, [https://en.wikipedia.org/wiki/Lojban\_grammar](https://en.wikipedia.org/wiki/Lojban_grammar)  
6. First-order logic \- Wikipedia, accessed February 11, 2026, [https://en.wikipedia.org/wiki/First-order\_logic](https://en.wikipedia.org/wiki/First-order_logic)  
7. accessed February 11, 2026, [https://lojban.org/resources/irclog/lojban/2015\_07/lojban-special.2015.07.log](https://lojban.org/resources/irclog/lojban/2015_07/lojban-special.2015.07.log)  
8. lojban/ilmentufa \- GitHub, accessed February 11, 2026, [https://github.com/lojban/ilmentufa](https://github.com/lojban/ilmentufa)  
9. nuzba/en \- La Lojban, accessed February 11, 2026, [https://mw.lojban.org/papri/nuzba/en](https://mw.lojban.org/papri/nuzba/en)  
10. lojban/camxes-rs \- GitHub, accessed February 11, 2026, [https://github.com/lojban/camxes.rs](https://github.com/lojban/camxes.rs)  
11. Syntax of pest parsers \- A thoughtful introduction to the pest parser, accessed February 11, 2026, [https://pest.rs/book/grammars/syntax.html](https://pest.rs/book/grammars/syntax.html)  
12. symbolic\_mgu \- Rust \- Docs.rs, accessed February 11, 2026, [https://docs.rs/symbolic-mgu/latest/symbolic\_mgu/](https://docs.rs/symbolic-mgu/latest/symbolic_mgu/)  
13. symbolic-mgu \- crates.io: Rust Package Registry, accessed February 11, 2026, [https://crates.io/crates/symbolic-mgu](https://crates.io/crates/symbolic-mgu)  
14. Crepe: fast, compiled Datalog in Rust \- Reddit, accessed February 11, 2026, [https://www.reddit.com/r/rust/comments/ikszdg/crepe\_fast\_compiled\_datalog\_in\_rust/](https://www.reddit.com/r/rust/comments/ikszdg/crepe_fast_compiled_datalog_in_rust/)  
15. pnevyk/rusty-graphs: Collection of examples for showcasing various Rust graph data structure libraries. \- GitHub, accessed February 11, 2026, [https://github.com/pnevyk/rusty-graphs](https://github.com/pnevyk/rusty-graphs)