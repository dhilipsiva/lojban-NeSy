# **Architectural Blueprint for a Neuro-Symbolic Reasoning Engine: Integrating Lojban, Rust, and Logic Tensor Networks**

## **1\. Introduction: The Convergence of Loglangs and Neuro-Symbolic AI**

The contemporary landscape of Artificial Intelligence is defined by a dichotomy: the unprecedented success of connectionist models (Deep Learning) in pattern recognition and language generation, contrasted against the enduring robustness of symbolic systems in logical reasoning and interpretability. Neuro-Symbolic AI (NeSy) has emerged as the synthesis of these paradigms, aiming to unite the learning capabilities of neural networks with the deductive power of formal logic.1 However, a persistent bottleneck in NeSy research remains the "Semantic Gap"—the inherent ambiguity of natural languages like English, which resists precise mapping to formal logical structures without error-prone intermediate parsing steps.3  
This report explores a novel architectural paradigm: the utilization of **Lojban**—a constructed language (conlang) engineered for syntactic unambiguousness and predicate logic isomorphism—as the native interface for a Neuro-Symbolic Reasoning Engine. By leveraging Lojban, we bypass the stochastic noise of natural language understanding (NLU) and interface directly with the logical substrate of intelligence. We propose a rigorous implementation strategy using **Rust**, a systems programming language that offers the requisite memory safety, type rigor, and performance for symbolic manipulation, alongside a rapidly maturing Machine Learning ecosystem represented by frameworks such as **Burn** and **Scallop**.4

### **1.1 The Semantic Gap in Natural Language Processing**

Traditional approaches to Neuro-Symbolic AI often involve a pipeline where natural language is first parsed into a logical form (e.g., First-Order Logic or $\\lambda$-calculus) and then processed by a symbolic reasoner. This "Semantic Parsing" stage is fraught with peril due to the irregularities of evolved languages. A sentence such as "I saw the man with the telescope" contains structural ambiguities that require extensive contextual knowledge to resolve—knowledge that is often probabilistic rather than deterministic. In high-stakes reasoning environments, such ambiguity introduces a fundamental fragility; if the parser misinterprets the syntax, the downstream logical engine reasons upon false premises.3  
Current State-of-the-Art (SOTA) architectures attempt to mitigate this through massive parameter counts in Large Language Models (LLMs), effectively memorizing statistical correlations between sentence structures and logical forms. However, this remains an approximation. The "Neural Theorem Prover" approach attempts to learn reasoning directly from text, but often hallucinates steps or fails to maintain consistency over long chains of deduction.6

### **1.2 Lojban: A Logical Substrate**

Lojban serves as a solution to this structural deficit. Developed by the Logical Language Group (LLG) as a successor to James Cooke Brown's Loglan, Lojban was explicitly designed to test the Sapir-Whorf hypothesis and to facilitate human-computer communication.8 Its pivotal feature for AI implementation is its **Parsing Expression Grammar (PEG)**, which guarantees that any valid string of text resolves to exactly one unique parse tree.9  
Unlike natural languages, where the mapping between syntax (grammar) and semantics (meaning) is opaque, Lojban enforces a strict isomorphism. A sentence (*bridi*) in Lojban is structurally identical to a predicate in formal logic. The word *klama*, for example, is not merely a verb meaning "to go"; it is a defined relation with five specific arguments (*sumti*): $x\_1$ comes to $x\_2$ from $x\_3$ via $x\_4$ using vehicle $x\_5$.11 This "place structure" allows a machine to extract the logical arguments directly from the syntax without semantic guessing.

### **1.3 The Rust Ecosystem for NeSy**

The choice of Rust for this implementation is strategic. While Python dominates the prototyping phase of ML research, Rust offers "Zero-Cost Abstractions," ensuring that high-level logical constructs do not incur runtime performance penalties—a critical factor when executing complex Datalog queries or traversing massive Knowledge Graphs (KGs).4  
Furthermore, the Rust ecosystem has recently seen the emergence of specialized tools for NeSy:

* **Burn:** A dynamic deep learning framework that supports custom differentiable operators, essential for implementing "Real Logic" tensor operations.12  
* **Scallop:** A Datalog engine written in Rust that supports differentiable reasoning via provenance semirings, allowing gradients to flow from logical proofs back to neural embeddings.5  
* **Camxes.rs:** A native Rust implementation of the official Lojban PEG parser, enabling high-performance, zero-copy AST generation.13

This report details a blueprint for integrating these components into a cohesive reasoning engine. We will analyze the theoretical mapping of Lojban to Logic Tensor Networks (LTN), the handling of complex n-ary relations via hyper-relational embeddings, and the specific Rust software architecture required to realize this system.

## ---

**2\. Theoretical Foundations: Lojban as Executable Logic**

To architect the reasoning engine, we must first establish a formal correspondence between Lojban's linguistic primitives and the computational structures of Neuro-Symbolic AI. This section dissects Lojban not as a language for communication, but as a data serialization format for First-Order Predicate Logic (FOPL).

### **2.1 The *Bridi* and N-ary Predicate Relations**

The fundamental unit of meaning in Lojban is the *bridi* (predication). In classical FOPL, a proposition is represented as $P(t\_1, t\_2, \\dots, t\_n)$, where $P$ is a predicate symbol and $t\_i$ are terms. Lojban mirrors this structure exactly.  
The core of the *bridi* is the *selbri* (the relationship), and the arguments are the *sumti*. Consider the *gismu* (root word) **dunda** 14:  
*dunda*: $x\_1$ (donor) gives $x\_2$ (gift) to $x\_3$ (recipient).  
In a binary Knowledge Graph (KG) format like RDF (Resource Description Framework), this ternary relationship must be reified or broken into multiple triples (e.g., \_:event type giving, \_:event agent x1, \_:event object x2, \_:event beneficiary x3). This "reification" explodes the graph size and complicates reasoning. Lojban, however, is natively **n-ary**. Most root words have between 2 and 5 places, with some having more.11  
This necessitates a reasoning engine that operates on **Hyper-Relational Knowledge Graphs** rather than simple dyadic graphs. The tensor representation of a *bridi* in our engine must therefore be a function $G(P): \\mathcal{E}^{n} \\rightarrow $, where $\\mathcal{E}$ is the embedding space of the entities. This directly challenges standard embedding techniques like TransE, which assume $h \+ r \\approx t$, and requires advanced architectures like **StarE** or **NeuInfer**.15

### **2.2 Place Structures and Positional Semantics**

The rigidity of Lojban's place structure is its greatest asset for symbolic reasoning. In English, the role of a noun is determined by prepositions ("to the market", "from the house") or word order, but these are inconsistent ("I gave the market a visit"). In Lojban, the position $x\_2$ of *klama* is *always* the destination, and $x\_3$ is *always* the origin.17  
This positional semantics allows for **Direct Slot Filling** in the neural architecture. We can define the input to our neural predicate as a fixed-size tensor list:

$$\\text{Input} \= \[ \\mathbf{v}\_{selbri}, \\mathbf{v}\_{x1}, \\mathbf{v}\_{x2}, \\mathbf{v}\_{x3}, \\mathbf{v}\_{x4}, \\mathbf{v}\_{x5} \]$$  
Where logical variables (empty places or *zo'e*) are masked or represented by a learnable "unknown" vector. This structure allows the implementation of **Logic Tensor Networks (LTN)** where the grounding of a predicate is a neural network explicitly architected to consume this exact input vector configuration.18

### **2.3 The *Tanru* Challenge: Neuro-Symbolic Composition**

While *gismu* have fixed logical definitions, Lojban allows the construction of *tanru*—metaphorical compounds where one term modifies another. For example, *sutra tavla* ("fast talker").19  
In strict symbolic logic, *tanru* are ill-defined; the relationship between *sutra* and *tavla* is not explicitly stated. It could mean "talks quickly," "talks about speed," or "talks while moving fast." This ambiguity, usually a defect in logical languages, is the primary entry point for the **Neural** component of our engine.

* **Symbolic Handling:** The parser identifies *tanru* as a binary tree structure: tanru(sutra, tavla).  
* **Neural Handling:** The engine employs **Vector Composition**. The embedding for the compound concept is a function of its parts: $\\mathbf{v}\_{compound} \= f(\\mathbf{v}\_{sutra}, \\mathbf{v}\_{tavla})$.

This requires the engine to implement a **Compositional Operator** (e.g., a multi-layer perceptron or a geometric intersection in a cone space) that learns the semantic interaction of these terms from data. The engine can then treat the composed vector $\\mathbf{v}\_{compound}$ as a new ad-hoc *selbri* with the place structure of the final term (*tavla*), preserving logical consistency while capturing fuzzy semantic nuance.20

### **2.4 Logical Connectives and Truth Functions**

Lojban's system of logical connectives (*ga/gi*, *je*, *nagi*, etc.) provides a complete set of Boolean operators (AND, OR, XOR, IFF) and their negations.21 Crucially, Lojban distinguishes between **Forethought** connectives (*ga... gi*) and **Afterthought** connectives (*....a...*).

* **Forethought (*ga A gi B*):** Allows the parser to construct the logical operation node *before* parsing the operands. This is computationally efficient for stack-based parsers and aligns with Polish Prefix Notation.  
* **Truth Functional Isomorphism:** The connective *ga* maps exclusively to the Inclusive OR ($\\lor$) operator. In a Neuro-Symbolic system, this maps to the **S-Norm** (T-Conorm) in fuzzy logic (e.g., $\\max(x, y)$ or $x+y-xy$).22

This explicit mapping means that the "Reasoning" layer of the engine does not need to *infer* the logical relationship (as it would with the English "and," which can imply temporal sequence or causation); it simply *executes* the operator defined by the *cmavo*.

## ---

**3\. Neuro-Symbolic Architectures: Selection and Adaptation**

Implementing this Lojbanic logic requires a specific class of AI architecture. We evaluate several candidates and propose a hybrid model combining **Logic Tensor Networks** for semantic grounding and **Differentiable Datalog** for structural inference.

### **3.1 Logic Tensor Networks (LTN) and "Real Logic"**

LTN is a framework that integrates learning and reasoning by embedding logic into a continuous space.18 It introduces **Real Logic**, where truth values are continuous in $$.  
**Mathematical Formulation for Lojban:**

1. **Grounding ($G$):**  
   * **Sumti (Terms):** Maps a symbol $s$ to a vector $\\mathbf{v}\_s \\in \\mathbb{R}^k$.  
   * **Selbri (Predicates):** Maps an n-ary relation $P$ to a function $G(P): \\mathbb{R}^{n \\cdot k} \\rightarrow $.  
2. **Operators:**  
   * **Conjunction ($A \\land B$):** Approximated by a T-Norm (e.g., product logic: $T(a,b) \= a \\cdot b$).  
   * **Disjunction ($A \\lor B$):** Approximated by an S-Norm (e.g., probabilistic sum: $S(a,b) \= a \+ b \- a \\cdot b$).  
   * **Quantifiers ($\\forall x P(x)$):** Aggregated via Generalized Mean or Softmax.

LTN is ideal for Lojban because it handles the "fuzzy" satisfaction of predicates like *melbi* (beautiful) or *cizra* (strange), which are subjective. The engine can learn the grounding function for *melbi* from a dataset of labeled examples, while enforcing logical constraints (e.g., $\\forall x (\\text{blanu}(x) \\rightarrow \\text{kloryzma}(x))$ — "all blue things are colored-things").

### **3.2 Differentiable Datalog (Scallop)**

While LTN is powerful for grounding, it struggles with deep multi-hop reasoning chains due to vanishing gradients in deep computation graphs. **Scallop** addresses this by using **Datalog**, a subset of Prolog, extended with **Provenance Semirings**.5  
**Scallop's Role:**

* **Structure:** Lojban *bridi* are naturally represented as Datalog facts: klama(x1, x2, x3, x4, x5).  
* **Provenance:** Scallop tracks the *provenance* of each fact—essentially the proof tree that derives it. By associating probabilities (from the Neural layer) with base facts, Scallop calculates the probability of derived facts.  
* **Recursion:** Scallop supports recursive rules, essential for processing Lojban's relative clauses (*poi* clauses) which can nest indefinitely.

### **3.3 The Hybrid "StarE" Architecture**

To handle the n-ary nature of Lojban predicates within this framework, we adopt the **StarE** (Star-Graph Encoder) approach.15

* Standard Graph Neural Networks (GNNs) operate on edges. In Lojban, a *bridi* is a hyper-edge connecting up to 5 nodes.  
* **StarE** splits the hyper-edge into a set of binary relations augmented with "qualifiers."  
  * Primary Triple: $(x\_1, \\text{selbri}, x\_2)$  
  * Qualifiers: $\\{(x\_3, \\text{place}\_3), (x\_4, \\text{place}\_4), \\dots\\}$  
* **Transformer Encoder:** The StarE model uses a Transformer to aggregate the embeddings of the primary triple and all qualifiers. This captures the interactions between arguments (e.g., how the *route* $x\_4$ modifies the *destination* $x\_2$).

**Integration in Rust:**  
This architecture can be implemented in **Burn** by creating a custom module that acts as the "Neural Predicate." This module takes the argument embeddings, processes them through a StarE Transformer block, and outputs a logit representing the truth value of the *bridi*.

## ---

**4\. The Rust Implementation Stack**

The rigorous implementation of this engine relies on a specific stack of Rust crates, chosen for their stability, performance, and alignment with Neuro-Symbolic goals.

### **4.1 Input Layer: Parsing with camxes.rs and pest**

The entry point of the system is the **Parser**. We rely on **camxes.rs**, a Rust port of the official Lojban parser.13  
**Technical Detail: Zero-Copy Parsing**  
Parsing large corpora requires efficiency. The pest parser generator (which underlies many Rust parsers) supports **Zero-Copy Parsing**.

* **Mechanism:** Instead of allocating new String objects for every node in the AST, the parser produces Pair objects that borrow slice references (\&str) from the original input string.  
* **Benefit:** drastically reduces heap allocation and garbage collection pressure (conceptually, though Rust has no GC).  
* **AST Structure:** The output is a tree of pest::iterators::Pair enums, which we map to a strongly-typed LojbanAST struct:  
  Rust  
  pub enum LojbanNode {  
      Bridi(Selbri, Vec\<Sumti\>),  
      Tanru(Box\<LojbanNode\>, Box\<LojbanNode\>),  
      //...  
  }

### **4.2 Neural Layer: The Burn Framework**

We select **Burn** over **Candle** or **tch-rs** for the neural substrate.

* **Reasoning:** Burn is a "comprehensive dynamic Deep Learning Framework" built entirely in Rust.12 It allows for the definition of dynamic computation graphs, which is essential for handling Lojban's variable sentence structures and recursive *tanru*.  
* **Backend Flexibility:** Burn supports WGPU (WebGPU) for cross-platform hardware acceleration and LibTorch for raw CUDA performance. This allows the reasoning engine to run on consumer hardware or edge devices.  
* **Custom Operators:** To implement "Real Logic," we need custom differentiable operators (e.g., fuzzy AND/OR). Burn's tensor API allows implementing these directly:  
  Rust  
  // Conceptual Rust code for Fuzzy AND (Godel T-Norm) in Burn  
  fn fuzzy\_and\<B: Backend\>(a: Tensor\<B, 1\>, b: Tensor\<B, 1\>) \-\> Tensor\<B, 1\> {  
      Tensor::min\_pair(a, b) // Element-wise minimum  
  }

### **4.3 Logical Layer: The Scallop Crate**

The reasoning core is built on **Scallop**.

* **Integration:** Scallop provides a Rust API (scallop-core) that allows embedding the Datalog compiler directly into the application.5  
* **Custom Provenance:** We implement a custom provenance semiring to bridge Burn's output with Scallop's logic.  
  * **Semiring Trait:** We define a struct LojbanProb implementing Scallop's Semiring trait.  
  * **Operations:** We map the add (OR) and mul (AND) methods of the trait to the fuzzy logic operators defined in the Burn layer.  
  * **Gradient Flow:** The crucial feature is that Scallop can compute the gradient of the proof. If the reasoning engine concludes a fact with probability 0.4 but the ground truth is 1.0, Scallop calculates $\\nabla \\mathcal{L}$ and propagates it back to the specific Burn embeddings that contributed to that proof.

## ---

**5\. Architectural Blueprint: The "Lojban-NeSy" Engine**

This section provides a detailed modular breakdown of the proposed engine.

### **5.1 Module 1: The Input Normalizer**

Before reasoning can occur, the raw AST from camxes must be normalized into a **Logical Intermediate Representation (LIR)**.

* **Rafsi Resolution:** Lojban compounds (*lujvo*) are made of affixes (*rafsi*). The normalizer decomposes *lujvo* (e.g., *brivla*) into their semantic roots (*bridi* \+ *valsi*), allowing the engine to reason about unknown words based on their components.25  
* **Anaphora Resolution:** Handles references like *ko'a* (he/it) or *di'u* (the previous sentence). This requires a stateful **Discourse Context Stack** that stores the LIR of previous utterances.  
* **Elision Recovery:** Fills explicit zo'e (unspecified arguments) into empty logical slots to ensure the tensor inputs have consistent dimensions.

### **5.2 Module 2: The Neural Grounding Service**

This module maintains the **VocabStore** and computes embeddings.

* **Data Structure:** A HashMap\<String, Tensor\<B, 1\>\> storing learnable vectors for all 1300 *gismu*.  
* **N-ary Encoder (StarE):** A Burn module that takes a *selbri* embedding and a list of *sumti* embeddings.  
  * **Input:** Queries like (klama,).  
  * **Process:**  
    1. Positional Encoding: Add a "Place Vector" ($P\_1 \\dots P\_5$) to each *sumti* embedding to enforce Lojban's strict positional semantics.  
    2. Self-Attention: Apply a Transformer layer to allow arguments to contextualize each other.  
    3. Pooling: Aggregate to a single scalar "truth score."

### **5.3 Module 3: The Symbolic Reasoner (Scallop Interface)**

This module manages the Datalog Knowledge Base (KB).

* **Fact Injection:** Converts high-confidence LIRs into Datalog facts.  
  * Lojban: *li pare cu nanca la djan* (John is 12 years old).  
  * Datalog: nanca(12, "John").  
* **Rule Engine:** Contains static rules derived from Lojban definitions (e.g., *If x1 is a mother of x2, x1 is a parent of x2*).  
* **Query Processing:**  
  * User asks: *ma klama* (Who comes?)  
  * Translated to: Query(X) :- klama(X, \_, \_, \_, \_).  
  * Scallop executes the query, retrieving all $X$ that satisfy the relation, ranked by their provenance probability.

### **5.4 Module 4: The Feedback Loop (Training)**

The system is trained end-to-end.

* **Loss Function:** We use a **Satisfiability Loss**. The objective is to maximize the truth value of observed Lojban sentences and minimize the truth value of contradictions (negative sampling).  
* **Loop:**  
  1. Parse batch of Lojban text.  
  2. Ground to tensors.  
  3. Compute truth scores via Burn (Neural) \+ Scallop (Symbolic).  
  4. Compute Loss $\\mathcal{L} \= 1 \- \\text{Score}(\\text{observed})$.  
  5. optimizer.step(): Updates the *gismu* embeddings and the StarE encoder weights.

## ---

**6\. Specific Implementation Challenges and Solutions**

### **6.1 The Hyper-Relational Data Problem**

Standard KGE models (TransE, RotatE) support only $(h, r, t)$ triples. Lojban's 5-place predicates break this.

* **Naive Solution:** Reification. Convert klama(a,b,c,d,e) to 5 triples. **Critique:** Loses the atomic nature of the *bridi* and increases computational cost 5x.  
* **Proposed Solution (StarE in Burn):** We implement **StarE** 15, which is designed for hyper-relational KGs.  
  * **Mechanism:** The *selbri* acts as the graph centroid. The *sumti* are neighbors. The message passing step updates the *selbri* embedding based on its arguments. This allows the neural network to "understand" that the meaning of *klama* shifts slightly depending on whether the route ($x\_4$) is a road or a wormhole.

### **6.2 Fuzzy Semantics and *Tanru***

*Tanru* (metaphors) are infinite. We cannot store embeddings for all of them.

* **Solution:** **Geometric Composition**. We implement **Cone Embeddings**.26  
  * Each *gismu* is modeled not as a point, but as a Cone in $\\mathbb{R}^n$.  
  * A *tanru* (intersection of concepts) is the geometric intersection of two cones.  
  * **Burn Implementation:** We define a custom ConeIntersection layer.  
  * **Logic:** If the intersection is empty (volume \= 0), the *tanru* is nonsensical. If non-empty, the centroid of the intersection region is the new embedding for the concept.

### **6.3 Handling "La'e" and Abstractions**

Lojban allows abstractors like *nu* (event of), *ka* (property of), *du'u* (predication of).

* **Problem:** These turn a whole *bridi* into a *sumti*.  
* **Solution:** **Recursive Embedding**.  
  * The encoder module is recursive. When it encounters a *nu* clause, it calls itself to encode the sub-bridi.  
  * The output vector of the sub-bridi (usually a truth score) is *not* used directly. Instead, we take the **hidden state** of the StarE encoder (the semantic representation before the truth pooling) and use *that* as the input embedding for the *nu* sumti in the parent sentence.

## ---

**7\. Case Study: A Reasoning Walkthrough**

To illustrate the engine's operation, we trace the processing of a single complex sentence.  
**Input:** *lo prenu cu sutra klama le zarci*  
("A person quickly-goes to the market.")  
**Step 1: Parsing (camxes.rs)**

* Output AST: Bridi { selbri: Tanru(sutra, klama), sumti: \[prenu, zarci\] }. (Note: *zarci* is in $x\_2$ because *cu* separates the head).

**Step 2: Normalization**

* Implicit $x\_1$: *lo prenu* fills $x\_1$.  
* Implicit $x\_2$: *le zarci* fills $x\_2$.  
* Explicit *zo'e*: Adds Unspecified tokens for $x\_3, x\_4, x\_5$.  
* LIR: REL: \[sutra, klama\], ARGS: \[prenu, zarci, zo'e, zo'e, zo'e\].

**Step 3: Neural Grounding (Burn)**

* **Tanru Comp:** $\\mathbf{v}\_{sk} \= \\text{Compose}(\\mathbf{v}\_{sutra}, \\mathbf{v}\_{klama})$.  
* **Sumti Emb:** Lookup $\\mathbf{v}\_{prenu}$, $\\mathbf{v}\_{zarci}$.  
* **StarE Encoder:**  
  * Input: $\[\\mathbf{v}\_{sk}, \\mathbf{v}\_{prenu} \\oplus P\_1, \\mathbf{v}\_{zarci} \\oplus P\_2, \\dots\]$  
  * Output: Truth Score $S \\in $ and Representation Vector $\\mathbf{h}\_{fact}$.

**Step 4: Symbolic Reasoning (Scallop)**

* Fact: sutra\_klama("person\_entity", "market\_entity")::0.92.  
* Rule in KB: sutra\_klama(X, Y) :- klama(X, Y), sutra(X).  
* Inference: The system infers that the *person* is fast (*sutra*) and that a *going* event occurred.  
* Query Check: If the user asks *ma sutra* (Who is fast?), the system returns *lo prenu* with confidence 0.92.

## ---

**8\. Conclusion**

The "Lojban-NeSy" engine represents a convergence of linguistic design and computational architecture. By utilizing Lojban, we eliminate the parsing ambiguity that plagues natural language AI. By utilizing Rust, we achieve the performance and safety requisite for a production-grade reasoning system. And by integrating **Logic Tensor Networks** with **Differentiable Datalog (Scallop)**, we create a hybrid mind capable of both learning fuzzy concepts from data (via *tanru* and embeddings) and reasoning rigorously about them (via Lojban's predicate logic structure).  
This blueprint provides a comprehensive technical roadmap. The implementation of the **StarE** architecture within **Burn** to handle Lojban's n-ary hyper-relations, combined with a **Scallop**\-based provenance engine for logical deduction, constitutes a novel and viable path toward robust Artificial General Intelligence (AGI) subsystems that are both logically sound and capable of learning.

### **8.1 Future Outlook**

The next phase of this research involves the creation of the **Lojban Knowledge Graph (LKG)**—a massive dataset of grounded Lojban *bridi* to train the embedding layers. Furthermore, the development of a "Lojban-to-Circuit" compiler could allow these logical structures to run directly on FPGA hardware, realizing the vision of a truly "Logical Machine."

## **9\. References**

* **Lojban & Logic:**.8  
* **Parsing (PEG/Rust):**.9  
* **Neuro-Symbolic AI (LTN/Scallop):**.1  
* **Machine Learning (Burn/Embeddings):**.12  
* **Rust Ecosystem:**.4  
* **Lojban Semantics:**.19

#### **Works cited**

1. Neuro-Symbolic AI in 2024: A Systematic Review \- arXiv, accessed February 13, 2026, [https://arxiv.org/html/2501.05435v1](https://arxiv.org/html/2501.05435v1)  
2. Neuro Symbolic Architectures with Artificial Intelligence for Collaborative Control and Intention Prediction \- GSC Online Press, accessed February 13, 2026, [https://gsconlinepress.com/journals/gscarr/sites/default/files/GSCARR-2025-0288.pdf](https://gsconlinepress.com/journals/gscarr/sites/default/files/GSCARR-2025-0288.pdf)  
3. A Natural Language Interface Using First-Order Logic \- Lojban, accessed February 13, 2026, [https://mw.lojban.org/images/b/b9/NaturalLanguageInterfaceUsingFOL.pdf](https://mw.lojban.org/images/b/b9/NaturalLanguageInterfaceUsingFOL.pdf)  
4. Rust Formal Methods Interest Group, accessed February 13, 2026, [https://rust-formal-methods.github.io/](https://rust-formal-methods.github.io/)  
5. scallop-lang/scallop: Framework and Language for Neurosymbolic Programming. \- GitHub, accessed February 13, 2026, [https://github.com/scallop-lang/scallop](https://github.com/scallop-lang/scallop)  
6. Neural Theorem Prover \- CS224d: Deep Learning for Natural Language Processing, accessed February 13, 2026, [https://cs224d.stanford.edu/reports/yuan.pdf](https://cs224d.stanford.edu/reports/yuan.pdf)  
7. Neural Theorem Provers: How Reasoning AI Is Learning Formal Math — and What It Means for Global Enterprises | by RAKTIM SINGH | Medium, accessed February 13, 2026, [https://medium.com/@raktims2210/neural-theorem-provers-how-reasoning-ai-is-learning-formal-math-and-what-it-means-for-944aa356fb06](https://medium.com/@raktims2210/neural-theorem-provers-how-reasoning-ai-is-learning-formal-math-and-what-it-means-for-944aa356fb06)  
8. Lojban \- Wikipedia, accessed February 13, 2026, [https://en.wikipedia.org/wiki/Lojban](https://en.wikipedia.org/wiki/Lojban)  
9. Parsing expression grammar \- Wikipedia, accessed February 13, 2026, [https://en.wikipedia.org/wiki/Parsing\_expression\_grammar](https://en.wikipedia.org/wiki/Parsing_expression_grammar)  
10. grammar \- La Lojban, accessed February 13, 2026, [https://mw.lojban.org/papri/grammar](https://mw.lojban.org/papri/grammar)  
11. The Complete Lojban Language (2016)/Chapter 2 \- Wikisource, the free online library, accessed February 13, 2026, [https://en.wikisource.org/wiki/The\_Complete\_Lojban\_Language\_(2016)/Chapter\_2](https://en.wikisource.org/wiki/The_Complete_Lojban_Language_\(2016\)/Chapter_2)  
12. huggingface/candle: Minimalist ML framework for Rust \- GitHub, accessed February 13, 2026, [https://github.com/huggingface/candle](https://github.com/huggingface/candle)  
13. lojban/camxes-rs \- GitHub, accessed February 13, 2026, [https://github.com/lojban/camxes.rs](https://github.com/lojban/camxes.rs)  
14. from Wikibooks: Lojban/Place structure, accessed February 13, 2026, [https://mw.lojban.org/papri/from\_Wikibooks:\_Lojban/Place\_structure](https://mw.lojban.org/papri/from_Wikibooks:_Lojban/Place_structure)  
15. Schema-Aware Hyper-Relational Knowledge Graph Embeddings for Link Prediction \- eXascale Infolab, accessed February 13, 2026, [https://exascale.info/assets/pdf/KnowledgeGraphEmbeddings\_TKDE2024.pdf](https://exascale.info/assets/pdf/KnowledgeGraphEmbeddings_TKDE2024.pdf)  
16. (PDF) NeuInfer: Knowledge Inference on N-ary Facts \- ResearchGate, accessed February 13, 2026, [https://www.researchgate.net/publication/343298846\_NeuInfer\_Knowledge\_Inference\_on\_N-ary\_Facts](https://www.researchgate.net/publication/343298846_NeuInfer_Knowledge_Inference_on_N-ary_Facts)  
17. The Basic Components (sumti and selbri) \- Lojban, accessed February 13, 2026, [https://www.lojban.org/publications/level0/brochure-utf/comp.html](https://www.lojban.org/publications/level0/brochure-utf/comp.html)  
18. Logic Tensor Networks arXiv:2012.13635v1 \[cs.AI\] 25 Dec 2020, accessed February 13, 2026, [https://arxiv.org/abs/2012.13635](https://arxiv.org/abs/2012.13635)  
19. tanru and lujvo-making \- Lojban, accessed February 13, 2026, [https://www.lojban.org/files/papers/lujvo-making.htm](https://www.lojban.org/files/papers/lujvo-making.htm)  
20. Semantic parsing using Lojban – On the middle ground between semantic ontology and language, accessed February 13, 2026, [https://www.inf.uni-hamburg.de/en/inst/ab/lt/teaching/theses/completed-theses/2014-ma-hinz.pdf](https://www.inf.uni-hamburg.de/en/inst/ab/lt/teaching/theses/completed-theses/2014-ma-hinz.pdf)  
21. The Complete Lojban Language \- ESP, accessed February 13, 2026, [https://esp.mit.edu/download/25c32ad5-dcf3-47d9-b42c-bf5a64969bc7/X11542\_cll\_v1.1\_book-1.pdf](https://esp.mit.edu/download/25c32ad5-dcf3-47d9-b42c-bf5a64969bc7/X11542_cll_v1.1_book-1.pdf)  
22. Logic Tensor Networks \- Neuro Symbolic AI, accessed February 13, 2026, [https://neurosymbolic.asu.edu/wp-content/uploads/sites/28/2023/03/Lecture06-LTN.pdf](https://neurosymbolic.asu.edu/wp-content/uploads/sites/28/2023/03/Lecture06-LTN.pdf)  
23. (PDF) Logic Tensor Networks \- ResearchGate, accessed February 13, 2026, [https://www.researchgate.net/publication/347965941\_Logic\_Tensor\_Networks](https://www.researchgate.net/publication/347965941_Logic_Tensor_Networks)  
24. josephmisiti/awesome-machine-learning: A curated list of awesome Machine Learning frameworks, libraries and software. \- GitHub, accessed February 13, 2026, [https://github.com/josephmisiti/awesome-machine-learning](https://github.com/josephmisiti/awesome-machine-learning)  
25. The Complete Lojban Language, accessed February 13, 2026, [https://la-lojban.github.io/uncll/uncll-1.2.4/xhtml\_no\_chunks/](https://la-lojban.github.io/uncll/uncll-1.2.4/xhtml_no_chunks/)  
26. Embedding Ontologies in the Description Logic ALC by Axis-Aligned Cones \- FIS Universität Bamberg, accessed February 13, 2026, [https://fis.uni-bamberg.de/bitstreams/73b2dbeb-7fdc-4ceb-a984-cc7dccc3c85a/download](https://fis.uni-bamberg.de/bitstreams/73b2dbeb-7fdc-4ceb-a984-cc7dccc3c85a/download)  
27. About Lojban, accessed February 13, 2026, [https://lojban.io/FAQ](https://lojban.io/FAQ)  
28. pest \- Rust \- Docs.rs, accessed February 13, 2026, [https://docs.rs/pest](https://docs.rs/pest)  
29. Introducing pest into glsl and hindsight about nom vs. pest (part 1\) \- strongly-typed-thoughts.net, accessed February 13, 2026, [https://phaazon.net/blog/glsl-pest-part-1](https://phaazon.net/blog/glsl-pest-part-1)  
30. Scallop: From Probabilistic Deductive Databases to Scalable Differentiable Reasoning \- Department of Computer Science, University of Toronto, accessed February 13, 2026, [https://www.cs.toronto.edu/\~six/data/neurips21b.pdf](https://www.cs.toronto.edu/~six/data/neurips21b.pdf)  
31. Scallop: A Language for Neurosymbolic Programming \- ResearchGate, accessed February 13, 2026, [https://www.researchgate.net/publication/369945806\_Scallop\_A\_Language\_for\_Neurosymbolic\_Programming](https://www.researchgate.net/publication/369945806_Scallop_A_Language_for_Neurosymbolic_Programming)  
32. DHGE: Dual-View Hyper-Relational Knowledge Graph Embedding for Link Prediction and Entity Typing \- AAAI Publications, accessed February 13, 2026, [https://ojs.aaai.org/index.php/AAAI/article/view/25795/25567](https://ojs.aaai.org/index.php/AAAI/article/view/25795/25567)  
33. Our teams' favorite Rust crates 2025 \- Freestyle, accessed February 13, 2026, [https://docs.freestyle.sh/blog/rust-crates-2025](https://docs.freestyle.sh/blog/rust-crates-2025)  
34. COMPILATION TECHNIQUES, ALGORITHMS, AND DATA STRUCTURES FOR EFFICIENT AND EXPRESSIVE DATA PROCESSING SYSTEMS \- Purdue University Graduate School, accessed February 13, 2026, [https://hammer.purdue.edu/ndownloader/files/43357701](https://hammer.purdue.edu/ndownloader/files/43357701)  
35. me lu ju'i lobypli li'u 14 moi \- La Lojban, accessed February 13, 2026, [https://mw.lojban.org/papri/me\_lu\_ju%27i\_lobypli\_li%27u\_14\_moi](https://mw.lojban.org/papri/me_lu_ju%27i_lobypli_li%27u_14_moi)