use super::*;

// ─── Zoo Ontology integration tests (REPL demo scenarios) ───────

#[test]
fn test_zoo_multi_hop_temporal_past() {
    // REPL: pu la .alis. gerku → ro lo gerku cu danlu → ro lo danlu cu jmive
    // Query: ?! pu la .alis. jmive → TRUE
    let kb = new_kb();
    assert_buf(&kb, make_temporal_event_assertion("alis", "gerku", past));
    assert_buf(&kb, make_temporal_event_universal("gerku", "danlu", past));
    assert_buf(&kb, make_temporal_event_universal("danlu", "jmive", past));
    assert!(
        query(&kb, make_temporal_event_query("alis", "jmive", past)),
        "multi-hop temporal: Past(gerku→danlu→jmive) should derive Past(jmive(alis))"
    );
}

#[test]
fn test_zoo_multi_hop_temporal_present() {
    // REPL: ca la .bob. mlatu → ro lo mlatu cu danlu → ro lo danlu cu jmive
    // Query: ?! ca la .bob. jmive → TRUE
    let kb = new_kb();
    assert_buf(&kb, make_temporal_event_assertion("bob", "mlatu", present));
    assert_buf(
        &kb,
        make_temporal_event_universal("mlatu", "danlu", present),
    );
    assert_buf(
        &kb,
        make_temporal_event_universal("danlu", "jmive", present),
    );
    assert!(
        query(&kb, make_temporal_event_query("bob", "jmive", present)),
        "multi-hop temporal: Present(mlatu→danlu→jmive) should derive Present(jmive(bob))"
    );
}

#[test]
fn test_zoo_tense_discrimination() {
    // Assert Past(gerku(alis)), derive Past(jmive(alis))
    // Query Present(jmive(alis)) → FALSE (strict tense discrimination)
    let kb = new_kb();
    assert_buf(&kb, make_temporal_event_assertion("alis", "gerku", past));
    assert_buf(&kb, make_temporal_event_universal("gerku", "danlu", past));
    assert_buf(&kb, make_temporal_event_universal("danlu", "jmive", past));

    // Past query should succeed
    assert!(
        query(&kb, make_temporal_event_query("alis", "jmive", past)),
        "Past(jmive(alis)) should hold"
    );
    // Present query should FAIL — alice was a dog in the past, not present
    assert!(
        query_false(&kb, make_temporal_event_query("alis", "jmive", present)),
        "Present(jmive(alis)) should NOT hold — wrong tense"
    );
}

#[test]
fn test_zoo_mixed_tenses() {
    // REPL demo: two entities with different tenses
    // pu la .alis. gerku + ca la .bob. mlatu + rules
    let kb = new_kb();
    assert_buf(&kb, make_temporal_event_assertion("alis", "gerku", past));
    assert_buf(&kb, make_temporal_event_assertion("bob", "mlatu", present));
    assert_buf(&kb, make_temporal_event_universal("gerku", "danlu", past));
    assert_buf(
        &kb,
        make_temporal_event_universal("mlatu", "danlu", present),
    );
    assert_buf(&kb, make_temporal_event_universal("danlu", "jmive", past));
    assert_buf(
        &kb,
        make_temporal_event_universal("danlu", "jmive", present),
    );

    // Each entity derivable only in its own tense
    assert!(query(&kb, make_temporal_event_query("alis", "jmive", past)));
    assert!(query(
        &kb,
        make_temporal_event_query("bob", "jmive", present)
    ));
    // Cross-tense queries fail
    assert!(query_false(
        &kb,
        make_temporal_event_query("alis", "jmive", present)
    ));
    assert!(query_false(
        &kb,
        make_temporal_event_query("bob", "jmive", past)
    ));
}

#[test]
fn test_zoo_witness_extraction_event_decomposed() {
    // REPL: ?? ma danlu — find witnesses after event-decomposed derivation
    let kb = new_kb();
    assert_buf(&kb, make_event_assertion("alis", "gerku"));
    assert_buf(&kb, make_event_assertion("bob", "gerku"));
    assert_buf(&kb, make_event_universal("gerku", "danlu"));

    // Query: ∃_v0. ∃_ev0. danlu(_ev0) ∧ danlu_x1(_ev0, _v0)
    let mut q_nodes = Vec::new();
    let q_type = pred(
        &mut q_nodes,
        "danlu",
        vec![LogicalTerm::Variable("_ev0".to_string())],
    );
    let q_role = pred(
        &mut q_nodes,
        "danlu_x1",
        vec![
            LogicalTerm::Variable("_ev0".to_string()),
            LogicalTerm::Variable("_v0".to_string()),
        ],
    );
    let q_and = and(&mut q_nodes, q_type, q_role);
    let q_ev = exists(&mut q_nodes, "_ev0", q_and);
    let q_root = exists(&mut q_nodes, "_v0", q_ev);
    let results = query_find(
        &kb,
        LogicBuffer {
            nodes: q_nodes,
            roots: vec![q_root],
        },
    );

    // Should find witnesses for both alis and bob
    assert!(
        !results.is_empty(),
        "should find witnesses for danlu after event-decomposed derivation"
    );
    // Extract entity bindings (filter to _v0 which is the entity variable)
    let entity_witnesses: Vec<String> = results
        .iter()
        .filter_map(|bindings| {
            bindings.iter().find_map(|b| {
                if b.variable == "_v0" {
                    match &b.term {
                        LogicalTerm::Constant(c) => Some(c.clone()),
                        _ => None,
                    }
                } else {
                    None
                }
            })
        })
        .collect();
    assert!(
        entity_witnesses.contains(&"alis".to_string()),
        "alis should be a witness for danlu"
    );
    assert!(
        entity_witnesses.contains(&"bob".to_string()),
        "bob should be a witness for danlu"
    );
}

#[test]
fn test_zoo_retraction_with_event_decomposition() {
    // REPL demo: retract alice's fact, bob should survive
    let kb = new_kb();
    assert_buf(&kb, make_temporal_event_universal("gerku", "danlu", past));
    assert_buf(
        &kb,
        make_temporal_event_universal("mlatu", "danlu", present),
    );
    assert_buf(&kb, make_temporal_event_universal("danlu", "jmive", past));
    assert_buf(
        &kb,
        make_temporal_event_universal("danlu", "jmive", present),
    );

    let alis_id = kb
        .assert_fact_inner(
            make_temporal_event_assertion("alis", "gerku", past),
            "past dog(Alis).".into(),
        )
        .unwrap();
    let _bob_id = kb
        .assert_fact_inner(
            make_temporal_event_assertion("bob", "mlatu", present),
            "now cat(Bob).".into(),
        )
        .unwrap();

    // Both should hold before retraction
    assert!(query(&kb, make_temporal_event_query("alis", "jmive", past)));
    assert!(query(
        &kb,
        make_temporal_event_query("bob", "jmive", present)
    ));

    // Retract alice's assertion
    kb.retract_fact_inner(alis_id).unwrap();

    // Alice's derivation should be gone
    assert!(
        query_false(&kb, make_temporal_event_query("alis", "jmive", past)),
        "after retracting alis's gerku fact, Past(jmive(alis)) should be FALSE"
    );
    // Bob's derivation should still hold
    assert!(
        query(&kb, make_temporal_event_query("bob", "jmive", present)),
        "bob's Present(jmive(bob)) should still hold after retracting alis"
    );
}

#[test]
fn test_zoo_proof_trace_multi_hop_temporal() {
    // Proof trace for a multi-hop, explicitly temporal derivation.
    let kb = new_kb();
    assert_buf(&kb, make_temporal_event_assertion("alis", "gerku", past));
    assert_buf(&kb, make_temporal_event_universal("gerku", "danlu", past));
    assert_buf(&kb, make_temporal_event_universal("danlu", "jmive", past));

    let (holds, trace) = kb
        .query_entailment_with_proof_inner(make_temporal_event_query("alis", "jmive", past))
        .unwrap();
    assert!(
        holds.is_true(),
        "Past(jmive(alis)) should hold with proof trace"
    );

    // Proof should contain Derived steps (from rule application)
    let derived_count = trace
        .steps
        .iter()
        .filter(|step| matches!(&step.rule, ProofRule::Derived { .. }))
        .count();
    assert!(
        derived_count >= 2,
        "multi-hop derivation should have at least 2 Derived steps (gerku→danlu, danlu→jmive), got {}",
        derived_count
    );

    // Proof should contain a ModalPassthrough for past tense
    let has_modal = trace
        .steps
        .iter()
        .any(|step| matches!(&step.rule, ProofRule::ModalPassthrough { kind: t } if t == "past"));
    assert!(
        has_modal,
        "proof trace should contain a ModalPassthrough(past) step"
    );
}

#[test]
fn explicit_temporal_rule_trace_has_no_synthetic_lift_label() {
    // The rule itself carries Past on both sides. Its ordinary rule label and
    // ModalPassthrough step are sufficient; no synthetic `[past]` lift marker is
    // permitted because no implicit lift occurred.
    let kb = new_kb();
    assert_buf(&kb, make_temporal_event_assertion("alis", "gerku", past));
    assert_buf(&kb, make_temporal_event_universal("gerku", "danlu", past));

    let (holds, trace) = kb
        .query_entailment_with_proof_inner(make_temporal_event_query("alis", "danlu", past))
        .unwrap();
    assert!(holds.is_true(), "Past(danlu(alis)) should hold");

    let derived_labels = trace
        .steps
        .iter()
        .filter_map(|s| match &s.rule {
            ProofRule::Derived { label, .. } => Some(label.clone()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert!(
        !derived_labels.is_empty(),
        "the explicit temporal rule should emit a Derived step"
    );
    assert!(
        derived_labels.iter().all(|label| !label.contains("[past]")),
        "explicit temporal rules must not masquerade as implicit lifts: {derived_labels:?}"
    );
}

#[test]
fn cross_flavor_rule_trace_discloses_both_wrappers() {
    let kb = new_kb();
    assert_buf(&kb, make_temporal_event_assertion("alis", "gerku", past));
    assert_buf(
        &kb,
        make_temporal_event_mapping("gerku", "danlu", past, present),
    );

    let (holds, trace) = kb
        .query_entailment_with_proof_inner(make_temporal_event_query("alis", "danlu", present))
        .unwrap();
    assert!(holds.is_true(), "Past gerku must derive Present danlu");

    assert!(
        trace.steps.iter().any(
            |step| matches!(&step.rule, ProofRule::Asserted { fact } if fact.starts_with("Past("))
        ),
        "the premise must remain visibly Past in the proof: {:?}",
        trace.steps
    );
    assert!(
        trace.steps.iter().any(
            |step| matches!(&step.rule, ProofRule::ModalPassthrough { kind } if kind == "present")
        ),
        "the query must disclose its Present wrapper: {:?}",
        trace.steps
    );
    let labels = trace
        .steps
        .iter()
        .filter_map(|step| match &step.rule {
            ProofRule::Derived { label, .. } => Some(label.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert!(!labels.is_empty(), "the declared rule label must appear");
    assert!(
        labels.iter().all(|label| !label.contains('[')),
        "cross-flavor rules use their ordinary labels, not synthetic lift labels: {labels:?}"
    );
}

#[test]
fn bare_rule_is_absent_from_a_tensed_failure_trace() {
    let kb = new_kb();
    kb.set_materialization(false);
    assert_buf(&kb, make_event_universal("gerku", "danlu"));
    assert_buf(&kb, make_temporal_event_assertion("alis", "gerku", past));

    let (holds, trace) = kb
        .query_entailment_with_proof_inner(make_temporal_event_query("alis", "danlu", past))
        .unwrap();
    assert_eq!(holds, QueryResult::False);
    assert!(
        trace
            .steps
            .iter()
            .all(|step| !matches!(step.rule, ProofRule::RuleAttemptFailed { .. })),
        "a Bare rule is not a candidate for a Past goal: {:?}",
        trace.steps
    );
}
