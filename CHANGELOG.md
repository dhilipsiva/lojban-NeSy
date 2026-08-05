# Changelog

All notable changes to the nibli workspace are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and the workspace adheres to lockstep [Semantic Versioning](https://semver.org/)
for its published crates (Tier A in
[RELEASING.md](RELEASING.md)'s decision table). The WIT component ABI version
(`nibli:engine@…` in `wit/world.wit`) is **independent** of crate semver.

During 0.x, minor versions may break APIs; every release documents its changes
here first.

## [Unreleased]

### Changed

- **WIT `nibli:engine@0.9.0` exposes generated-witness provenance:**
  `witness-origin` gains `generated-witness`, so find bindings and existential
  proof steps distinguish opaque reasoner-minted witnesses from user constants
  without interpreting the display-only `sk_N` spelling. Universal proof
  payloads now carry origin-bearing `witness-binding` records too, instead of
  bare terms. These are breaking component, native Rust, and proof-JSON changes
  during 0.x.
- **Internal Skolem identity is structural and source-scoped:** generated
  independent and dependent witnesses are typed terms keyed by assertion id,
  binder ordinal, sort, and origin. A user constant such as `"sk_0"` or
  `"sk_0(adam)"` can no longer alias a generated witness through equality,
  event joins, rule identity, find/count, proof memoization, retraction, or
  rebuild. `sk_N` remains presentation only. Compute dispatch refuses opaque
  generated arguments as `UNKNOWN (backend-unavailable)` instead of leaking
  them into the string-only backend protocol; equal-looking user constants are
  still dispatched normally.
- **Typed fact-store schema is v2:** authoritative v1/unversioned populated
  stores fail closed. The engine's disposable typed mirror is erased and
  restamped atomically, then rebuilt from the authoritative `LogicBuffer`
  registry without decoding legacy rows. Compiler-only pattern/placeholder terms
  are rejected at persistence ingress.
- **Persistent native direct/text assertions now share one source-id space:**
  `assert_fact_direct` is durable when `NibliEngine` is opened with a store, and
  both assertion paths reconcile the live and durable allocators. A reasoning
  rejection physically deletes the row written before evaluation, so a failed
  call cannot resurrect on reopen.
- **`KnowledgeBase::with_store` accepts only an empty custom store:** typed rows
  alone cannot restore rules, domain/equality indexes, source
  provenance, or retraction semantics. Embedders must install an empty mirror and
  replay their authoritative buffers with stable ids via `assert_fact_with_id`.
- **Existential import is now an explicit, coherent legacy profile:** clean-core
  is the default, so a description universal does not manufacture existence and
  `some` is plain ∃. `NIBLI_EXISTENTIAL_IMPORT=1`,
  `:existential-import on`, or the programmatic setter opts into the old xorlo
  witness behavior. An imported witness now participates consistently in ∃, ∀,
  find, exact-count, `count_witnesses`, and aggregate enumeration instead of
  being visible only to boolean reasoning. Profile changes transactionally replay
  the active assertion registry (rollback preserves the old profile on failure),
  so toggles apply immediately to existing rules. The host reports the effective
  profile at startup/toggle and the browser UI labels it.
- **WIT `nibli:engine@0.8.0` exposes witness provenance and effective import
  state:** `witness-binding` and `exists-witness-rule` gain `witness-origin`,
  `count-result-rule` gains `existential-imported`, and
  `set-existential-import` is now fallible because it rebuilds the KB; the new
  `existential-import-enabled` getter reports what the session is actually using.
  The native and wasm-bindgen APIs expose the same origin/profile information.
  This is a breaking component/proof-JSON/API change during 0.x.
- **Ordinary-predicate temporal rules are now explicit and flavor-exact:** bare
  rules no longer get
  an implicit second firing pass for `past`/`now`/`future` queries. Bare means
  unqualified, not rigid or timeless; authors declare the intended mapping on
  each rule literal (for example, `all $x: past dog($x) -> past animal($x).`).
  Bare NAF stays bare, explicit tensed NAF keeps its flavor, flavored rules keep
  ordinary proof labels, and materialisation continues to fall back rather than
  erase wrappers. The Vampire/clingo flavorizer now emits each rule once and
  differentially checks bare taxonomy/causal/NAF non-lifting alongside explicit
  same- and cross-flavor controls; every curated temporal case must reach its
  oracle rather than silently skip.
- **Mixed tense×deontic stacks now fail closed instead of losing a wrapper:** one
  atom may carry one temporal prefix or one deontic prefix, but not both.
  `must past P` and `past must P` are compile errors; AST compilation/rendering
  rejects programmatic dual-field propositions, and raw `LogicBuffer` ingress
  rejects any second flavor wrapper on the same path before facts, rules, NAF,
  queries, finds, proofs, materialisation, or replay can observe it. Separate
  rule literals may still have different flavors. This is a breaking surface
  syntax restriction during 0.x; the individual WIT node variants are unchanged.
  A legacy persisted buffer containing a mixed stack fails replay with a reasoning
  error rather than being rewritten, deleted, or interpreted with one wrapper.

### Fixed

- **Sound semantic identity:** rule deduplication now buckets by digest but compares a
  full alpha-canonical identity, including flat/grouped NAF metadata, instead of treating
  one 64-bit hash as equality. Opaque abstractions now emit a versioned
  `__abs_v1_<digest>_<lossless-key>` marker whose tagged, length-delimited key includes
  abstraction kind and the complete alpha-canonical body. Ingress recomputes the digest
  from that key, so the digest is non-semantic; malformed/unknown marker versions and exact
  legacy `__abs_<16hex>` persisted buffers fail closed with an actionable database/re-import
  recovery path rather than silently returning a false verdict against newly compiled queries.
  Programmatic constraints, custom fact stores, and authoritative typed-store rows now share
  the same validation; the engine's disposable typed mirror is erased before decoding and
  rebuilt from the canonical LogicBuffer registry, so an obsolete mirror cannot block recovery.
- **`nibli-formalize`:** the shipped LLM system prompt demonstrated
  `every dog $d: animal($d) & barks($d).` — `barks` is not a corpus name, so
  that statement is a compile error, in a prompt whose own text warns that the
  compiler fails closed on unknown words. Corrected to `runs`. The
  gate-validity guard covered the few-shot examples only; it now also compiles
  every complete statement written in the instruction prose
  (`prose_kr_statements_are_gate_valid`), so this class cannot recur.

## [0.1.0] - 2026-08-03

The first tagged release: the engine as it has shipped on `main` — a
deterministic theorem prover compiled to WebAssembly (WASI P2), with the
**nibli KR** predicate-call language as its sole front-end. **0.x caveat:**
minor versions may break APIs; the embed surface (`nibli-engine`) is not yet
declared stable.

### Added

- **The pipeline**: nibli KR (`nibli-kr`, pest-grammar front-end) →
  First-Order Logic IR (`nibli-semantics`, spec in `LOGIC_IR.md`) →
  demand-driven backward chaining over an indexed fact store
  (`nibli-reason`), shared by every runtime surface through
  `nibli-session::CoreSession`.
- **Runtime surfaces**: the `nibli-pipeline` WASM component
  (`nibli:engine@0.7.0` WIT) under the `nibli-host` Wasmtime REPL; the native
  `nibli-engine` embedding; the browser `nibli-wasm` / `nibli-ui` bundles;
  dev tooling in `nibli` (REPL, validate, import CLI, `nibli-pin`, benches).
- **Guarantees and gates** (`GUARANTEES.md`): differential soundness against
  Vampire and clingo, six mechanized Lean 4 proofs with Rust conformance
  bridges, the KR seam gate, corpus/dictionary differentials, KB-level
  behavioural pins, fuzzing, and a mutation-testing baseline.
- **The committed English corpus** (`nibli-lexicon`): the dictionary as
  validated Rust source — one build mode, no network at build time.
- Persistent store (`nibli-store`, redb schema v3), English rendering
  (`nibli-render`), proof-trace wire format (`nibli-protocol`), RDF/OWL
  import (`nibli-import`), the agentic formalizer (`nibli-formalize`), and
  built-in authorization (`nibli-auth`, unpublished).
- Workspace release packaging (R0): lockstep `[workspace.package]` `0.1.0`
  inherited by every member, `[workspace.dependencies]` (path + version) for
  the internal crates (internal *dev*-dependencies stay path-only on purpose
  — they are stripped at publish and must not constrain the publish order),
  `publish = false` on the non-publishable tier, per-crate descriptions and
  READMEs, this CHANGELOG, and the `just release-check` consistency gate.

[Unreleased]: https://github.com/dhilipsiva/nibli/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/dhilipsiva/nibli/releases/tag/v0.1.0
