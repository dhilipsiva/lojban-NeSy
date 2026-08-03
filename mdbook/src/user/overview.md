# User guide — overview

*Audience: people writing `.nibli` knowledge bases, using the REPL or playground, or embedding the engine.*

| Page | What it covers |
|------|----------------|
| [What Nibli guarantees](guarantees.md) | Four-valued verdicts, closed world/domain, trusted compute |
| [Quickstart](quickstart.md) | Nix dev shell, `just run`, first claims |
| [nibli KR cookbook](kr-cookbook.md) | Surface syntax stubs + link to the full spec |
| [Playground](playground.md) | Hosted triad UI and Formalize |
| [GDPR walkthrough](gdpr-walkthrough.md) | A worked compliance KB — engine-checked verdicts and the consent-withdrawal flip |
| [Drug-interactions walkthrough](ddi-walkthrough.md) | A worked safety KB — the three-step mechanism and its negative controls |
| [Belief revision](belief-revision.md) | `:retract`, retract ≡ never-asserted, and edit-and-re-query in the playground |
| [Authorization](authorization.md) | Builtin policy, `can` / fields / explain, Rust + Python adapters |

## Deeper sources (repo root)

| Topic | Where |
|-------|--------|
| Product overview | [README.md](https://github.com/dhilipsiva/nibli/blob/main/README.md) |
| Formal contracts | [GUARANTEES.md](https://github.com/dhilipsiva/nibli/blob/main/GUARANTEES.md) |
| Language (normative) | [NIBLI_KR.md](https://github.com/dhilipsiva/nibli/blob/main/NIBLI_KR.md) |
| Example corpora | `gdpr.nibli`, `drug-interactions.nibli`, `readme.nibli` |
| Host / WASM ship path | [DEPLOY.md](https://github.com/dhilipsiva/nibli/blob/main/DEPLOY.md) |

**Query model:** state a claim to check for entailment (e.g. `dog(Adam).`), not an interrogative. The playground’s decorative `?` is not sent to the engine.

The [developer guide](../developer/overview.md) covers the crate map, the IR, the WASM/host/compute path, the CI gates, and the WIT surface; published crate APIs are in the [API index](../api-index.md). Remaining docs work is tracked in [TODO.md](https://github.com/dhilipsiva/nibli/blob/main/TODO.md).
