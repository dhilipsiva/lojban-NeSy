# GDPR walkthrough

A worked compliance knowledge base: a formalizable slice of the EU General Data
Protection Regulation (Articles 5, 6, 7, 9, 15, 17, 33), answered by deterministic,
auditable deduction.

**Source of truth:** [`gdpr.nibli`](https://github.com/dhilipsiva/nibli/blob/main/gdpr.nibli)
at the repo root. Every verdict quoted below is pinned by the `gdpr_*` regression
tests in `nibli-engine/tests/integration.rs`, so this page cannot silently drift
from the engine. The same corpus ships as the playground preset
**“GDPR compliance (Ch 19)”** — an example KB, not a chapter of any third-party book.

## The scenario

| Entity | Role |
|--------|------|
| `Adam` | Data subject; has given consent |
| `Akmes` | Controller that suffered a breach |
| `Gugli` | Controller with no breach (clean control) |
| `Kanrek` | Adam's health record (special-category data, Art 9) |
| `Ordrek` | An ordinary personal-data record |

Second-order legal concepts are mapped onto proxy predicates, disclosed in the
corpus header:

| Predicate | Reads as |
|-----------|----------|
| `permitted(x)` | processing of `x` has a lawful basis (Art 6) |
| `~permitted(x)` | no lawful basis remains → right to erasure (Art 17) |
| `obliged(x, event { P() })` | `x` is under a legal obligation that `P` |
| `permitted(x, event { P() })` | `x` has the right that `P` |

## Load it in the REPL

```text
:load gdpr.nibli
[Load] Done: 24 asserted, 77 skipped, 0 errors
```

The 77 skipped lines are comments and blanks. Fact ids are assigned in file
order; the consent fact `approves(Adam).` lands as **fact #21** — the corpus
deliberately defines the Article 17 rule *after* the scenario so these ids stay
stable (a rule's position in the file never affects reasoning).

## Engine-checked queries

Query by stating a claim with the `?` prefix. All verdicts below are asserted by
the regression suite against the full loaded corpus:

| Claim | Verdict | Why |
|-------|---------|-----|
| `? permitted(Adam).` | **TRUE** | Consent is a lawful basis (Art 6(1)(a)) |
| `? ~permitted(Adam).` | **FALSE** | A lawful basis stands, so no erasure right |
| `? permitted(Gugli).` | **FALSE** | A controller is not a consenting subject — an exhaustive, deduced FALSE |
| `? data(Kanrek).` | **TRUE** | Health record → personal data, derived via `data(every healthy data).` |
| `? obliged(Kanrek, event { correct() }).` | **TRUE** | Art 5 accuracy reaches health data through the category chain |
| `? obliged(Kanrek, event { exact() }).` | **TRUE** | Special-category data needs a stricter basis (Art 9) |
| `? obliged(Ordrek, event { exact() }).` | **FALSE** | Ordinary data does not |
| `? permitted(Adam, event { data discovers() }).` | **TRUE** | Right of access / DSAR (Art 15) |
| `? permitted(Akmes, event { data discovers() }).` | **FALSE** | A controller does not acquire the subject's access right |
| `? obliged(Akmes, event { message() }).` | **TRUE** | Breached controller must notify (Art 33) |
| `? obliged(Gugli, event { message() }).` | **FALSE** | No breach, no notification duty |
| `? obliged(Adam, event { removes() }).` | **FALSE** | Consent present → no erasure obligation (Art 17 rule) |

Every FALSE here is a **deduced** false under the closed-world assumption — the
engine exhausted the search — not a shrug. `?` also prints a plain-English
`[Why]` summary and the proof tree; see [What Nibli guarantees](guarantees.md)
for the verdict contract.

## Belief revision: withdrawing consent

The headline demo. Adam's only lawful basis is consent (fact #21). Retract it
and re-query — both verdicts flip:

```text
:retract 21
[Retract] Fact #21 retracted. KB rebuilt.

? permitted(Adam).
[Query] FALSE

? ~permitted(Adam).
[Query] TRUE
```

No lawful basis remains, so the right to erasure (Art 17(1)(b)) arises. The
erasure verdict is derived by negation-as-failure and its proof carries the
`naf_dependent` flag — the engine discloses that the conclusion rests on an
absence. Nothing was edited by hand: the same rules, re-derived over the
surviving facts. See [Belief revision](belief-revision.md) for the mechanics.

The corpus also stores Article 17 as a rule with a negated restrictor:

```nibli-kr
obliged(every person where ~approves, event { removes() }).
```

`where ~approves` compiles to a negation-as-failure check *per subject*: a
consenting person carries no erasure obligation while a non-consenting one does
(pinned by `gdpr_erasure_rule_is_per_subject` and the rule-level
belief-revision test).

## Honest boundaries

Two scope decisions the corpus makes explicitly, in its own comments:

- **Art 7 (“freely given” consent) is deliberately not encoded.** Whether a
  consent was un-coerced is a case-by-case human judgment — it stays outside
  the deductive firewall rather than being faked as a rule.
- **The erasure rule keys on `~approves`, not `~permitted`.** The corpus derives
  a lawful basis *from* a legal obligation, so a `where ~permitted` rule would
  close a negative basis↔obligation cycle — the engine correctly rejects that
  as unstratifiable at assert time. For Adam, whose only basis is consent, the
  two formulations coincide, which is why the right can equivalently be queried
  as `~permitted(Adam).`

## Try it in the playground

Select **“GDPR compliance (Ch 19)”** in the [playground](playground.md) header
dropdown. Its preset queries are exactly the first four claims above:
*lawful basis? (Art 6)* · *right to erasure? (Art 17)* · *a controller is not a
consenting person—exhaustive FALSE* · *health record → personal data (Art 4/9,
derived)*. Proofs render with the curated legal-domain overlay (“has a lawful
basis for processing”), never a bare variable.
