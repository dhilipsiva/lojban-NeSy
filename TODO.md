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

## Independent — no ordering constraints between these

- **Make aggregation fail closed instead of returning a partial numeric result.** *(effort: medium)*
  `KnowledgeBase::aggregate` (`nibli-reason/src/lib.rs`:1559-1593) uses `filter_map`
  over witness bindings (:1572), silently dropping a witness when the requested variable
  is absent or nonnumeric and then summing/averaging the survivors (:1586-1591, with no
  non-finite/overflow rejection); the success payload is only `Option<f64>`, so
  `Ok(None)` (:1583) conflates "no witnesses" with "witnesses exist but none numeric".
  Since 5580618 the ENUMERATION half is pinned: `aggregate` inherits `query_find`'s
  incomplete-enumeration refusal via `?` at :1569 (any non-definitive final candidate
  leaf errors instead of undercounting — `src/tests/witness_completeness.rs`:82, with
  the definitive-empty → `Ok(None)` control at :161-172), and the doc (:1561-1562)
  states the numeric-projection behavior is deliberately unchanged — so what remains is
  exactly the typed-outcome half. Define a typed outcome that distinguishes empty input,
  incomplete or mixed bindings, non-finite input/result, and a valid aggregate;
  preserve the (now-pinned) incompleteness refusal. **Exit:** missing-variable, mixed
  string/number, NaN/infinity, overflow, empty, and all-numeric controls are pinned;
  session/engine/WIT callers propagate the outcome without collapsing it; aggregate
  provenance states the contributing witnesses/count or the book narrows its proof
  claim. Ripple: Chapter 20 and Appendices B/E.

- **Remove the retired two-path retraction story from active docs and benchmarks.** *(effort: medium)*
  `retract_fact_inner` (`nibli-reason/src/lib.rs`:455-465) has no branch at all: it
  flips `r.retracted = true` (:460) and calls `rebuild_inner` unconditionally (:462),
  which ID-sorts the survivors (:532). Its OWN doc block (:435-454) correctly narrates
  the 2026-08-01 retirement and cites `retract_diff.rs` — but four other sites still
  tell the old story:
  (1) `nibli-reason/src/lib.rs`:1612-1613 — the stale public `retract_fact` doc,
  unchanged since 009a663 (2026-04-07): "Uses incremental removal for ground facts,
  full rebuild for facts that compiled into rules";
  (2) `nibli-engine/benches/engine_bench.rs`:198-204 narrates "two groups, one per
  retraction path", so `bench_retraction_incremental` (:229-247, comment "Flat
  direct-inject ground facts … → incremental path" at :235, registered at :257) measures
  the rebuild path under a false label;
  (3) `nibli-reason/src/kb.rs`:1675-1677 documents `rule_source_map` as "Used for
  incremental retraction: … the corresponding rules can be removed without full
  rebuild", but at HEAD that map is WRITE-ONLY — writes at `rules.rs`:674 and :1727,
  clears at `lib.rs`:514 and `kb.rs`:2051, clone at `kb.rs`:1929, init at `kb.rs`:1990,
  and no production reader anywhere (the only read-adjacent site is the test-only
  state-equality snapshot, `src/tests/compute_ingest.rs`:148/:252). Decide whether the
  map itself is now vestigial;
  (4) `nibli-reason/src/tests/assertions.rs`:133-134 — a comment "(a ground-fact
  retraction is incremental and would not rebuild)", false at HEAD (found in the
  2026-08-16 sweep).
  Chapter 11 still describes and measures the same retired path. Rename or remove the
  stale benchmark leg, measure the one real path, and reconcile API prose plus
  Chapter 11. **Exit:** the four sites above are fixed. Do NOT use
  `rg -n 'incremental.*retract|retraction_incremental|two-path'` as the gate — it
  matches NOTHING in `nibli-reason/src/lib.rs` (the stale doc reads "Retract … Uses
  incremental removal": wrong word order, capital R), so it goes green with the worst
  site untouched; a case-insensitive `incremental` sweep over `nibli-reason/src` plus
  the bench does fire (today at lib.rs:438 honest, :1612 stale, kb.rs:1675, rules.rs:666,
  plus tests). The replacement benchmark asserts its fixture/verdicts and reports
  reproducible hardware/profile/methodology; retraction tests remain green.

- **README's REPL-commands table is missing entries** *(effort: low)* (README.md:359
  `### REPL Commands`, header still `| Command | Description |`). The table omits the
  host's `:dump` and `:export` (`nibli-host/src/main.rs`:1659, :1694), all six short
  aliases (`:b :f :h :m :q :r`), and has no rows for `:quit`/`:help` themselves. NOT
  missing (struck on earlier re-checks, re-confirmed 2026-08-16): the debug-REPL-only
  commands (a paragraph below the table already separates that surface as different),
  `:materialize`/`:db`/`:proof-verbose` (:378-380), and 5777ced's bare-`:compute`
  report (already documented in the row at :372). A Surface column would still read
  better than the prose caveat, but that is polish, not a defect.

- **The component's import list is not gated.** *(effort: low)* Docs (book Ch 13/15/App C) state
  "no clock or filesystem imports" — true today (imports are the
  `wasi:cli`/`wasi:io` set + `wasi:random/insecure-seed`), but no CI check pins it and
  `wasm-tools` sits unused in the flake (its sole mention repo-wide is `flake.nix`:56).
  A one-line `ci-wasm` smoke
  (`wasm-tools component wit target/wasm32-wasip2/release/nibli.wasm` + grep for the
  absent interfaces) closes the gap.

- **`wit/world.wit`:281 `proof-ref` doc comment is wrong.** *(effort: low)* It claims "No children —
  the full proof was shown at its first occurrence," but the memo-hit arm of
  `trace_predicate_provenance_typed` (`nibli-reason/src/reasoning.rs`:3314-3326) always
  pushes `children: vec![cached_idx]` (:3323), and nothing strips it: `children` lives
  on `proof-step` (`wit/world.wit`:292-296), not on the rule variant, and
  `nibli-pipeline/src/lib.rs`:202 clones it across the component boundary (map closure
  :199-203; the `ProofRef` arm at :167-168 converts only the fact string). The verbose
  text renderer re-expands that child while the collapsed/UI renderings drop it. So the
  COMMENT is what is wrong, not the behaviour — the "decide whether that divergence is
  intentional" step concerns only the renderers. The sibling field doc at
  `world.wit`:227 ("its full proof was shown at first occurrence") is accurate as
  written — keep the two consistent when fixing. If a test ships with the fix, note
  that `nibli-reason/src/tests/memo_regressions.rs`:540
  (`test_proof_ref_carries_cached_index`) is an `if let` inside a `for` with no "at
  least one ProofRef was seen" assertion, so it passes vacuously on a trace containing
  none. Ripple: the book's Appendix C reproduces `world.wit` in full and must be
  updated together.

- **Settle the RDF export contract.** *(effort: high; narrow-to-fact-label-dump
  exit: low)* `nibli-import/src/export.rs`:1-18 (the whole
  file, byte-stable through the sweep) calls its output N-Triples (:1) but the sole
  emitter (:15) writes `# fact:<id> <label>` — a valid but EMPTY N-Triples document,
  all comment lines. Its doc comment still describes labels as "Lojban source or
  `:assert` form" (:6) and points at "the original Lojban text (the canonical source of
  truth for the KB)" (:8-10), wording that is doubly stale since gismu stopped
  resolving at the committed-corpus milestone. The labels it reaches are not
  round-trippable either: importing `ex:Rex a ex:dog .` and running
  `nibli-import <f>.ttl --export` emits `# fact:0 :assert a` (the `a` keyword is never
  expanded to rdf:type — `rdf.rs`:181-204 → `owl.rs`:36/:46-52 — and the label is
  minted as `:assert {relation}` at `nibli-session/src/lib.rs`:350 /
  `nibli-engine/src/lib.rs`:397; CLI routing `nibli/src/bin/import.rs`:40, :107-108).
  This is neither RDF nor a typed round-trip, while Chapter 21 advertises Turtle import
  and export. Either implement valid Turtle/N-Triples export from typed facts
  (`list_facts`, not labels) with tested IRI/literal mapping, or rename the module and
  feature to a fact-label dump and remove the RDF-export claims. Note for the typed
  design: since 07734c8 an RDF predicate locally named `exponential`/`logarithm`
  refuses at import ingress (query-only reference compute names), so an export/import
  round trip must account for that refusal class. **Exit:** real RDF takes an export ->
  independent parser -> re-import round trip with identity/literal/alias tests; a
  narrowed dump gets an exact-format contract and stale comments removed. Synchronize
  Chapters 16/21, CLI help, README, and the reference gate.

---

## Chain — verdict rendering (do 1 before or with 2)

Order: step 1 defines the honest FALSE/UNKNOWN/resource sentences; step 2's Depth hint
prints alongside them, and its repro exhibits exactly the false `[Why]` step 1 removes.

1. **Render computed FALSE as a decision, not closed-world non-derivability.** *(effort: high)* A query
   such as `greater(3, 5)` carries a false `ComputeCheck` leaf, but `summarize_false`
   (`nibli-render/src/summary.rs`:223-249) has arms only for `PredicateNotFound`,
   `ForallCounterexample` and `ExistsFailed`, and otherwise returns the fallback at
   :248, "This could not be derived from the known facts and rules." `summarize_true`
   calls `collect_extras` (:475) at :85 and that is the ONLY place a `ComputeCheck`
   becomes English (through `computed_extra_label`, :466); the FALSE path never calls
   it. Observed through nibli-host: `? greater(3, 5).` prints that CWA sentence one
   line above `⊢ greater  [computed (local)] -> FALSE`. The discriminator already
   exists and already crosses the WIT boundary — `ProofTrace.cwa_false` — and
   nibli-render already reads it at `collapse.rs`:221 and `proof.rs`:99, just never in
   `summary.rs`, so the fix need not re-derive it from the steps. The same fallback
   also fires under a `RESOURCE_EXCEEDED (depth)` verdict, so cover resource verdicts,
   not only `ComputeCheck`. Handle local arithmetic/numeric FALSE and trusted-backend
   FALSE explicitly, without a CWA caveat, while ordinary missing-fact FALSE keeps the
   non-derivability explanation. **Exit:** renderer, host, UI, protocol and WASM tests
   distinguish local computed FALSE, backend FALSE, backend unavailable, non-finite
   UNKNOWN, resource-exceeded, and ordinary CWA FALSE; Chapter 17 is recaptured from
   real bytes. (Surfaced by the book's 2026-07-26 review pass.)

2. **`resource_hint`'s Depth arm is dead code.** *(effort: medium; drop-the-dead-arm exit: low)* `resource_hint`
   (`nibli-host/src/main.rs`:587) has exactly one call site, :1846, inside the `Err(e)`
   trap arm of `run_proof_query` (fn at :1798); an engine-returned Depth verdict takes
   the `Ok(Ok(…))` arm at :1802 and never reaches it, and `classify_resource_trap`
   (:566) cannot yield Depth — its own doc says so at :564-565, so dropping the arm
   contradicts nothing already written. Reproduced: a 13-link `X(every Y).` chain at
   the default `max_chain_depth` of 10 (`awake(every actual). … dark(every cyan).
   actual(Rex).` then `? dark(Rex).`) prints `[Query] RESOURCE_EXCEEDED (depth)` with
   no hint line — and with the same false `[Why] This could not be derived from the
   known facts and rules.` that step 1 fixes. The unreachable hint text (:595)
   recommends raising `max_chain_depth`, a knob no shipped surface exposes: the only
   setter is the Rust API `KnowledgeBase::set_max_chain_depth`
   (`nibli-reason/src/lib.rs`:603) — there is no `:depth` command and no `NIBLI_DEPTH`,
   and GUARANTEES.md:133 states the shipped runtime surfaces keep the default. Wire a
   real Depth hint into the verdict path (and something for it to recommend), or drop
   the dead arm.

---

## Chain — proof certificate (1 → 2 → 3)

Order: step 1 is a small self-contained fail-safe whose typed "cannot index this proof"
outcome step 2's envelope should carry; step 2 defines the versioned contract; step 3
extends that contract to materialised verdicts — design 2 knowing 3's needs, or the
envelope bakes in the per-rule-derivation assumption C2 exists to relax.

1. **Check proof indices instead of casting them.** *(effort: medium)* The display-string half of this
   entry LANDED in `706bfb9`: the provenance tracer keys structurally on `StoredFact`
   (`memo: &mut HashMap<StoredFact, u32>` — signature sites now
   `nibli-reason/src/reasoning.rs`:596 and :3309, plus a THIRD site since the TraceSink
   refactor, the `RecordingSink` field at :2893; read at :3314), so human rendering is
   no longer an identity boundary and display stays at the render edge. What remains is
   the index half: every step index is produced by an unchecked `steps.len() as u32` —
   still exactly nine sites, now `reasoning.rs`:2898, :3319, :3343, :3389, :3411,
   :3430, :3472, :3586, :3615 (no `try_from` anywhere in the file) — so a trace that
   outgrows `u32` wraps into a valid-looking back-reference instead of failing. Fail
   cleanly if a proof cannot be indexed. **Exit:** oversized/deep traces return a typed
   resource/error outcome rather than a silently truncated index; the memo regression
   suite and `just verify-proofs` remain green.

2. **Bind proof traces to the full verdict and durable evidence.** *(effort: extra; narrow-the-public-contract
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

3. **Materialisation: the trace story (C2).** *(effort: extra; declining as a decision of record:
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
