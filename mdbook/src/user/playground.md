# Playground

The Transparency Triad UI runs the full pipeline (**nibli-kr → nibli-semantics → nibli-reason**) **in the browser**. There is no nibli server.

<p><a href="https://dhilipsiva.dev/nibli-playground/"><strong>Open playground →</strong></a></p>

Local equivalent: `just ui` (Dioxus on port 8080). Ship path: [DEPLOY.md](https://github.com/dhilipsiva/nibli/blob/main/DEPLOY.md).

## Panes

| Pane | Role |
|------|------|
| **Source** | Plain English (optional Formalize input) |
| **nibli KR** | The knowledge base — formal claims the engine asserts |
| **Back-translation** | Structure-exposing gloss of the KR |

The **nibli KR pane is the knowledge base**. Each query rebuilds a fresh engine, re-asserts the KB, then runs the claim.

## Query model

**State a claim**, do not ask a question:

```nibli-kr
eats(Adam).
```

The UI may show a decorative `?` next to the query box — it is **not** part of the text sent to the engine. Verdicts are `TRUE` / `FALSE` / `UNKNOWN` (see [guarantees](guarantees.md)).

Exact-count forms (`exactly N` and `no`) belong here in the query box, not in
the KR knowledge-base pane. They report the current KB snapshot; the engine has
no persistent cardinality-constraint assertion.

## Formalize (optional)

**Formalize** (not “compile”) is a bring-your-own-key LLM step from the Source tab. The key stays in tab memory only; the request goes from your browser to the provider you choose. Drafts are checked by the real nibli-kr + nibli-semantics + render round-trip gates, plus a KB-assertability guard that rejects query-only exact counts, before they land in the KR pane. Formalize sits **outside** the deterministic reasoning core — always review the KR and back-translation.

## Example knowledge bases (preset hooks)

The header dropdown loads preloaded KBs used in regression tests and demos. Treat them as **example corpora**, not as chapters of any third-party book. In example mode the KR is read-only and Formalize is disabled; the query control becomes a preset list that auto-runs.

The dropdown names and preset labels below are **byte-stable hooks** — they are defined in `nibli-ui/src/examples.rs`, pinned by the `shipped_examples_compile` guard (`just test-ui`), and safe to reference from docs and links:

| Dropdown name | Corpus | Preset queries |
|---------------|--------|----------------|
| **Syllogism (Ch 18)** | inline 3-line KB | *does Adam eat?—a 2-hop proof* · *is Adam an animal?—1 hop* · *is Adam a bird?—a real FALSE* |
| **GDPR compliance (Ch 19)** | [`gdpr.nibli`](https://github.com/dhilipsiva/nibli/blob/main/gdpr.nibli) — see the [GDPR walkthrough](gdpr-walkthrough.md) | *lawful basis? (Art 6)* · *right to erasure? (Art 17)* · *a controller is not a consenting person—exhaustive FALSE* · *health record → personal data (Art 4/9, derived)* |
| **Constitutional core (utopia)** | [`utopia.nibli`](https://github.com/dhilipsiva/nibli/blob/main/utopia.nibli) — extra playground corpus, not a book chapter | 14 presets over the constitutional scenario (floor duties, voiding multi-sig, imprisonment routing, whistleblower shield) |
| **Drug interactions (Ch 20)** | [`drug-interactions.nibli`](https://github.com/dhilipsiva/nibli/blob/main/drug-interactions.nibli) — see the [drug-interactions walkthrough](ddi-walkthrough.md) | *concentration rising?* · *toxicity risk?* · *safety alert?—a 3-hop proof* · *negative control—no alert* |

The GDPR, utopia, and drug KBs are `include_str!`-ed from the same repo-root `.nibli` files the engine's regression tests pin, so the playground cannot drift from the tested corpora.

## Belief revision: edit and re-query

Each query rebuilds a fresh engine from the KR pane, so revising the KB is just editing it: delete or `#`-comment a fact line and re-run the claim. Presets are read-only — to revise one, paste its corpus into **Custom** mode first. Worked demos: [Belief revision](belief-revision.md).

## Built-in vs external compute

In-browser: built-in arithmetic (`product` / `sum` / `quotient`) and ground numeric comparisons work locally. External compute backend predicates need the host + backend path (`just run-with-backend`) — not the pure playground.

## More

- [Quickstart](quickstart.md) for the CLI REPL
- [nibli KR cookbook](kr-cookbook.md) for syntax
- Product UI notes in the [README](https://github.com/dhilipsiva/nibli/blob/main/README.md)
