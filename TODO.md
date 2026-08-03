# Engine TODO

**Future-facing only.** An entry is here because it is still true and still wants doing —
delete it when it lands rather than marking it done, and put the record in the commit.
Every entry was re-verified against the tree on 2026-08-01; where a claim cites a line,
that line was checked, not remembered.

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

- **NIBLI_KR.md §6 co-reference — QUERY SIDE ONLY now.** The compiler still gives each
  top-level `&` side its own existential scope (`bite($x, Bel) & bite($x, Dana).` compiles
  to `∃$x.(…) ∧ ∃$x.(…)`; `nibli-semantics/src/semantic/compile.rs`:274-289 closes per
  PROPOSITION and :567-588 compiles each `&` side separately). But the two paths diverge,
  and only one disagrees with §6:
  - **Asserted — §6 holds, by accident.** `collect_exists_for_skolem`
    (`nibli-reason/src/rules.rs`:37-49) keys its Skolem map by variable NAME across the
    whole buffer, so both scopes collapse onto ONE constant: the statement stores
    `bite(sk_0, Bel)` and `bite(sk_0, Dana)`, while `$p`/`$q` give `sk_0`/`sk_2`. A rule
    needing one entity to do both fires for the shared name and not for distinct ones.
  - **Queried — the disagreement.** No such collapse: `bite(Ann, Bel). bite(Cy, Dana).`
    then `? bite($x, Bel) & bite($x, Dana).` answers TRUE though no single biter did both.
    This is the fluent-but-wrong case Ch 21 discloses.

  Decide: bind `$name` statement-wide on the query path (matching what assert already does
  by name-keying), or amend §6 to a per-conjunct scope rule for queries. **The assert-side
  agreement is INCIDENTAL** — a name-keyed Skolem map, not a designed co-reference pass —
  so a refactor to fresh-per-scope Skolems would silently break it; whichever way this
  resolves, it wants a pin. Ripple: Ch 21 + NIBLI_KR.md §6 together.

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
