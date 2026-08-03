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

### Fixed

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
