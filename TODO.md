# Engine TODO

**Future-facing only.** An entry is here because it is still true and still wants doing —
delete it when it lands rather than marking it done, and put the record in the commit.

Every surviving entry was re-verified against HEAD on 2026-08-08, after the 15-commit
merge `a7d288a` — against CURRENT source and, where the claim was behaviourally testable,
against a command actually run rather than remembered. That merge moved a great many line
numbers, so the citations below were corrected in the same pass; still check a line before
trusting it, and correct it in place when you do. A prior sweep dated 2026-08-01 is
superseded: reading this file against a pre-merge checkout will report items as open that
the merge closed. Entries the merge closed were deleted by the commits that closed them.

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

- **Decide whether a numeric comparison in a rule should COMPUTE.** The ambiguous syntax
  is now REFUSED rather than silently divergent (`validate_no_operational_comparisons`,
  `nibli-reason/src/kb.rs`), so the three wrong behaviours are gone: a positive guard no
  longer sits inert, a negated guard no longer overfires into a definitive wrong TRUE, and
  a rule head no longer derives a fact the query twin ignores. The refusal tests the
  OPERANDS, not the relation, so `greater(Alis, Bob)` keeps the relational reading — store
  lookup on both sides, hence no divergence.
  WHAT IS STILL OPEN is the capability: there is currently NO way to write "if quantity is
  greater than 15 then …". A rule cannot carry a comparison over a variable or a literal,
  and the ground fact it would need is refused too. That is a real limit on the
  quantitative direction Chapter 20 gestures at (dosage thresholds), and the alternative
  the original entry named is still available — give rule bodies a typed comparison atom
  that dispatches on bound values at firing time, so the syntax means what it looks like.
  Refusing first was chosen deliberately as forward-compatible: that change would only ever
  ACCEPT more, so nothing refused today becomes wrong later.
  If it is taken up, the parts a refusal did not have to answer are: operands unbound or
  non-numeric at firing time (fail closed), NAF semantics over a computed guard,
  stratification (a comparison is currently classified `base`, so its extension reads as
  complete-and-empty — that is precisely what made `~greater` overfire), materialisation
  refusal (`Ineligible::ComputeCondition` already exists), and proof children.
  **Exit:** surface and raw-IR tests cover positive/negated antecedents and heads, numeric
  and nonnumeric bindings, and non-finite values; the stratifier stops calling a computed
  comparison a base relation; materialisation refuses a rule carrying one; and the
  Vampire/clingo differentials cover the new shape.

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
  `assert; query; assert; query` REPL session recomputes the next requested query cone
  and its cumulative root union from scratch, and `nibli-ui` re-asserts its whole tab
  per run by design. Datalog is monotone, so a
  seed addition can only GROW the model: a three-state dirty flag (`Clean` /
  `GrewBy(Vec<StoredFact>)` / `Invalid`) could resume the semi-naive loop from a
  one-tuple delta rather than rebuild — `eval_rule`'s `delta_pos` marker is already a
  delta-driven round. `Invalid` for retraction, rebuild, reset, rule registration, and
  any non-`Bare` or `equals` insert (both can retroactively disqualify a relation).

  **Now the top materialisation cost, measured.** With the per-candidate binding clone
  gone (`9671e2e`), a 555-pin run over a 2004-line constitution spends its time in
  `eval_rule::walk` itself. The suite carries 66 KB-mutating directives interleaved
  with 492 queries, and each one drops the saturation: 20 identical queries with no
  mutation cost 0.13 s, the same 20 with a `:accept-scoped` between each cost 1.16 s
  (~9x). A `:refuse` costs the same as a real mutation even though the KB ends
  semantically identical — `rebuild_inner` (`nibli-reason/src/lib.rs`:479) nulls the
  saturation unconditionally on the rollback. Preserving it across a rollback that
  restores the prior state is a smaller, strictly-sound subset of C3 and would cover
  36 of those 66 directives.

- **Index the join on already-bound positions.** `eval_rule`'s inner `walk` does a FULL
  relation scan per level (`nibli-reason/src/materialize.rs`, the `for tuple in tuples`
  loop) with no index on the positions a partial binding has already fixed. For a
  transitive-closure shape (`earlier($a,$b) & earlier($b,$c) -> earlier($a,$c)`) that is
  O(|R|²) per round where an index on the bound position is O(|R| · fanout). Since the
  binding clone landed this is the largest remaining cost: `walk` went from 4.98% to
  13.97% of self time and is now the top symbol. The set of positions bound on entry to
  level `i` is statically derivable from the rule's templates, so the index key is known
  per (rule, level); the index must be rebuilt per round because `ext`/`delta` grow. If
  it comes with join REORDERING, note that the undo trail is order-independent by
  construction, which is what makes permuting `positive` safe.

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

- **Check proof indices instead of casting them.** The display-string half of this entry
  LANDED in `706bfb9`: the provenance tracer now keys structurally on `StoredFact`
  (`memo: &mut HashMap<StoredFact, u32>`, `nibli-reason/src/reasoning.rs`:556 and :3163,
  read at :3168), so human rendering is no longer an identity boundary and display stays at
  the render edge. What remains is the index half: every step index is produced by an
  unchecked `steps.len() as u32` — `reasoning.rs`:2752, :3173, :3197, :3243, :3265, :3284,
  :3326, :3440, :3469 — so a trace that outgrows `u32` wraps into a valid-looking
  back-reference instead of failing. Fail cleanly if a proof cannot be indexed. **Exit:**
  oversized/deep traces return a typed resource/error outcome rather than a silently
  truncated index; the memo regression suite and `just verify-proofs` remain green.

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

- **The mutation baseline is stale enough that `just mutants` fails on `main`.**
  `mutants-baseline.txt` was cut 2026-07-19 from a 985-mutant sweep; the tree now
  generates **1507**, and 24 commits have touched soundness paths since — so roughly
  312 mutants of code that has never been through the gate. That is not caused by any
  one change and cannot be triaged as part of one: a re-cut means adjudicating
  survivors across `reasoning.rs`, `rules.rs`, `kb.rs` and `nibli-semantics`, each
  either killed with a test or added to the baseline with a documented reason.
  `materialize.rs` joined `examine_globs` on 2026-08-07 (+210 mutants) and its slice
  was swept once — 152 caught / 24 missed / 5 timeout / 29 unviable — but those 24
  survivors are deliberately NOT in the baseline for the same reason. Until the re-cut,
  `cargo mutants --in-diff` is the working gate, as CLAUDE.md already says.
  **Gotcha for whoever runs it:** `cargo mutants -f <path>` does NOT scope the sweep
  when `examine_globs` is set — the config wins and you get the full run. Check the
  "Found N mutants to test" line before walking away.

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
  as `greater(3, 5)` carries a false `ComputeCheck` leaf, but `summarize_false`
  (`nibli-render/src/summary.rs`:223-249) has arms only for `PredicateNotFound`,
  `ForallCounterexample` and `ExistsFailed`, and otherwise returns the fallback at :248,
  "This could not be derived from the known facts and rules." `summarize_true` calls
  `collect_extras` (:475) at :85 and that is the ONLY place a `ComputeCheck` becomes English
  (through `computed_extra_label`, :466); the FALSE path never calls it. Observed through
  nibli-host: `? greater(3, 5).` prints that CWA sentence one line above
  `⊢ greater  [computed (local)] -> FALSE`. The discriminator already exists and already
  crosses the WIT boundary — `ProofTrace.cwa_false` — and nibli-render already reads it at
  `collapse.rs`:221 and `proof.rs`:99, just never in `summary.rs`, so the fix need not
  re-derive it from the steps. The same fallback also fires under a
  `RESOURCE_EXCEEDED (depth)` verdict, so cover resource verdicts, not only `ComputeCheck`.
  Handle local arithmetic/numeric FALSE and trusted-backend FALSE explicitly, without a CWA
  caveat, while ordinary missing-fact FALSE keeps the non-derivability explanation.
  **Exit:** renderer, host, UI, protocol and WASM tests distinguish local computed FALSE,
  backend FALSE, backend unavailable, non-finite UNKNOWN, resource-exceeded, and ordinary
  CWA FALSE; Chapter 17 is recaptured from real bytes.

- **Settle the RDF export contract.** `nibli-import/src/export.rs`:1-18 (the whole file)
  calls its output N-Triples (:1) but the sole emitter (:15) writes
  `# fact:<id> <label>` — a valid but EMPTY N-Triples document, all comment lines. Its doc
  comment still describes labels as "Lojban source or `:assert` form" (:6) and points at
  "the original Lojban text (the canonical source of truth for the KB)" (:8-10), wording
  that is doubly stale since gismu stopped resolving at the committed-corpus milestone. The
  labels it reaches are not round-trippable either: importing `ex:Rex a ex:dog .` and
  running `nibli-import <f>.ttl --export` emits `# fact:0 :assert a`. This is neither RDF
  nor a typed round-trip, while Chapter 21 advertises Turtle import and export. Either
  implement valid Turtle/N-Triples export from typed facts (`list_facts`, not labels) with
  tested IRI/literal mapping, or rename the module and feature to a fact-label dump and
  remove the RDF-export claims. **Exit:** real RDF takes an export -> independent parser ->
  re-import round trip with identity/literal/alias tests; a narrowed dump gets an
  exact-format contract and stale comments removed. Synchronize Chapters 16/21, CLI help,
  README, and the reference gate.

- **Remove the retired two-path retraction story from active docs and benchmarks.**
  `retract_fact_inner` (`nibli-reason/src/lib.rs`:411-421) has no branch at all: it flips
  `r.retracted = true` and calls `rebuild_inner` unconditionally at :418, which ID-sorts the
  survivors (:488). Its OWN doc block (:391-410) correctly narrates the 2026-08-01
  retirement and cites `retract_diff.rs` — but the public API comment further down, at
  :1497-1498, still says "Uses incremental removal for ground facts, full rebuild for facts
  that compiled into rules." Two comments on adjacent functions disagree and the public one
  is the wrong one. Three sites:
  (1) `nibli-reason/src/lib.rs`:1497-1498 — the stale public `retract_fact` doc, unchanged
  since 009a663 (2026-04-07);
  (2) `nibli-engine/benches/engine_bench.rs`:196-204 narrates "two groups, one per
  retraction path", so `bench_retraction_incremental` (:229-247, comment "Flat
  direct-inject ground facts … → incremental path" at :235, registered at :257) measures
  the rebuild path under a false label;
  (3) `nibli-reason/src/kb.rs`:1423-1425 documents `rule_source_map` as "Used for
  incremental retraction: … the corresponding rules can be removed without full rebuild",
  but at HEAD that map is WRITE-ONLY — writes at `rules.rs`:674 and :1727, clears at
  `lib.rs`:470 and `kb.rs`:1762, a clone at `kb.rs`:1644, and no reader anywhere. Decide
  whether the map itself is now vestigial.
  Chapter 11 still describes and measures the same retired path. Rename or remove the stale
  benchmark leg, measure the one real path, and reconcile API prose plus Chapter 11.
  **Exit:** the three sites above are fixed. Do NOT use
  `rg -n 'incremental.*retract|retraction_incremental|two-path'` as the gate — it matches
  NOTHING in `nibli-reason/src/lib.rs` (the stale doc reads "Retract … Uses incremental
  removal": wrong word order, capital R), so it goes green with the worst site untouched; a
  case-insensitive `incremental` sweep over `nibli-reason/src` plus the bench does fire.
  The replacement benchmark asserts its fixture/verdicts and reports reproducible
  hardware/profile/methodology; retraction tests remain green.

- **`obliged`-spelled duties render the wrong obligated party (TWO defects, one entry).**
  `obliged(every data governs, event { message() }).` back-translates as "For every X, if X
  governs and X is data, then **Y** is obligated to notify", while the converted
  `obligated_by` spelling correctly binds X. (a) WHO-SELECTION — both
  `collapse_deontic_event_duties` (`nibli-render/src/logic.rs`:379-383) and `render_frame`'s
  early deontic branch (:405-419, taken whenever place 1 is a Constant and place 2 exists,
  bypassing `frame_template`/`fill_template` entirely) hardcode place 2 as the duty-holder.
  That is right only for the CONVERTED argument order: both spellings compile to the SAME
  base relation with the places SWAPPED (`obliged(Adam, Bel)` emits
  `obliged_x1(_ev0, adam)`/`obliged_x2(_ev0, bel)`; `obligated_by(Adam, Bel)` emits x1=bel,
  x2=adam), and the corpus places are `[bound, duty, standard]` — the bound party is x1
  (`nibli-lexicon/src/corpus/predicates.rs`:1627). (b) INVERTED TEMPLATE — the override row
  `("obliged", "{x2} is obligated that {x1}")` (`nibli-render/src/frame.rs`:19) is likewise
  converted-ordered. It is NOT reached for (a) — the early branch wins — but it IS reached
  at arity 1, where `fill_template`'s trailing-elision cut (frame.rs:243-251) drops the
  whole string: `obliged(Adam).` renders as the EMPTY string, and
  `permitted(every person where obliged).` (`gdpr.nibli`:52) renders "For every X, if ,
  then …" — GDPR Article 6(1)(c) with its antecedent silently gone, in a shipped corpus the
  Transparency Triad asks reviewers to check. Fixing either half alone leaves the other. The
  `obligated_by` row at frame.rs:18 is dead for rendering but cannot be deleted without
  updating its assertion at frame.rs:310-313. Ripple: re-check nibli-wasm's
  `c18_draft_error_glosses_are_verbatim` pin (`nibli-wasm/src/lib.rs`:454) and the book's
  Ch 18 alias note.
- **README's REPL-commands table is missing four entries** (README.md:**329**
  `### REPL Commands`, header still `| Command | Description |`). RE-CHECKED
  2026-08-07 and most of this entry's original complaint no longer holds — `41a0c43`
  fixed it. What is actually left: the table omits the host's `:dump` and `:export`
  (`nibli-host/src/main.rs`:1516, :1551) and all six short aliases
  (`:b :f :h :m :q :r`). NOT still true, and struck from this entry: it does NOT list
  `:contradictions`/`:trace`/`:untrace`/`:traces` (a paragraph below the table already
  separates the debug REPL as a different surface), and it DOES list
  `:materialize`/`:db`/`:proof-verbose`. A Surface column would still read better than
  the prose caveat, but that is polish, not a defect.
- **The component's import list is not gated.** Docs (book Ch 13/15/App C) state
  "no clock or filesystem imports" — true today (imports are the
  `wasi:cli`/`wasi:io` set + `wasi:random/insecure-seed`), but no CI check pins
  it and `wasm-tools` sits unused in the flake. A one-line `ci-wasm` smoke
  (`wasm-tools component wit target/wasm32-wasip2/release/nibli.wasm` + grep for
  the absent interfaces) closes the gap.
- **`resource_hint`'s Depth arm is dead code.** `resource_hint`
  (`nibli-host/src/main.rs`:587) has exactly one call site, :1803, inside the `Err(e)` trap
  arm of `run_proof_query`; an engine-returned Depth verdict takes the `Ok(Ok(…))` arm at
  :1758 and never reaches it, and `classify_resource_trap` (:566) cannot yield Depth — its
  own doc says so at :564-565, so dropping the arm contradicts nothing already written.
  Reproduced: a 13-link `X(every Y).` chain at the default `max_chain_depth` of 10
  (`awake(every actual). … dark(every cyan). actual(Rex).` then `? dark(Rex).`) prints
  `[Query] RESOURCE_EXCEEDED (depth)` with no hint line — and with the same false
  `[Why] This could not be derived from the known facts and rules.` the compute-FALSE entry
  above describes, so that renderer fix should cover resource verdicts too. The unreachable
  hint text (:595) recommends raising `max_chain_depth`, a knob no shipped surface exposes:
  the only setter is the Rust API `KnowledgeBase::set_max_chain_depth`
  (`nibli-reason/src/lib.rs`:559) — there is no `:depth` command and no `NIBLI_DEPTH`, and
  GUARANTEES.md:131 states the shipped runtime surfaces keep the default. Wire a real Depth
  hint into the verdict path (and something for it to recommend), or drop the dead arm.
- **`wit/world.wit`:281 `proof-ref` doc comment is wrong.** It claims "No children — the
  full proof was shown at its first occurrence," but the memo-hit arm of
  `trace_predicate_provenance_typed` (`nibli-reason/src/reasoning.rs`:3168-3180) always
  pushes `children: vec![cached_idx]` (:3177), and nothing strips it: `children` lives on
  `proof-step` (`wit/world.wit`:292-296), not on the rule variant, and
  `nibli-pipeline/src/lib.rs`:199 clones it across the component boundary with no ProofRef
  special case. The verbose text renderer re-expands that child while the collapsed/UI
  renderings drop it. So the COMMENT is what is wrong, not the behaviour — the "decide
  whether that divergence is intentional" step concerns only the renderers. The merge
  `a7d288a` edited the variant list directly above this comment (adding `presupposed`) and
  left it. If a test ships with the fix, note that
  `nibli-reason/src/tests/memo_regressions.rs`:540
  (`test_proof_ref_carries_cached_index`) is an `if let` inside a `for` with no "at least
  one ProofRef was seen" assertion, so it passes vacuously on a trace containing none.
  Ripple: the book's Appendix C reproduces `world.wit` in full and must be updated together.

---

## Pointers

| Tracker / doc | Scope |
|---------|--------|
| **This file** | Engine runtime, editors/tooling, docs hosting, open semantics decisions — the ONE repo tracker since `DOCS_TODO.md` retired |
| **`RELEASING.md`** | Release decisions of record (tiers, lockstep, tags) + the operator runbook |
| **`DEPLOY.md`** | Hosting: playground, wasm demo, mdBook primary + Pages mirror |
| **`book/TODO.md`** | Manuscript only (private checkout; Orange AVA) |
