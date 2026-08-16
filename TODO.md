# Engine TODO

**Future-facing only.** An entry is here because it is still true and still wants doing —
delete it when it lands rather than marking it done, and put the record in the commit.

Every surviving entry was re-verified against HEAD on 2026-08-16 (HEAD `5777ced`) by a
12-way parallel sweep: every cited file opened at the cited line, behavioural claims
checked against current source, and every negative grep positive-controlled. Five commits
landed after the previous 2026-08-08 sweep (`ef669b8`, `1784c67`, `07734c8`, `5580618`,
`5777ced` — the witness-enumeration fail-closed decision and the corpus-scoped
compute-registration decision). They closed NOTHING below, but they moved many lines in
`nibli-reason/src/lib.rs` (+~40..110), `kb.rs` (+~90..290), `reasoning.rs` (~+100 in the
tracer region), and README — citations below were corrected in the same pass — and
5580618's complete-or-error collection contract joined what the book-sync entries must
carry. One sub-claim died and was deleted (`ProofRule::Asserted` DOES carry fact-id
citations since `d421a6d` / WIT 0.10.0), and the mutants figures were refreshed. Still
check a line before trusting it, and correct it in place when you do.

**Ordered by dependency since 2026-08-16.** Four tiers: constraints of record (no open
work), INDEPENDENT entries (no ordering constraints between them — land any time, any
order), dependency CHAINS (numbered; do them in the stated order, the reason is given in
place), and entries blocked on external repos. Effort is NOT the axis — an independent
entry may be week-sized and a chain step an afternoon.

**Effort tags (added 2026-08-16):** every open entry carries *(effort: …)*, sized
against the code by a 10-agent assess-and-calibrate sweep. **low** = under an hour —
one file or recipe, existing coverage. **medium** = a focused session — a few files
plus targeted tests, no open design decisions. **high** = days — cross-crate work,
new batteries/gates, design within established patterns. **extra** = a week-plus — a
new contract/subsystem, a WIT bump threaded through every surface, or Lean
proof-gate work. **max** = a multi-week program — external professional review on
the critical path. **ultracode** = open-ended program-scale work whose scope is only
discoverable by doing it; currently unused — every entry here has a stated
deliverable and bounded scope. Where an entry documents a cheaper exit, that exit's
tier is noted in the same tag. Constraints of record carry no tag (nothing to
implement).

Release runbook: **`RELEASING.md`**. Docs hosting: **`DEPLOY.md`**. Book manuscript:
separate `book/` repo (`book/TODO.md`). The docs + release tracker `DOCS_TODO.md` was
RETIRED 2026-08-03.

---

## Constraints of record — nothing to do; rules a future change must respect

**Authorization (A0–A5 SHIPPED in full):** `nibli-auth` + `nibli-auth-py`, `authorizer`
on `nibli:engine@0.6.0` (package now @0.11.0 — `wit/world.wit`:14; the export survives at
:531/:580), `policy/auth-0.1.0.nibli`, Rust and Python adapters, both examples, the
mdBook chapter, and a dedicated `auth` job in CI (`just test-auth && just
check-auth-axum`). The phase-by-phase record is in git history; these outlive it:

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

(The one open item this section used to carry — Python-bridge CI coverage — LANDED
2026-08-16: the `auth-py` CI job runs `just test-auth-py` through the default Nix
devshell, so both halves of the adapter surface are gated.)

**Pin runner — `:accept-scoped` cannot scope a one-way declaration, by design.**
`derived_only`/`admits` are deliberately absent from `rebuild_inner`'s clear list, so
they survive the replay a retraction performs; the runner refuses to scope them rather
than silently no-op (`nibli/src/bin/pin.rs`:321-332, the refusal at :945-957). If a KB
ever needs a *scoped* vocabulary control, the engine would have to gain an explicit
un-declare — which was rejected once already, because a vocabulary that can be re-opened
at runtime gives back exactly the capability the declaration removes. Revisit only with
a concrete need.

---

## Chain — proof certificate (1 → 2)

Order: step 1 defines the versioned contract (the index-cap refusal LANDED
2026-08-16: `push_proof_step` is the one indexing point, overflow refuses the
whole trace as a typed error — fold that outcome into the envelope); step 2
extends the contract to materialised verdicts — design 1 knowing 2's needs, or
the envelope bakes in the per-rule-derivation assumption C2 exists to relax.

1. **Bind proof traces to the full verdict and durable evidence.** *(effort: extra; narrow-the-public-contract
   exit: medium)* `ProofStep` exposes
   only `holds: bool`, while `ProofTrace` carries `naf_dependent` and `cwa_false` but
   not the root `QueryResult`, UNKNOWN reason, or RESOURCE_EXCEEDED kind
   (`nibli-types/src/logic.rs`:366-394 — `ProofStep` :366-370, `ProofTrace` :375-394).
   Native callers receive a separate tuple, but a serialized/cached trace can no longer
   prove which non-TRUE result it accompanied. (Direct-assertion evidence ids DO exist
   since `d421a6d` / WIT 0.10.0: `ProofRule::Asserted`/`Derived`/`Presupposed` carry
   `AssertionCitation`/`RuleCitation` with `FactId` — logic.rs:315-340 — so the
   envelope can build on them rather than invent them; the earlier claim here that
   `Asserted` held only a display string is dead.) Define one versioned
   result-plus-certificate envelope (or narrow the public "proof" contract), with
   configuration/corpus/engine versions and stable evidence ids sufficient for an
   independent checker. **Exit:** round-trip and independent-validation tests cover
   TRUE, closed-world FALSE, arithmetic FALSE, every UNKNOWN reason, every resource
   kind, NAF, equality, duplicate assertions, proof-local compute evidence, and replay;
   WIT/protocol/host/UI and Appendix C evolve together.

2. **Materialisation: the trace story (C2).** *(effort: extra; declining as a decision of record:
   low)* Proof-traced queries keep the
   backward-chaining path (`positive_lookup` lowered for their duration —
   `nibli-reason/src/lib.rs`:1024, restored at :1052/:1072; the fast-path gates read it
   at `reasoning.rs`:1003 and :2705) because a materialised verdict has no derivation
   to record. To let them use the fast path, four things need answering:
   `trace_predicate_provenance_typed` (reasoning.rs:3304) falls to a `holds:false`
   `PredicateNotFound` (:3616-3620) for a TRUE reachable only by materialisation; a
   materialised FALSE has no per-rule blocking premise, which `proofs/Trace.lean`'s
   `Neg` constructor and `trace_soundness_conformance` both require; `ProofRule::
   ExistsWitness` names a witness term the projection eliminated; and `naf_dependent`
   can flip true→false when a positive lookup deletes the `Negation` steps beneath it
   (a user-visible honesty marker moving because of an optimisation). Minimum-churn
   option if pursued: `ProofRule::PredicateCheck { method: "materialized" }` — no WIT
   change — plus a `validate_cert` arm and a `factAx`-analogue bridge against `m.ext`.

---

## Chain — materialisation performance (1 → 2; gate each step with `--in-diff`)

Order: step 1 is the top measured cost and the smaller change; step 2's rollback subset
is strictly sound and independent of delta design, and the full delta machinery comes
last. Both steps edit `examine_globs` files, so each one grows the mutation debt below —
gate with `cargo mutants --in-diff` per change, and see the baseline re-cut entry.

1. **Index the join on already-bound positions.** *(effort: medium)* `eval_rule`'s inner `walk`
   (`nibli-reason/src/materialize.rs`:1054 / :1076) does a FULL relation scan per level
   (the `for tuple in tuples` loop at :1125, fed by `source.get(&atom.relation)` over
   the whole extension, :1117-1148) with no index on the positions a partial binding
   has already fixed. For a transitive-closure shape
   (`earlier($a,$b) & earlier($b,$c) -> earlier($a,$c)`) that is O(|R|²) per round
   where an index on the bound position is O(|R| · fanout). Since the binding clone
   landed this is the largest remaining cost: `walk` went from 4.98% to 13.97% of self
   time and is now the top symbol. The set of positions bound on entry to level `i` is
   statically derivable from the rule's templates, so the index key is known per
   (rule, level); the index must be rebuilt per round because `ext`/`delta` grow. If it
   comes with join REORDERING, note that the undo trail is order-independent by
   construction, which is what makes permuting `positive` safe.

2. **Materialisation: incremental re-saturation (C3) — rollback subset first.** *(effort: high; the rollback subset alone: medium)* Every
   fact insert drops the saturation (`assert_typed_fact` → `invalidate_materialization`,
   `rules.rs`:873 → :892 → :1044; the invalidation itself is `reasoning.rs`:2016-2018,
   `*inner.materialized.borrow_mut() = None`), so an interleaved
   `assert; query; assert; query` REPL session recomputes the next requested query cone
   and its cumulative root union from scratch, and `nibli-ui` re-asserts its whole tab
   per run by design. Datalog is monotone, so a seed addition can only GROW the model:
   a three-state dirty flag (`Clean` / `GrewBy(Vec<StoredFact>)` / `Invalid`) could
   resume the semi-naive loop from a one-tuple delta rather than rebuild —
   `eval_rule`'s `delta_pos` marker is already a delta-driven round. `Invalid` for
   retraction, rebuild, reset, rule registration, and any non-`Bare` or `equals` insert
   (both can retroactively disqualify a relation).

   **Now the top materialisation cost, measured.** With the per-candidate binding clone
   gone (`9671e2e`), a 555-pin run over a 2004-line constitution spends its time in
   `eval_rule::walk` itself. The suite carries 66 KB-mutating directives interleaved
   with 492 queries, and each one drops the saturation: 20 identical queries with no
   mutation cost 0.13 s, the same 20 with a `:accept-scoped` between each cost 1.16 s
   (~9x). A `:refuse` costs the same as a real mutation even though the KB ends
   semantically identical — `rebuild_inner` (`nibli-reason/src/lib.rs`:475) nulls the
   saturation unconditionally on the rollback (the `None` assignment at :523).
   Preserving it across a rollback that restores the prior state is a smaller,
   strictly-sound subset of C3 and would cover 36 of those 66 directives — do that
   subset first.

**The mutation baseline is stale enough that `just mutants` fails on `main`** *(effort: high)* — and the
chain above will widen the gap, so sequence the re-cut before it or immediately after.
`mutants-baseline.txt` was cut 2026-07-19 from a 985-mutant sweep (the file's own
header; last touched 2026-08-05 by `2385012`, a 4-line survivor shrink, not a re-cut);
the tree generated **1507** at the 2026-08-08 sweep and has grown since (5580618 added
~153 in-scope lines to `reasoning.rs`; `lib.rs`'s +107 is NOT in `examine_globs` —
re-derive with `cargo mutants --list | wc -l` at the re-cut), and **47** commits have
touched soundness paths since the cut (as of 2026-08-16; the previous figure of 24 was
already stale) — so well over 300 mutants of code have never been through the gate.
That is not caused by any one change and cannot be triaged as part of one: a re-cut
means adjudicating survivors across `reasoning.rs`, `rules.rs`, `kb.rs` and
`nibli-semantics`, each either killed with a test or added to the baseline with a
documented reason. `materialize.rs` joined `examine_globs` on 2026-08-07 (+210 mutants)
and its slice was swept once — 152 caught / 24 missed / 5 timeout / 29 unviable — but
those 24 survivors are deliberately NOT in the baseline for the same reason (verified:
zero `materialize` lines in the baseline file, which by itself guarantees the failing
sweep). Until the re-cut, `cargo mutants --in-diff` is the working gate, as CLAUDE.md
already says. **Gotcha for whoever runs it:** `cargo mutants -f <path>` does NOT scope
the sweep when `examine_globs` is set — the config wins and you get the full run. Check
the "Found N mutants to test" line before walking away.

---

## Chain — case-study corpora and their rendering (1 → 2; 3 independent)

Order: fix the rendering (1) before recapturing GDPR (2) — `gdpr.nibli`:52 currently
renders with its antecedent silently gone, and the redesigned corpus will be reviewed
through the same renderer. The drug-interactions redesign (3) does not depend on 1 and
can run in parallel with 2.

1. **`obliged`-spelled duties render the wrong obligated party (TWO defects, one
   entry).** *(effort: medium)* `obliged(every data governs, event { message() }).` back-translates as
   "For every X, if X governs and X is data, then **Y** is obligated to notify", while
   the converted `obligated_by` spelling correctly binds X. (a) WHO-SELECTION — both
   `collapse_deontic_event_duties` (`nibli-render/src/logic.rs`:381-385) and
   `render_frame`'s early deontic branch (:406-419, taken whenever place 1 is a
   Constant and place 2 exists, bypassing `frame_template`/`fill_template` entirely)
   hardcode place 2 as the duty-holder. That is right only for the CONVERTED argument
   order: both spellings compile to the SAME base relation with the places SWAPPED
   (`obliged(Adam, Bel)` emits `obliged_x1(_ev0, adam)`/`obliged_x2(_ev0, bel)`;
   `obligated_by(Adam, Bel)` emits x1=bel, x2=adam), and the corpus places are
   `[bound, duty, standard]` — the bound party is x1
   (`nibli-lexicon/src/corpus/predicates.rs`:1627). (b) INVERTED TEMPLATE — the
   override row `("obliged", "{x2} is obligated that {x1}")`
   (`nibli-render/src/frame.rs`:19) is likewise converted-ordered. (The converse-alias
   corpus work DID de-invert the lexicon templates — predicates.rs:1625/:1627 — but
   `TEMPLATE_OVERRIDES` wins over corpus templates, so that fix does not touch either
   defect here.) The override is NOT reached for (a) — the early branch wins — but it
   IS reached at arity 1, where `fill_template`'s trailing-elision cut
   (frame.rs:243-251) drops the whole string: `obliged(Adam).` renders as the EMPTY
   string, and `permitted(every person where obliged).` (`gdpr.nibli`:52) renders
   "For every X, if , then …" — GDPR Article 6(1)(c) with its antecedent silently gone,
   in a shipped corpus the Transparency Triad asks reviewers to check. Fixing either
   half alone leaves the other. The `obligated_by` row at frame.rs:18 is dead for
   rendering in the common path (a who-less collapse acc with place 2 absent can still
   fall through to it, so "dead" is approximate) and cannot be deleted without updating
   its assertion at frame.rs:310-313. Ripple: re-check nibli-wasm's
   `c18_draft_error_glosses_are_verbatim` pin (`nibli-wasm/src/lib.rs`:454) and the
   book's Ch 18 alias note.

2. **Replace person-level GDPR proxies with operation-scoped legal facts.** *(effort: max)*
   `gdpr.nibli`:46-78 derives a generic `permitted(person)` from
   `approves/promise/obliged` (the three rules at :48/:50/:52) and uses absence of
   `approves` under NAF as the erasure trigger (rationale at :64-78; the erasure rule
   statement itself, `obligated_by(every person where ~approves, event { removes() }).`,
   sits at :101). It does not identify the processing operation, controller, data,
   purpose, consent scope/withdrawal time, alternative lawful bases, Article 17
   exceptions, or jurisdiction/effective text; NAF absence is not a legal finding.
   Design an operation-scoped schema and corpus with legal review, explicit coverage
   assumptions, exceptions, and dated primary sources. **Exit:**
   multi-controller/multi-purpose/withdrawal/alternative-basis/exception
   counterexamples are pinned; missing evidence yields an honest
   non-compliance-neutral result rather than a legal conclusion; engine/UI fixtures and
   Chapter 19 consume the same live artifact.

3. **Redesign `drug-interactions.nibli` around a patient-local exposure event.** *(effort: max)* The
   concentration rules at :71-72 derive drug-level risk from global
   inhibition/metabolism, and the alert rule at :94 checks only that Adam uses the
   risky substrate. Retracting `uses(Adam, Flukonazol)` therefore does not withdraw the
   warfarin alert; a second patient taking warfarin alone can inherit risk from an
   inhibitor nobody in that regimen takes. Model co-administration explicitly and
   decide/document the dosage, route, timing, phenotype, evidence-quality, and
   uncertainty boundary with a pharmacology reviewer. **Exit:** patient-isolation and
   inhibitor-retraction negatives, dose/route/time boundary tests, and clinically
   reviewed scenario fixtures pass in `nibli-engine`, `nibli-ui`, and the pin/seam
   gates; Chapter 20 is recaptured from the live corpus and is labeled a synthetic
   teaching model until that review exists.

---

## Chain — Editor / Linguist / LSP (syntax tooling)

Policy: book and gates keep fence tags **`nibli` / `nibli-kr`** (never a foreign
language alias). Seed grammar lives in `grammars/` (`nibli.tmLanguage.json` +
injection sketch + `grammars/README.md`). Runtime lexer remains
`nibli-kr/src/highlight.rs` (REPL / UI) — keep token classes aligned when anything
here changes. Keywords must stay equal to `nibli_lexicon::RESERVED_WORDS`.
(Whole section re-verified 2026-08-16: no artifacts have appeared — still no
`editors/`, no `nibli-lsp`, no tree-sitter grammar, no `tower-lsp`/`lsp-server` in
`Cargo.lock`; `grammars/` is read by exactly one thing, the keyword ratchet.)

Independent starters (any order):

- **VS Code / Cursor extension (local):** *(effort: medium)* package `grammars/nibli.tmLanguage.json`
  as language id `nibli` for `*.nibli`, plus markdown fence injection for
  `nibli` and `nibli-kr` via `grammars/nibli-markdown-injection.json`. Ship as
  `editors/vscode/` (or a small sibling repo) with `language-configuration.json`
  (`#` line / `/* */` block). Optional: publish to Open VSX / Marketplace.
  → Unblocks the editor docs page below.
- **GitHub Linguist PR:** *(effort: medium — the submission work; merge is
  externally gated on Linguist's usage threshold)* submit `source.nibli` + samples from shipped corpora
  (`gdpr.nibli`, `drug-interactions.nibli`, `readme.nibli`, …); language name
  **Nibli**, extensions `.nibli`, aliases `nibli-kr` / `nibli kr`. Until merge,
  github.com fences stay uncolored — do not retag book fences to `prolog` etc.
- **Tree-sitter grammar (`tree-sitter-nibli`):** *(effort: high)* new crate/repo tracking
  `nibli-kr.pest` by discipline (not codegen). Queries: `highlights.scm`,
  `locals.scm`, `injections.scm` for Markdown. Consumers: nvim-treesitter,
  Helix, Zed. Does **not** replace Linguist/TextMate for github.com.
  → Unblocks the tree-sitter keyword ratchet below.
- **`nibli-lsp` (thin LSP):** *(effort: high; diagnostics-only first cut: medium)* workspace
  bin on `tower-lsp` (or `lsp-server`) using
  `nibli_kr::parse_checked` diagnostics, lexicon hover/completion (gloss,
  places, templates), optional format via `nibli_kr::render`, optional semantic
  tokens from `nibli_kr::highlight::lex`. Single-file first; multi-file KB
  projects later. → Also unblocks the editor docs page below.

Blocked followers:

- **Conformance guard — TextMate half LANDED** *(effort of the still-owed tree-sitter half: low)*
  (`just verify-grammar-parity`, in
  `ci`: set, order and the `\b` anchors vs `RESERVED_WORDS`). Still owed: the same
  ratchet for the Tree-sitter keyword list — blocked on the tree-sitter grammar
  existing.
- **Editor/LSP docs (mdBook)** *(effort: low, once unblocked)* — blocked,
  precondition UNMET, and the last docs
  item that outlived `DOCS_TODO.md`. Everything above is design + seed artifacts:
  `grammars/` holds a TextMate grammar and an injection sketch that **no build or
  CI job consumes** (only the keyword ratchet reads it), and there is no
  `editors/` dir, no `nibli-lsp` crate, no tree-sitter grammar, and no `tower-lsp`
  anywhere — `Cargo.lock` included. Writing the page now would present aspiration
  as shipped, which the docs' own epistemic rule forbids. Revisit when
  `editors/vscode/` or `nibli-lsp` actually exists.
- **Book fence rename (optional, coordinated):** *(effort: medium)* if/when fences go plain
  `nibli` only, retarget `verify_book.py` + capture harness + injection regex in
  one PR; keep `nibli-kr` as Linguist/VS Code alias forever.

---

## Blocked on external repos — nothing in this repo is missing

- **Synchronize the manuscript with the resolved existential-import algebra (engine
  decision complete 2026-08-05; book-repo work only).** *(effort: high)* The engine now uses clean-core
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
  `some`/forall/find/exact-0/exact-1/count/aggregate/retraction matrix and live toggles
  (:1710, :1831, :1906, :1927 as of HEAD `5777ced`); `smoke-host-existential-import`
  pins the component path; the independent oracle engines explicitly select clean-core.
  **Remaining work:** in the separate `book/` repository, update Chapters 5, 8, and 12
  plus Appendices A, B, E, and K to describe this exact algebra, the default/opt-in
  controls, structural witness origin, the 0.9.0-introduced witness-origin proof
  surfaces (current package is `nibli:engine@0.11.0` — cite the live version, not
  0.9.0), the fact that ON→OFF/OFF→ON applies immediately to already-loaded rules, AND
  — since `5580618` (2026-08-12) — the complete-or-error collection contract: `find`/
  `count_witnesses`/`aggregate` now refuse ("witness enumeration incomplete") whenever
  any final evaluated candidate leaf is non-definitive, under either polarity, instead
  of undercounting (GUARANTEES "Collections are complete or error"). Chapter 12's
  consent capture currently depends on the historical import profile (the engine's
  `ch12_consent_case_study_traced_query_completes` watchdog —
  `nibli-engine/tests/known_failures.rs`:333 — now selects it explicitly), so either
  mark that profile in the capture or recast the example for clean-core. Run the book
  validator, reference checker, capture checker, and two byte-idempotence passes before
  removing this residual with the `book-todo` workflow.

- **Synchronize the manuscript with query-only exact-count semantics (engine
  decision complete 2026-08-05; book-repo work only).** *(effort: medium)* The engine no longer
  generates witnesses for an asserted `CountNode`: every assertion, assumption,
  and preassigned/replay ingress rejects `exactly N`/`no` in asserted position before
  allocating an id or mutating state. Counts remain valid queries over the current
  closed domain, collapse equality classes, include explicitly enabled legacy-import
  witnesses with structured `CountResult` provenance, and may change after facts,
  equality links, import-profile changes, or retractions. `exactly 0` is a query,
  never a stored prohibition. Counts inside opaque abstractions are quoted content,
  not outer-KB constraints. Legacy persisted count-assertion rows fail replay
  non-destructively. Root specs/docs and Formalize's assertion-authoring guard are
  synchronized; do not reintroduce the retired generative reading. (The separate
  exact-`CountNode` evaluator was explicitly PRESERVED through `5580618`'s
  witness-enumeration change — both CHANGELOG and GUARANTEES say so — so this entry's
  semantics are unaffected by that commit.) Update Chapters 4 and 8 plus Appendices
  A/B/E and any renderer/capture exposition to present exact counts only on query
  surfaces; rewrite examples that currently load them as KB statements. Run the book
  validator, reference checker, capture checker, and two byte-idempotence passes
  before removing this residual with the `book-todo` workflow.

- **Synchronize the manuscript with the fail-closed aggregate outcome (engine done
  2026-08-16; book-repo work only).** *(effort: low)* `KnowledgeBase::aggregate` /
  `aggregate_text` now return the typed `AggregateOutcome` — `Empty` for a complete
  zero-witness enumeration, `Value { value, witnesses }` with contributing-witness
  provenance — and REFUSE (distinct reasoning errors) a binding set missing the
  variable, a non-numeric value, a non-finite operand, and an overflowed result;
  the 5580618 incomplete-enumeration refusal is unchanged, and no WIT surface
  exposes aggregation. Update Chapter 20 and Appendices B/E to present this
  contract (and drop any "silently skips non-numeric witnesses" phrasing). Run the
  book validator, reference checker, capture checker, and two byte-idempotence
  passes before removing this residual with the `book-todo` workflow.

- **Synchronize Chapter 11 with the one-path retraction model and the renamed
  benchmark (engine done 2026-08-16; book-repo work only).** *(effort: low)* The
  engine repo no longer contains any live two-path retraction claim: the public
  `retract_fact` doc, the benchmark (now ONE criterion group, `retraction`, with
  `text_event_decomposed` / `flat_direct_inject` fixture VARIANTS of the same
  rebuild path, verdict-asserted fixtures, and a methodology header), the
  write-only `rule_source_map` (removed outright), and the stale test
  names/comments are all reconciled. Chapter 11 still describes and measures the
  retired incremental path and quotes the old `retraction_rebuild` /
  `retraction_incremental` group names — re-derive its table from the new bench
  (`cargo bench -p nibli-engine --bench engine_bench -- retraction`) on
  documented hardware and rewrite the prose to the one-path story. Run the book
  validator, reference checker, capture checker, and two byte-idempotence passes
  before removing this residual with the `book-todo` workflow.

- **Synchronize Chapters 16/21 with the real N-Triples export (engine done
  2026-08-16; book-repo work only).** *(effort: low)* `nibli-import --export` now
  emits valid, independently-parsed N-Triples for the representable fragment
  (arity-2 surface tuples over IRI-safe constants/finite numbers, re-projected
  from the store's event decomposition under the minted `http://nibli.dev/kb#`
  base) and REFUSES everything else with per-fact reasons on stderr — the old
  `# fact:<id> <label>` comment-line dump and its stale Lojban wording are gone.
  Update Chapter 21's Turtle import/export exposition (and Chapter 16 if it
  quotes the old dump) to the fragment-plus-refusals contract; re-capture any
  transcripts. Run the book validator, reference checker, capture checker, and
  two byte-idempotence passes before removing this residual with the
  `book-todo` workflow.

- **Recapture Chapter 17 from the new verdict-class `[Why]` renderer (engine done
  2026-08-16; book-repo work only).** *(effort: low)* `[Why]` is now verdict-driven:
  a computed FALSE prints "Decided by computation: … (computed locally / by the
  trusted backend)" instead of the closed-world sentence, UNKNOWN prints a
  per-reason "No verdict: …" line (backend-unavailable, non-finite), and
  RESOURCE_EXCEEDED prints the budget-cutoff sentence. Chapter 17's transcripts
  quote the old fallback under these verdicts — recapture from real bytes. Run the
  book validator, reference checker, capture checker, and two byte-idempotence
  passes before removing this residual with the `book-todo` workflow.

- **Wire `dhilipsiva.dev/docs/nibli/`** *(effort: low)* in the external `dhilipsiva/dhilipsiva.dev`
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

## Pointers

| Tracker / doc | Scope |
|---------|--------|
| **This file** | Engine runtime, editors/tooling, docs hosting, open semantics decisions — the ONE repo tracker since `DOCS_TODO.md` retired |
| **`RELEASING.md`** | Release decisions of record (tiers, lockstep, tags) + the operator runbook |
| **`DEPLOY.md`** | Hosting: playground, wasm demo, mdBook primary + Pages mirror |
| **`book/TODO.md`** | Manuscript only (private checkout; Orange AVA) |
