# Adjudication layer — nibli above Soufflé / CodeQL

A worked, self-verifying example of nibli used as the **policy layer over an
external static analyzer**, not as the analyzer.

```
Soufflé / CodeQL          extract.py           policy.nibli
──────────────────  ──►  ───────────  ──►  ──────────────────  ──►  verdict
compute the flows        escape into KR      decide what they mean     + proof
(recursion lives here)   ground facts        (no recursion here)
```

Run it:

```
just verify-adjudication
```

## Why the seam is here

The split is forced by measurement, not taste. Measured 2026-09-03 on this
repo (release build, `nibli-pin`):

**Recursion is not viable.** A textbook transitive closure
(`earlier($a,$b) & earlier($b,$c) -> earlier($a,$c)`) over a linear chain:

| chain length | 5 | 8 | 10 | 12 | 15 |
|---|---|---|---|---|---|
| time | 0.21 s | 1.31 s | 4.82 s | 18.3 s | >60 s |

That is ~3.7x per two edges added. Materialisation does not rescue it:
saturation triggers on NAF-completeness need, and a positive query cone that
resolves definitively stays lazy (`NIBLI_MATERIALIZE=1` vs `=0` at n=12 measured
17.69 s vs 17.25 s). Even when saturation *is* forced, the derivation rate is
~100–200 tuples/s; Soufflé is 10^6–10^7.

**Non-recursive adjudication is comfortable — if you ask the right question.**
The same engine, on the shape this directory demonstrates, querying the *finding*
relation (see below):

| analyzer flows | facts | time | peak RSS |
|---|---|---|---|
| 100 | 256 | 0.10 s | 7 MB |
| 2,000 | 5,006 | 0.20 s | 54 MB |
| 10,000 | 25,006 | 1.10 s | 251 MB |
| 30,000 | 75,006 | 3.80 s | 764 MB |
| 60,000 | 150,006 | 8.31 s | 1.5 GB |

Linear in time, roughly 10 KB per fact — **memory binds before time does**.

## Ask for the finding, not the clearance

The two clearance rules are purely **positive**, and a purely positive query cone
that resolves definitively is never materialised (GUARANTEES § Completeness). So
`? authorized(F, Release, S).` returning FALSE — the case you care about, the
finding — is an exhaustive backward-chaining search, and it degrades badly:

| flows | `authorized(...)` FALSE | `warns(Gate, ...)` TRUE |
|---|---|---|
| 500 | 0.90 s | 0.10 s |
| 2,000 | 19.2 s | 0.20 s |
| 10,000 | — | 1.20 s |

Same knowledge base, same verdict, ~190x apart at 500 flows (`just bench-closure`
measures both legs). The finding rule reads clearance under `~`, which requests
the cone, saturates the whole `authorized` relation bottom-up, and answers from a
complete set:

```
all $f, $d, $s, $o, $r: carries($f, $d, $s, $o, $r) & ~authorized($f, Release, $s) -> warns(Gate, $f, $s).
```

**This does not weaken the fail-closed reading** — that is exactly why the rule is
stated over *clearance* rather than over evidence. A flow whose sink the analyzer
never classified is not cleared, so it warns. `flow:4` pins that in both forms.

This is the same trigger policy as the recursion cliff above, showing up with no
recursion at all. Run the finding query at scale; keep the clearance query for
single-flow explanations.

So the rule for any pipeline built this way: **never hand nibli a relation that
needs recursion to derive.** Reachability, def-use chains, points-to and taint
propagation are all closures — compute them upstream and pass the *closed*
relation in as ground facts.

## What the layer is for

Soufflé and CodeQL already decide these questions faster and at more scale than
nibli ever will. What they do not give you is a conclusion that a third party
can check without trusting the tool that produced it. This layer adds three
things, and nothing else:

**1. The analyzer cannot state a conclusion.** `derived_only("authorized")`
closes the verdict relation to derivation. Evidence goes in; verdicts come out
of the policy or not at all.

**2. The analyzer cannot widen the record.** `admits(...)` closes the base
vocabulary to the four relations the policy names. A new analyzer output
relation that nobody reviewed fails the load rather than quietly entering
the record. Ordering is enforced — the `admits` block must precede every
ordinary assertion, so load `policy.nibli` *before* `facts.nibli`.

**3. The verdict carries a checkable derivation.** `NibliEngine::certify_text`
returns a `ProofEnvelope` (verdict + trace + profile + lockstep engine version),
validated by the KB-independent `nibli_types::logic::validate_envelope`
(round-trip pinned in `nibli-engine/tests/integration.rs`).

The policy is also **fail-closed by construction**: a dangerous flow is cleared
only on positive evidence — a sanitizer on the path, or a recorded waiver. There
is deliberately no "not known to be dangerous" rule, because under the
closed-world assumption that would auto-clear every sink the analyzer failed to
classify. `flow:4` in `facts.nibli` pins that boundary.

## Analyzer identifiers are attacker-influenced

Columns coming out of the analyzer carry identifiers from the code under
analysis — file paths, variable names, string literals. In a security pipeline
whoever can name a file in the analyzed repository chooses those bytes.
Interpolating them into KR text would let a crafted name close the quoted term
and append statements of its own; `permits` is admitted vocabulary, so the
`admits` closure does **not** catch that.

`extract.py` therefore escapes backslash and double-quote, and *refuses* any
field carrying a control character rather than rewriting it. `flow:5` in
`facts.nibli` is a real injection attempt carried through the extractor, pinned
to stay a finding — if that pin ever flips, analyzer-controlled bytes are
executing as policy.

## Files

| File | Role |
|---|---|
| `policy.nibli` | The reviewed policy + the trust boundary. The artifact humans argue about. |
| `facts.nibli` | Generated analyzer output. Committed as a sample so the example runs with no analyzer installed. |
| `policy.pins.nibli` | The policy's regression suite — **content pins**, so the fixture is the live `policy.nibli` via `--kb`, never an inlined copy. |
| `extract.py` | Soufflé/CodeQL tabular output → escaped KR ground facts. |

## Wiring it to a real analyzer

```sh
# Soufflé — tab-separated by default
souffle -F facts -D out taint.dl
./extract.py --relation carries   out/TaintFlow.csv  >  facts.nibli
./extract.py --relation dangerous out/DangerSink.csv >> facts.nibli

# CodeQL
codeql query run --output=r.bqrs taint.ql
codeql bqrs decode --format=csv r.bqrs \
  | ./extract.py --relation carries --format csv --skip-header - >> facts.nibli
```

`extract.py` knows each admitted relation's arity and refuses to pad or truncate
a row that does not match, so a widened analyzer query is a loud failure rather
than a silently reshaped fact.

## Known limits

- **Vocabulary is borrowed.** `carries(carrier, cargo, destination, origin, route)`
  and `dangerous(danger, victim, conditions)` are corpus predicates whose place
  structures came from Lojban gismu; they fit this domain well, but the next
  relation you need may not exist or may read subtly wrong. Only 66 of the 1,356
  corpus entries are `Curated` tier with hand-verified places. User-authored
  `pred` declarations (NIBLI_KR §14.1) are a v2 feature and a compile error today
  — this is the single change that would most help this use case.
- **Enumeration is not shown here.** The pin language has no find/count form, so
  "list every flow that warns" needs `NibliEngine::query_find_text` from a native
  embedding. The scaling tables above measure ground queries.
- **Which query you write is a performance decision**, and the engine gives no
  warning when you write the slow one. See "Ask for the finding" above.
- **Files are line-oriented.** A rule wrapped across two lines is a syntax error
  in `:load` and in `--kb` fixtures.
