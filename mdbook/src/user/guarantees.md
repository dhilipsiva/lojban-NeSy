# What Nibli guarantees

Derived from the engine [README](https://github.com/dhilipsiva/nibli/blob/main/README.md) and [GUARANTEES.md](https://github.com/dhilipsiva/nibli/blob/main/GUARANTEES.md). The full contract text lives in those files; this page is a short operator summary.

## One surface language

Nibli has **one** front-end language: **nibli KR** (predicate-call surface: `dog(Adam).`, `animal(every dog).`). Name resolution is **fail-closed**: an unknown word is a compile error, never an arity-guessed new predicate. Normative spec: [NIBLI_KR.md](https://github.com/dhilipsiva/nibli/blob/main/NIBLI_KR.md).

## Soundness (relative to what you asserted)

The engine never returns **TRUE** for a formula that does not follow from the asserted facts and compiled rules, **given a correct implementation**. A TRUE answer comes with a formal proof trace. Bugs would be deterministic and testable — not stochastic fabrication.

This is **not** omniscience: change the premises and the verdict can change.

Ordinary-predicate temporal rule literals are **flavor-exact**. Bare rules are
unqualified and do not silently lift into `past`, `now`, or `future`; authors
declare same- or cross-flavor mappings explicitly. Negation-as-failure follows the same rule:
bare `~P` checks bare `P`, while `past ~P` checks only for a Past witness.
Stratification remains conservatively keyed by surface relation, so temporal
flavors cannot be used to evade a negative-cycle rejection. Identity and
query-time compute built-ins retain their separately documented semantics.
One formula path may carry one temporal prefix or one deontic prefix, never
both. `must past P` and `past must P` are compile errors rather than formulas
whose outer wrapper could be lost in the one-flavor fact/rule store. The same
fail-closed rule applies to manually nested raw IR before assertion or query.
It is path-scoped, so separate rule literals may use different flavors, as in
an explicitly declared `past P -> must Q` mapping.

## Closed world and closed domain

Inference assumes:

- **Closed world** — a fact you did not assert is taken to be *false*, not unknown.
- **Closed domain** — quantifiers range only over entities the knowledge base knows.

The default **clean-core** profile does not infer existence from a universal:
`animal(every dog).` alone makes no dog, and `some dog` remains plain ∃. Legacy
xorlo import is an explicit opt-in (`NIBLI_EXISTENTIAL_IMPORT=1` or
`:existential-import on`). When enabled, the imported witness is a full logical
entity: it participates consistently in ∃/∀/find/count/aggregate. Find and proof
metadata label it `existential-import`, count proofs disclose the imported share,
and the host/UI show which profile is active. Toggling the profile rebuilds the
current KB transactionally, so existing rules change immediately.

Ordinary reasoner-minted witnesses are labeled `generated-witness`; asserted user
constants remain `knowledge-base`, even when their display text resembles `sk_N`.
Internally, generated witnesses have source-scoped typed identities (source
assertion, binder ordinal, sort, and origin). Their friendly `sk_N` or
`sk_N(argument)` rendering is never semantic, so equal-looking user constants stay
distinct through equality, event joins, find/count, proof, persistence, retraction,
and rebuild. A compute call that would expose an internal witness to the
string-only backend protocol fails closed as `UNKNOWN (backend-unavailable)`; an
equal-looking user constant still dispatches normally.

## Exact counts are observations, not constraints

`exactly N` and `no` are query-only formulas over the current closed domain.
They count identity-equivalence classes and disclose any legacy
existential-import contribution in the proof. Assertion ingress rejects them
before allocating a fact id: Nibli does not persist or enforce cardinality
constraints, and `exactly 0` is never a stored prohibition. Assert ordinary
facts, then re-query after additions, equality changes, or retractions. Counts
inside opaque `fact { … }` or `event { … }` content remain quoted content and
do not constrain the outer KB.

## Four-valued outcomes

How to read a query result (product README wording):

| Verdict | Meaning |
|---------|---------|
| **TRUE** | A proof exists from your premises (facts + rules + trusted backend results). |
| **FALSE** | *Not derivable* from those premises. This is **not** a proof of ¬P. |
| **UNKNOWN** | The search could not decide (e.g. a cycle, incomplete knowledge, or negation over an undecided sub-goal). |
| **RESOURCE_EXCEEDED** | A budget ran out before the search finished — `depth`, `fuel`, or `memory`. Not a verdict about the claim: raise the budget and re-run. |

All four are `QueryResult` variants in the engine itself, not host conventions — `RESOURCE_EXCEEDED` carries which limit was hit. Raise them with the `NIBLI_FUEL` / `NIBLI_MEMORY_MB` env vars or the `:fuel` / `:memory` REPL commands; see [GUARANTEES.md](https://github.com/dhilipsiva/nibli/blob/main/GUARANTEES.md).

## Trusted compute backend

Results from the **external compute backend** (`exponential`, `logarithm`, or predicates you register) are a **trusted oracle**, not a derivation: a `true` reply is auto-asserted mid-query. Built-in arithmetic (`product` / `sum` / `quotient`) is local. Any conclusion that passes through the backend is only as sound as that oracle.

## Where the full story lives

- [GUARANTEES.md](https://github.com/dhilipsiva/nibli/blob/main/GUARANTEES.md) — differential oracles (Vampire / clingo), Lean proofs, determinism, mutation baseline.
- [LOGIC_IR.md](https://github.com/dhilipsiva/nibli/blob/main/LOGIC_IR.md) — the FOL intermediate form the reasoner consumes.
- CI: `just ci`, `just verify-soundness`, `just verify-proofs`.
