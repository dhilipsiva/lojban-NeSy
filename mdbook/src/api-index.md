# API index

Rustdoc for the embeddable crates, on **docs.rs** since the first publish
(v0.1.0, 2026-08-03). Embedding starts with one line:

```bash
cargo add nibli-engine
```

Every crate below is version `0.1.0` in workspace lockstep,
`MIT OR Apache-2.0`, published in the dependency order the decisions table in
[DOCS_TODO.md](https://github.com/dhilipsiva/nibli/blob/main/DOCS_TODO.md)
locks. (docs.rs builds each crate shortly after publish — a page may briefly
show “building” right after a release.) For local API docs from a checkout,
`cargo doc -p <crate> --open` still works.

## Publishable crates (Tier A, in dependency order)

| Crate | API docs | What its API gives you |
|-------|----------|------------------------|
| `nibli-types` | [docs.rs/nibli-types/0.1.0](https://docs.rs/nibli-types/0.1.0) | Canonical type definitions shared across the pipeline — `LogicBuffer`, `AstBuffer`, `NibliError`, the shared arithmetic evaluator |
| `nibli-lexicon` | [docs.rs/nibli-lexicon/0.1.0](https://docs.rs/nibli-lexicon/0.1.0) | The committed English predicate corpus — lookups, place labels, provenance bridge |
| `nibli-protocol` | [docs.rs/nibli-protocol/0.1.0](https://docs.rs/nibli-protocol/0.1.0) | Shared wire-format proof-trace types and the JSON helpers |
| `nibli-kr` | [docs.rs/nibli-kr/0.1.0](https://docs.rs/nibli-kr/0.1.0) | The nibli KR surface-syntax front-end — `parse_checked`, `parse_text`, `render`, the pest grammar |
| `nibli-semantics` | [docs.rs/nibli-semantics/0.1.0](https://docs.rs/nibli-semantics/0.1.0) | Semantic compiler — flat AST buffer to First-Order Logic IR |
| `nibli-reason` | [docs.rs/nibli-reason/0.1.0](https://docs.rs/nibli-reason/0.1.0) | Reasoning engine — backward-chaining inference over the typed fact store, proof traces |
| `nibli-render` | [docs.rs/nibli-render/0.1.0](https://docs.rs/nibli-render/0.1.0) | Shared human-readable rendering for back-translation and proof traces |
| `nibli-session` | [docs.rs/nibli-session/0.1.0](https://docs.rs/nibli-session/0.1.0) | The shared session core: the one compile/assert/query chain every runtime surface wraps |
| (`nibli-store`) | [docs.rs/nibli-store/0.1.0](https://docs.rs/nibli-store/0.1.0) | Persistent redb-backed knowledge-base store with tombstone retraction — parenthesized in the decision table: needed only when embedding *with* persistence |
| **`nibli-engine`** | [docs.rs/nibli-engine/0.1.0](https://docs.rs/nibli-engine/0.1.0) | The native embedding — `NibliEngine`: `assert_text`, `query_text_with_proof`, `query_find_text`, `retract_fact`, optional persistence. **Start here for embedding** |
| `nibli-formalize` *(optional)* | [docs.rs/nibli-formalize/0.1.0](https://docs.rs/nibli-formalize/0.1.0) | Agentic English→KR formalizer: LLM + validation gates + self-correction loop |
| `nibli-import` *(optional)* | [docs.rs/nibli-import/0.1.0](https://docs.rs/nibli-import/0.1.0) | RDF/OWL import and KB export utilities |
| `nibli` *(optional)* | [docs.rs/nibli/0.1.0](https://docs.rs/nibli/0.1.0) | The dev bins: native REPL, `nibli-validate`, `nibli-import` CLI, `nibli-pin` (bin-only; the bench bins are repo-only behind the `bench-bins` feature) |

## Not on the list

The Tier Z crates (`nibli-pipeline`, `nibli-host`, `nibli-ui`, `nibli-wasm`,
`nibli-verify`, `nibli-lexigen`, plus the workspace-excluded `fuzz/` harness)
are `publish = false`: they ship as the WASM component, host binary, hosted
sites, and CI gates rather than as libraries — the
[v0.1.0 GitHub Release](https://github.com/dhilipsiva/nibli/releases/tag/v0.1.0)
carries the built component and host. Their internals are covered by the
[developer guide](developer/overview.md). The auth crates (`nibli-auth`,
`nibli-auth-py`) and the `auth-axum` example are also `publish = false` for
now — they sit in neither tier row of the decision table; the
[authorization guide](user/authorization.md) covers their APIs.

Conceptual documentation (this site) stays on the primary/mirror hosts only —
docs.rs carries API docs, not these pages.
