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

- **Synchronize the manuscript with explicit, flavor-exact temporal rules (engine
  decision complete 2026-08-04).** The engine no longer gives a tensed goal a hidden
  second pass over Bare rules: unprefixed ordinary-predicate literals remain Bare, and
  authors write same- or cross-flavor mappings explicitly (`past P -> past Q`,
  `past P -> now Q`). Engine/root docs, KR seam, determinism, materialisation ON/OFF,
  proof traces, and zero-skip Vampire/clingo cases pin that contract. The separate
  `book/` repo still teaches the removed behavior: Chapter 5 calls bare rules timeless
  and automatically lifted; Chapter 7 documents the deleted `strip_tense_from_fact` /
  `apply_tense_to_fact` path; Chapter 9 makes lifting a pipeline step and exercise;
  Appendix H lists it as supported. Sweep Chapters 4, 5, 7, 9, 11, 17, and 21;
  Appendices F, G, H, and I; and `book-outline.md` for derivative claims. This is a
  `book-todo` task in the nested repo (its Maintenance umbrella already names temporal
  lifting), not an engine-code change. **Exit:** examples distinguish bare-rule FALSE
  from explicitly flavored TRUE and cross-flavor causation; implementation walkthroughs
  contain no deleted helper/phase; limitations describe conservative surface-relation
  stratification and the separately tracked stacked-modifier gap; `verify_book.py`,
  `verify_book_refs.py`, and `capture_book.py --check` pass in the book repo, and root
  `just verify-book-refs` reports no stale temporal claim.

- **Make stacked tense×deontic semantics compositional or reject the syntax.**
  KR accepts and the seam structurally pins `must past ~P` as
  `Obligatory(Past(Not(P)))` (`NIBLI_KR.md` §6; `nibli_kr_seam_gate.rs`), but runtime
  facts/rule templates have one mutually exclusive `StoredFact` flavor and the
  evaluator/flatteners overwrite an outer wrapper when they descend into an inner one
  (`nibli-reason/src/reasoning.rs`:561-637; `rules.rs`:186-237,361-409). The compiled
  tree therefore advertises a modifier product the reasoner cannot represent. Choose a
  real product type with defined NAF/proof/materialisation semantics, or fail closed on
  stacked modifiers before assertion/query; do not silently pick one wrapper. **Exit:**
  fact, query, antecedent, conclusion, NAF, proof-trace, and retraction tests cover both
  modifier orders; the seam and external oracle either model the product or pin rejection;
  KR/IR/guarantees and the relevant book chapters state the same implemented contract.

- **Reconcile existential import with find/count semantics.** A description universal
  mints `presupposition_witnesses` when existential import is on
  (`nibli-reason/src/rules.rs`:1290-1320). Those constants satisfy existential and
  universal queries, but `CountNode` explicitly removes them
  (`nibli-reason/src/reasoning.rs`:1046-1068), and the KB contract says `??` does too
  (`nibli-reason/src/kb.rs`:734-740). The same KB can therefore report that some `P`
  exists while exposing zero `P` witnesses/count, which is not ordinary FOL counting and
  is easy to misread as a contradiction. Choose one coherent profile: include imported
  witnesses everywhere, expose separate logical-versus-observed count/find operations,
  or make clean-core/no-import the high-assurance default. In every case expose witness
  origin. **Exit:** metamorphic tests cover `some P`, `?? P`, `exactly 0/1 P`, and count
  before/after retraction with import on/off; UI/host report the active profile; Chapters
  5 and 8 plus Appendices A, B, E, and K match the implemented algebra.

- **Separate internal Skolem identity from user constants.** `GroundTerm::Constant`
  represents both user data and generated witnesses (`nibli-reason/src/kb.rs`:145-165),
  the generator emits unchecked `sk_<counter>` strings (:956-965), and reasoning uses
  that spelling as a semantic classifier (`nibli-reason/src/reasoning.rs`:2440-2452).
  Direct LogicBuffer/WIT, replay, and compute paths can supply the same string even if the
  KR surface normally cannot, aliasing user data with an internal witness. Introduce a
  tagged/opaque Skolem term, or reserve and reject the namespace at every ingress and stop
  inferring origin from a prefix. **Exit:** adversarial `sk_0` constants remain distinct
  through equality, event roles, find/count, proof, persistence, and retraction/rebuild;
  rendering may show a friendly `sk_N` without using that display as identity; direct,
  component, replay, and compute collision tests pass with `just test`,
  `just test-engine`, `just ci-wasm`, and `just verify-soundness`.

- **Stop presenting generative exact-count assertions as persistent cardinality
  constraints.** A top-level `big(exactly 1 dog).` currently creates one fresh matching
  witness (`nibli-reason/src/rules.rs`:1783-1834); it does not register an invariant.
  A later matching assertion can make the original count false, and `exactly 0` stores no
  prohibition at all. Choose one explicit contract: persist/enforce an atomic cardinality
  constraint (including zero), reject count assertions and retain counts as queries, or
  introduce separately named generative syntax. **Exit:** sequences for exactly-one then a
  second match, zero then a match, duplicate/equality-collapsed entities, retraction,
  rebuild/reopen, proof provenance, and import on/off are pinned through the KR surface;
  the renderer/spec/Chapters 4 and 8 plus Appendices A/B/E stop calling witness generation
  an exact-cardinality assertion.

---

## Compute / fact lifecycle

- **Auto-ingested compute facts bypass the fact registry.** An arithmetic/backend
  `true` auto-asserted mid-query (`assert_typed_fact`) writes only the typed fact
  store: no `FactRecord`, no fact id — invisible to `:facts`, not retractable, and
  dropped by any rebuild (retraction, failed-assert rollback, and trap replay all
  rebuild from the registry). README now DISCLOSES both halves (the ghost-fact lifecycle and the outage-cache
  read-back), so this is no longer an undocumented surprise — but the DECISION is still
  unmade, and the store-shadowing policy question in `nibli-reason/src/compute.rs` still
  points here. Decide the lifecycle: give ingested
  facts registry records (listable, retractable, replay-safe), or codify the
  current outage-cache semantics as the contract (dispatch-first on every query;
  the stored fact is consulted only when dispatch errors) and say so in
  GUARANTEES.md.

- **Track fact origin so traces cannot call derived/cache facts “asserted.”**
  `trace_predicate_provenance_typed` treats any exact `fact_store` hit as
  `ProofRule::Asserted` (`nibli-reason/src/reasoning.rs`:2724-2735), but that same store
  receives forward-derived facts (`nibli-reason/src/rules.rs`:985-1098) and compute
  auto-ingestion. Neither carries the user `FactRecord` id/label/source. This can turn a
  derived or trusted-oracle premise into a displayed `[given]`, defeating the proof's
  trust-boundary story. Introduce explicit origin metadata (user assertion id, rule id,
  compute request/response provenance, presupposition, or other internal source), retain
  it through equivalence and rebuild, and render origins honestly. **Exit:** tests force
  the same ground fact through each origin and require distinct serialized/WIT proof
  steps; duplicate user assertions remain separately citable; retraction/reopen retains
  provenance; Chapters 10, 11, 16 and Appendices C/E recapture real output.

- **Define a high-assurance compute admission policy.** Today a backend `true` crosses a
  plaintext JSONL/TCP seam and is inserted as a premise. The current disclosure is honest,
  but there is no authenticated request/response binding, backend/schema version,
  freshness/expiry/revocation policy, or durable provenance. Decide whether v0.1 remains
  explicitly low-assurance or add an opt-in policy that verifies identity/integrity and
  records the exact request, response, backend version, timestamp/nonce, and admission
  decision before assertion. **Exit:** tampered, replayed, stale, mismatched, unavailable,
  and revoked responses have fail-closed tests; proof/export surfaces identify oracle
  premises; protocol and host docs state the residual TCB. Ripple: Chapters 2, 16, 21 and
  Appendices C/E/H.

---

## Reasoning / evaluation

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
  equality, duplicate assertions, compute premises, and replay; WIT/protocol/host/UI and
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
