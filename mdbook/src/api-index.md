# API index

Rustdoc for the embeddable crates. **Not on docs.rs yet** — docs.rs pages
exist only after a crate's first crates.io publish, and no nibli crate has
been published (the release track in
[DOCS_TODO.md](https://github.com/dhilipsiva/nibli/blob/main/DOCS_TODO.md) is
at R0: packaging landed, no `vX.Y.Z` release tags, no publishes). Until then,
build the API docs locally from a checkout:

```bash
cargo doc -p nibli-engine --open
```

Any crate below works the same way (`cargo doc -p <crate> --open`). When the
first publish lands (R2), each row here gains its versioned docs.rs link —
that flip is the remainder of docs Phase 4.

## Publishable crates (Tier A, in dependency order)

The publish order is the locked decision of record; every crate is version
`0.1.0` in workspace lockstep, `MIT OR Apache-2.0`.

| Crate | What its API gives you |
|-------|------------------------|
| `nibli-types` | Canonical type definitions shared across the pipeline — `LogicBuffer`, `AstBuffer`, `NibliError`, the shared arithmetic evaluator |
| `nibli-lexicon` | The committed English predicate corpus — lookups, place labels, provenance bridge |
| `nibli-protocol` | Shared wire-format proof-trace types and the JSON helpers |
| `nibli-kr` | The nibli KR surface-syntax front-end — `parse_checked`, `parse_text`, `render`, the pest grammar |
| `nibli-semantics` | Semantic compiler — flat AST buffer to First-Order Logic IR |
| `nibli-reason` | Reasoning engine — backward-chaining inference over the typed fact store, proof traces |
| `nibli-render` | Shared human-readable rendering for back-translation and proof traces |
| `nibli-session` | The shared session core: the one compile/assert/query chain every runtime surface wraps |
| (`nibli-store`) | Persistent redb-backed knowledge-base store with tombstone retraction — parenthesized in the decision table: needed only when embedding *with* persistence |
| **`nibli-engine`** | The native embedding — `NibliEngine`: `assert_text`, `query_text_with_proof`, `query_find_text`, `retract_fact`, optional persistence. **Start here for embedding** |
| `nibli-formalize` *(optional)* | Agentic English→KR formalizer: LLM + validation gates + self-correction loop |
| `nibli-import` *(optional)* | RDF/OWL import and KB export utilities |
| `nibli` *(optional)* | The dev bins: native REPL, `nibli-validate`, `nibli-import` CLI, `nibli-pin`, bench bins (bin-only — `cargo doc -p nibli` documents the binaries, not a library API) |

## Not on the list

The Tier Z crates (`nibli-pipeline`, `nibli-host`, `nibli-ui`, `nibli-wasm`,
`nibli-verify`, `nibli-lexigen`, plus the workspace-excluded `fuzz/` harness)
are `publish = false`: they ship as the WASM component, host binary, hosted
sites, and CI gates rather than as libraries. Their internals are covered by
the [developer guide](developer/overview.md). The auth crates (`nibli-auth`,
`nibli-auth-py`) and the `auth-axum` example are also `publish = false` for
now — they sit in neither tier row of the decision table; the
[authorization guide](user/authorization.md) covers their APIs.

Conceptual documentation (this site) stays on the primary/mirror hosts only —
docs.rs will carry API docs, not these pages.
