# WASM, host & compute

How the engine ships: one WASM component, a native Wasmtime host, an optional
external compute backend, and two browser surfaces that skip WASI entirely.

**Sources:** `nibli-host/src/main.rs`, `nibli-protocol/src/compute_client.rs`,
`python/nibli_backend.py`,
[DEPLOY.md](https://github.com/dhilipsiva/nibli/blob/main/DEPLOY.md).

## One component, four runtimes

`nibli-pipeline` is the **single** WASM component (WIT world `nibli-pipeline`):
nibli-kr / nibli-semantics / nibli-reason are internal Rust crate dependencies,
not separate components. It imports `compute-backend` from the host and exports
the `engine` + `authorizer` interfaces ([WIT surface](wit-surface.md)).

| Runtime | What runs | Compute dispatch |
|---------|-----------|------------------|
| `nibli-host` (Wasmtime, WASI P2) | loads `nibli.wasm`; the canonical operator REPL | The component registers dispatch at Session creation, bridging to the host's `compute-backend` implementation |
| `nibli-engine` (native, in-process) | the same crates as plain Rust | Opt-in: `enable_compute_backend(addr)` wires the native TCP client; otherwise external compute stays unregistered |
| `nibli-wasm` / `nibli-ui` (browser, wasm32-unknown-unknown) | the same crates via wasm-bindgen / Dioxus | External compute deliberately unregistered (no TCP in the browser); built-in arithmetic still resolves in-engine |

All four wrap the same `nibli_session::CoreSession`, so they agree by
construction. The dispatch **hook** is per-KB-instance function pointers
(`KnowledgeBase::set_compute_dispatch`) — nibli-reason itself holds no
thread-locals or globals, which is what lets multithreaded embedders register
it at all. The native TCP client those pointers call into does live per-thread
in the embedder (`nibli-engine/src/compute_client.rs`), so a multithreaded
native embedder calls `enable_compute_backend` on each worker thread that
reasons.

## nibli-host mechanics

`just run` builds the component and launches the host
(`NIBLI_WASM_PATH=target/wasm32-wasip2/debug/nibli.wasm` by default).

**Environment variables** (read at startup):

| Var | Default | Effect |
|-----|---------|--------|
| `NIBLI_WASM_PATH` | `target/wasm32-wasip2/debug/nibli.wasm` | Component location |
| `NIBLI_FUEL` | `50_000_000_000` | Wasmtime fuel budget **per command** (debug WASM is ~6× hungrier than release; `~1.5e11` covers the heaviest demo corpus on debug) |
| `NIBLI_MEMORY_MB` | `512` | Guest memory cap (`trap_on_grow_failure`) |
| `NIBLI_COMPUTE_ADDR` | unset | External backend `host:port`; unset = built-in arithmetic only |
| `NIBLI_DB_PATH` | unset | Optional persistent redb store (migrated + replayed fail-closed at startup) |
| `NIBLI_QUIET` | off | `=1` suppresses the `[Fact #N]` / `[Skolem]` / `[Rule]` bookkeeping echoes (forwarded into the guest's WASI env) |
| `NIBLI_STRICT` | off | `=1` makes arity/integrity violations reject atomically instead of warn-and-insert |
| `NIBLI_EXISTENTIAL_IMPORT` | off | `=1` opts into legacy xorlo witnesses; imported witnesses participate in ∃/∀/find/count/aggregate |
| `NIBLI_MATERIALIZE` | on | `=0` opts out of NAF saturation, sending every NAF check back through backward chaining |

Runtime toggles: `:fuel [n]`, `:memory [mb]`, `:backend [addr]`,
`:strict on|off`, `:existential-import on|off`, `:materialize on|off` (bare
`:materialize` prints the saturation report). Script mode (`--script <file>`
or piped stdin) captures transcripts byte-faithfully.

The host prints the effective existential-import profile at startup and after a
toggle. Changing it is a fallible transactional replay of the active assertion
registry, so existing universals gain or lose imported witnesses immediately;
find/proof/count results carry structured import provenance. The browser UI
labels the same clean-core vs legacy-import profile.

**Resource traps don't brick the session.** Fuel exhaustion
(`wasmtime::Trap::OutOfFuel`) and memory-grow denials are classified and — for
queries — synthesized into a `RESOURCE_EXCEEDED (fuel|memory)` verdict with a
remediation hint. A trap poisons the component instance, so the host keeps a
**journal** of every successful KB mutation and lazily rebuilds a
byte-identical session on the next call (the engine is deterministic:
identical fact ids and Skolem numbering). Raising `:fuel` between trap and
re-query applies to the replay. Depth limits are engine-level, never a trap.

## The compute backend

An external process the reasoner can consult for computed predicates —
**JSON Lines over TCP**, one object per line:

```json
{"relation": "exponential", "args": [{"type": "number", "value": 8.0}, {"type": "number", "value": 2.0}, {"type": "number", "value": 3.0}]}
{"result": true}
```

Responses are `{"result": true|false}` or `{"error": "..."}`. Argument tags:
`variable`, `constant`, `description`, `unspecified`, `number`.

- **Built-in vs forwarded:** `product` / `sum` / `quotient` with fully
  numeric arguments evaluate locally (the shared
  `nibli_types::eval_arithmetic`); a call whose arguments don't resolve to
  numbers falls through to the backend — which is why the reference server
  implements all three too. Everything else registered via `:compute <name>`
  forwards.
- **Tolerant equality (disclosed):** arithmetic equality is `isclose` with
  `rel_tol 1e-9, abs_tol 0` — `0.3 = 0.1 + 0.2` is TRUE. The comparison
  predicate `num_equal` is exact `==`. Non-finite operands yield
  `UNKNOWN (non-finite)`.
- **Trust boundary (disclosed):** a backend `true` reply is **auto-asserted as
  a ground fact mid-query** which downstream rules chain on. The backend is
  part of the trusted computing base — a plaintext, unauthenticated peer; run
  it on localhost or a segment you control. Auto-asserted compute facts are
  non-durable — never journaled or replayed, recomputed on demand, and they do
  not survive a restart (the persistent engine's typed mirror is cleared and
  rebuilt from the fact registry on open).
- **No backend configured?** A registered external predicate answers
  `UNKNOWN (backend-unavailable)` — an outage is never a derived falsehood
  (pinned by the `smoke-host-backend-unavailable` gate).
- **Opaque generated argument?** Internal `Skolem`/`SkolemFn` identity has no
  lossless representation in this string-only protocol. Dispatch therefore
  fails closed as `UNKNOWN (backend-unavailable)` before the host callback runs.
  A user constant whose literal text is `sk_0` is not internal and is forwarded.
- **Client behavior:** lazy connect, reused connection (idle reap after 300 s,
  read timeout 10 s, write 5 s — `NIBLI_BACKEND_*` env overrides), retry-once
  on connection errors, and a batch path that pipelines all requests in one
  burst to amortize WASM-boundary and TCP round trips.

The reference server is `python/nibli_backend.py` (`just backend`, port 5555):
handlers `product`, `sum`, `quotient`, `exponential`, `logarithm` in a
`HANDLERS` dict — extend by adding an entry. `just run-with-backend` wires
host + backend together.

## Browser surfaces

No WASI, no component, no server: `nibli-wasm` (wasm-bindgen `Session` for JS;
powers the live demo) and `nibli-ui` (the Dioxus playground) compile the
engine crates straight into the browser bundle. `nibli-wasm` keeps two
deprecated no-op shims (`set_language`, `back_translate`) for deployed-site
compatibility until the site migration lands; the live back-translation is
IR-driven (`back_translate_ir`).

## Ship paths

| Target | What | How |
|--------|------|-----|
| `dhilipsiva.dev/nibli-playground` | nibli-ui bundle | Built by the external `dhilipsiva.dev` site repo; this repo pings it via `redeploy-site.yml` (`repository_dispatch: nibli-updated`) on every push to main — self-skips until the `SITE_DISPATCH_TOKEN` secret exists |
| `dhilipsiva.dev/nibli` | nibli-wasm live demo | Same site repo |
| `dhilipsiva.github.io/nibli/` | this docs site (mirror) | `docs-pages.yml`: `just docs /nibli/` → GitHub Pages, on any `mdbook/**` / `Justfile` / flake push |
| `dhilipsiva.dev/docs/nibli/` | this docs site (primary, pending) | Site repo copies the default `just docs` build (DEPLOY.md §2b) |

`just build-ui` produces the exact shipping bundle locally
(`target/dx/nibli-ui/release/web/public/`) as a pre-merge sanity check — the
production build runs in the site repo. Since the committed corpus, **no build
needs a dictionary fetch** — the full vocabulary is compiled in (the site
repo's leftover fetch step is obsolete and unread; see DEPLOY.md).
