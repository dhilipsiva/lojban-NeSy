use super::*;
use std::cell::Cell;

thread_local! {
    static SCALAR_CALLS: Cell<usize> = const { Cell::new(0) };
    static BATCH_CALLS: Cell<usize> = const { Cell::new(0) };
}

fn flapping_scalar(_relation: &str, _args: &[LogicalTerm]) -> Result<bool, String> {
    Ok(SCALAR_CALLS.with(|calls| {
        let prior = calls.get();
        calls.set(prior + 1);
        prior == 0
    }))
}

fn all_true_batch(requests: &[ComputeRequest]) -> Vec<Result<bool, String>> {
    BATCH_CALLS.with(|calls| calls.set(calls.get() + 1));
    requests.iter().map(|_| Ok(true)).collect()
}

fn scalar_true(_relation: &str, _args: &[LogicalTerm]) -> Result<bool, String> {
    Ok(true)
}

fn scalar_false(_relation: &str, _args: &[LogicalTerm]) -> Result<bool, String> {
    Ok(false)
}

fn all_false_batch(requests: &[ComputeRequest]) -> Vec<Result<bool, String>> {
    BATCH_CALLS.with(|calls| calls.set(calls.get() + 1));
    requests.iter().map(|_| Ok(false)).collect()
}

fn external_compute(args: Vec<LogicalTerm>) -> LogicBuffer {
    LogicBuffer {
        nodes: vec![LogicNode::ComputeNode(("zzflap".to_string(), args))],
        roots: vec![0],
    }
}

#[test]
fn traced_query_dispatches_each_external_call_once_across_probe_and_trace() {
    SCALAR_CALLS.with(|calls| calls.set(0));
    let kb = new_kb();
    kb.set_compute_dispatch(flapping_scalar, all_true_batch);

    let (result, trace) = kb
        .query_entailment_with_proof_inner(external_compute(vec![LogicalTerm::Constant(
            "same-call".to_string(),
        )]))
        .unwrap();

    assert_eq!(result, QueryResult::True);
    assert_eq!(SCALAR_CALLS.with(Cell::get), 1);
    let root = &trace.steps[trace.root as usize];
    assert!(root.holds);
    assert!(matches!(
        &root.rule,
        ProofRule::ComputeCheck { method, detail }
            if method == "backend" && detail == "zzflap"
    ));
}

#[test]
fn count_trace_carries_the_memoized_compute_evidence_for_each_member() {
    BATCH_CALLS.with(|calls| calls.set(0));
    let kb = new_kb();
    assert_buf(&kb, make_assertion("alice", "person"));
    assert_buf(&kb, make_assertion("bob", "person"));
    kb.set_compute_dispatch(flapping_scalar, all_true_batch);

    let mut query = external_compute(vec![LogicalTerm::Variable("x".to_string())]);
    query
        .nodes
        .push(LogicNode::CountNode(("x".to_string(), 2, 0)));
    query.roots = vec![1];

    let (result, trace) = kb.query_entailment_with_proof_inner(query).unwrap();
    assert_eq!(result, QueryResult::True);
    assert_eq!(BATCH_CALLS.with(Cell::get), 1);

    let root = &trace.steps[trace.root as usize];
    assert!(matches!(
        root.rule,
        ProofRule::CountResult {
            expected: 2,
            actual: 2,
            ..
        }
    ));
    assert_eq!(root.children.len(), 2);
    for &child in &root.children {
        assert!(matches!(
            &trace.steps[child as usize],
            ProofStep {
                rule: ProofRule::ComputeCheck { method, detail },
                holds: true,
                ..
            } if method == "backend" && detail == "zzflap"
        ));
    }
}

fn two_member_kb() -> KnowledgeBase {
    let kb = new_kb();
    assert_buf(&kb, make_assertion("alice", "person"));
    assert_buf(&kb, make_assertion("bob", "person"));
    kb
}

fn quantified_external_compute(kind: &str, expected: u32) -> LogicBuffer {
    let mut query = external_compute(vec![LogicalTerm::Variable("x".to_string())]);
    let root = match kind {
        "exists" => {
            query
                .nodes
                .push(LogicNode::ExistsNode(("x".to_string(), 0)));
            1
        }
        "count" => {
            query
                .nodes
                .push(LogicNode::CountNode(("x".to_string(), expected, 0)));
            1
        }
        _ => unreachable!("unsupported quantified compute fixture"),
    };
    query.roots = vec![root];
    query
}

#[test]
fn false_exists_records_every_compute_decision_and_is_not_cwa_false() {
    BATCH_CALLS.with(|calls| calls.set(0));
    let kb = two_member_kb();
    kb.set_compute_dispatch(scalar_false, all_false_batch);

    let (result, trace) = kb
        .query_entailment_with_proof_inner(quantified_external_compute("exists", 0))
        .unwrap();
    assert_eq!(result, QueryResult::False);
    assert!(!trace.cwa_false, "all candidates were refuted by compute");
    assert_eq!(BATCH_CALLS.with(Cell::get), 1);

    let root = &trace.steps[trace.root as usize];
    assert!(matches!(root.rule, ProofRule::ExistsFailed));
    assert_eq!(root.children.len(), 2);
    for child in &root.children {
        assert!(matches!(
            &trace.steps[*child as usize],
            ProofStep {
                rule: ProofRule::ComputeCheck { method, detail },
                holds: false,
                ..
            } if method == "backend" && detail == "zzflap"
        ));
    }
}

#[test]
fn count_mismatch_uses_structural_compute_evidence_in_both_directions() {
    for (expected, scalar, batch, expected_actual) in [
        (0, scalar_true as EvalFn, all_true_batch as BatchEvalFn, 2),
        (2, scalar_false as EvalFn, all_false_batch as BatchEvalFn, 0),
    ] {
        let kb = two_member_kb();
        kb.set_compute_dispatch(scalar, batch);
        let (result, trace) = kb
            .query_entailment_with_proof_inner(quantified_external_compute("count", expected))
            .unwrap();
        assert_eq!(result, QueryResult::False);
        assert!(
            !trace.cwa_false,
            "expected {expected}, actual {expected_actual}: mismatch was compute-decided"
        );
        let root = &trace.steps[trace.root as usize];
        assert!(matches!(
            root.rule,
            ProofRule::CountResult { actual, .. } if actual == expected_actual
        ));
        assert_eq!(root.children.len(), 2);
    }
}

fn composed_external_query(conjunction: bool) -> LogicBuffer {
    let mut nodes = Vec::new();
    let compute = compute(
        &mut nodes,
        "zzflap",
        vec![LogicalTerm::Constant("alice".to_string())],
    );
    let missing = pred(
        &mut nodes,
        "missing",
        vec![LogicalTerm::Constant("alice".to_string())],
    );
    let root = if conjunction {
        and(&mut nodes, compute, missing)
    } else {
        or(&mut nodes, compute, missing)
    };
    LogicBuffer {
        nodes,
        roots: vec![root],
    }
}

#[test]
fn cwa_false_is_determined_by_formula_structure_not_a_global_compute_scan() {
    let kb = new_kb();
    kb.set_compute_dispatch(scalar_false, all_false_batch);

    let (or_result, or_trace) = kb
        .query_entailment_with_proof_inner(composed_external_query(false))
        .unwrap();
    assert_eq!(or_result, QueryResult::False);
    assert!(
        or_trace.cwa_false,
        "compute false OR a closed-world miss becomes Unknown under OWA"
    );

    let (and_result, and_trace) = kb
        .query_entailment_with_proof_inner(composed_external_query(true))
        .unwrap();
    assert_eq!(and_result, QueryResult::False);
    assert!(
        !and_trace.cwa_false,
        "the compute-false left conjunct decides the conjunction without CWA"
    );

    kb.set_compute_dispatch(scalar_true, all_true_batch);
    let (cwa_and_result, cwa_and_trace) = kb
        .query_entailment_with_proof_inner(composed_external_query(true))
        .unwrap();
    assert_eq!(cwa_and_result, QueryResult::False);
    assert!(
        cwa_and_trace.cwa_false,
        "compute true AND a missing predicate still rests on CWA"
    );
}

fn double_negated_leaf(compute_leaf: bool) -> LogicBuffer {
    let mut nodes = Vec::new();
    let leaf = if compute_leaf {
        compute(
            &mut nodes,
            "zzflap",
            vec![LogicalTerm::Constant("alice".to_string())],
        )
    } else {
        pred(
            &mut nodes,
            "missing",
            vec![LogicalTerm::Constant("alice".to_string())],
        )
    };
    let inner = not(&mut nodes, leaf);
    let root = not(&mut nodes, inner);
    LogicBuffer {
        nodes,
        roots: vec![root],
    }
}

#[test]
fn nested_negation_preserves_definitive_compute_false_but_not_cwa_absence() {
    let kb = new_kb();
    kb.set_compute_dispatch(scalar_false, all_false_batch);

    let (compute_result, compute_trace) = kb
        .query_entailment_with_proof_inner(double_negated_leaf(true))
        .unwrap();
    assert_eq!(compute_result, QueryResult::False);
    assert!(
        !compute_trace.cwa_false,
        "double negation does not erase a definitive compute false"
    );

    let (missing_result, missing_trace) = kb
        .query_entailment_with_proof_inner(double_negated_leaf(false))
        .unwrap();
    assert_eq!(missing_result, QueryResult::False);
    assert!(
        missing_trace.cwa_false,
        "double negation over an ordinary absence remains CWA-dependent"
    );
}
