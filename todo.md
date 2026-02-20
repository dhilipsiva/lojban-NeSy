### Phase 1: Make the semantics crate stop lying

These are bugs where the parser does real work that gets silently discarded. No new features — just honoring what already exists.

**1.1 — Handle `bridi.negated` in `compile_bridi`**
The parser sets `bridi.negated = true` for `na`-prefixed sentences. `compile_bridi` never reads this field. Add `Not` to `LogicalForm` in `ir.rs`, wrap the final form when `bridi.negated` is true.

**1.2 — Handle `Selbri::Negated` in `apply_selbri`**
Currently falls to the `_ =>` unknown predicate catch-all. Should recurse into the inner selbri and wrap in `Not`.

**1.3 — Handle `Sumti::Tagged` in `compile_bridi`**
Currently falls to `_ => LogicalTerm::Unspecified`. Match on `Tagged((tag, inner_id))`, resolve the inner sumti, and insert at `tag.to_index()` positionally into the args vector instead of appending sequentially. This requires changing the args-building logic from a linear push to a positional insert with a pre-allocated `Vec<Option<LogicalTerm>>` of size `target_arity`.

**1.4 — Handle `Sumti::Restricted` (relative clauses) in `compile_bridi`**
Currently falls to catch-all. For `lo gerku poi barda`, the restrictor `barda(x)` should be conjoined with the description's existential. Resolve the inner sumti, compile the rel clause body as a `LogicalForm`, and conjoin it inside the quantifier scope.

**1.5 — Handle `Selbri::WithArgs` (be/bei) in `apply_selbri`**
Currently falls to unknown. Should extract the core selbri, bind the `args` from the be/bei clause into the predicate's argument positions (starting at x2), then apply the core selbri with the merged argument list.

**1.6 — Handle `Selbri::Connected` in `apply_selbri`**
Currently falls to unknown. For `je` (AND): apply both selbri to the same args, conjoin. Requires `Or` in the IR for `ja`.

**1.7 — Handle `Selbri::Grouped` (ke/ke'e) in `apply_selbri`**
Trivial — just recurse into the inner selbri. Currently falls to unknown for no reason.

### Phase 2: Extend the type system to carry the new semantics

**2.1 — Add `Not`, `Or`, `Implies` to `ir::LogicalForm`**
Required by 1.1, 1.2, 1.6. Trivial enum extension.

**2.2 — Add `not-node`, `or-node`, `implies-node` to WIT `logic-node` variant**
The WIT type is the serialization boundary. Without this, the new IR types can't cross to the reasoning crate.

**2.3 — Extend `flatten_form` in `semantics/lib.rs`**
Handle the new `LogicalForm` variants when serializing to `LogicBuffer`.

**2.4 — Extend `reconstruct_sexp` in `reasoning/lib.rs`**
Handle the new `LogicNode` variants when generating egglog s-expressions.

**2.5 — Extend `reconstruct_debug_sexp` in `orchestrator/lib.rs`**
Same function, duplicated. (Or fix the duplication — see 4.2.)

### Phase 3: Make the reasoning engine actually reason

**3.1 — Replace `(run 10)` with `(run-schedule (saturate ...))`**
Deterministic fixpoint instead of arbitrary step count.

**3.2 — Add conjunction elimination/introduction rules**
```
(rule ((IsTrue (And A B))) ((IsTrue A) (IsTrue B)))
(rule ((IsTrue A) (IsTrue B)) ((IsTrue (And A B))))
```
Without these, `And` is opaque to queries.

**3.3 — Add modus ponens**
```
(rule ((IsTrue (Implies A B)) (IsTrue A)) ((IsTrue B)))
```

**3.4 — Add negation rules (double negation, De Morgan's)**
Required once `Not` flows through from Phase 2.

**3.5 — Design existential instantiation strategy**
Hardest item. egglog doesn't natively support Skolemization. Options: Skolem functions at the semantics level (replace `∃x.P(x)` with `P(sk_n)` before it hits egglog), or use egglog's `function` declarations to generate fresh constants. This is a design decision, not just code.

### Phase 4: Architectural hygiene

**4.1 — Move `?` query routing from orchestrator to runner**
The runner should parse `:quit`, `?`, and any future commands. The WIT interface should export `assert` and `query` separately. The orchestrator shouldn't know about UI sigils.

**4.2 — Extract `reconstruct_sexp` into shared code**
Either a shared Rust crate that compiles into each component, or a WIT-level `debug-print` function owned by one component.

**4.3 — Improve `execute` return type**
Change from `bool` to `result<execution-result, string>` with a variant enum so the runner can distinguish assertion success, query true/false, and specific error types.

**4.4 — Add `roots: list<u32>` to `ast-buffer`**
Currently `sentences` mixes top-level sentences with rel clause bodies. Either add an explicit roots list (like `logic-buffer` has) or separate them.

**4.5 — Rename/split `ast-types` interface**
`logical-term` and `logic-node` don't belong in `ast-types`. Split into `ast-types` and `logic-types`, or rename to `types`.

**4.6 — Verify wasip1 vs wasip2 target alignment**
Your flake advertises wasip2, your build outputs target wasip1 paths. Pick one and align the flake shellHook, Justfile, and cargo-component config.

### Phase 5: Parser hardening (lower priority)

**5.1 — Add recursion depth limit to `grammar.rs`**
Nested `poi` clauses can stack overflow in WASM's 1MB stack. Add a depth counter to the `Parser` struct, check on each recursive call.

**5.2 — Fix place tag backtracking**
`try_parse_term` consumes a place tag but doesn't restore position if no sumti follows. Save/restore around the tag parse.

**5.3 — Fix `bevri` arity in `CORE_GISMU_ARITIES`**
Listed as 4-place, CLL defines it as 5-place.

**5.4 — Validate `sa` degradation behavior**
Currently acts as `si` (single-word erase). Document this limitation or implement selma'o-based class erasure.

### Phase 6: Long-term research (not immediate)

**6.1 — Neo-Davidsonian event semantics**
Reify events: `∃e. prami(e) ∧ agent(e, mi) ∧ theme(e, do)`. This changes the entire IR structure and predicate arity model. Don't touch this until Phases 1-3 are solid.

**6.2 — Quantifier scope ambiguity**
`lo`/`le`/`ro`/`su'o` with proper scope resolution. Currently only `lo` (existential) is handled.

**6.3 — Non-monotonic reasoning / belief revision**
Retraction of facts, default reasoning. Fundamentally changes the egglog model.
