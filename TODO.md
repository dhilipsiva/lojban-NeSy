# Engine TODO

**Future-facing only.** An entry is here because it is still true and still wants doing —
delete it when it lands rather than marking it done, and put the record in the commit.
The pre-existing entries were re-verified against the tree on 2026-08-01. The
review-derived entries added on 2026-08-04 were checked against the cited current source;
where a claim cites a line, that line was checked, not remembered.

Release runbook: **`RELEASING.md`**. Docs hosting: **`DEPLOY.md`**. Book manuscript:
separate `book/` repo (`book/TODO.md`). The docs + release tracker `DOCS_TODO.md` was
RETIRED 2026-08-03 — docs Phases 0–5 and releases R0–R3 all landed, so what survived it
is the two entries below plus the policy that moved into RELEASING.md and CLAUDE.md.

---

## Authorization — SHIPPED, constraints that outlive it

The A0–A5 track landed in full: `nibli-auth` + `nibli-auth-py`, `authorizer` on
`nibli:engine@0.6.0`, `policy/auth-0.1.0.nibli`, Rust and Python adapters, both
examples, the mdBook chapter, and a dedicated `auth` job in CI
(`just test-auth && just check-auth-axum`). The phase-by-phase record lived here and
is now in git history; what remains are the rules a FUTURE change has to respect.

- **Forbidden names for auth heads:** `can`, `field`, `principal` — all three collide
  with corpus entries (`lante` tin/can, `foldi` agricultural field, `ralju`). The KR
  heads are `authorized(agent, action, resource)` and
  `visible_attr(agent, resource, attr)`; the host-language method names stay
  `can` / `allowed_fields` / `explain`.
- **Do not overload `permits`/`permitted` as HTTP auth.** They are the deontic corpus
  pair and mean something else.
- **UNKNOWN on the hot path DENIES.** Only an engine TRUE allows; proofs stay opt-in
  via `explain`.
- **One warm `Authorizer` per process** — never per-request, and never `reset()`
  between requests (it drops the policy).
- **Still non-goals:** Extism as primary runtime; per-request engine spawn; weakening
  CWA/CDA; inventing vocabulary outside the fail-closed corpus; framework-specific
  policy languages.
- **Open, small:** `just test-auth-py` stays local (maturin), so the Python bridge has
  no CI coverage. Either add a maturin job or state that the Python adapter is
  tested-by-hand.

---

## Editor / Linguist / LSP (syntax tooling)

Policy: book and gates keep fence tags **`nibli` / `nibli-kr`** (never a foreign
language alias). Seed grammar lives in `grammars/` (`nibli.tmLanguage.json` +
injection sketch + `grammars/README.md`). Runtime lexer remains
`nibli-kr/src/highlight.rs` (REPL / UI) — keep token classes aligned when anything
here changes. Keywords must stay equal to `nibli_lexicon::RESERVED_WORDS`.

- **VS Code / Cursor extension (local):** package `grammars/nibli.tmLanguage.json`
  as language id `nibli` for `*.nibli`, plus markdown fence injection for
  `nibli` and `nibli-kr` via `grammars/nibli-markdown-injection.json`. Ship as
  `editors/vscode/` (or a small sibling repo) with `language-configuration.json`
  (`#` line / `/* */` block). Optional: publish to Open VSX / Marketplace.
- **GitHub Linguist PR:** submit `source.nibli` + samples from shipped corpora
  (`gdpr.nibli`, `drug-interactions.nibli`, `readme.nibli`, …); language name
  **Nibli**, extensions `.nibli`, aliases `nibli-kr` / `nibli kr`. Until merge,
  github.com fences stay uncolored — do not retag book fences to `prolog` etc.
- **Tree-sitter grammar (`tree-sitter-nibli`):** new crate/repo tracking
  `nibli-kr.pest` by discipline (not codegen). Queries: `highlights.scm`,
  `locals.scm`, `injections.scm` for Markdown. Consumers: nvim-treesitter,
  Helix, Zed. Does **not** replace Linguist/TextMate for github.com.
- **`nibli-lsp` (thin LSP):** workspace bin on `tower-lsp` (or `lsp-server`) using
  `nibli_kr::parse_checked` diagnostics, lexicon hover/completion (gloss,
  places, templates), optional format via `nibli_kr::render`, optional semantic
  tokens from `nibli_kr::highlight::lex`. Single-file first; multi-file KB
  projects later.
- **Conformance guard — TextMate half LANDED** (`just verify-grammar-parity`, in
  `ci`: set, order and the `\b` anchors vs `RESERVED_WORDS`). Still owed: the same
  ratchet for the Tree-sitter keyword list once that grammar exists.
- **Editor/LSP docs (mdBook)** — blocked, precondition UNMET, and the last docs
  item that outlived `DOCS_TODO.md`. Everything above is design + seed artifacts:
  `grammars/` holds a TextMate grammar and an injection sketch that **no build or
  CI job consumes** (only the keyword ratchet reads it), and there is no
  `editors/` dir, no `nibli-lsp` crate, no tree-sitter grammar, and no `tower-lsp`
  anywhere — `Cargo.lock` included. Writing the page now would present aspiration
  as shipped, which the docs' own epistemic rule forbids. Revisit when
  `editors/vscode/` or `nibli-lsp` actually exists.
- **Book fence rename (optional, coordinated):** if/when fences go plain
  `nibli` only, retarget `verify_book.py` + capture harness + injection regex in
  one PR; keep `nibli-kr` as Linguist/VS Code alias forever.

---

## Language semantics

- **Synchronize the manuscript with the resolved existential-import algebra (engine
  decision complete 2026-08-05; book-repo work only).** The engine now uses clean-core
  as the high-assurance default: a description universal mints no entity, so `some`,
  `??`/`query_find`, `count_witnesses`, and `exactly 0/1` all range over the same
  ordinary domain. Explicit legacy import remains available
  (`set_existential_import(true)`, `NIBLI_EXISTENTIAL_IMPORT=1`,
  `:existential-import on`); when enabled, every imported witness participates in
  ∃/∀/find/count/aggregate-derived enumeration rather than being selectively hidden.
  `WitnessBinding.origin`, `ExistsWitness.origin`, and
  `CountResult.existential_imported` expose that contribution; a profile toggle
  transactionally rebuilds the loaded KB, and the host/browser surfaces report the
  active profile. Engine evidence: `nibli-engine/tests/integration.rs` pins the ON/OFF
  `some`/forall/find/exact-0/exact-1/count/aggregate/retraction matrix and live toggles;
  `smoke-host-existential-import` pins the component path; the independent oracle
  engines explicitly select clean-core. **Remaining work:** in the separate `book/`
  repository, update Chapters 5, 8, and 12 plus Appendices A, B, E, and K to describe
  this exact algebra, the default/opt-in controls, structural witness origin, WIT 0.9.0
  (including generated origin on find/existential/universal proof surfaces),
  and the fact that ON→OFF/OFF→ON applies immediately to already-loaded rules. Chapter
  12's consent capture currently depends on the historical import profile (the engine's
  `ch12_consent_case_study_traced_query_completes` watchdog now selects it explicitly),
  so either mark that profile in the capture or recast the example for clean-core. Run the
  book validator, reference checker, capture checker, and two byte-idempotence passes
  before removing this residual with the `book-todo` workflow.

- **Synchronize the manuscript with query-only exact-count semantics (engine
  decision complete 2026-08-05; book-repo work only).** The engine no longer
  generates witnesses for an asserted `CountNode`: every assertion, assumption,
  and preassigned/replay ingress rejects `exactly N`/`no` in asserted position before
  allocating an id or mutating state. Counts remain valid queries over the current
  closed domain, collapse equality classes, include explicitly enabled legacy-import
  witnesses with structured `CountResult` provenance, and may change after facts,
  equality links, import-profile changes, or retractions. `exactly 0` is a query,
  never a stored prohibition. Counts inside opaque abstractions are quoted content,
  not outer-KB constraints. Legacy persisted count-assertion rows fail replay
  non-destructively. Root specs/docs and Formalize's assertion-authoring guard are
  synchronized; do not reintroduce the retired generative reading. Update Chapters
  4 and 8 plus Appendices A/B/E and any renderer/capture exposition to present
  exact counts only on query surfaces; rewrite examples that currently load them as
  KB statements. Run the book validator, reference checker, capture checker, and
  two byte-idempotence passes before removing this residual with the `book-todo`
  workflow.

---

## Reasoning / evaluation

- **Make numeric comparisons in rules compositional or reject the ambiguous
  syntax.** `greater` / `less` / `num_equal` remain ordinary `Predicate` nodes
  in the IR; query evaluation recognizes their event-decomposed numeric shape
  operationally in `try_evaluate_numeric_group`, but rule compilation lowers
  the same atoms to plain `StoredFact` templates. A positive numeric guard is
  therefore inert, a negated guard can overfire under NAF because the stored
  extension is empty, and a numeric comparison in a rule head is shadowed by
  query-time evaluation. The new `ComputeNode` assertion guard cannot cover
  this without also banning legitimate nonnumeric relational uses such as
  `greater(Alis, Bob)`. Choose a typed rule-atom representation with bound-value
  numeric dispatch and four-valued/proof propagation, or define a conservative
  assertion-time sort rule that rejects only potentially operational comparison
  atoms. **Exit:** surface and raw-IR tests cover positive/negated antecedents and
  heads, numeric and nonnumeric bindings, non-finite values, materialisation
  refusal, stratification, proof children, and external-oracle differentials;
  accepted syntax can never compile to a store lookup whose query twin computes.

- **Materialisation: the trace story (C2).** Proof-traced queries keep the
  backward-chaining path (`positive_lookup` lowered for their duration) because a
  materialised verdict has no derivation to record. To let them use the fast path,
  four things need answering: `trace_predicate_provenance_typed` falls to a
  `holds:false` `PredicateNotFound` for a TRUE reachable only by materialisation; a
  materialised FALSE has no per-rule blocking premise, which `proofs/Trace.lean`'s
  `Neg` constructor and `trace_soundness_conformance` both require; `ProofRule::
  ExistsWitness` names a witness term the projection eliminated; and `naf_dependent`
  can flip true→false when a positive lookup deletes the `Negation` steps beneath it
  (a user-visible honesty marker moving because of an optimisation). Minimum-churn
  option if pursued: `ProofRule::PredicateCheck { method: "materialized" }` — no WIT
  change — plus a `validate_cert` arm and a `factAx`-analogue bridge against `m.ext`.
- **Materialisation: incremental re-saturation (C3).** Every fact insert drops the
  saturation (`assert_typed_fact` → `invalidate_materialization`), so an interleaved
  `assert; query; assert; query` REPL session pays a full fixpoint per query, and
  `nibli-ui` re-asserts its whole tab per run by design. Datalog is monotone, so a
  seed addition can only GROW the model: a three-state dirty flag (`Clean` /
  `GrewBy(Vec<StoredFact>)` / `Invalid`) could resume the semi-naive loop from a
  one-tuple delta rather than rebuild — `eval_rule`'s `delta_pos` marker is already a
  delta-driven round. `Invalid` for retraction, rebuild, reset, rule registration, and
  any non-`Bare` or `equals` insert (both can retroactively disqualify a relation).

- **Bind proof traces to the full verdict and durable evidence.** `ProofStep` exposes only
  `holds: bool`, while `ProofTrace` carries `naf_dependent` and `cwa_false` but not the
  root `QueryResult`, UNKNOWN reason, or RESOURCE_EXCEEDED kind
  (`nibli-types/src/logic.rs`:268-297). Native callers receive a separate tuple, but a
  serialized/cached trace can no longer prove which non-TRUE result it accompanied.
  `ProofRule::Asserted` also contains only a display string, not a fact id. Define one
  versioned result-plus-certificate envelope (or narrow the public “proof” contract), with
  configuration/corpus/engine versions and stable evidence ids sufficient for an
  independent checker. **Exit:** round-trip and independent-validation tests cover TRUE,
  closed-world FALSE, arithmetic FALSE, every UNKNOWN reason, every resource kind, NAF,
  equality, duplicate assertions, proof-local compute evidence, and replay; WIT/protocol/host/UI and
  Appendix C evolve together.

- **Use structural proof-memo keys and checked proof indices.** The provenance tracer
  memoizes by `fact.to_display_string()` (`nibli-reason/src/reasoning.rs`:2700-2719), even
  though `StoredFact` already implements structural `Eq + Hash`. Human rendering is not a
  safe identity boundary (numeric bit patterns, wrappers, descriptions, and future display
  changes can alias), and step indices repeatedly cast `usize` to `u32`. Key memoization by
  `StoredFact` (plus any proof context that affects derivation), keep display at the render
  edge, and fail cleanly if a proof cannot be indexed. **Exit:** collision-shaped tests
  exercise structurally distinct facts with identical/ambiguous renderings and confirm no
  cross-reference reuse; oversized/deep traces return a typed resource/error outcome; the
  memo regression suite and `just verify-proofs` remain green.

- **Preserve or explicitly invalidate rule execution settings across rebuild.**
  `set_rule_forward` and `set_rule_priority` mutate compiled `UniversalRuleRecord`s, but
  those settings are not part of `FactRecord`; `rebuild_inner` recreates rules at default
  `forward=false, priority=0` and suppresses forward firing while `rebuilding` is true
  (`nibli-reason/src/lib.rs`:238-308; `rules.rs`:599-607,985-988). An unrelated retraction
  therefore changes rule configuration and fact-store/proof shape even when definitive
  query answers remain backward-derivable. Either persist/reapply the settings as session
  configuration or make the setters explicitly ephemeral and remove claims of replay
  equivalence that include them. **Exit:** set forward/priority, retract an unrelated id,
  rebuild/reopen, and assert the documented configuration, eager facts, proof origin, and
  verdict; add this case to the retraction differential.

- **Make aggregation fail closed instead of returning a partial numeric result.**
  `KnowledgeBase::aggregate` uses `filter_map` over witness bindings
  (`nibli-reason/src/lib.rs`:1128-1159), silently dropping a witness when the requested
  variable is absent or nonnumeric and then summing/averaging the survivors. It also does
  not reject non-finite operands or overflowed results, while the embedding API exposes
  only `Option<f64>`. Define a typed outcome that distinguishes empty input, incomplete or
  mixed bindings, non-finite input/result, and a valid aggregate; preserve the existing
  depth/cycle incompleteness refusal. **Exit:** missing-variable, mixed string/number,
  NaN/infinity, overflow, empty, and all-numeric controls are pinned; session/engine/WIT
  callers propagate the outcome without collapsing it; aggregate provenance states the
  contributing witnesses/count or the book narrows its proof claim. Ripple: Chapter 20 and
  Appendices B/E.

---

## Shipped case-study corpora

- **Redesign `drug-interactions.nibli` around a patient-local exposure event.** The
  concentration rules at :71-72 derive drug-level risk from global inhibition/metabolism,
  and the alert rule at :94 checks only that Adam uses the risky substrate. Retracting
  `uses(Adam, Flukonazol)` therefore does not withdraw the warfarin alert; a second patient
  taking warfarin alone can inherit risk from an inhibitor nobody in that regimen takes.
  Model co-administration explicitly and decide/document the dosage, route, timing,
  phenotype, evidence-quality, and uncertainty boundary with a pharmacology reviewer.
  **Exit:** patient-isolation and inhibitor-retraction negatives, dose/route/time boundary
  tests, and clinically reviewed scenario fixtures pass in `nibli-engine`, `nibli-ui`, and
  the pin/seam gates; Chapter 20 is recaptured from the live corpus and is labeled a
  synthetic teaching model until that review exists.

- **Replace person-level GDPR proxies with operation-scoped legal facts.**
  `gdpr.nibli`:46-78 derives a generic `permitted(person)` from `approves/promise/obliged`
  and uses absence of `approves` under NAF as the erasure trigger. It does not identify the
  processing operation, controller, data, purpose, consent scope/withdrawal time,
  alternative lawful bases, Article 17 exceptions, or jurisdiction/effective text; NAF
  absence is not a legal finding. Design an operation-scoped schema and corpus with legal
  review, explicit coverage assumptions, exceptions, and dated primary sources. **Exit:**
  multi-controller/multi-purpose/withdrawal/alternative-basis/exception counterexamples
  are pinned; missing evidence yields an honest non-compliance-neutral result rather than
  a legal conclusion; engine/UI fixtures and Chapter 19 consume the same live artifact.

---

## Pin runner / harness

- **`:accept-scoped` cannot scope a one-way declaration, and that asymmetry may want
  closing.** `derived_only`/`admits` are deliberately absent from `rebuild_inner`'s clear
  list, so they survive the replay a retraction performs; the runner refuses to scope them
  rather than silently no-op. If a KB ever needs a *scoped* vocabulary control, the engine
  would have to gain an explicit un-declare — which was rejected once already, because a
  vocabulary that can be re-opened at runtime gives back exactly the capability the
  declaration removes. Revisit only with a concrete need.
- **`--allow-shell` has no test that it stays OFF for `pins/`.** The gate is what keeps
  the pin language closed during `just ci`; nothing pins that `just verify-pins` never
  passes the flag. A one-line grep guard in the Justfile, or a test asserting the recipe's
  argv, would stop a future edit from quietly opening it.

---

## Docs hosting — site primary

- **Wire `dhilipsiva.dev/docs/nibli/`** in the external `dhilipsiva/dhilipsiva.dev`
  repo, on the `nibli-updated` dispatch this repo already fires: checkout nibli →
  `just docs` (default `site-url=/docs/nibli/`) → copy `mdbook/book/` →
  `public/docs/nibli/`. Recipe and requirements are `DEPLOY.md` §2b. **Do not call
  the primary URL live until it returns HTTP 200** — until then the canonical
  public docs URL is the GitHub Pages mirror, `dhilipsiva.github.io/nibli/`, which
  `docs-pages.yml` already deploys. Blocked here only in the sense that the work
  is in another repo; nothing in this one is missing. (Search needs no attention
  under either base path: every mdBook asset, the searcher and the index included,
  is referenced through `path_to_root`, verified in the built output.)

---

## Docs / surface defects

Surfaced by the book's review passes (2026-07-26) and RE-VERIFIED against the tree on
2026-08-01 — every entry below still reproduces. Each names its book ripple where one
exists.

- **Render computed FALSE as a decision, not closed-world non-derivability.** A query such
  as `greater(3, 5)` carries a false `ComputeCheck` leaf, but
  `nibli-render/src/summary.rs`:191-211 has no compute-false arm and falls back to “could
  not be derived from the known facts and rules.” Handle local arithmetic/numeric FALSE
  and trusted-backend FALSE explicitly, without a CWA caveat, while ordinary missing-fact
  FALSE keeps the non-derivability explanation. **Exit:** renderer, host, UI, protocol and
  WASM tests distinguish local computed FALSE, backend FALSE, backend unavailable,
  non-finite UNKNOWN, and ordinary CWA FALSE; Chapter 17 is recaptured from real bytes.

- **Settle the RDF export contract.** `nibli-import/src/export.rs`:1-17 calls its output
  N-Triples/RDF-like but emits only comment lines of the form `# fact:<id> <label>` and
  still describes labels as Lojban source. This is neither RDF nor a typed round-trip,
  while Chapter 21 advertises Turtle import and export. Either implement valid
  Turtle/N-Triples export from typed facts with tested IRI/literal mapping, or rename the
  feature to a fact-label dump and remove RDF-export claims. **Exit:** real RDF takes an
  export -> independent parser -> re-import round trip with identity/literal/alias tests;
  a narrowed dump gets an exact-format contract and stale comments removed. Synchronize
  Chapters 16/21, CLI help, README, and the reference gate.

- **Remove the retired two-path retraction story from active docs and benchmarks.**
  `KnowledgeBase::retract_fact_inner` now always rebuilds and ID-sorts survivors
  (`nibli-reason/src/lib.rs`:200-229,286-295), but the public API comment at :1179-1180,
  `nibli-engine/benches/engine_bench.rs`:198-246, and Chapter 11 still describe or measure
  an incremental/O(1) ground-fact path. Rename or remove the stale benchmark leg, measure
  the one real path, and reconcile API prose plus Chapter 11. **Exit:**
  `rg -n 'incremental.*retract|retraction_incremental|two-path' nibli-reason nibli-engine/benches book/P3_C11*`
  finds no active two-path claim; the replacement benchmark asserts its fixture/verdicts
  and reports reproducible hardware/profile/methodology; retraction tests remain green.

- **`obliged`-spelled every-duty renders the wrong obligated party.**
  `obliged(every data governs, event { message() }).` back-translates as "For every
  X, if X governs and X is data, then **Y** is obligated to notify" — the
  post-`8286738` deontic collapse picks the event variable as the duty-holder for
  the BASE spelling, while the converted `obligated_by` spelling correctly binds X.
  The who-selection appears keyed to the converted routing. Fix the base-spelling
  collapse; ripple: re-check nibli-wasm's `c18_draft_error_glosses_are_verbatim`
  pin and the book's Ch 18 alias note.
- **README's merged REPL-commands table** (README.md:241 `### REPL Commands`, table header still `| Command | Description |`) lists host-only commands
  (`:backend`/`:fuel`/`:memory`/`:strict`/`:existential-import`) and
  debug-REPL-only commands (`:contradictions`/`:trace`/`:untrace`/`:traces`) in
  one table with no binary column. It has ALSO gone stale a second way: it omits the
  host's `:materialize`/`:db`/`:dump`/`:export`/`:proof-verbose`
  (`nibli-host/src/main.rs`:1194, 1070, 1516, 1551, 1568) and the short aliases entirely.
  Split per surface or add a Surface column, and sweep the omissions in the same pass.
- **The component's import list is not gated.** Docs (book Ch 13/15/App C) state
  "no clock or filesystem imports" — true today (imports are the
  `wasi:cli`/`wasi:io` set + `wasi:random/insecure-seed`), but no CI check pins
  it and `wasm-tools` sits unused in the flake. A one-line `ci-wasm` smoke
  (`wasm-tools component wit target/wasm32-wasip2/release/nibli.wasm` + grep for
  the absent interfaces) closes the gap.
- **`resource_hint`'s Depth arm is dead code.** Hints fire only on the trap path
  (`nibli-host/src/main.rs`:1696) and `classify_resource_trap` never yields
  Depth, so a `RESOURCE_EXCEEDED (depth)` verdict prints no hint line — and the
  unreachable hint text (~main.rs:537) recommends raising `max_chain_depth`, a
  knob no shipped surface exposes. Wire a real Depth hint into the verdict path
  or drop the dead arm.
- **nibli-engine persist-before-assert leaves an orphan that BRICKS the database.** The
  store write-through (`nibli-engine/src/lib.rs`:292-308) mints the id and writes the
  durable row (`insert_fact`, :298) BEFORE `assert_fact_with_id` (:303). If the assert
  fails the in-memory KB rolls back (`nibli-reason/src/lib.rs`:179-186) but the persisted
  row gets no compensating tombstone — even though `NibliStore::retract_fact` exists and is
  used on the retract path (lib.rs:410). Consequence, reproduced with an unstratifiable-NAF
  rule: `replay_from_store` hard-fails on the orphan (`?` at lib.rs:229) and `open()` calls
  it unconditionally (:210), so EVERY later `NibliEngine::open` on that path returns
  `Err("Replay error (fact 2): Unstratifiable negation…")`. In-memory stays clean; the file
  is unopenable, with no engine-level recovery. Fix: write the durable row only AFTER the
  assert succeeds, or tombstone it on the error path — and add a nibli-engine test that
  asserts a refused fact and then reopens.
- **`wit/world.wit`:234 `proof-ref` doc comment is wrong.** It claims "No children —
  the full proof was shown at its first occurrence," but the step always carries
  exactly one back-reference child (the memo hit pushes
  `children: vec![cached_idx]`, `nibli-reason/src/reasoning.rs`:3050); the verbose
  text renderer re-expands that child while the collapsed/UI renderings drop it.
  Decide whether that divergence is intentional, then fix the comment; ripple:
  the book's Appendix C reproduces `world.wit` in full and must be updated
  together.

---

## Pointers

| Tracker / doc | Scope |
|---------|--------|
| **This file** | Engine runtime, editors/tooling, docs hosting, open semantics decisions — the ONE repo tracker since `DOCS_TODO.md` retired |
| **`RELEASING.md`** | Release decisions of record (tiers, lockstep, tags) + the operator runbook |
| **`DEPLOY.md`** | Hosting: playground, wasm demo, mdBook primary + Pages mirror |
| **`book/TODO.md`** | Manuscript only (private checkout; Orange AVA) |
