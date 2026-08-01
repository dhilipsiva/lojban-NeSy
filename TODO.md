# Engine TODO

Plain bullets — delete when fully landed. Docs & release track: **`DOCS_TODO.md`**.
Book manuscript: separate `book/` repo (`book/TODO.md`).

---

## Authorization (first-class multi-language auth)

**Goal:** Built-in authorization as a **native** nibli surface — committed policy KB +
stable WIT + thin Python/Rust framework adapters — with **low runtime overhead** and
**unchanged** zero-hallucination / CWA / CDA guarantees. Not an external Python package;
not Extism-primary (optional future PDK note only).

**Consumers:** FastAPI, DRF + django-spectacular, Strawberry, Graphene, OpenAPI helpers;
axum, tower, async-graphql, juniper (same KR policy).

**Efficiency:** long-lived engine; policy once; ephemeral context facts only; decision
cache (`policy_version` + agent/action/resource + context hash); proofs **opt-in**.

### A0 — Design (landed)

Inspected: `wit/world.wit` (`nibli:engine@0.5.0`, world exports `engine` only),
`nibli-session` / `nibli-engine` (assert/query/retract), lexicon collisions, `python/`
(compute backend only — no engine binding).

### Decisions of record (locked — A0)

| Decision | Choice |
|----------|--------|
| **Primary ABI** | **`interface authorizer` inside `nibli:engine@0.6.0`** (single-package shipping; logical auth v0.1). World exports `engine` + `authorizer`. Split `nibli:auth@0.1.0` package deferred. Extism **not** primary. |
| **Policy home** | `nibli-auth/policy/auth-0.1.0.nibli` via `include_str!`; `POLICY_VERSION = "0.1.0"` |
| **API decision head** | **`authorized(agent, action, resource)`** — **not** `can` (**`can` is tin/can `lante` in corpus**) |
| **Field-level head** | **`visible_attr(agent, resource, attr)`** — **not** `field` / `visible_field` (**`field` is agricultural `foldi`**) |
| **Principal sort** | Prefer **`agent`** (FREE); do **not** reuse corpus **`principal`** (`ralju`) |
| **Ownership** | Reuse existing **`owns(owner, owned, basis)`** |
| **Roles / tenants** | NEW: `has_role(agent, role)`, `in_tenant(agent, tenant)`; optional `resource_tenant` |
| **Action / attr tokens** | KR **quoted strings** (`"update"`, `"title"`) → `Constant` |
| **Roles/tenants style** | Prefer **quoted strings** (parity with actions) |
| **UNKNOWN on hot path** | **Deny** (`allowed=false`, verdict=unknown); only engine TRUE ⇒ allow |
| **Decision payload** | `allowed`, `verdict` (true/false/unknown/resource-exceeded), optional `reason`, `fields` list; proof only on `explain` |
| **Context** | Multi-line **context KR** string: assert → query → **retract** fact ids |
| **Session** | One warm `Authorizer` wrapping engine/session; never per-request spawn; never `reset()` between requests (drops policy) |
| **Python bridge** | **PyO3** primary (`nibli_auth`); no subprocess hot path |
| **Deontic corpus** | Do **not** overload `permits`/`permitted` as HTTP auth |

**Forbidden names for auth heads:** `can`, `field`, `principal` (corpus collisions).

### Locked file layout

| Path | Role |
|------|------|
| `nibli-auth/policy/auth-0.1.0.nibli` | Built-in policy |
| `nibli-auth/src/{lib,policy,cache}.rs` | Authorizer, Decision, cache |
| `nibli-lexicon/.../predicates.rs` | NEW rows: `authorized`, `visible_attr`, `has_role`, `in_tenant`, optional `agent`/`resource`/`resource_tenant` |
| `wit/auth.wit` | `nibli:auth@0.1.0` |
| `wit/world.wit` | Export `authorizer` (A2) |
| `python/nibli_auth/` | Client + FastAPI/DRF/Strawberry/Graphene/spectacular |
| `examples/auth-axum/`, `examples/auth-fastapi/` | **Same policy** E2E |
| `mdbook/src/user/authorization.md` | A5 / DOCS_TODO Phase 2b |

### WIT sketch (locked shape — land in A2)

```wit
package nibli:auth@0.1.0;

interface types {
  enum verdict { true_, false_, unknown, resource-exceeded }
  record decision {
    allowed: bool,
    verdict: verdict,
    reason: option<string>,
    fields: list<string>,
  }
  record explained {
    decision: decision,
    proof-json: option<string>,
  }
}

interface authorizer {
  use types.{decision, explained};
  resource session {
    constructor();
    load-policy: func(extra-kr: option<string>) -> result<string, string>;
    assert-facts: func(kr: string) -> result<list<u64>, string>;
    retract: func(id: u64) -> result<_, string>;
    can: func(agent: string, action: string, resource: string,
              context-kr: string) -> result<decision, string>;
    allowed-fields: func(agent: string, action: string, resource: string,
                         context-kr: string) -> result<list<string>, string>;
    explain: func(agent: string, action: string, resource: string,
                  context-kr: string) -> result<explained, string>;
    policy-version: func() -> string;
    clear-ephemeral: func() -> result<_, string>;
  }
}
```

API method names stay `can` / `allowed-fields` / `explain` (host language); KR uses **`authorized`**.

### Policy sketch (compile-clean form preferred in A1)

```nibli-kr
# Prefer prenex if description-universals are awkward:
# all $a, $r: owns($a, $r) -> authorized($a, "read", $r).
# all $a, $r: has_role($a, "admin") -> authorized($a, "update", $r).
```

### Phased delivery

#### A0 — Inspect & design

**Landed:** collisions, ontology, WIT package choice, layout, PyO3, efficiency/deny defaults (this section).

#### A1 — Built-in policy KB + engine core

**Landed:** lexicon (`authorized`, `visible_attr`, `has_role`, `in_tenant`, `agent`, `resource`, `resource_tenant`); `nibli-auth` crate + `policy/auth-0.1.0.nibli`; warm `Authorizer` (load once, ephemeral context, decision cache); `can` / `allowed_fields(candidates)` / `explain`; tests via `just test-auth`.

**Policy note:** conclusion vars must appear in a positive body condition (admin rules use `resource($r)`). v0.1 field masking = all candidates if row `can(..., "read"|action)` allows.

#### A2 — WIT + pipeline / host

**Landed:** `authorizer` interface on `nibli:engine@0.6.0`; world `export authorizer`; pipeline `AuthSession` wraps `nibli_auth::Authorizer`; `nibli-auth` uses `nibli-session` (wasm-safe, no redb); host rebuilds against 0.6.0. WIT param `object` = resource id (`resource` is a WIT keyword).

#### A3 — Rust adapters + example

**Landed:** `nibli_auth::tls` (thread-local warm policy); features `axum` / `async-graphql` / `juniper` guards; `examples/auth-axum` (context_kr from app DB; `X-Agent` header). `just test-auth`, `just check-auth-axum`, `just run-auth-axum`.

#### A4 — Python adapters + example

**Landed:** `nibli-auth-py` (PyO3 → `nibli_auth_native` via maturin); `python/nibli_auth` (FastAPI helpers + optional DRF/Strawberry/Graphene/spectacular); `examples/auth-fastapi`. `just build-auth-py` / `test-auth-py` / `run-auth-fastapi`. Same tls + policy as auth-axum.

#### A5 — Docs, CI

**Landed:** mdBook [Authorization](mdbook/src/user/authorization.md); README pointer; CI job `auth` (`just test-auth` + `check-auth-axum`); Extism documented as non-primary. Python `test-auth-py` remains local (maturin).

### Success criteria — **met** (A0–A5)

- Warm `Authorizer` + policy; `can` / `allowed_fields` / `explain` from **Rust and Python**.
- Same KR policy in `examples/auth-axum` and `examples/auth-fastapi` (and framework guards).
- Field masking shares `Decision` family; proofs only via `explain`.
- `just test-auth` + docs build; zero-hallucination = entailment-only allow.

### Explicit non-goals (still)

- Extism as primary runtime; per-request engine spawn; CWA/CDA weakening;
  inventing vocabulary outside fail-closed corpus; framework-specific policy languages.

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
- **Conformance guard (optional ratchet):** test or script that TextMate keyword
  alternation ≡ `RESERVED_WORDS` (and document Tree-sitter keyword list the same
  way when that grammar exists) so reserved-list edits cannot drift the editors.
- **Book fence rename (optional, coordinated):** if/when fences go plain
  `nibli` only, retarget `verify_book.py` + capture harness + injection regex in
  one PR; keep `nibli-kr` as Linguist/VS Code alias forever.

---

## Language semantics

- **NIBLI_KR.md §6 co-reference vs the compiler.** The spec (§6) says all
  occurrences of one `$name` in a statement co-refer; the compiler gives each
  top-level `&` side its OWN existential scope — verified: `bite($x, Bel) &
  bite($x, Dana).` compiles to two separate `∃ $x` scopes, asserted and queried
  alike (the shared CoreSession compile chain), which is Ch 21's disclosed
  fluent-but-wrong case. Decide: implement statement-wide co-reference (the
  "correlated multi-witness" roadmap item) or amend §6 to document the
  per-conjunct scope rule. The book documents the engine's behavior and flags
  this disagreement; whichever way it resolves, update Ch 21 + NIBLI_KR.md
  together.

---

## Compute / fact lifecycle

- **Auto-ingested compute facts bypass the fact registry.** An arithmetic/backend
  `true` auto-asserted mid-query (`assert_typed_fact`) writes only the typed fact
  store: no `FactRecord`, no fact id — invisible to `:facts`, not retractable, and
  dropped by any rebuild (retraction, failed-assert rollback, and trap replay all
  rebuild from the registry). `set_compute_dispatch`'s trust-boundary doc already
  discloses the non-durability; the store-shadowing policy question in
  `nibli-reason/src/compute.rs` points here. Decide the lifecycle: give ingested
  facts registry records (listable, retractable, replay-safe), or codify the
  current outage-cache semantics as the contract (dispatch-first on every query;
  the stored fact is consulted only when dispatch errors) and say so in
  GUARANTEES.md.

---

## Reasoning / evaluation

- **The numeric quantifier-domain exclusion is WIDER than the universal, and two of
  its faces are WRONG VERDICTS, not just silent ones.** Surfaced by the adversarial
  review of the `[Domain]` diagnostic (2026-08-01), which covers only `ForAllNode`.
  Numbers never enter `known_entities` (`collect_and_note_constants`, rules.rs), and
  every other consumer of the domain inherits that. Ranked by severity:
  - **`CountNode` (`exactly N` / `no`) counts over the same number-free domain.** With
    `big(5). dog(5). dog(Rex).` the engine entails `big(5).` and `dog(5).` yet answers
    `dog(no big).` = TRUE and `dog(exactly 0 big).` = TRUE, with nothing on any channel.
    Whether that is a *wrong* verdict or merely a surprising one depends on whether
    `exactly N` is read as counting the quantifier domain (in which case it is correct and
    merely undisclosed) or as counting satisfying values (in which case it is wrong) — an
    adversarial check disputed the "wrong verdict" reading on exactly that ground. Either
    way the count semantics need stating, and this is the same domain decision as the rest
    of this entry.
  - **`some X` answers a definitive FALSE when the matrix is arithmetic/comparison.**
    `big(5).` + `sum(some big, 2, 3).` = FALSE though `big(5)` and `sum(5,2,3)` are both
    TRUE. Entity control (`dog(some big)`) is TRUE, so it fires exactly on
    number-bearing quantification.
  - **`query_find` / `count_witnesses` / `aggregate` report a truncated enumeration as
    DEFINITIVE.** `extract_rule_candidates_for_entailment` instantiates from
    `all_typed_domain_members()`, and `find_horizon_hit` is not set on that path, so the
    `INCOMPLETE_MSG` refusal never fires. This is the one place the gap escapes as a
    NUMBER a user pastes into a report rather than as a TRUE/FALSE.
  - **A universal has a different domain as an ASSERTION than as a QUERY.** Asserted, it
    becomes a `UniversalRuleRecord` and rule firing unifies a `PatternVar` with a
    `GroundTerm::Number` — so it DOES reach numbers; queried, the `ForAllNode` arm
    enumerates `all_non_event_domain_members()` and does not. Same sentence, two domains,
    and only the query side is disclosed.
  Fixing these properly means deciding whether numbers belong in the quantifier domain at
  all — which would move the pinned verdicts at `numeric_terms_are_not_universal_domain_members`
  and is a semantics decision, not a diagnostics one. Until then the disclosure at
  GUARANTEES §Disclosed Sharp Edges understates the blast radius: it describes the
  vacuous universal and not the wrong `no`/`some` verdicts.

- **The `[Domain]` diagnostic only sees ASSERTED extensions.** Its candidate source is
  `arg_position_index`, which holds stored facts, so a RULE-DERIVED (IDB) restrictor stays
  silent: `dog(5). animal(every dog).` + `sum(every animal, 2, 2).` gets no note although 5
  is in `animal`'s extension, while the one-hop twin `sum(every dog, 2, 2).` does. That is
  the fail-closed direction (a missed note, never a fabricated one) and is disclosed, but
  it means the caveat appears or vanishes purely on whether a rule sits in between — the
  shipped Syllogism shape. Candidates could instead come from the materialised extension
  when the relation is complete (`Materialized::is_complete_for`).

- **`smoke-host-quiet` does not cover `[Domain]`.** Every other engine echo has a
  ci-wasm smoke pinning both directions of `NIBLI_QUIET`; this one rides the same
  `inner.verbose` gate but is unpinned at the component boundary.

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

## Book-review upstream items (2026-07-26 manuscript review)

Surfaced by the book's verified review passes; each hand-verified at filing time
(2026-07-27) and naming its book ripple where one exists.

- **`obliged`-spelled every-duty renders the wrong obligated party.**
  `obliged(every data governs, event { message() }).` back-translates as "For every
  X, if X governs and X is data, then **Y** is obligated to notify" — the
  post-`8286738` deontic collapse picks the event variable as the duty-holder for
  the BASE spelling, while the converted `obligated` spelling correctly binds X.
  The who-selection appears keyed to the converted routing. Fix the base-spelling
  collapse; ripple: re-check nibli-wasm's `c18_draft_error_glosses_are_verbatim`
  pin and the book's Ch 18 alias note.
- **README's merged REPL-commands table** (~243–263) lists host-only commands
  (`:backend`/`:fuel`/`:memory`/`:strict`/`:existential-import`) and
  debug-REPL-only commands (`:contradictions`/`:trace`/`:untrace`/`:traces`) in
  one table with no binary column — a documented confusion source. Split the
  table per surface or add a Surface column.
- **The component's import list is not gated.** Docs (book Ch 13/15/App C) state
  "no clock or filesystem imports" — true today (imports are the
  `wasi:cli`/`wasi:io` set + `wasi:random/insecure-seed`), but no CI check pins
  it and `wasm-tools` sits unused in the flake. A one-line `ci-wasm` smoke
  (`wasm-tools component wit target/wasm32-wasip2/release/nibli.wasm` + grep for
  the absent interfaces) closes the gap.
- **`resource_hint`'s Depth arm is dead code.** Hints fire only on the trap path
  (`nibli-host/src/main.rs` ~1604) and `classify_resource_trap` never yields
  Depth, so a `RESOURCE_EXCEEDED (depth)` verdict prints no hint line — and the
  unreachable hint text (~main.rs:537) recommends raising `max_chain_depth`, a
  knob no shipped surface exposes. Wire a real Depth hint into the verdict path
  or drop the dead arm.
- **nibli-engine persist-before-assert leaves an orphaned durable row on
  failure.** The store write-through (`nibli-engine/src/lib.rs` ~272–293) mints
  the id and writes the durable row BEFORE `assert_fact_with_id`; if the assert
  fails, the in-memory KB rolls back via the registry rebuild but the persisted
  row has no compensating delete on that path. Verify the intended semantics
  (does the schema-v3 open/replay tolerate the orphan fail-closed?) and either
  compensate or document.
- **`wit/world.wit` `proof-ref` doc comment is wrong.** It claims "No children —
  the full proof was shown at its first occurrence," but the step always carries
  exactly one back-reference child (the memo hit pushes
  `children: vec![cached_idx]`, `nibli-reason/src/reasoning.rs`); the verbose
  text renderer re-expands that child while the collapsed/UI renderings drop it.
  Decide whether that divergence is intentional, then fix the comment; ripple:
  the book's Appendix C reproduces `world.wit` in full and must be updated
  together.

---

## Pointers

| Tracker | Scope |
|---------|--------|
| **`DOCS_TODO.md`** | mdBook docs, GH Pages / site primary, crates.io + GitHub Releases (R0–R3); auth chapter when A5 ships |
| **`book/TODO.md`** | Manuscript only (private checkout; Orange AVA) |
| **This file** | Engine runtime, **authorization track (A0–A5)**, editors |
