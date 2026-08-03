# Drug-interactions walkthrough

A worked safety knowledge base: pharmacokinetic drug-drug interaction (DDI)
reasoning — “should this co-prescription raise a safety alert?” — decided by
deduction over an explicit mechanism instead of a statistical guess.

**Source of truth:** [`drug-interactions.nibli`](https://github.com/dhilipsiva/nibli/blob/main/drug-interactions.nibli)
at the repo root. Every verdict quoted below is pinned by the `ddi_*` regression
tests in `nibli-engine/tests/integration.rs`. The same corpus ships as the
playground preset **“Drug interactions (Ch 20)”** — an example KB, not a chapter
of any third-party book.

## The scenario

The warfarin + fluconazole interaction, mediated by the CYP2C9 enzyme:
fluconazole inhibits CYP2C9; warfarin (narrow therapeutic index) is metabolised
by CYP2C9, so its concentration rises → toxicity risk → safety alert.
**Apixaban** is the negative control: metabolised by CYP3A4, which fluconazole
does not inhibit → no alert, as a real deduced FALSE. **Phenytoin** is the
second control: pharmacologically at risk, but not on the patient's chart.

The corpus has no native pharmacology vocabulary, so it maps onto the nearest
committed relations and discloses the mapping in its header:

| Predicate | Reads as |
|-----------|----------|
| `chemical(d)` | `d` is a drug |
| `uses(p, d)` | patient `p` takes drug `d` |
| `prevents(d, e)` | drug `d` inhibits enzyme `e` |
| `metabolized_by(d, e)` | drug `d` is metabolised by enzyme `e` |
| `thin(d)` | narrow therapeutic index |
| `increases(d)` | blood concentration is raised |
| `dangerous(d)` | at toxicity risk |
| `warns(d)` | warrants a safety alert |

Enzymes are opaque rigid Names: `Siptucin` = CYP2C9, `Sipcivon` = CYP3A4.

## The three-step mechanism

**Step 1 — concentration rise.** Grounded conditionals per affected substrate
(the fully general join rule `all $a, $b, $e: prevents($a,$e) &
metabolized_by($b,$e) -> increases($b).` also compiles and reasons correctly —
the grounded form is an encoding choice, not a limitation):

```nibli-kr
prevents(Flukonazol, Siptucin) & metabolized_by(Varfarin, Siptucin) -> increases(Varfarin).
```

**Step 2 — toxicity risk.** One general rule with a *conjunctive* restrictor;
both conditions are required:

```nibli-kr
dangerous(every chemical where increases where thin).
```

A wide-margin drug whose concentration rises is not flagged, and a narrow-index
drug with no interaction is not flagged (both negative controls are pinned by
`ddi_toxicity_requires_both_conditions`).

**Step 3 — the alert is patient-gated.** Risk is drug-level pharmacology; the
actionable alert only fires for an at-risk drug this patient actually takes:

```nibli-kr
all $da: dangerous($da) & uses(Adam, $da) -> warns($da).
```

## Load it in the REPL

```text
:load drug-interactions.nibli
[Load] Done: 16 asserted, 78 skipped, 0 errors
```

Two fact ids matter for the belief-revision demos below (assigned in file
order, pinned by `ddi_corpus_transcript_pins`): the inhibition fact
`prevents(Flukonazol, Siptucin).` is **#4**, and the regimen fact
`uses(Adam, Varfarin).` is **#10**.

## Engine-checked queries

| Claim | Verdict | Why |
|-------|---------|-----|
| `? increases(Varfarin).` | **TRUE** | Step 1: inhibited enzyme + substrate |
| `? dangerous(Varfarin).` | **TRUE** | Step 2: raised concentration + narrow index |
| `? warns(Varfarin).` | **TRUE** | Step 3: at risk *and* on Adam's chart — a 3-hop proof |
| `? increases(Apiksaban).` | **FALSE** | CYP3A4 is not inhibited by fluconazole |
| `? warns(Apiksaban).` | **FALSE** | The negative control: a deduced FALSE, not unknown |
| `? increases(Fenitoin).` | **TRUE** | Same shared inhibitor, same general rules |
| `? dangerous(Fenitoin).` | **TRUE** | Risk is drug-level — no per-drug rule needed |
| `? warns(Fenitoin).` | **FALSE** | But Adam does not take it: the regimen gate |

The phenytoin pair is the point of step 3: pharmacological risk is general,
the actionable alert is patient-specific.

Witness extraction (`??`) enumerates bindings instead of checking one claim —
“which drugs are CYP2C9 substrates?”:

```text
?? metabolized_by($da, Siptucin).
```

lists warfarin and phenytoin as witnesses for `$da`; apixaban (a CYP3A4
substrate) does not appear (pinned by `ddi_witness_cyp2c9_substrates`).

## Belief revision: two clinical moves

Alerts are never baked in — they are re-derived from current facts, so the two
canonical chart edits are single retractions
(see [Belief revision](belief-revision.md)):

**Discontinue the inhibitor** (retract `prevents(Flukonazol, Siptucin).`, #4):
the mechanism's entry premise disappears, so the concentration rise, the
toxicity risk, and the alert all dissolve in one step — for *both* substrates,
since they share the inhibitor:

```text
:retract 4
[Retract] Fact #4 retracted. KB rebuilt.

? warns(Varfarin).
[Query] FALSE

? dangerous(Fenitoin).
[Query] FALSE
```

**Discontinue the drug** (retract `uses(Adam, Varfarin).`, #10): the alert is
withdrawn while the drug-level risk stays derivable — the alert is gated on the
regimen, the risk is not:

```text
:retract 10
[Retract] Fact #10 retracted. KB rebuilt.

? dangerous(Varfarin).
[Query] TRUE

? warns(Varfarin).
[Query] FALSE
```

Both moves are pinned by `ddi_belief_revision_discontinue_inhibitor` and
`ddi_belief_revision_discontinue_drug`.

## Try it in the playground

Select **“Drug interactions (Ch 20)”** in the [playground](playground.md)
header dropdown. Its presets are the headline chain plus the negative control:
*concentration rising?* · *toxicity risk?* · *safety alert?—a 3-hop proof* ·
*negative control—no alert*. Proofs render with the curated pharmacology
overlay (“fluconazole inhibits CYP2C9”, “warfarin is at toxicity risk”), never
a bare variable or a raw transliterated name.
