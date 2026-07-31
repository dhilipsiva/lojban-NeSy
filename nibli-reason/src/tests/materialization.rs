//! Stratum-ordered materialisation ([`crate::materialize`]).
//!
//! These are SURFACE tests (`compile_surface`), not flat ones, and that is not a style
//! choice: materialisation is defined on the event-decomposed shape — it projects
//! `∃ev. rel(ev) ∧ rel_x1(ev,a) ∧ …` back to `rel(a, …)` — so a hand-built flat buffer
//! exercises none of it. See the FLAT vs SURFACE note at the top of `tests.rs`.
//!
//! The load-bearing property is the FIRST test: materialisation must never change a
//! verdict, only how fast one is reached. Everything else here pins a way that could
//! stop being true.

use super::*;

/// Both engines, same KB, same queries. Returns `(with_materialisation, without)`.
fn both_ways(kb_lines: &[&str], queries: &[&str]) -> (Vec<QueryResult>, Vec<QueryResult>) {
    let run = |on: bool| -> Vec<QueryResult> {
        let kb = new_kb();
        kb.set_materialization(on);
        for l in kb_lines {
            assert_buf(&kb, compile_surface(l));
        }
        queries
            .iter()
            .map(|q| query_result(&kb, compile_surface(q)))
            .collect()
    };
    (run(true), run(false))
}

/// A stratified-NAF KB shaped like utopia's Article 3/4 pair: a wide multi-variable
/// rule concludes `false`, and a merit rule reads it under `~`.
const AUDIT_KB: &[&str] = &[
    "person(Ara).",
    "person(Bel).",
    "person(Cyd).",
    "teaches(Ara, Cyd).",
    "teaches(Bel, Cyd).",
    "judge(Ara, Bel).",
    "capture(Ara, Bel).",
    "all $a: all $x: judge($a, $x) & capture($a, $x) & ~deceive($a, $x) -> false($x).",
    "all $t: all $s: teaches($t, $s) & ~false($t) -> reward($t).",
];

/// THE property. Materialisation is an optimisation: it may make a previously
/// non-definitive verdict definitive (that is the completeness gain), but it must never
/// turn one definitive verdict into a different one.
#[test]
fn naf_verdicts_are_unchanged_by_materialization() {
    let queries = [
        "reward(Ara).", // Ara is not voided → merit rule fires
        "reward(Bel).", // Bel IS voided by Ara's audit → blocked
        "false(Bel).",  // the voided party
        "false(Ara).",  // nobody audited Ara
        "reward(Cyd).", // teaches nobody
    ];
    let (on, off) = both_ways(AUDIT_KB, &queries);
    for ((q, a), b) in queries.iter().zip(&on).zip(&off) {
        if a == b {
            continue;
        }
        assert!(
            !b.is_definitive(),
            "materialisation changed a DEFINITIVE verdict for {q}: {b:?} → {a:?}"
        );
        assert!(
            a.is_definitive(),
            "materialisation made {q} LESS definitive: {b:?} → {a:?}"
        );
    }
    // And the verdicts are the ones the KB actually entails.
    assert_eq!(on[0], QueryResult::True, "Ara is unvoided and teaches");
    assert_eq!(on[1], QueryResult::False, "Bel is voided, so no reward");
    assert_eq!(on[2], QueryResult::True, "Bel was audited");
    assert_eq!(on[3], QueryResult::False, "Ara was not audited");
}

/// THE deliberate behaviour change (GUARANTEES §Completeness).
///
/// A saturated extension is complete regardless of `max_chain_depth`, so a NAF whose
/// positive would need a chain past the bound now answers definitively where it used to
/// return `ResourceExceeded(Depth)`. That is a completeness GAIN — non-definitive
/// becoming definitive — never a flip between two definitive verdicts, which is what
/// the first test in this file pins.
///
/// Its positive twin is `positive_goal_past_the_depth_bound_becomes_definitive`. Since
/// both halves are shortcut, `depth_boundary_contract` (tests/traces.rs) now runs with
/// materialisation OFF — it pins the SEARCH contract, and a complete extension is exactly
/// what removes the search.
#[test]
fn naf_over_a_chain_past_the_depth_bound_becomes_definitive() {
    let kb_lines = [
        "dog(Rex).",
        "all $x: dog($x) -> animal($x).",
        "all $x: animal($x) -> alive($x).",
        "all $x: alive($x) -> beautiful($x).",
        // `beautiful` is 3 rule hops from the `dog` fact; read it under `~`.
        "all $x: person($x) & ~beautiful($x) -> rotten($x).",
        "person(Rex).",
    ];
    let verdict = |on: bool| {
        let kb = new_kb();
        kb.set_materialization(on);
        // A bound too small for the 3-hop chain the NAF has to refute.
        kb.set_max_chain_depth(1);
        for l in kb_lines {
            assert_buf(&kb, compile_surface(l));
        }
        query_result(&kb, compile_surface("rotten(Rex)."))
    };
    let off = verdict(false);
    assert!(
        !off.is_definitive(),
        "without materialisation a NAF past the depth bound must be non-definitive, got {off:?}"
    );
    let on = verdict(true);
    assert!(
        on.is_definitive(),
        "with materialisation the saturated extension decides it regardless of the bound, got {on:?}"
    );
    // Rex IS beautiful (via the chain), so `~beautiful(Rex)` fails and `rotten` does not hold.
    assert_eq!(on, QueryResult::False);
}

/// A saturation is a claim about the KB's CONTENT, so a later assertion must drop it.
/// Getting this wrong is not a stale-cache annoyance: the NAF would keep answering TRUE
/// for a positive the new fact just made derivable.
#[test]
fn a_later_assertion_invalidates_the_saturation() {
    let kb = new_kb();
    for l in [
        "person(Ara).",
        "all $x: person($x) & ~rotten($x) -> fit($x).",
    ] {
        assert_buf(&kb, compile_surface(l));
    }
    assert_eq!(
        query_result(&kb, compile_surface("fit(Ara).")),
        QueryResult::True,
        "nothing makes Ara rotten yet"
    );
    // This is the fact the stale extension would not know about.
    assert_buf(&kb, compile_surface("rotten(Ara)."));
    assert_eq!(
        query_result(&kb, compile_surface("fit(Ara).")),
        QueryResult::False,
        "the new `rotten` fact must block the NAF — a stale saturation would still say TRUE"
    );
}

/// The retraction twin. `rebuild_inner` clears the saturation directly rather than
/// relying on its callers pairing with `invalidate_pred_cache`, because
/// `KnowledgeBase::rebuild` does not.
#[test]
fn retraction_invalidates_the_saturation() {
    let kb = new_kb();
    assert_buf(&kb, compile_surface("person(Ara)."));
    assert_buf(
        &kb,
        compile_surface("all $x: person($x) & ~rotten($x) -> fit($x)."),
    );
    let rotten = assert_id(&kb, compile_surface("rotten(Ara)."), "rotten");
    assert_eq!(
        query_result(&kb, compile_surface("fit(Ara).")),
        QueryResult::False
    );
    kb.retract_fact_inner(rotten).unwrap();
    assert_eq!(
        query_result(&kb, compile_surface("fit(Ara).")),
        QueryResult::True,
        "retracting the blocker must re-open the NAF — a saturation surviving the rebuild would not"
    );
}

/// `KnowledgeBase::rebuild` is the one rebuild entry point that does NOT invalidate the
/// predicate cache, so the saturation must be dropped inside `rebuild_inner` itself.
/// Without that, this query answers from a knowledge base that no longer exists.
#[test]
fn a_bare_rebuild_drops_the_saturation() {
    let kb = new_kb();
    assert_buf(&kb, compile_surface("person(Ara)."));
    assert_buf(
        &kb,
        compile_surface("all $x: person($x) & ~rotten($x) -> fit($x)."),
    );
    assert_eq!(
        query_result(&kb, compile_surface("fit(Ara).")),
        QueryResult::True
    );
    assert!(
        kb.inner.borrow().materialized.borrow().is_some(),
        "the query should have built a saturation"
    );
    kb.rebuild().unwrap();
    assert!(
        kb.inner.borrow().materialized.borrow().is_none(),
        "rebuild must drop the saturation itself, not rely on caller discipline"
    );
}

/// Tense makes rule firing flavour-polymorphic and the projection does not model that,
/// so a flavoured relation must be REFUSED rather than saturated with the flavour
/// silently dropped (which would merge `past P(x)` and `P(x)` into one tuple).
#[test]
fn a_flavoured_relation_is_refused_and_still_answers_correctly() {
    let kb_lines = [
        "person(Ara).",
        "past rotten(Ara).",
        "all $x: person($x) & ~rotten($x) -> fit($x).",
    ];
    let (on, off) = both_ways(&kb_lines, &["fit(Ara)."]);
    assert_eq!(
        on, off,
        "a flavoured KB must fall back, giving byte-identical verdicts"
    );
    let kb = new_kb();
    for l in kb_lines {
        assert_buf(&kb, compile_surface(l));
    }
    let _ = query_result(&kb, compile_surface("fit(Ara)."));
    let (complete, _) = kb.materialization_report();
    assert!(
        !complete.iter().any(|r| r == "rotten"),
        "a relation with a Past fact must not be reported complete: {complete:?}"
    );
}

/// `du` makes fact lookup modulo union-find, so a plain set-membership test on a
/// projected tuple could miss an equivalent variant and report "no witness" wrongly.
/// The whole KB is refused when any equivalence class exists.
#[test]
fn equality_classes_refuse_the_whole_kb() {
    let kb_lines = [
        "person(Ara).",
        "rotten(Bel).",
        "Ara = Bel.",
        "all $x: person($x) & ~rotten($x) -> fit($x).",
    ];
    let (on, off) = both_ways(&kb_lines, &["fit(Ara)."]);
    assert_eq!(
        on, off,
        "with `du` present the engine must fall back, not answer from a projection"
    );
    let kb = new_kb();
    for l in kb_lines {
        assert_buf(&kb, compile_surface(l));
    }
    let _ = query_result(&kb, compile_surface("fit(Ara)."));
    let (complete, _) = kb.materialization_report();
    assert!(
        complete.is_empty(),
        "no relation may be saturated while equivalence classes exist: {complete:?}"
    );
}

/// The report is the only way a knowledge base can tell whether its slow `~p(x)`
/// actually got the lookup, so it must name what was saturated.
#[test]
fn the_report_names_what_was_saturated() {
    let kb = new_kb();
    for l in AUDIT_KB {
        assert_buf(&kb, compile_surface(l));
    }
    let _ = query_result(&kb, compile_surface("reward(Ara)."));
    let (complete, refused) = kb.materialization_report();
    assert!(
        complete.iter().any(|r| r == "false"),
        "`false` is read under `~` and is projectable — expected it saturated: \
         complete={complete:?} refused={refused:?}"
    );
    // Every refusal carries a human-readable reason, never an empty string.
    assert!(refused.iter().all(|(_, why)| !why.is_empty()));
}

/// Since the POSITIVE fast path, a negation-free KB saturates too — that is the point:
/// a completed extension answers an ordinary query by lookup, not only a `~p(x)`.
///
/// (This replaces an earlier `a_kb_without_negation_saturates_nothing`, which pinned the
/// NAF-only target set. That was correct while `~` was the sole consumer and became wrong
/// the moment positive goals could read the same extension.)
#[test]
fn a_kb_without_negation_still_saturates_for_positive_lookups() {
    let kb = new_kb();
    for l in ["dog(Rex).", "all $x: dog($x) -> animal($x)."] {
        assert_buf(&kb, compile_surface(l));
    }
    assert!(query(&kb, compile_surface("animal(Rex).")));
    let (complete, refused) = kb.materialization_report();
    assert!(
        complete.iter().any(|r| r == "animal"),
        "`animal` is rule-derived and projectable — expected it saturated: \
         complete={complete:?} refused={refused:?}"
    );
}

/// The positive twin of `naf_over_a_chain_past_the_depth_bound_becomes_definitive`, and
/// the reason `depth_boundary_contract` had to be scoped: a completed extension decides
/// regardless of `max_chain_depth`, so a positive goal past the bound now answers instead
/// of returning `ResourceExceeded(Depth)`. Non-definitive → definitive, the sound
/// direction; the differential gate is what forbids the other one.
#[test]
fn positive_goal_past_the_depth_bound_becomes_definitive() {
    let kb_lines = [
        "dog(Rex).",
        "all $x: dog($x) -> animal($x).",
        "all $x: animal($x) -> alive($x).",
        "all $x: alive($x) -> beautiful($x).",
    ];
    let verdict = |on: bool| {
        let kb = new_kb();
        kb.set_materialization(on);
        kb.set_max_chain_depth(1);
        for l in kb_lines {
            assert_buf(&kb, compile_surface(l));
        }
        query_result(&kb, compile_surface("beautiful(Rex)."))
    };
    let off = verdict(false);
    assert!(
        !off.is_definitive(),
        "a 3-hop chain under a depth-1 bound must be non-definitive without \
         materialisation, got {off:?}"
    );
    assert_eq!(
        verdict(true),
        QueryResult::True,
        "the saturated extension decides it regardless of the bound"
    );
}

/// A proof-traced query keeps the BACKWARD-CHAINING path: a lookup has no derivation, and
/// the trace contract assumes one. Gating that per-sink rather than per-query is what
/// broke `flat_vs_surface::transitive_chain_true` in development — phase 1 resolved at
/// depth 1 by lookup, then phase 2 rebuilt the trace by chaining at that same depth and
/// could not reach it, turning a TRUE into `ResourceExceeded(Depth)`.
#[test]
fn a_traced_query_agrees_with_its_untraced_twin() {
    let kb = new_kb();
    for l in [
        "dog(Rex).",
        "all $x: dog($x) -> animal($x).",
        "all $x: animal($x) -> alive($x).",
    ] {
        assert_buf(&kb, compile_surface(l));
    }
    let untraced = query_result(&kb, compile_surface("alive(Rex)."));
    let (traced, trace) = kb
        .query_entailment_with_proof_inner(compile_surface("alive(Rex)."))
        .unwrap();
    assert_eq!(untraced, QueryResult::True);
    assert_eq!(
        traced, untraced,
        "traced and untraced verdicts must not diverge"
    );
    assert!(
        trace
            .steps
            .get(trace.root as usize)
            .is_some_and(|s| s.holds),
        "a TRUE verdict must carry a holding root step, not a not-found leaf"
    );
}

/// Recursion through a positive cycle is one stratum, evaluated to a fixpoint — the
/// case a non-recursive "evaluate each rule once" saturation would silently truncate,
/// under-deriving the extension and turning a NAF FALSE into a wrong TRUE.
#[test]
fn a_recursive_positive_relation_saturates_to_its_fixpoint() {
    let kb_lines = [
        "person(Ara).",
        "parent(Ara, Bel).",
        "parent(Bel, Cyd).",
        "parent(Cyd, Dee).",
        // Transitive closure: `judge` is recursive through itself (a POSITIVE cycle,
        // which stratification accepts and which must be run to a fixpoint).
        "all $x: all $y: parent($x, $y) -> judge($x, $y).",
        "all $x: all $y: all $z: judge($x, $y) & parent($y, $z) -> judge($x, $z).",
        // Read the closure under `~`.
        "all $x: person($x) & ~judge($x, Dee) -> rotten($x).",
    ];
    let (on, off) = both_ways(&kb_lines, &["rotten(Ara).", "judge(Ara, Dee)."]);
    assert_eq!(on, off, "recursion must not change under materialisation");
    assert_eq!(
        on[1],
        QueryResult::True,
        "Ara reaches Dee in three hops of the transitive closure"
    );
    assert_eq!(
        on[0],
        QueryResult::False,
        "so `~judge(Ara, Dee)` fails — a truncated fixpoint would wrongly say TRUE"
    );
}

/// The kill switch is what makes the ON/OFF differential expressible, and it must take
/// effect immediately rather than at the next mutation.
#[test]
fn toggling_materialization_off_drops_the_saturation() {
    let kb = new_kb();
    for l in AUDIT_KB {
        assert_buf(&kb, compile_surface(l));
    }
    let _ = query_result(&kb, compile_surface("reward(Ara)."));
    assert!(kb.inner.borrow().materialized.borrow().is_some());
    kb.set_materialization(false);
    assert!(
        kb.inner.borrow().materialized.borrow().is_none(),
        "turning the switch off must drop the extension now, not later"
    );
    assert!(!kb.is_materialization());
    // And the verdict is unchanged.
    assert_eq!(
        query_result(&kb, compile_surface("reward(Ara).")),
        QueryResult::True
    );
}

/// Configuration, not derived state — `reset()` wipes the KB but keeps the mode, exactly
/// like `strict` and `existential_import`.
#[test]
fn the_materialization_mode_survives_reset() {
    let kb = new_kb();
    kb.set_materialization(false);
    kb.reset();
    assert!(
        !kb.is_materialization(),
        "the mode is session configuration, not KB content"
    );
}

/// THE BUG THE DIFFERENTIAL CAUGHT (`mat_seed4` / `mat_seed35`, 2026-07-31).
///
/// `eligible_relations` closes downward over relations it cannot PROJECT. But a relation
/// can also become unusable later, when `seed_edb` refuses its STORED FACTS — a `past`
/// fact, a role gap, an arity clash. Those refusals are invisible to the eligibility
/// analysis, so a rule reading `~rotten` was still saturated while `rotten`'s extension
/// was ABSENT. An absent extension reads as "nothing derived", so the negated condition
/// passed and the head was derived for everyone: a definitive WRONG TRUE, the exact
/// failure this module exists to prevent.
///
/// Here `rotten` carries a Past fact, so it cannot be projected; `fit` reads it under `~`
/// and must therefore be refused too, not silently completed over a hole.
#[test]
fn a_relation_whose_negated_dependency_is_unseedable_is_refused_not_completed() {
    let kb_lines = [
        "person(Ara).",
        "rotten(Ara).",
        // Makes `rotten` unprojectable: rule firing is flavour-polymorphic and the
        // projection drops flavours, so the relation is refused wholesale.
        "past rotten(Ara).",
        "all $x: person($x) & ~rotten($x) -> fit($x).",
    ];
    let (on, off) = both_ways(&kb_lines, &["fit(Ara)."]);
    assert_eq!(
        on[0],
        QueryResult::False,
        "Ara IS rotten, so `~rotten(Ara)` fails and `fit` must not hold"
    );
    assert_eq!(on, off, "materialisation must not change this verdict");

    let kb = new_kb();
    for l in kb_lines {
        assert_buf(&kb, compile_surface(l));
    }
    let _ = query_result(&kb, compile_surface("fit(Ara)."));
    let (complete, refused) = kb.materialization_report();
    assert!(
        !complete.iter().any(|r| r == "fit"),
        "`fit` reads an unseedable relation under `~` — it must NOT be complete: \
         complete={complete:?}"
    );
    assert!(
        refused
            .iter()
            .any(|(rel, why)| rel == "fit" && why.contains("rotten")),
        "the refusal must NAME the dependency that caused it: refused={refused:?}"
    );
    // And the reason for `rotten` itself must point at the DATA, not at a rule — the two
    // are different repairs.
    assert!(
        refused
            .iter()
            .any(|(rel, why)| rel == "rotten" && why.contains("stored fact")),
        "refused={refused:?}"
    );
}
