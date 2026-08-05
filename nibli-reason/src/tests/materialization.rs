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

/// The projection does not model tense, so a flavoured relation must be REFUSED
/// rather than saturated with the flavour
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

#[test]
fn bare_temporal_non_lifting_is_identical_with_materialization_on_or_off() {
    let kb_lines = ["all $x: dog($x) -> animal($x).", "past dog(Ara)."];
    let (on, off) = both_ways(&kb_lines, &["past animal(Ara)."][..]);
    assert_eq!(on, vec![QueryResult::False]);
    assert_eq!(on, off, "materialization must not invent temporal lifting");
}

#[test]
fn explicit_temporal_rule_falls_back_without_changing_the_verdict() {
    let kb_lines = ["all $x: past dog($x) -> past animal($x).", "past dog(Ara)."];
    let (on, off) = both_ways(&kb_lines, &["past animal(Ara)."][..]);
    assert_eq!(on, vec![QueryResult::True]);
    assert_eq!(on, off, "the exact backward-chain fallback must agree");

    let kb = new_kb();
    for line in kb_lines {
        assert_buf(&kb, compile_surface(line));
    }
    let _ = query_result(&kb, compile_surface("past animal(Ara)."));
    let (complete, refused) = kb.materialization_report();
    assert!(
        !complete.iter().any(|relation| relation == "animal"),
        "a flavored rule head must not be reported complete: {complete:?}"
    );
    assert!(
        refused.iter().any(|(relation, why)| {
            relation == "animal" && why.contains("tense/deontic flavour")
        }),
        "the report must disclose the flavored-rule refusal: {refused:?}"
    );
}

#[test]
fn explicit_temporal_naf_never_uses_the_bare_materialized_extension() {
    let kb_lines = [
        "fit(every person where past ~rotten(it)).",
        "person(Ara).",
        "rotten(Ara).",
    ];
    let (on, off) = both_ways(&kb_lines, &["fit(Ara)."][..]);
    assert_eq!(
        off,
        vec![QueryResult::True],
        "a bare rotten witness must not block `past ~rotten`"
    );
    assert_eq!(
        on, off,
        "materialization must refuse the flavored NAF group and fall back"
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
    let _ = kb.reset();
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
        // Makes `rotten` unprojectable: the projection detects the flavor and
        // refuses the relation wholesale rather than dropping it.
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

/// THE ABSTRACTION PROJECTION, and the firewall it must not break.
///
/// `entitled(every person, event { P() }).` compiles to a head carrying an abstraction
/// referent — `event(sk_1(x))` and `__abs_<id>(sk_1(x))`, two arity-1 atoms on ONE event
/// term, plus `entitled_x2(sk_3(x), sk_1(x))`, the referent in a role VALUE. Untreated
/// that is `AmbiguousAnchor` + `SkolemInValue`, and EVERY abstraction-bearing rule sits
/// outside the saturation.
///
/// The projection maps the referent to its marker relation name as an opaque constant —
/// the identity that crosses compiles is that NAME, not any term. What makes this safe to
/// materialise is a property of the compiled form, not of the refusal: the abstraction
/// body's event term is `sk_2(Unspecified)`, INDEPENDENT of the universal, so its role
/// values are `Unspecified` and the body projects to a tuple about nobody. An entitlement
/// therefore cannot fabricate the actuality it guarantees.
#[test]
fn an_entitlement_is_materialised_without_fabricating_the_actuality() {
    let kb_lines = ["entitled(every person, event { eats() }).", "person(Adam)."];
    let (on, off) = both_ways(
        &kb_lines,
        &[
            "entitled(Adam, event { eats() }).",
            "eats(Adam).",
            "eats(some person).",
            "entitled(Adam, event { choose() }).",
            "entitled(Adam, fact { eats() }).",
        ],
    );
    assert_eq!(on, off, "the projection must not change any verdict");
    assert_eq!(on[0], QueryResult::True, "the entitlement must still MATCH");
    assert_eq!(
        on[1],
        QueryResult::False,
        "the actuality must still MISS — an entitlement does not feed anyone"
    );
    assert_eq!(on[2], QueryResult::False, "nor for anyone else");
    assert_eq!(
        on[3],
        QueryResult::False,
        "a different body is a different lossless identity"
    );
    assert_eq!(
        on[4],
        QueryResult::False,
        "abstraction kind survives projection into the marker identity"
    );

    // And it is genuinely materialised, not passing by falling back to the chainer.
    let kb = new_kb();
    for l in kb_lines {
        assert_buf(&kb, compile_surface(l));
    }
    let _ = query_result(&kb, compile_surface("eats(Adam)."));
    let (complete, refused) = kb.materialization_report();
    assert!(
        complete.iter().any(|r| r == "eats"),
        "`eats` should be saturable now: complete={complete:?} refused={refused:?}"
    );
    // The typing markers must be REFUSED, never merely omitted: `is_edb` is
    // "no rule and not refused", and an omitted marker would be marked complete over an
    // empty seed, so `~event(x)` would answer TRUE where the chainer answers FALSE.
    assert!(
        refused.iter().any(|(r, _)| r == "event"),
        "the `event` typing anchor must be refused, not left to look like EDB: \
         refused={refused:?}"
    );
    assert!(
        refused.iter().any(|(r, _)| r.starts_with("__abs_")),
        "the abstraction marker must be refused too: refused={refused:?}"
    );
}

fn force_abstraction_digest(buffer: &mut LogicBuffer, digest: &str) {
    assert_eq!(digest.len(), 16);
    let digest = u64::from_str_radix(digest, 16).unwrap();
    let mut rewritten = 0;
    for node in &mut buffer.nodes {
        if let LogicNode::Predicate((relation, _)) = node
            && relation.starts_with("__abs_v1_")
        {
            let (_, key_hex) = relation
                .strip_prefix("__abs_v1_")
                .unwrap()
                .split_once('_')
                .unwrap();
            let key: Vec<u8> = key_hex
                .as_bytes()
                .chunks_exact(2)
                .map(|pair| {
                    let pair = std::str::from_utf8(pair).unwrap();
                    u8::from_str_radix(pair, 16).unwrap()
                })
                .collect();
            *relation = nibli_types::abstraction::encode_v1_with_digest(&key, digest);
            rewritten += 1;
        }
    }
    assert_eq!(
        rewritten, 1,
        "collision seam must rewrite exactly one abstraction marker"
    );
}

fn abstraction_marker_relation(buffer: &LogicBuffer) -> String {
    let markers: Vec<String> = buffer
        .nodes
        .iter()
        .filter_map(|node| match node {
            LogicNode::Predicate((relation, _)) if relation.starts_with("__abs_v1_") => {
                Some(relation.clone())
            }
            _ => None,
        })
        .collect();
    assert_eq!(markers.len(), 1);
    markers.into_iter().next().unwrap()
}

#[test]
fn abstraction_digest_prefix_is_canonicalized_from_the_full_key() {
    let mut asserted = compile_surface("believe(me, fact { goes(Adam) }).");
    let mut same_query = compile_surface("believe(me, fact { goes(Adam) }).");
    let mut different_query = compile_surface("believe(me, fact { goes(Bel) }).");
    force_abstraction_digest(&mut asserted, "0000000000000000");
    force_abstraction_digest(&mut same_query, "ffffffffffffffff");
    force_abstraction_digest(&mut different_query, "0000000000000000");

    let kb = new_kb();
    assert_buf(&kb, asserted);
    assert!(
        query(&kb, same_query),
        "the same full key must match even when supplied digest prefixes differ"
    );
    assert!(
        query_false(&kb, different_query),
        "equal digest prefixes must not conflate different proposition bodies"
    );
}

#[test]
fn equal_abstraction_digest_prefixes_do_not_match_distinct_full_keys() {
    let mut first = compile_surface("believe(me, fact { goes(Adam) }).");
    let mut second = compile_surface("believe(me, fact { goes(Bel) }).");
    force_abstraction_digest(&mut first, "0000000000000000");
    force_abstraction_digest(&mut second, "0000000000000000");
    let first_relation = abstraction_marker_relation(&first);
    let second_relation = abstraction_marker_relation(&second);
    assert_ne!(first_relation, second_relation);
    assert_eq!(
        &first_relation[.."__abs_v1_".len() + 16],
        &second_relation[.."__abs_v1_".len() + 16],
        "the seam must create a real digest-prefix collision"
    );

    let referent = GroundTerm::Constant("opaque".to_string());
    let stored = StoredFact::Bare(GroundFact::new(first_relation, vec![referent.clone()]));
    let colliding_query = StoredFact::Bare(GroundFact::new(second_relation, vec![referent]));
    let kb = new_kb();
    let mut inner = kb.inner.borrow_mut();
    assert_typed_fact(stored.clone(), &mut inner);
    assert!(typed_fact_is_stored(&stored, &inner));
    assert!(
        !typed_fact_is_stored(&colliding_query, &inner),
        "the core matcher must compare the complete key, not the equal digest prefix"
    );
}

#[test]
fn legacy_hash_only_abstraction_markers_fail_closed_on_every_buffer_path() {
    let mut legacy = compile_surface("believe(me, fact { goes(Adam) }).");
    for node in &mut legacy.nodes {
        if let LogicNode::Predicate((relation, _)) = node
            && relation.starts_with("__abs_v1_")
        {
            *relation = "__abs_0123456789abcdef".to_string();
        }
    }

    let kb = new_kb();
    let assertion_error = kb
        .assert_fact_inner(legacy.clone(), "legacy persisted buffer".to_string())
        .expect_err("hash-only assertion replay must be rejected");
    assert!(assertion_error.contains("recompile/re-import"));
    let query_error = kb
        .query_entailment_inner(legacy.clone())
        .expect_err("hash-only entailment queries must be rejected");
    assert!(query_error.contains("legacy hash-only"));
    let proof_error = kb
        .query_entailment_with_proof_inner(legacy.clone())
        .expect_err("hash-only proof queries must be rejected");
    assert!(proof_error.contains("legacy hash-only"));
    let find_error = kb
        .query_find_inner(legacy)
        .expect_err("hash-only find/count/aggregate queries must be rejected");
    assert!(find_error.contains("legacy hash-only"));
}

#[test]
fn malformed_and_unknown_abstraction_markers_fail_closed() {
    for malformed_relation in [
        "__abs_v2_0123456789abcdef_00",
        "__abs_v1_short_00",
        "__abs_v1_0123456789abcdef_0",
        "__abs_v1_0123456789abcdef_FF",
    ] {
        let mut malformed = compile_surface("believe(me, fact { goes(Adam) }).");
        for node in &mut malformed.nodes {
            if let LogicNode::Predicate((relation, _)) = node
                && relation.starts_with("__abs_v1_")
            {
                *relation = malformed_relation.to_string();
            }
        }
        let error = new_kb()
            .assert_fact_inner(malformed, "malformed marker".to_string())
            .expect_err("unknown/malformed markers must never enter the fact store");
        assert!(
            error.contains("unsupported or malformed opaque-abstraction marker"),
            "unexpected error for {malformed_relation}: {error}"
        );
    }

    let mut wrong_arity = compile_surface("believe(me, fact { goes(Adam) }).");
    for node in &mut wrong_arity.nodes {
        if let LogicNode::Predicate((relation, args)) = node
            && relation.starts_with("__abs_v1_")
        {
            args.push(LogicalTerm::Unspecified);
        }
    }
    let arity_error = new_kb()
        .assert_fact_inner(wrong_arity, "non-unary marker".to_string())
        .expect_err("an internal marker must remain unary");
    assert!(arity_error.contains("unary Predicate") && arity_error.contains("arity 2"));

    let mut compute_marker = compile_surface("believe(me, fact { goes(Adam) }).");
    let marker_index = compute_marker
        .nodes
        .iter()
        .position(|node| {
            matches!(node, LogicNode::Predicate((relation, _)) if relation.starts_with("__abs_v1_"))
        })
        .expect("fixture must contain a marker");
    let (relation, args) = match compute_marker.nodes[marker_index].clone() {
        LogicNode::Predicate(pair) => pair,
        _ => unreachable!("selected a Predicate marker"),
    };
    compute_marker.nodes[marker_index] = LogicNode::ComputeNode((relation, args));
    let kb = new_kb();
    let assertion_error = kb
        .assert_fact_inner(compute_marker.clone(), "compute marker".to_string())
        .expect_err("an internal marker must never be asserted as a ComputeNode");
    assert!(
        assertion_error.contains("never ComputeNode"),
        "marker validation must run before treating anything as opaque: {assertion_error}"
    );
    assert_eq!(
        kb.next_fact_id().unwrap(),
        0,
        "malformed marker rejection must precede id allocation"
    );
    let query_error = kb
        .query_entailment_inner(compute_marker)
        .expect_err("an internal marker must never be queried as a ComputeNode");
    assert!(query_error.contains("never ComputeNode"));
}

#[test]
fn nested_abstraction_identity_keeps_the_complete_inner_proposition() {
    let asserted = "believe(me, fact { believe(Bel, fact { goes(Adam) }) }).";
    let same = asserted;
    let different_body = "believe(me, fact { believe(Bel, fact { goes(Gia) }) }).";
    let different_kind = "believe(me, fact { believe(Bel, event { goes(Adam) }) }).";

    let kb = new_kb();
    assert_buf(&kb, compile_surface(asserted));
    assert!(query(&kb, compile_surface(same)));
    assert!(query_false(&kb, compile_surface(different_body)));
    assert!(query_false(&kb, compile_surface(different_kind)));
    assert!(
        query_false(&kb, compile_surface("believe(Bel, fact { goes(Adam) }).")),
        "the nested abstraction body must remain opaque"
    );
}

/// A MATERIALISED relation must flip when an assertion changes what it derives.
///
/// This is the engine-side twin of the consuming project's release sequence
/// (`13-the-one-thing-taken.pins.nibli`: `dwell(Hano)` TRUE at :46, `free(Hano).` at :72,
/// `dwell(Hano)` FALSE at :101). Before the abstraction projection those relations were
/// REFUSED, so they were backward-chained and invalidation was irrelevant to them; now
/// they are saturated and the flip is a live test that the extension is dropped.
///
/// Distinct from `a_later_assertion_invalidates_the_saturation` in asserting that the
/// relation really was COMPLETE on both sides — otherwise a silent fallback would make
/// this pass while pinning nothing.
#[test]
fn a_materialised_relation_flips_across_an_assertion() {
    let kb = new_kb();
    for l in [
        "person(Ara).",
        "all $x: person($x) & ~rotten($x) -> fit($x).",
    ] {
        assert_buf(&kb, compile_surface(l));
    }
    assert_eq!(
        query_result(&kb, compile_surface("fit(Ara).")),
        QueryResult::True
    );
    let (before, _) = kb.materialization_report();
    assert!(
        before.iter().any(|r| r == "fit"),
        "`fit` must be materialised for this to test anything: {before:?}"
    );

    assert_buf(&kb, compile_surface("rotten(Ara)."));
    assert_eq!(
        query_result(&kb, compile_surface("fit(Ara).")),
        QueryResult::False,
        "the new fact must block the NAF — a surviving extension would still say TRUE"
    );
    let (after, _) = kb.materialization_report();
    assert!(
        after.iter().any(|r| r == "fit"),
        "and it must be re-saturated, not silently demoted to fallback: {after:?}"
    );
}

// ─── Stratification report (the machine-readable dump) ───────────────────────

/// Load a shipped corpus into a fresh KB, skipping lines that are not plain KB text.
fn kb_from_corpus(src: &str) -> KnowledgeBase {
    let kb = new_kb();
    for raw in src.lines() {
        let line = raw.trim();
        if line.is_empty()
            || line.starts_with('#')
            || line.starts_with(':')
            || line.starts_with('?')
        {
            continue;
        }
        if let Ok(ast) = nibli_kr::parse_checked(line)
            && let Ok(mut buf) = nibli_semantics::compile_from_ast(ast)
        {
            transform_compute_nodes(&mut buf, &default_compute_predicates());
            let _ = kb.assert_fact(buf, line.to_string());
        }
    }
    kb
}

#[test]
fn strata_surface_projection_is_lossless() {
    // `stratification_report` collapses event-decomposed role predicates (`p_x1`) onto
    // their anchor (`p`). That is only honest if every member of a decomposed atom lands
    // in the SAME stratum — which it must, since the roles and the anchor are conditions
    // and conclusions of exactly the same rules and therefore carry identical dependency
    // sets. This pins it on the shipped corpora rather than trusting the argument: if it
    // ever fails, the report is silently picking a stratum and the collapse must go.
    for (name, src) in [
        ("utopia", include_str!("../../../utopia.nibli")),
        ("gdpr", include_str!("../../../gdpr.nibli")),
        (
            "drug-interactions",
            include_str!("../../../drug-interactions.nibli"),
        ),
        (
            "determinism",
            include_str!("../../../determinism-corpus.nibli"),
        ),
    ] {
        let kb = kb_from_corpus(src);
        let inner = kb.inner.borrow();
        let strata = crate::materialize::compute_strata(&inner.pred_dep_graph);
        let mut by_surface: std::collections::BTreeMap<&str, std::collections::BTreeSet<usize>> =
            std::collections::BTreeMap::new();
        for (raw, lvl) in &strata {
            by_surface
                .entry(crate::materialize::surface_relation(raw))
                .or_default()
                .insert(*lvl);
        }
        for (surface, levels) in &by_surface {
            assert_eq!(
                levels.len(),
                1,
                "{name}: `{surface}` spans strata {levels:?} — the anchor and its role \
                 predicates disagree, so collapsing them loses information"
            );
        }
    }
}

#[test]
fn stratification_report_is_stable_and_well_formed() {
    let kb = kb_from_corpus(include_str!("../../../utopia.nibli"));
    let rows = kb.stratification_report();
    assert!(!rows.is_empty(), "utopia must produce a non-empty report");

    // Deterministic: same KB, same bytes. `pred_dep_graph` is a HashMap, so this is the
    // property a consumer diffing across runs actually depends on.
    let again = kb.stratification_report();
    assert_eq!(rows, again, "two reports off one KB must be identical");

    // Sorted by predicate, edges sorted and deduplicated.
    let names: Vec<&str> = rows.iter().map(|r| r.predicate.as_str()).collect();
    let mut sorted = names.clone();
    sorted.sort_unstable();
    assert_eq!(names, sorted, "rows must be sorted by predicate");
    for r in &rows {
        let mut e = r.edges.clone();
        e.sort();
        e.dedup();
        assert_eq!(
            e, r.edges,
            "{}: edges must be sorted and deduplicated",
            r.predicate
        );
        // No role predicate survives the surface projection.
        assert_eq!(
            crate::materialize::surface_relation(&r.predicate),
            r.predicate,
            "a role predicate leaked into the report"
        );
    }

    // base/derived agrees with "a rule concludes it", the definition the dump documents.
    let inner = kb.inner.borrow();
    let derived: std::collections::BTreeSet<&str> = inner
        .universal_rules
        .keys()
        .map(|k| crate::materialize::surface_relation(k))
        .collect();
    for r in &rows {
        assert_eq!(
            r.base,
            !derived.contains(r.predicate.as_str()),
            "{}: base/derived disagrees with the rule-head set",
            r.predicate
        );
    }

    // utopia reads `false` under negation, so a negative edge must be present somewhere —
    // otherwise the polarity column is uniformly `+` and pins nothing.
    assert!(
        rows.iter().any(|r| r.edges.iter().any(|e| e.negative)),
        "utopia has NAF rules; the report must mark at least one negative edge"
    );
}

#[test]
fn a_negative_edge_raises_the_stratum_it_reads_from() {
    // The property the whole dump exists to communicate: a predicate read under `~` must
    // sit STRICTLY below the predicate reading it, and a positive edge must not raise it.
    let kb = new_kb();
    for line in [
        "person(Adam).",
        "all $x: person($x) & ~home($x) -> prisoner($x).",
        "all $x: prisoner($x) -> reward($x).",
    ] {
        assert_buf(&kb, compile_surface(line));
    }
    let rows = kb.stratification_report();
    let get = |p: &str| {
        rows.iter()
            .find(|r| r.predicate == p)
            .unwrap_or_else(|| panic!("{p}"))
    };

    let home = get("home");
    let prisoner = get("prisoner");
    let watched = get("reward");
    assert!(
        prisoner.stratum > home.stratum,
        "a NAF read must raise the reader's stratum: prisoner={} home={}",
        prisoner.stratum,
        home.stratum
    );
    assert_eq!(
        watched.stratum, prisoner.stratum,
        "a POSITIVE edge must not raise the stratum"
    );
    assert!(home.base, "`home` is concluded by no rule");
    assert!(!prisoner.base, "`prisoner` is concluded by a rule");
    assert!(
        prisoner.edges.iter().any(|e| e.to == "home" && e.negative),
        "the prisoner -> home edge must be marked negative: {:?}",
        prisoner.edges
    );
    assert!(
        watched
            .edges
            .iter()
            .any(|e| e.to == "prisoner" && !e.negative),
        "the watched -> prisoner edge must be marked positive: {:?}",
        watched.edges
    );
}
