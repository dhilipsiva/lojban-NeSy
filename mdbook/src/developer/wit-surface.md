# WIT surface

The component boundary, as declared in
[`wit/world.wit`](https://github.com/dhilipsiva/nibli/blob/main/wit/world.wit)
— package **`nibli:engine@0.7.0`**, world **`nibli-pipeline`**:

```wit
world nibli-pipeline {
    import compute-backend;
    export engine;
    export authorizer;
}
```

One component, one world. The WIT package version is **independent of crate
semver** (a locked decision of record). Only flat, `u32`-indexed data crosses
the boundary — no heap pointers.

## Interfaces

### `error-types`

`variant nibli-error` identifies the pipeline stage that failed:
`syntax(syntax-detail)` (with `message` + 1-based `line`/`column`),
`semantic(string)`, `reasoning(string)`, `backend(tuple<string, string>)`.

### `logic-types`

The IR and verdict types ([Pipeline & IR](pipeline-and-ir.md)):

- `logical-term` (5 cases) and `logic-node` (13 cases) mirror
  `nibli_types::logic` exactly — kebab-case names, identical declaration order
  (the component-model discriminant is positional).
- `logic-buffer { nodes, roots }`.
- `query-result`: `true` | `false` | `unknown(unknown-reason)` |
  `resource-exceeded(resource-kind)`, with `unknown-reason` ∈ {`cycle-cut`,
  `incomplete-knowledge`, `naf-dependent`, `backend-unavailable`,
  `non-finite`} and `resource-kind` ∈ {`depth`, `fuel`, `memory`}.
- `witness-binding { variable, term }`, `fact-id` (u64),
  `fact-summary { id, label, root-count }`.
- `proof-rule` (19 cases) + 15 named-field payload records (WIT variant cases
  hold at most one payload type, so each data-carrying case gets a record —
  the interface self-documents instead of using positional tuples);
  `proof-step { rule, holds, children }`;
  `proof-trace { steps, root, naf-dependent, cwa-false }` — the NAF/CWA flags
  are computed once in the engine and carried across so consumers never
  recompute them.
- `materialization-report { complete, refused }` — which relations the NAF
  saturation completed, and which it refused with a one-line reason each.

### `compute-backend` (the host import)

- `evaluate(relation, args) -> result<bool, nibli-error>`
- `evaluate-batch(requests) -> list<compute-result>` — one boundary crossing
  for N requests; results in input order, one failure never poisons the batch.

The engine calls this for predicates registered for compute dispatch; the host
answers built-in arithmetic locally and forwards the rest over TCP
([WASM, host & compute](wasm-host-compute.md)).

### `engine` (export) — the `session` resource

| Method | Contract |
|--------|----------|
| `constructor()` | Fresh KB |
| `assert-text(input)` | → `list<(fact-id, logic-buffer)>` — multi-statement input splits into one independent fact per root (a connective stays one compound fact); each pair carries the compiled buffer so a persisting host can replay without recompiling |
| `query-text(input)` | → `query-result` |
| `query-text-with-proof(input)` | → `(query-result, proof-trace)` |
| `query-find-text(input)` | → witness binding sets |
| `compile-debug(input)` | Compile without asserting; the host renders the buffer |
| `assert-fact(relation, args)` / `assert-fact-with-id(…)` | Ground fact, bypassing text parsing; the `-with-id` form takes a caller-chosen id for restart replay |
| `assert-buffer-with-id(buffer, label, id)` | **The** recompile-free replay primitive (the legacy `assert-text-with-id` was removed at 0.5.0 with store schema v3) |
| `retract-fact(id)` / `list-facts()` / `reset-kb()` | KB management |
| `register-compute-predicate(name)` | Marks a relation for compute dispatch |
| `set-strict(bool)` | Off = permissive warn-and-insert; on = arity/integrity violations reject atomically |
| `set-existential-import(bool)` | Default on (a description universal mints a presupposition witness); off = clean-core classical ∃ |
| `set-materialization(bool)` / `materialization-report()` | NAF saturation toggle + its report — added in 0.7.0 because the optimisation is invisible when it fails: without the report, a KB whose `~p(x)` stays slow has no way to learn which relation fell out of the materialisable fragment, or why. A definitive TRUE/FALSE can never flip (the `materialize_diff` gate enforces it); a non-definitive OFF verdict may become definitive under ON — the deliberate depth-bound completeness gain |

### `authorizer` (export)

The built-in authorization surface (logical auth v0.1), wrapping the same
policy as the native `nibli-auth` crate. Types: `decision { allowed, verdict,
reason, fields }` (`allowed` is true **only** on entailment TRUE) and
`explained { decision, proof-json }`. Session methods: `load-policy`,
`assert-facts`, `retract`, `can`, `can-any` (batch), `allowed-fields`,
`explain`, `policy-version`, `clear-ephemeral`. Two conventions worth knowing:
the protected-resource parameter is named `object` (WIT reserves the keyword
`resource`), and error results are plain `string`, not `nibli-error`.

## Bindings: the types ARE `nibli_types`

`nibli-pipeline` is the only crate with WIT bindings, regenerated by
`cargo component build -p nibli-pipeline` (the `just build-wasm` recipe, which
also `cargo fmt`s the output; `src/bindings.rs` is gitignored — CI regenerates
it on every run).

`[package.metadata.component.bindings.with]` remaps the **ten** ABI-matching
boundary types onto the canonical `nibli_types` definitions —
`logical-term`, `logic-node`, `logic-buffer`, `query-result`,
`unknown-reason`, `resource-kind`, `witness-binding`, `fact-summary` from
`logic-types`, plus `nibli-error` and `syntax-detail` from `error-types` — so
the guest passes the canonical types straight through instead of maintaining a
mirror-conversion layer.

The one exception is the proof trio: `proof-rule` is named-field in Rust but
wit-bindgen emits only tuple/newtype variants, so it keeps the single
hand-written `convert_proof_rule` bridge in `nibli-pipeline/src/lib.rs`
(`proof-step`/`proof-trace` reference it and stay generated too).

**Version-bump checklist:** the remap keys pin the interface version
(`nibli:engine/logic-types@0.7.0/…`). Any WIT version change must bump **all
ten keys** — a missed key silently stops remapping and resurrects the mirror
types.

## Version history

| WIT | Change |
|-----|--------|
| 0.7.0 | `set-materialization` + `materialization-report` |
| 0.6.0 | `export authorizer` (wrapping native `nibli-auth`) |
| 0.5.0 | Removed legacy `assert-text-with-id` (store schema v3) |
| 0.4.0 | `set-existential-import` |
| 0.3.0 | Named-field `proof-rule` payload records |
| 0.2.0 | `set-language` (no longer present in the current WIT — the Lojban front-end retired at THE DROP) |
