# nibli-formalize

The **agentic English→KB formalizer engine** for the Transparency Triad
(`fanva` = Lojban "translate" — the crate name predates THE FLIP). An LLM
formalizes English into **nibli KR**; the real Nibli compiler verifies every
candidate and feeds deterministic errors back until the KB text is valid.
Surfaced inside `nibli-ui` as **Formalize** mode (this crate holds no UI).
"Formalize", never "compile": the LLM step is interpretive and sits outside
the reasoning firewall, behind the deterministic gates below.

## The loop

Every candidate must clear a four-gate, fail-fast, local firewall:

1. `nibli_kr::parse_checked` — grammar and fail-closed name resolution.
2. `nibli_semantics::compile_from_ast` — semantics and arity.
3. The render round-trip gate — the canonical `nibli_kr::render` re-spelling
   must recompile to the same `LogicBuffer` (pure Rust, native and WASM).
4. The KB-assertability gate — query-only `exactly N` / `no` formulas in
   asserted position are rejected instead of being presented as persistent
   cardinality constraints. Opaque `fact { ... }` / `event { ... }` content
   remains quoted. If an exact source claim cannot be represented by explicit,
   source-supported ordinary facts, it stays an explicit unsupported failure;
   it is never silently omitted, weakened, or fabricated around.

A rejection feeds the gate's own message back through `gates::feedback_for`.
The LLM retries up to `max_attempts`, with an oscillation guard. A gate-clean
candidate then faces the semantic verification turn (`verify.rs`): a fresh
context judge reads the engine's IR-level back-translation of each KB line.
A mismatch retries through the same loop; verifier transport or parse failure
is best-effort/fail-open. This runs before the KB text is shown and is separate
from the engine's later nibli-kr → nibli-semantics → nibli-reason execution.

```mermaid
flowchart TD
    src(["English source"]) --> loop{"attempt n ≤ max_attempts?"}
    loop -->|"no · cap reached"| exh["Exhausted<br/>best effort + last error"]
    loop -->|yes| gen["LLM proposes candidate KB text"]
    gen --> clean["clean_output"]
    clean -->|"per non-comment KB line"| g1{"gate 1 · nibli-kr<br/>grammar + names"}
    g1 -->|ok| g2{"gate 2 · nibli-semantics<br/>semantics + arity"}
    g2 -->|ok| g3{"gate 3 · render round-trip<br/>same LogicBuffer"}
    g3 -->|ok| g4{"gate 4 · KB assertability<br/>reject outer exactly N / no"}
    g4 -->|ok| ver{"semantic verification turn<br/>IR back-translation (advisory)"}
    ver -->|"MATCH / fail-open"| ok["Success<br/>validated KB text → KB tab"]

    g1 -->|reject| osc
    g2 -->|reject| osc
    g3 -->|reject| osc
    g4 -->|reject| osc
    ver -->|MISMATCH| osc
    osc{"same candidate<br/>as previous attempt?"} -->|"yes · oscillation"| exh
    osc -->|"no · append feedback_for"| loop
    gen -.->|"provider / network / auth error"| cf["ChatFailed<br/>transport error"]

    classDef good fill:#1a7f37,stroke:#116329,color:#fff;
    classDef warn fill:#9a6700,stroke:#7d4e00,color:#fff;
    classDef bad fill:#cf222e,stroke:#a40e26,color:#fff;
    class ok good;
    class exh warn;
    class cf bad;
```

The compiler and round-trip portion is `gates::local_gates`; the full
assertion-authoring contract is `gates::validate` / `gates::validate_kb`.

## Test discipline

- Native `cargo test -p nibli-formalize --lib` (`just test-formalize`) covers
  local gates, the exact-count assertion guard, provider/agent behavior,
  bounded history, prompt grounding, and the semantic verification turn with
  mocked `chat()`.
- The shipped system prompt's examples and prose snippets are compiled by
  guard tests so prompting guidance cannot silently drift from the grammar.
