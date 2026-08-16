use super::*;

// ─── STRICT MODE (opt-in): reject instead of warn-and-insert ─────────

/// Strict arity: the mismatching fact is rejected and the assertion fails
/// atomically; the permissive default (pinned above in
/// `test_predicate_arity_mismatch_detected`) is unchanged.
#[test]
fn strict_mode_rejects_arity_mismatch() {
    let kb = new_kb();
    kb.set_strict(true);
    assert_buf(&kb, make_assertion("alis", "gerku")); // registers arity 2

    let mut nodes = Vec::new();
    let root = pred(
        &mut nodes,
        "gerku",
        vec![LogicalTerm::Constant("bob".to_string())],
    );
    let err = kb
        .assert_fact_inner(
            LogicBuffer {
                nodes,
                roots: vec![root],
            },
            String::new(),
        )
        .expect_err("strict mode must fail an arity-mismatched assertion");
    assert!(
        err.contains("strict mode rejected") && err.contains("arity mismatch"),
        "unexpected error: {err}"
    );

    // The mismatching fact must NOT be in the store; the original must be.
    let inner = kb.inner.borrow();
    let dog_facts = inner.fact_store.lookup_predicate("gerku").unwrap();
    assert!(
        !dog_facts.iter().any(|f| f.inner().args.len() == 1),
        "the rejected arity-1 fact must not be stored"
    );
}

/// Strict constraints: an assertion completing a violating set is rejected and
/// rolled back atomically — the KB is byte-identical to the pre-assertion
/// state, and the earlier (non-violating) fact survives.
#[test]
fn strict_mode_rejects_constraint_violation_atomically() {
    let kb = new_kb();
    kb.set_strict(true);

    // Constraint: gerku(alis) and mlatu(alis) must not both hold.
    let c1 = StoredFact::Bare(GroundFact::new(
        "gerku",
        vec![GroundTerm::Constant("alis".to_string())],
    ));
    let c2 = StoredFact::Bare(GroundFact::new(
        "mlatu",
        vec![GroundTerm::Constant("alis".to_string())],
    ));
    kb.register_constraint("not-both".to_string(), vec![c1, c2])
        .unwrap();

    let flat = |rel: &str| {
        let mut nodes = Vec::new();
        let root = pred(
            &mut nodes,
            rel,
            vec![LogicalTerm::Constant("alis".to_string())],
        );
        LogicBuffer {
            nodes,
            roots: vec![root],
        }
    };

    assert_buf(&kb, flat("gerku")); // fine — the set is incomplete
    let err = kb
        .assert_fact_inner(flat("mlatu"), String::new())
        .expect_err("strict mode must fail the violation-completing assertion");
    assert!(
        err.contains("strict mode rejected") && err.contains("integrity constraint"),
        "unexpected error: {err}"
    );

    // The violating fact is gone; the earlier fact survives; verdicts agree.
    assert!(query(&kb, flat("gerku")), "the pre-existing fact survives");
    assert!(
        query_false(&kb, flat("mlatu")),
        "the rejected fact must not be queryable"
    );

    // The permissive default is untouched: same sequence on a fresh KB
    // warns and inserts.
    let kb2 = new_kb();
    kb2.register_constraint(
        "not-both".to_string(),
        vec![
            StoredFact::Bare(GroundFact::new(
                "gerku",
                vec![GroundTerm::Constant("alis".to_string())],
            )),
            StoredFact::Bare(GroundFact::new(
                "mlatu",
                vec![GroundTerm::Constant("alis".to_string())],
            )),
        ],
    )
    .unwrap();
    assert_buf(&kb2, flat("gerku"));
    assert_buf(&kb2, flat("mlatu")); // warns, does not error
    assert!(query(&kb2, flat("mlatu")), "permissive mode still inserts");
}

#[test]
fn constraint_abstraction_identity_is_canonicalized_or_rejected() {
    fn valid_marker_with_digest(digest: &str) -> String {
        let mut key = vec![0xa0, 0x10];
        key.extend_from_slice(&1_u64.to_be_bytes());
        key.push(b'p');
        key.extend_from_slice(&0_u64.to_be_bytes());
        let key_hex: String = key.iter().map(|byte| format!("{byte:02x}")).collect();
        format!("__abs_v1_{digest}_{key_hex}")
    }

    let kb = new_kb();
    kb.set_strict(true);
    let forged = valid_marker_with_digest("0000000000000000");
    let term = GroundTerm::Constant("abstract".to_string());
    let mut canonical_fact = StoredFact::Bare(GroundFact::new(&forged, vec![term.clone()]));
    canonicalize_stored_fact_abstraction_marker(&mut canonical_fact).unwrap();
    let canonical = canonical_fact.relation().to_string();
    assert_ne!(forged, canonical, "the test digest must be non-canonical");

    kb.register_constraint("no-abstract-p".to_string(), vec![canonical_fact])
        .unwrap();
    let logic = LogicBuffer {
        nodes: vec![LogicNode::Predicate((
            forged,
            vec![LogicalTerm::Constant("abstract".to_string())],
        ))],
        roots: vec![0],
    };
    let err = kb
        .assert_fact(logic, "forged-digest assertion".to_string())
        .expect_err("canonical-equivalent constraint must reject the assertion");
    assert!(
        err.to_string().contains("integrity constraint"),
        "unexpected error: {err}"
    );

    let legacy = StoredFact::Bare(GroundFact::new("__abs_0123456789abcdef", vec![term]));
    assert!(
        kb.register_constraint("legacy".to_string(), vec![legacy])
            .is_err(),
        "legacy hash-only constraint markers must fail closed"
    );
}

/// Strict mode is inert during retraction-replay rebuilds: facts accepted
/// before strict was enabled replay faithfully.
#[test]
fn strict_mode_is_inert_during_rebuild() {
    let kb = new_kb();
    // Permissively insert an arity mismatch (warned, stored).
    assert_buf(&kb, make_assertion("alis", "gerku"));
    let mut nodes = Vec::new();
    let root = pred(
        &mut nodes,
        "gerku",
        vec![LogicalTerm::Constant("bob".to_string())],
    );
    let id = kb
        .assert_fact_inner(
            LogicBuffer {
                nodes,
                roots: vec![root],
            },
            String::new(),
        )
        .unwrap();

    // Turn strict ON, then force a rebuild by retracting an unrelated fact:
    // the mismatched fact must survive the replay.
    kb.set_strict(true);
    let unrelated = kb
        .assert_fact_inner(make_assertion("kim", "mlatu"), String::new())
        .unwrap();
    kb.retract_fact(unrelated).unwrap();

    let inner = kb.inner.borrow();
    let dog_facts = inner.fact_store.lookup_predicate("gerku").unwrap();
    assert!(
        dog_facts.iter().any(|f| f.inner().args.len() == 1),
        "fact {id}: a previously-accepted mismatch must survive a strict-era rebuild"
    );
}

// ─── Internal strict rollback (unassert_typed_fact) ──────────────────

/// Kills three rules.rs mutants in `unassert_typed_fact`:
/// `replace unassert_typed_fact with ()` (the rejected fact would stay in the
/// store), `delete !` (early return after the store remove — the index leaves
/// keep the ghost), and `replace != with ==` (the leaf `retain` flips polarity
/// and scrubs the INNOCENT co-leaf facts instead of the rejected one).
///
/// Internal insertions have no registry-rebuild rollback behind them
/// (`assert_fact_inner`'s rebuild masks the user-assertion twin), so
/// `unassert_typed_fact` itself must surgically undo the insert: store AND every
/// index leaf, innocents untouched.
#[test]
fn strict_internal_constraint_rejection_scrubs_store_and_index() {
    let kb = new_kb();

    // Innocent co-leaf fact: same relation, shares the (relation, position,
    // value) index leaves at positions 0 and 1 with the fact the constraint
    // will reject. Asserted BEFORE strict so nothing rejects it.
    let flat_num = |a: f64, b: f64, c: f64| {
        let mut nodes = Vec::new();
        let root = pred(
            &mut nodes,
            "zzoracle",
            vec![
                LogicalTerm::Number(a),
                LogicalTerm::Number(b),
                LogicalTerm::Number(c),
            ],
        );
        LogicBuffer {
            nodes,
            roots: vec![root],
        }
    };
    assert_buf(&kb, flat_num(8.0, 2.0, 9.0));

    kb.set_strict(true);
    let stored = |c: f64| {
        StoredFact::Bare(GroundFact::new(
            "zzoracle",
            vec![
                GroundTerm::from_f64(8.0),
                GroundTerm::from_f64(2.0),
                GroundTerm::from_f64(c),
            ],
        ))
    };
    kb.register_constraint("no-zzoracle-8-2-3".to_string(), vec![stored(3.0)])
        .unwrap();

    // Exercise the internal insertion primitive directly. The constraint
    // rejects the fact and strict mode must roll it out immediately.
    {
        let mut inner = kb.inner.borrow_mut();
        assert_typed_fact(stored(3.0), &mut inner);
    }

    let inner = kb.inner.borrow();
    let rejected = stored(3.0);
    let innocent = stored(9.0);
    assert!(
        !inner.fact_store.contains(&rejected),
        "the constraint-rejected fact must not remain in the store"
    );
    assert!(
        inner.fact_store.contains(&innocent),
        "the innocent same-relation fact must survive the rollback"
    );
    for (pos, val) in [(0usize, 8.0f64), (1, 2.0)] {
        let leaf = inner
            .arg_position_index
            .get(&("zzoracle".to_string(), pos))
            .expect("index position map must exist")
            .get(&GroundTerm::from_f64(val))
            .expect("shared index leaf must exist");
        assert!(
            !leaf.contains(&rejected),
            "index leaf ({pos}, {val}) must be scrubbed of the rejected fact: {leaf:?}"
        );
        assert!(
            leaf.contains(&innocent),
            "index leaf ({pos}, {val}) must retain the innocent co-leaf fact: {leaf:?}"
        );
    }
}

#[test]
fn numeric_comparison_set_matches_the_evaluator_domain() {
    // Conformance: try_numeric_comparison handles exactly
    // relations::NUMERIC_COMPARISONS (the single-source name sets) —
    // built-in arithmetic falls through to the tolerant evaluator instead.
    use nibli_types::logic::LogicalTerm;
    let subs = std::collections::HashMap::new();
    let args = vec![LogicalTerm::Number(2.0), LogicalTerm::Number(1.0)];
    for r in nibli_types::relations::NUMERIC_COMPARISONS {
        assert!(
            crate::compute::try_numeric_comparison(r, &args, &subs).is_some(),
            "{r} must be a decidable comparison"
        );
    }
    for r in nibli_types::relations::BUILTIN_ARITHMETIC {
        assert!(
            crate::compute::try_numeric_comparison(r, &args, &subs).is_none(),
            "{r} must not be treated as a comparison"
        );
    }
}

// ─── Constraint ingress shares the assertion name/comparison guards ──────

/// A constraint conjunct naming a reference external-compute relation is
/// refused: assertion ingress refuses `exponential` facts, so the constraint
/// could never match anything — inert by construction while reading as a
/// guarantee. Role spellings collapse onto the anchor exactly as at assertion
/// ingress.
#[test]
fn constraint_ingress_refuses_external_compute_names() {
    let kb = new_kb();
    for relation in ["exponential", "exponential_x1", "logarithm"] {
        let conjunct = StoredFact::Bare(GroundFact::new(
            relation,
            vec![GroundTerm::from_f64(8.0), GroundTerm::from_f64(2.0)],
        ));
        let err = kb
            .register_constraint("inert".to_string(), vec![conjunct])
            .expect_err("an external-compute conjunct must be refused");
        let msg = format!("{err:?}");
        assert!(
            msg.contains("query-only") && msg.contains("inert by construction"),
            "refusal must explain the vacuity, got: {msg}"
        );
    }
    assert!(
        kb.inner.borrow().integrity_constraints.is_empty(),
        "a refused constraint must not be registered"
    );
}

/// A constraint conjunct over an operational numeric comparison is refused —
/// with Number operands (flat and decomposed spellings) and with a
/// PatternVar operand (could bind a number at match time) — while the
/// RELATIONAL reading (`greater(Alis, Bob)`, non-numeric constants) still
/// registers: the same boundary `pins/numeric-comparison-boundary.nibli`
/// pins for assertions, so a name-based ban cannot creep in here either.
#[test]
fn constraint_ingress_refuses_operational_comparisons_but_keeps_relational() {
    let kb = new_kb();
    let refused: Vec<StoredFact> = vec![
        StoredFact::Bare(GroundFact::new(
            "greater",
            vec![GroundTerm::from_f64(20.0), GroundTerm::from_f64(15.0)],
        )),
        StoredFact::Bare(GroundFact::new(
            "less",
            vec![
                GroundTerm::PatternVar("n".to_string()),
                GroundTerm::from_f64(15.0),
            ],
        )),
        StoredFact::Bare(GroundFact::new(
            "num_equal",
            vec![
                GroundTerm::Constant("alis".to_string()),
                GroundTerm::from_f64(3.0),
            ],
        )),
        // Decomposed role spelling: the VALUE slot (args[1]) is numeric.
        StoredFact::Bare(GroundFact::new(
            "greater_x2",
            vec![
                GroundTerm::Constant("_ev0".to_string()),
                GroundTerm::from_f64(15.0),
            ],
        )),
    ];
    for conjunct in refused {
        let relation = conjunct.relation().to_string();
        let err = kb
            .register_constraint("inert".to_string(), vec![conjunct])
            .expect_err("an operational-comparison conjunct must be refused");
        let msg = format!("{err:?}");
        assert!(
            msg.contains("computed comparison") && msg.contains("inert by construction"),
            "refusal for `{relation}` must explain the vacuity, got: {msg}"
        );
    }
    assert!(
        kb.inner.borrow().integrity_constraints.is_empty(),
        "no refused constraint may be registered"
    );

    // The relational dual must keep working: non-numeric operands are an
    // ordinary fact, and the constraint over them must register AND fire.
    let relational = |who: &str| {
        StoredFact::Bare(GroundFact::new(
            "greater",
            vec![
                GroundTerm::Constant(who.to_string()),
                GroundTerm::Constant("bob".to_string()),
            ],
        ))
    };
    kb.register_constraint("taller-ban".to_string(), vec![relational("alis")])
        .expect("the relational reading must stay registrable");
    assert_eq!(
        kb.inner.borrow().integrity_constraints.len(),
        1,
        "the relational constraint must be registered"
    );
    // And it is live: strict mode rejects the matching assertion.
    kb.set_strict(true);
    {
        let mut inner = kb.inner.borrow_mut();
        assert_typed_fact(relational("alis"), &mut inner);
    }
    assert!(
        !kb.inner.borrow().fact_store.contains(&relational("alis")),
        "the registered relational constraint must actually fire under strict mode"
    );
}
