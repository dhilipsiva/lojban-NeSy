//! Materialisation metamorphic differential: **saturated ≡ backward-chained**.
//!
//! Stratum-ordered materialisation (`nibli_reason::materialize`) answers `~p(x)` by
//! looking `p`'s tuple up in a bottom-up saturated extension instead of trying to prove
//! `p(x)` and failing. That is an OPTIMISATION with exactly one unsound failure mode: if
//! a saturation UNDER-derives, the missing tuple reads as "not derivable" and the NAF
//! flips from FALSE to a wrong TRUE. Nothing about that failure is loud — the verdict
//! stays definitive, the proof trace stays well-formed, and the query gets faster.
//!
//! So this harness runs every generated program TWICE on fresh engines — materialisation
//! ON and OFF — over an entity×predicate battery, and demands:
//!
//! * the two verdicts are EQUAL, or
//! * the OFF verdict was non-definitive (`Unknown`/`ResourceExceeded`) and the ON verdict
//!   is definitive.
//!
//! The second arm is the deliberate completeness gain: a saturated extension is complete
//! regardless of `max_chain_depth`, so a NAF whose positive needs a chain past the bound
//! now decides instead of returning `ResourceExceeded(Depth)`. Anything else — a
//! definitive verdict changing, or ON becoming LESS definitive than OFF — fails.
//!
//! In practice the generated chains are short enough that the gain arm rarely fires here
//! (the run reports how many verdicts it covered); it is pinned directly by
//! `naf_over_a_chain_past_the_depth_bound_becomes_definitive` in nibli-reason. What this
//! gate is really for is the FIRST arm, across thousands of verdicts.
//!
//! The generator deliberately mixes in the shapes materialisation must REFUSE rather
//! than approximate (`du` identity links, `past`-flavoured facts, recursion through a
//! positive cycle), because a refusal that silently became an approximation is exactly
//! the regression this gate exists to catch. `NIBLI_VERIFY_MATERIALIZE_RANDOM_COUNT`
//! sets the batch size. Native-only, never skips — no external solver involved.

use crate::generator::Lcg;

/// Base UNARY predicates: facts and negated restrictors only, never rule heads.
const UNARY_BASE: &[&str] = &["dog", "cat", "rotten", "broken"];
/// Derived UNARY predicates: rule heads. Index-ordered, so positive edges among them
/// descend and the accepted rule set stays acyclic-positive unless recursion is added
/// deliberately below.
const UNARY_DERIVED: &[&str] = &["animal", "alive", "beautiful", "big", "fit", "reward"];
/// Base BINARY predicates: facts and negated restrictors only.
const BINARY_BASE: &[&str] = &["parent", "teaches"];
/// Derived BINARY predicate: the recursion head.
const BINARY_DERIVED: &str = "judge";
/// The domain every entity is asserted into, so a universal has somewhere to range.
const DOMAIN: &str = "person";

const ENTS: &[&str] = &["Adam", "Bel", "Cyd"];

/// Backward-chaining depth both sides run at. See `run_side` for why it is not the
/// engine default.
const DIFF_CHAIN_DEPTH: usize = 3;

/// A generated program plus the battery to run against it.
pub struct MatCase {
    pub name: String,
    pub kb: Vec<String>,
    pub queries: Vec<String>,
}

/// Generate the case for `seed`. Deterministic: the same seed always yields the same
/// program, so a failure is reproducible from the reported name alone.
pub fn random_materialize_case(seed: u64) -> MatCase {
    let mut rng = Lcg::new(seed);
    let mut kb: Vec<String> = Vec::new();

    // ── Facts ──
    for e in ENTS {
        kb.push(format!("{DOMAIN}({e})."));
    }
    let n_unary_facts = 1 + rng.below(4);
    for _ in 0..n_unary_facts {
        kb.push(format!("{}({}).", rng.pick(UNARY_BASE), rng.pick(ENTS)));
    }
    // Binary base facts are DAG-oriented (strictly ascending entity index) — the same
    // discipline `strat_diff`/`retract_diff` apply to rule edges, and for the same
    // reason. A self-loop `parent(Cyd, Cyd)` or a 2-cycle, combined with the transitive
    // closure below, sends the OFF side regressing to the depth horizon at every
    // deepening level: measured 13.5 s for ONE such program in a debug build. That is a
    // fair picture of what materialisation fixes, but it is not a cost worth paying 150
    // times in a gate. Orienting the edges keeps the closure non-trivial (A→B, B→C ⟹
    // A→C) while removing the regress.
    let n_binary_facts = 1 + rng.below(4);
    for _ in 0..n_binary_facts {
        let i = rng.below(ENTS.len() - 1);
        let j = i + 1 + rng.below(ENTS.len() - i - 1);
        kb.push(format!(
            "{}({}, {}).",
            rng.pick(BINARY_BASE),
            ENTS[i],
            ENTS[j]
        ));
    }

    // ── Rules ──
    // (1) The plain unary NAF rule — the shape the whole optimisation is named for.
    let r = rng.pick(UNARY_BASE);
    let q_idx = rng.below(UNARY_DERIVED.len());
    kb.push(format!(
        "all $x: {DOMAIN}($x) & ~{r}($x) -> {}($x).",
        UNARY_DERIVED[q_idx]
    ));

    // (2) Optionally a positive chain onward from it. Strictly descending index, so
    //     these edges cannot close a positive cycle.
    if q_idx + 1 < UNARY_DERIVED.len() && rng.below(2) == 0 {
        let s_idx = q_idx + 1 + rng.below(UNARY_DERIVED.len() - q_idx - 1);
        kb.push(format!(
            "all $x: {}($x) -> {}($x).",
            UNARY_DERIVED[q_idx], UNARY_DERIVED[s_idx]
        ));
    }

    // (3) Optionally the utopia shape: a multi-variable rule with a `~($a = $b)`
    //     built-in cutting the diagonal out of a self-join. This is the case that
    //     forced `equals` to be treated as a decidable built-in rather than an
    //     unmaterialisable relation.
    if rng.below(2) == 0 {
        let b = rng.pick(BINARY_BASE);
        let head = rng.pick(UNARY_DERIVED);
        kb.push(format!(
            "all $a: all $b: all $x: {b}($a, $x) & {b}($b, $x) & ~({} = {}) -> {head}($x).",
            "$a", "$b"
        ));
    }

    // (4) Optionally recursion through a POSITIVE cycle — transitive closure. A
    //     saturation that stopped after one round instead of running to a fixpoint
    //     would under-derive this, so a NAF over it would wrongly say TRUE.
    let mut recursive = false;
    if rng.below(2) == 0 {
        let b = rng.pick(BINARY_BASE);
        kb.push(format!(
            "all $x: all $y: {b}($x, $y) -> {BINARY_DERIVED}($x, $y)."
        ));
        kb.push(format!(
            "all $x: all $y: all $z: {BINARY_DERIVED}($x, $y) & {b}($y, $z) -> {BINARY_DERIVED}($x, $z)."
        ));
        recursive = true;
    }

    // (5) A `du` identity link. Materialisation must REFUSE the whole KB here: fact
    //     lookup is modulo union-find, and set membership on a projected tuple would
    //     miss an equivalent variant.
    //
    //     NEVER combined with the recursion above. A `du` link sends BOTH sides down the
    //     backward-chaining path (materialisation refuses the KB outright), and each
    //     equality-variant probe is a full derivation — up to `DU_VARIANT_BOUND` = 256 of
    //     them, each re-entering the transitive closure. Measured: one such program did
    //     not finish in eight minutes in a debug build. It also tests nothing extra —
    //     what the product would exercise is "materialisation fell back", which the
    //     non-recursive `du` cases already cover completely.
    if !recursive && rng.below(3) == 0 {
        kb.push(format!("{} = {}.", ENTS[0], ENTS[1]));
    }

    // (6) A tense-flavoured fact. Rule firing is flavour-polymorphic; the projection
    //     does not model that, so the relation must be refused rather than saturated
    //     with the flavour dropped.
    if rng.below(4) == 0 {
        kb.push(format!(
            "past {}({}).",
            rng.pick(UNARY_BASE),
            rng.pick(ENTS)
        ));
    }

    // ── Battery ──
    let mut queries: Vec<String> = Vec::new();
    for e in ENTS {
        for p in UNARY_BASE.iter().chain(UNARY_DERIVED.iter()) {
            queries.push(format!("{p}({e})."));
        }
    }
    if recursive {
        for a in ENTS {
            for b in ENTS {
                queries.push(format!("{BINARY_DERIVED}({a}, {b})."));
            }
        }
    }

    MatCase {
        name: format!("mat_seed{seed}"),
        kb,
        queries,
    }
}

/// What one case's ON/OFF comparison found.
#[derive(Debug)]
pub enum MatOutcome {
    /// Every battery query agreed (or improved definiteness). Carries the number of
    /// queries checked and how many became definitive under materialisation.
    Agree {
        name: String,
        checked: usize,
        gained: usize,
    },
    /// A DEFINITIVE verdict changed — the unsound direction.
    Diverge {
        name: String,
        query: String,
        on: String,
        off: String,
    },
    /// Materialisation made a verdict LESS definitive. Not unsound, but it means the
    /// saturation is silently losing information and must be investigated.
    LessDefinite {
        name: String,
        query: String,
        on: String,
        off: String,
    },
    /// Harness noise: a line failed to assert on one side but not the other, or a query
    /// errored. Reported separately so it can never be mistaken for a soundness finding.
    Error { name: String, detail: String },
}

/// Load `case.kb` into a fresh engine with materialisation set to `on`, then answer the
/// battery. Returns one `Debug`-formatted verdict per query, or the first error.
fn run_side(case: &MatCase, on: bool) -> Result<Vec<String>, String> {
    let engine = crate::kr_engine();
    engine.set_materialization(on);
    // A shallower bound than the default 10, applied IDENTICALLY to both sides so the
    // comparison stays honest. Two reasons, both about what this gate is for:
    //
    // * Cost. The OFF side backward-chains a recursive `judge` closure under iterative
    //   deepening, which is the pathology materialisation exists to remove — at depth 10
    //   in a debug build a single program costs seconds, and a gate that takes twenty
    //   minutes does not get run.
    // * Coverage. A tighter bound makes the OFF side hit `ResourceExceeded(Depth)` more
    //   often, which is exactly the "OFF non-definitive → ON definitive" arm — the
    //   deliberate completeness gain this differential is supposed to exercise, not
    //   merely tolerate.
    engine.kb().set_max_chain_depth(DIFF_CHAIN_DEPTH);
    for line in &case.kb {
        // An assert-time rejection (e.g. an arity clash the generator stumbled into) is
        // fine as long as BOTH sides reject it identically — materialisation is a
        // read-side concern and cannot touch the write path. The comparison below
        // catches any asymmetry, because a rejected line changes what the battery sees.
        if let Err(e) = engine.assert_text(line) {
            return Err(format!("assert {line:?}: {e:?}"));
        }
    }
    let mut out = Vec::with_capacity(case.queries.len());
    for q in &case.queries {
        match engine.query_holds(q) {
            Ok(v) => out.push(format!("{v:?}")),
            Err(e) => return Err(format!("query {q:?}: {e:?}")),
        }
    }
    Ok(out)
}

/// True for the `Debug` spelling of a verdict that DECIDED. Kept as a string test so the
/// harness never has to mirror `QueryResult`'s shape.
fn is_definitive(v: &str) -> bool {
    v == "True" || v == "False"
}

/// Run one case both ways and compare.
pub fn run_case(case: &MatCase) -> MatOutcome {
    let on = match run_side(case, true) {
        Ok(v) => v,
        Err(e) => {
            return MatOutcome::Error {
                name: case.name.clone(),
                detail: format!("materialisation ON: {e}"),
            };
        }
    };
    let off = match run_side(case, false) {
        Ok(v) => v,
        Err(e) => {
            return MatOutcome::Error {
                name: case.name.clone(),
                detail: format!("materialisation OFF: {e}"),
            };
        }
    };
    if on.len() != off.len() {
        return MatOutcome::Error {
            name: case.name.clone(),
            detail: format!("battery length {} vs {}", on.len(), off.len()),
        };
    }

    let mut gained = 0usize;
    for ((q, a), b) in case.queries.iter().zip(&on).zip(&off) {
        if a == b {
            continue;
        }
        if is_definitive(b) {
            // A decided verdict changed. This is the failure the gate exists for.
            return MatOutcome::Diverge {
                name: case.name.clone(),
                query: q.clone(),
                on: a.clone(),
                off: b.clone(),
            };
        }
        if !is_definitive(a) {
            return MatOutcome::LessDefinite {
                name: case.name.clone(),
                query: q.clone(),
                on: a.clone(),
                off: b.clone(),
            };
        }
        gained += 1;
    }
    MatOutcome::Agree {
        name: case.name.clone(),
        checked: on.len(),
        gained,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Same seed, same program — a divergence must be reproducible from its name.
    #[test]
    fn generator_is_deterministic() {
        for seed in [0u64, 7, 99] {
            let a = random_materialize_case(seed);
            let b = random_materialize_case(seed);
            assert_eq!(a.kb, b.kb);
            assert_eq!(a.queries, b.queries);
        }
    }

    /// Non-vacuity: the batch must actually contain the shapes materialisation has to
    /// refuse, or the gate would only ever exercise the happy path.
    #[test]
    fn the_batch_covers_the_refusal_shapes() {
        let mut du = 0;
        let mut tensed = 0;
        let mut recursive = 0;
        for seed in 0..200u64 {
            let c = random_materialize_case(seed);
            if c.kb.iter().any(|l| l.contains(" = ")) {
                du += 1;
            }
            if c.kb.iter().any(|l| l.starts_with("past ")) {
                tensed += 1;
            }
            if c.kb.iter().any(|l| l.contains("-> judge(")) {
                recursive += 1;
            }
        }
        assert!(du > 20, "too few `du` cases: {du}");
        assert!(tensed > 20, "too few tensed cases: {tensed}");
        assert!(recursive > 40, "too few recursive cases: {recursive}");
    }

    /// Every generated program must have a non-empty battery, or an "Agree" would be
    /// vacuous.
    #[test]
    fn every_case_has_a_battery() {
        for seed in 0..50u64 {
            assert!(!random_materialize_case(seed).queries.is_empty());
        }
    }
}
