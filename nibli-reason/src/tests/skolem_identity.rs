use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};

fn existential_pair_assertion() -> LogicBuffer {
    let mut nodes = Vec::new();
    let p = pred(
        &mut nodes,
        "p",
        vec![LogicalTerm::Variable("x".to_string())],
    );
    let q = pred(
        &mut nodes,
        "q",
        vec![LogicalTerm::Variable("x".to_string())],
    );
    let body = and(&mut nodes, p, q);
    let root = exists(&mut nodes, "x", body);
    LogicBuffer {
        nodes,
        roots: vec![root],
    }
}

fn unary_user_assertion(relation: &str, constant: &str) -> LogicBuffer {
    let mut nodes = Vec::new();
    let root = pred(
        &mut nodes,
        relation,
        vec![LogicalTerm::Constant(constant.to_string())],
    );
    LogicBuffer {
        nodes,
        roots: vec![root],
    }
}

fn unary_find(relation: &str) -> LogicBuffer {
    let mut nodes = Vec::new();
    let body = pred(
        &mut nodes,
        relation,
        vec![LogicalTerm::Variable("x".to_string())],
    );
    let root = exists(&mut nodes, "x", body);
    LogicBuffer {
        nodes,
        roots: vec![root],
    }
}

fn dependent_find(relation: &str) -> LogicBuffer {
    let mut nodes = Vec::new();
    let body = pred(
        &mut nodes,
        relation,
        vec![
            LogicalTerm::Constant("adam".to_string()),
            LogicalTerm::Variable("y".to_string()),
            LogicalTerm::Unspecified,
        ],
    );
    let root = exists(&mut nodes, "y", body);
    LogicBuffer {
        nodes,
        roots: vec![root],
    }
}

fn dependent_user_assertion(relation: &str, value: &str) -> LogicBuffer {
    let mut nodes = Vec::new();
    let root = pred(
        &mut nodes,
        relation,
        vec![
            LogicalTerm::Constant("adam".to_string()),
            LogicalTerm::Constant(value.to_string()),
            LogicalTerm::Unspecified,
        ],
    );
    LogicBuffer {
        nodes,
        roots: vec![root],
    }
}

fn assert_identity_matrix(kb: &KnowledgeBase) {
    let found = query_find(kb, unary_find("p"));
    assert_eq!(
        found.len(),
        2,
        "internal and user terms must both enumerate"
    );
    let observed: Vec<_> = found
        .iter()
        .map(|set| {
            assert_eq!(set.len(), 1);
            (set[0].origin, set[0].term.clone())
        })
        .collect();
    assert!(observed.iter().any(|(origin, term)| {
        *origin == nibli_types::logic::WitnessOrigin::GeneratedWitness
            && matches!(term, LogicalTerm::Constant(label) if label == "sk_0")
    }));
    assert!(observed.iter().any(|(origin, term)| {
        *origin == nibli_types::logic::WitnessOrigin::KnowledgeBase
            && matches!(term, LogicalTerm::Constant(label) if label == "sk_0" || label == "adam")
    }));
    assert_eq!(kb.count_witnesses(unary_find("p")).unwrap(), 2);

    let mut nodes = Vec::new();
    let body = pred(
        &mut nodes,
        "p",
        vec![LogicalTerm::Variable("x".to_string())],
    );
    let root = nodes.len() as u32;
    nodes.push(LogicNode::CountNode(("x".to_string(), 2, body)));
    assert!(query(
        kb,
        LogicBuffer {
            nodes,
            roots: vec![root],
        }
    ));

    assert!(query_false(kb, unary_user_assertion("q", "sk_0")));
    assert!(query_false(kb, unary_user_assertion("q", "adam")));
}

#[test]
fn user_sk_0_and_generated_skolem_stay_distinct_through_equality_find_count_and_rebuild() {
    let kb = new_kb();
    let generated_id = assert_id(&kb, existential_pair_assertion(), "generated");
    let (generated_result, generated_trace) = query_with_proof(&kb, unary_find("p"));
    assert!(generated_result);
    assert!(matches!(
        generated_trace.steps[generated_trace.root as usize].rule,
        ProofRule::ExistsWitness {
            origin: nibli_types::logic::WitnessOrigin::GeneratedWitness,
            ..
        }
    ));
    let user_id = assert_id(&kb, unary_user_assertion("p", "sk_0"), "user");
    let equality_id = assert_id(&kb, make_equals("sk_0", "adam"), "user equality");

    assert_identity_matrix(&kb);

    kb.retract_fact_inner(equality_id).unwrap();
    assert_identity_matrix(&kb);

    kb.retract_fact_inner(user_id).unwrap();
    let generated_only = query_find(&kb, unary_find("p"));
    assert_eq!(generated_only.len(), 1);
    assert_eq!(
        generated_only[0][0].origin,
        nibli_types::logic::WitnessOrigin::GeneratedWitness
    );

    let replacement_user_id = assert_id(&kb, unary_user_assertion("p", "sk_0"), "user again");
    assert_identity_matrix(&kb);

    kb.retract_fact_inner(generated_id).unwrap();
    let user_only = query_find(&kb, unary_find("p"));
    assert_eq!(user_only.len(), 1);
    assert_eq!(
        user_only[0][0].origin,
        nibli_types::logic::WitnessOrigin::KnowledgeBase
    );
    kb.retract_fact_inner(replacement_user_id).unwrap();
    assert!(query_find(&kb, unary_find("p")).is_empty());
}

#[test]
fn forall_proof_preserves_origin_for_equal_looking_entities() {
    let kb = new_kb();
    assert_buf(&kb, existential_pair_assertion());
    assert_buf(&kb, unary_user_assertion("p", "sk_0"));

    let mut nodes = Vec::new();
    let body = pred(
        &mut nodes,
        "p",
        vec![LogicalTerm::Variable("x".to_string())],
    );
    let root = forall(&mut nodes, "x", body);
    let (result, trace) = query_with_proof(
        &kb,
        LogicBuffer {
            nodes,
            roots: vec![root],
        },
    );
    assert!(result);
    let ProofRule::ForallVerified { entities } = &trace.steps[trace.root as usize].rule else {
        panic!(
            "expected a forall proof, got {:?}",
            trace.steps[trace.root as usize].rule
        );
    };
    assert_eq!(entities.len(), 2);
    assert!(
        entities.iter().all(
            |binding| matches!(&binding.term, LogicalTerm::Constant(label) if label == "sk_0")
        )
    );
    assert!(
        entities
            .iter()
            .any(|binding| binding.origin == nibli_types::logic::WitnessOrigin::GeneratedWitness)
    );
    assert!(
        entities
            .iter()
            .any(|binding| binding.origin == nibli_types::logic::WitnessOrigin::KnowledgeBase)
    );
}

#[test]
fn equal_looking_user_event_cannot_complete_generated_event_roles() {
    let kb = new_kb();

    let mut generated_nodes = Vec::new();
    let generated_body = pred(
        &mut generated_nodes,
        "happens",
        vec![LogicalTerm::Variable("_ev".to_string())],
    );
    let generated_root = exists(&mut generated_nodes, "_ev", generated_body);
    assert_buf(
        &kb,
        LogicBuffer {
            nodes: generated_nodes,
            roots: vec![generated_root],
        },
    );

    let mut role_nodes = Vec::new();
    let role_root = pred(
        &mut role_nodes,
        "happens_x1",
        vec![
            LogicalTerm::Constant("sk_0".to_string()),
            LogicalTerm::Constant("bob".to_string()),
        ],
    );
    assert_buf(
        &kb,
        LogicBuffer {
            nodes: role_nodes,
            roots: vec![role_root],
        },
    );

    let mut query_nodes = Vec::new();
    let head = pred(
        &mut query_nodes,
        "happens",
        vec![LogicalTerm::Variable("ev".to_string())],
    );
    let role = pred(
        &mut query_nodes,
        "happens_x1",
        vec![
            LogicalTerm::Variable("ev".to_string()),
            LogicalTerm::Constant("bob".to_string()),
        ],
    );
    let joined = and(&mut query_nodes, head, role);
    let root = exists(&mut query_nodes, "ev", joined);
    assert!(query_false(
        &kb,
        LogicBuffer {
            nodes: query_nodes,
            roots: vec![root],
        }
    ));
}

static COMPUTE_CALLS: AtomicUsize = AtomicUsize::new(0);

fn recording_compute(_relation: &str, args: &[LogicalTerm]) -> Result<bool, String> {
    COMPUTE_CALLS.fetch_add(1, Ordering::SeqCst);
    Ok(matches!(args, [LogicalTerm::Constant(value)] if value == "sk_0"))
}

fn recording_batch(requests: &[ComputeRequest]) -> Vec<Result<bool, String>> {
    COMPUTE_CALLS.fetch_add(requests.len(), Ordering::SeqCst);
    requests.iter().map(|_| Ok(true)).collect()
}

#[test]
fn compute_dispatch_refuses_internal_witness_but_accepts_user_sk_0() {
    COMPUTE_CALLS.store(0, Ordering::SeqCst);
    let kb = new_kb();
    kb.set_compute_dispatch(recording_compute, recording_batch);
    assert_buf(&kb, existential_pair_assertion());

    let mut generated_nodes = Vec::new();
    let compute = generated_nodes.len() as u32;
    generated_nodes.push(LogicNode::ComputeNode((
        "external".to_string(),
        vec![LogicalTerm::Variable("x".to_string())],
    )));
    let generated_root = exists(&mut generated_nodes, "x", compute);
    let generated = kb
        .query_entailment_inner(LogicBuffer {
            nodes: generated_nodes,
            roots: vec![generated_root],
        })
        .unwrap();
    assert!(matches!(
        generated,
        QueryResult::Unknown(UnknownReason::BackendUnavailable)
    ));
    assert_eq!(COMPUTE_CALLS.load(Ordering::SeqCst), 0);

    let user_query = LogicBuffer {
        nodes: vec![LogicNode::ComputeNode((
            "external".to_string(),
            vec![LogicalTerm::Constant("sk_0".to_string())],
        ))],
        roots: vec![0],
    };
    assert!(kb.query_entailment_inner(user_query).unwrap().is_true());
    assert_eq!(COMPUTE_CALLS.load(Ordering::SeqCst), 1);
}

#[test]
fn user_skolem_fn_spelling_stays_distinct_from_dependent_skolem_identity() {
    let kb = new_kb();
    assert_id(
        &kb,
        make_dependent_skolem_universal("p", "q"),
        "dependent rule",
    );
    assert_id(&kb, make_assertion("adam", "p"), "restrictor");

    let (generated_result, generated_trace) = query_with_proof(&kb, dependent_find("q"));
    assert!(generated_result);
    assert!(matches!(
        generated_trace.steps[generated_trace.root as usize].rule,
        ProofRule::ExistsWitness {
            term: LogicalTerm::Constant(ref label),
            origin: nibli_types::logic::WitnessOrigin::GeneratedWitness,
            ..
        } if label == "sk_0(adam)"
    ));

    let user_id = assert_id(
        &kb,
        dependent_user_assertion("q", "sk_0(adam)"),
        "equal-looking user function spelling",
    );
    let found = query_find(&kb, dependent_find("q"));
    assert_eq!(found.len(), 2);
    assert!(found.iter().any(|set| {
        set.iter().any(|binding| {
            binding.variable == "y"
                && binding.origin == nibli_types::logic::WitnessOrigin::GeneratedWitness
        })
    }));
    assert!(found.iter().any(|set| {
        set.iter().any(|binding| {
            binding.variable == "y"
                && binding.origin == nibli_types::logic::WitnessOrigin::KnowledgeBase
        })
    }));
    assert_eq!(kb.count_witnesses(dependent_find("q")).unwrap(), 2);

    assert!(query(&kb, dependent_user_assertion("q", "sk_0(adam)")));
    kb.retract_fact_inner(user_id).unwrap();
    assert!(query_false(
        &kb,
        dependent_user_assertion("q", "sk_0(adam)")
    ));
    assert!(query(&kb, dependent_find("q")));
}

#[test]
fn proof_memo_uses_structural_fact_identity_not_equal_display_text() {
    let kb = new_kb();
    let first = StoredFact::Bare(GroundFact::new(
        "p",
        vec![GroundTerm::Skolem(SkolemSymbol::new(
            10,
            0,
            0,
            SkolemSort::Individual,
            SkolemOrigin::Generated,
        ))],
    ));
    let second = StoredFact::Bare(GroundFact::new(
        "p",
        vec![GroundTerm::Skolem(SkolemSymbol::new(
            11,
            0,
            0,
            SkolemSort::Individual,
            SkolemOrigin::Generated,
        ))],
    ));
    assert_eq!(first.to_display_string(), second.to_display_string());

    let mut inner = kb.inner.borrow_mut();
    assert_typed_fact(first.clone(), &mut inner);
    assert_typed_fact(second.clone(), &mut inner);
    let mut steps = Vec::new();
    let mut memo = HashMap::new();
    let mut visited = HashSet::new();
    trace_predicate_provenance_typed(&first, &inner, &mut steps, 0, &mut memo, &mut visited);
    trace_predicate_provenance_typed(&second, &inner, &mut steps, 0, &mut memo, &mut visited);
    assert_eq!(memo.len(), 2);
    assert!(matches!(steps[0].rule, ProofRule::Asserted { .. }));
    assert!(matches!(steps[1].rule, ProofRule::Asserted { .. }));
}

#[test]
fn negative_group_bridge_never_retypes_internal_skolem_as_user_constant() {
    let user = StoredFact::Bare(GroundFact::new(
        "p",
        vec![GroundTerm::Constant("sk_0".to_string())],
    ));
    let user_buffer = negative_group_to_query_buffer(&[user])
        .expect("a genuine user constant must round-trip through the query bridge");
    assert!(matches!(
        user_buffer.nodes.first(),
        Some(LogicNode::Predicate((relation, args)))
            if relation == "p"
                && args == &vec![LogicalTerm::Constant("sk_0".to_string())]
    ));

    let internal = StoredFact::Bare(GroundFact::new(
        "p",
        vec![GroundTerm::Skolem(SkolemSymbol::new(
            1,
            0,
            0,
            SkolemSort::Individual,
            SkolemOrigin::Generated,
        ))],
    ));
    assert!(
        negative_group_to_query_buffer(&[internal]).is_none(),
        "a typed internal identity has no forgeable LogicalTerm::Constant encoding"
    );
}

#[test]
fn duplicate_assertion_id_is_rejected_before_skolem_identity_can_be_reused() {
    let kb = new_kb();
    kb.assert_fact_with_id(existential_pair_assertion(), "first".to_string(), 7)
        .unwrap();

    let mut nodes = Vec::new();
    let r = pred(
        &mut nodes,
        "r",
        vec![LogicalTerm::Variable("x".to_string())],
    );
    let r = exists(&mut nodes, "x", r);
    let duplicate = LogicBuffer {
        nodes,
        roots: vec![r],
    };
    let err = kb
        .assert_fact_with_id(duplicate, "duplicate".to_string(), 7)
        .expect_err("a fact id must never identify two assertion sources");
    assert!(err.contains("cannot be reused"), "unexpected error: {err}");

    assert!(query(&kb, unary_find("p")));
    assert!(query_false(&kb, unary_user_assertion("r", "sk_0")));
    let facts = kb.list_facts_inner().unwrap();
    assert_eq!(facts.len(), 1);
    assert_eq!(facts[0].label, "first");
}

#[test]
fn assertion_id_allocator_fails_closed_at_u64_boundary_without_wrapping() {
    let kb = new_kb();
    let err = kb
        .assert_fact_with_id(
            existential_pair_assertion(),
            "impossible max id".to_string(),
            u64::MAX,
        )
        .expect_err("u64::MAX has no safe monotonic successor");
    assert!(err.contains("no collision-free successor"));
    assert!(kb.list_facts_inner().unwrap().is_empty());

    let id = kb
        .assert_fact(existential_pair_assertion(), "ordinary".to_string())
        .unwrap();
    assert_eq!(
        id, 0,
        "a rejected max id must not advance or wrap the allocator"
    );

    let exhausted = new_kb();
    exhausted.inner.borrow_mut().fact_counter = u64::MAX;
    let err = exhausted
        .assert_fact(existential_pair_assertion(), "exhausted".to_string())
        .expect_err("fresh allocation must fail rather than wrap to zero");
    assert!(err.to_string().contains("fact-id space exhausted"));
    assert!(exhausted.list_facts_inner().unwrap().is_empty());
}
