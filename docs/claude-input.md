# Lojban NeSy Engine — Comprehensive Review

## BUGS (Will cause incorrect behavior or crashes)

### B1. `(let f1 ...)` rebinding in reasoning `assert_fact` [reasoning/lib.rs]

Every call to `assert_fact` runs `(let f1 <sexp>)`. egglog `let` binds a global name. The second assertion either shadows or errors depending on egglog version. After two assertions, the engine is in undefined territory.

**Fix:** Maintain an atomic counter alongside the `Mutex<EGraph>` and generate unique names (`f1`, `f2`, `f3`...), or drop `let` entirely and inline the expression directly into the `IsTrue` call:

```rust
let command = format!("(IsTrue {})\n(run 10)", sexp);
```

---

### B2. Bumpalo arena dropped while AST is still borrowed [parser/lib.rs]

```rust
fn parse_text(input: String) -> Result<AstBuffer, String> {
    let arena = Bump::new();          // created here
    let raw_tokens = tokenize(&input);
    let normalized = preprocess(...);
    let ast = parse_tokens_to_ast(&normalized, &arena)?;  // borrows arena
    Ok(flatten_to_buffer(ast))        // arena dropped here
}
```

This works *today* only because `flatten_to_buffer` eagerly `.to_string()`s everything. But the arena allocation is wasted — you pay for bump allocation then immediately copy to the heap. If anyone later tries to return arena-borrowed data, it's use-after-free.

**Fix:** Either remove bumpalo entirely (use owned types in the AST), or move the arena to a caller-controlled scope where the borrow is valid for the full lifetime needed.

---

### B3. `Box<Selbri>` in arena-allocated AST [parser/ast.rs]

`Sumti::Description(Box<Selbri<'a>>)` and `Selbri::Tanru(Box<Selbri<'a>>, Box<Selbri<'a>>)` allocate on the global heap, not the bump arena. This defeats the purpose of using bumpalo and creates a mixed allocation model (some data in arena, some on heap).

**Fix:** Either:
- Use `&'a Selbri<'a>` references with `arena.alloc()` instead of `Box`, or
- Drop bumpalo entirely and use `Box` everywhere (simpler, and you're copying to owned strings at the WIT boundary anyway)

---

### B4. `sa` (EraseClass) panics at runtime [parser/preprocessor.rs]

```rust
LojbanToken::EraseClass => {
    unimplemented!("'sa' (EraseClass) resolution requires...");
}
```

Any Lojban input containing `sa` causes a WASM trap, killing the pipeline. Same for valid Lojban text that happens to contain `sa` as a substring matched by the lexer.

**Fix:** Replace with a no-op that logs a warning, or return `Err`:

```rust
LojbanToken::EraseClass => {
    // V1: treat as no-op, log warning
    eprintln!("Warning: 'sa' erasure not yet supported, ignoring");
}
```

---

### B5. `to_sexp` panics on `ForAll`/`Exists` [semantics/semantic.rs]

```rust
_ => unimplemented!("Other forms deferred for V1 MVP"),
```

If any code path ever constructs a quantified `LogicalForm` and serializes it, the WASM component traps. The variants exist in the IR enum but can't be serialized.

**Fix:** Either implement stub serialization or remove the variants from the enum until supported:

```rust
LogicalForm::ForAll(var, body) => {
    format!("(ForAll \"{}\" {})", self.interner.resolve(var), self.to_sexp(body))
}
LogicalForm::Exists(var, body) => {
    format!("(Exists \"{}\" {})", self.interner.resolve(var), self.to_sexp(body))
}
```

---

### B6. Cmevla regex is too greedy, overlaps with Gismu [parser/lexer.rs]

```rust
#[regex(r"[a-zA-Z'\.]+[^aeiouy \t\n\f\.]")]
Cmevla,
```

This pattern will match words that should be classified as Gismu. For example, `donri` (a valid gismu meaning "daytime") ends in `i` (vowel), so it *shouldn't* match Cmevla — but the character class `[a-zA-Z'\.]+` could greedily consume partial matches in edge cases, especially with names adjacent to other words. More critically, the pattern matches any string ending in a consonant, which will eat multi-word sequences without proper word boundary anchoring.

**Fix:** Add word boundary anchors or restructure the regex to be non-greedy. Test specifically with: `djan`, `donri`, `la .djan.`, `alis`.

---

### B7. Query entailment uses error-as-control-flow [reasoning/lib.rs]

```rust
match egraph.parse_and_run_program(None, &command) {
    Ok(_) => true,
    Err(e) => false, // conflates "not entailed" with "malformed query" or egglog bug
}
```

A malformed S-expression, an egglog internal error, and a legitimate `false` result are all treated identically.

**Fix:** Inspect the error message — egglog's `check` failure has a specific format. At minimum, distinguish between check-failure and other errors:

```rust
Err(e) => {
    let msg = format!("{}", e);
    if msg.contains("Check failed") {
        false
    } else {
        eprintln!("[Reasoning] Unexpected error: {}", e);
        false
    }
}
```

---

### B8. Runner REPL crashes on WASM traps instead of recovering [runner/main.rs]

```rust
reasoning_inst
    .lojban_nesy_reasoning()
    .call_assert_fact(&mut store, &sexp)?;
```

The `?` operator propagates any WASM trap (from bugs B4, B5, or malformed input) as an `anyhow::Error`, which exits the REPL entirely. One bad input kills the session.

**Fix:** Match the error and continue the REPL loop:

```rust
if let Err(e) = reasoning_inst
    .lojban_nesy_reasoning()
    .call_assert_fact(&mut store, &sexp)
{
    eprintln!("[Host] Reasoning error: {}", e);
    continue;
}
```

---

## IMPROVEMENTS (Ordered by impact and dependency chain)

### Phase 1: Stabilize what exists

#### I1. Delete dead code and unused dependencies

| Item | Location |
|---|---|
| `reasoning.rs` | `reasoning/src/` — old dynamic-injection implementation that won't compile (references `dictionary::JbovlasteSchema` and `semantic::SemanticCompiler` which don't exist in this crate). Delete. |
| `orchestrator` crate | Root tree — not in workspace `members`. Delete or add to workspace. |
| `quick-xml` | `reasoning/Cargo.toml` — declared but unused. Remove. |
| `nom` | `parser/Cargo.toml` — declared but unused. Remove. |
| `lojban.pest` | `parser/src/` — file exists but no `pest`/`pest_derive` dependency. Remove. |
| `rustix` (commented) | `runner/Cargo.toml` — dead comment. Remove. |
| `serde` | `semantics/Cargo.toml` — declared but no `Serialize`/`Deserialize` derives in code. Remove unless planned. |

#### I2. Align Rust editions

Runner uses `edition = "2021"`, everything else uses `edition = "2024"`. Align to `2024`.

#### I3. Add `result` types to WIT interface for semantics and reasoning

```wit
interface semantics {
    use ast-types.{ast-buffer};
    compile-buffer: func(ast: ast-buffer) -> result<list<string>, string>;
}

interface reasoning {
    assert-fact: func(sexp: string) -> result<_, string>;
    query-entailment: func(sexp: string) -> result<bool, string>;
}
```

This makes errors part of the contract instead of causing WASM traps.

#### I4. Fix `build.rs` XML parsing [semantics/build.rs]

Replace `split("<valsi ")` + manual string extraction with `quick-xml` or `roxmltree` in `build-dependencies`. Current approach silently breaks on CDATA sections, XML entities (`&amp;`), multiline attributes, or `<valsi` appearing in definition text.

Also: the arity extraction (`extract_arity`) searches for `x5`/`x4` etc. in definitions after stripping tags, but jbovlaste definitions use inconsistent formats (`$x_1$`, `{x1}`, prose descriptions). Validate against a known-correct arity table for at least the core gismu.

#### I5. Make default arity explicit, not silent [semantics/dictionary.rs]

```rust
*JBOVLASTE_ARITIES.get(word).unwrap_or(&2)
```

Silently defaulting to 2 produces wrong logical forms for unknown words. Return `Option<usize>` and let the caller decide. The hardcoded `klama` fast-path suggests the PHF might have a bug for that entry — investigate and remove the override if the dictionary is correct.

---

### Phase 2: Parser — the critical bottleneck

#### I6. Implement recursive descent parser with `nom`

The current parser is a flat single-pass loop that only handles `[sumti*] cu? selbri [sumti*]`. Everything downstream is bottlenecked by this. Priority constructs to implement, in order:

1. **Sentence separator `.i`** — enables multi-sentence input
2. **Place tags `fa`/`fe`/`fi`/`fo`/`fu`** — explicit argument positions
3. **Negation `na`/`naku`** — required for reasoning to be non-trivial
4. **Relative clauses `poi`/`noi`** — restrictive vs. non-restrictive (maps to conjunction vs. apposition in FOL)
5. **Sumti raising `be`/`bei`** — arguments inside selbri
6. **Grouping `ke`/`ke'e`** — tanru structure
7. **Connectives `je`/`ja`/`jo`/`ju`** — logical connectives map directly to FOL ∧/∨/↔/⊕
8. **Terminators `ku`/`vau`/`kei`** — proper scope closure

You already have `nom` in deps. Use it. `nom` is a better fit than `pest` for this — Lojban's grammar is context-sensitive in places (metalinguistic operators, elidable terminators), which PEG handles poorly.

#### I7. Handle `lo`-descriptions as quantified forms in semantics

This is the single biggest semantic gap. Currently:

```
"mi prami lo gerku" → prami(mi, Desc("gerku"))
```

Correct FOL:

```
"mi prami lo gerku" → ∃x. gerku(x) ∧ prami(mi, x)
```

Your IR already has `Exists` and `And` constructors. Use them. The `SemanticCompiler` needs to generate fresh variables (e.g., a counter-based `x0`, `x1`, ...) and wrap predicates with existential quantifiers when processing `Sumti::Description`.

---

### Phase 3: Reasoning improvements

#### I8. Batch `(run N)` instead of per-assertion

Move `(run N)` out of `assert_fact` and into a separate `saturate()` call, or invoke it only before queries:

```rust
fn query_entailment(sexp: String) -> bool {
    let mut egraph = get_egraph().lock().unwrap();
    egraph.parse_and_run_program(None, "(run 100)").ok();  // saturate before checking
    let command = format!("(check (IsTrue {}))", sexp);
    // ...
}
```

#### I9. Replace `Pred1`..`Pred5` with variadic representation

The arity-indexed constructors don't scale to BAI-extended arities (which you've explicitly said you want to preserve by not truncating args). The `.clamp(1, 5)` in `to_sexp` silently drops data for arity > 5.

Options:
- Variadic S-expression: `(Pred "klama" (Args (Const "mi") (Const "do") (Zoe) (Zoe) (Zoe)))`
- Keep `Pred1`..`Pred5` but add `PredN String (Vec Term)` as overflow

#### I10. Add Lojban-specific rewrite rules to egglog

The current rules (commutativity of `And`, double-negation elimination) are generic FOL. Lojban has productive transformations that map directly to e-graph rewrites:

```scheme
;; se-conversion: swap x1 and x2
(rewrite (IsTrue (Pred2 rel a b)) (IsTrue (Pred2 (se rel) b a)))

;; te-conversion: swap x1 and x3
(rewrite (IsTrue (Pred3 rel a b c)) (IsTrue (Pred3 (te rel) c b a)))
```

These would enable non-trivial entailment queries: assert `mi prami do`, query `do se prami mi` → `true`.

#### I11. Add `se`/`te`/`ve`/`xe` conversion support (parser + semantics + reasoning)

This is a cross-cutting feature:
- **Parser**: Recognize `se`/`te`/`ve`/`xe` as selbri modifiers
- **Semantics**: Permute argument positions in the generated `LogicalForm`
- **Reasoning**: Add rewrite rules (I10) for bidirectional inference

---

### Phase 4: Architecture improvements

#### I12. Eliminate `map_buffer_to_semantics` in runner

The 50-line identity mapping between parser and semantics types exists only because `bindgen!` generates separate types per world. Options:

- **Best:** Use `wasm-tools compose` to compose the three `.wasm` components into a single `engine-pipeline` component (your WIT already defines this world). The runner then instantiates one component.
- **Good:** Write a macro that generates `From` impls between the structurally identical types.
- **Minimum:** At least add a comment explaining why this exists so it doesn't look like a mistake.

#### I13. Separate REPL commands for assert vs. query [runner/main.rs]

Currently, every input is parsed → compiled → asserted → trivially queried (same fact). Implement command prefixes:

```
lojban> mi prami do          # default: assert
lojban> :query do se prami mi   # query entailment
lojban> :facts                  # dump knowledge base
lojban> :clear                  # reset e-graph
```

#### I14. Consider separate `Store` instances per component

Currently all three WASM components share one Wasmtime `Store` (one linear memory). A bug in the parser can corrupt reasoning state. For production sandboxing, use separate stores. The tradeoff is negligible — you're already serializing across the WIT canonical ABI.

#### I15. Verify egglog compiles to `wasm32-wasip2`

egglog's dependency tree includes `log`, `hashbrown`, symbol tables, and a parser. Some of these may use `std::time` or platform-specific features that fail under WASI P2. If you haven't done a clean `cargo component build --release --target wasm32-wasip2 -p reasoning`, this is a likely blocker. Test early.

---

### Phase 5: Long-term semantic completeness

#### I16. Tanru modifier preservation

`Selbri::Tanru` currently extracts only the head's string, discarding the modifier. `sutra gerku` becomes just `gerku`. The modifier relationship should be represented in the IR, e.g.:

```scheme
(And (IsTrue (Pred1 "gerku" x)) (IsTrue (Pred1 "sutra" x)))
```

#### I17. Non-monotonic reasoning / negation

egglog is inherently monotonic. You can add equalities, never remove them. This blocks:
- `na` (bridi negation)
- `naku` (logical negation)
- `da'i` (hypothetical/counterfactual)
- Belief revision / retraction

Long-term options: fork the e-graph per hypothetical context, layer a Datalog engine with stratified negation (ascent, crepe, or Scallop as originally planned) alongside egglog, or implement a truth-maintenance system (TMS) on top.

#### I18. Proper quantifier scope for `lo`/`le`/`la`

Beyond I7 (basic existential for `lo`), Lojban has a rich quantifier system:
- `lo` — veridical description (∃)
- `le` — non-veridical reference (specific referent, not necessarily satisfying the predicate)
- `la` — named entity (constant)
- `ro` — universal (∀)
- `su'o` — existential with cardinality
- `pa`, `re`, `ci`... — numeric quantifiers

Each requires different FOL translation. This is where the semantics crate becomes genuinely interesting.

#### I19. Event semantics (neo-Davidsonian)

The current IR uses flat predicates: `prami(mi, do)`. The standard approach in formal semantics is to reify events:

```
∃e. prami(e) ∧ agent(e, mi) ∧ theme(e, do)
```

This enables temporal reasoning, adverbial modification, and causality chains — all expressible in Lojban via tense/aspect system (`pu`/`ca`/`ba`, `za'o`/`co'a`, etc.). This is a fundamental architectural shift but the correct long-term target for a neuro-symbolic engine.
