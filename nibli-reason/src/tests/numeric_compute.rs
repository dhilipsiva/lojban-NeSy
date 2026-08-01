use super::*;

// ─── Numeric Comparison Tests ────────────────────────────────

#[test]
fn test_greater_numeric_true() {
    let kb = new_kb();
    assert!(query(&kb, make_numeric_query("greater", 2.0, 1.0)));
}

#[test]
fn test_greater_numeric_false() {
    let kb = new_kb();
    assert!(query_false(&kb, make_numeric_query("greater", 1.0, 2.0)));
}

#[test]
fn test_greater_numeric_equal_false() {
    let kb = new_kb();
    assert!(query_false(&kb, make_numeric_query("greater", 2.0, 2.0)));
}

#[test]
fn test_less_numeric_true() {
    let kb = new_kb();
    assert!(query(&kb, make_numeric_query("less", 1.0, 2.0)));
}

#[test]
fn test_less_numeric_false() {
    let kb = new_kb();
    assert!(query_false(&kb, make_numeric_query("less", 2.0, 1.0)));
}

#[test]
fn test_num_equal_numeric_true() {
    let kb = new_kb();
    assert!(query(&kb, make_numeric_query("num_equal", 5.0, 5.0)));
}

#[test]
fn test_num_equal_numeric_false() {
    let kb = new_kb();
    assert!(query_false(&kb, make_numeric_query("num_equal", 5.0, 3.0)));
}

#[test]
fn test_greater_negated() {
    let kb = new_kb();
    // NOT (1 > 2) should be TRUE
    let mut nodes = Vec::new();
    let cmp = make_numeric_pred(&mut nodes, "greater", 1.0, 2.0);
    let root = not(&mut nodes, cmp);
    assert!(query(
        &kb,
        LogicBuffer {
            nodes,
            roots: vec![root]
        }
    ));
}

#[test]
fn test_greater_non_numeric_fallback() {
    let kb = new_kb();
    // Non-numeric zmadu: assert then query via standard KB path
    let mut a_nodes = Vec::new();
    let a_root = pred(
        &mut a_nodes,
        "greater",
        vec![
            LogicalTerm::Constant("alis".to_string()),
            LogicalTerm::Constant("bob".to_string()),
            LogicalTerm::Unspecified,
            LogicalTerm::Unspecified,
        ],
    );
    assert_buf(
        &kb,
        LogicBuffer {
            nodes: a_nodes,
            roots: vec![a_root],
        },
    );

    let mut q_nodes = Vec::new();
    let q_root = pred(
        &mut q_nodes,
        "greater",
        vec![
            LogicalTerm::Constant("alis".to_string()),
            LogicalTerm::Constant("bob".to_string()),
            LogicalTerm::Unspecified,
            LogicalTerm::Unspecified,
        ],
    );
    assert!(query(
        &kb,
        LogicBuffer {
            nodes: q_nodes,
            roots: vec![q_root]
        }
    ));
}

#[test]
fn test_greater_large_numbers() {
    let kb = new_kb();
    assert!(query(
        &kb,
        make_numeric_query("greater", 1_000_000.0, 999_999.0)
    ));
}

#[test]
fn test_greater_negative_numbers() {
    let kb = new_kb();
    assert!(query(&kb, make_numeric_query("greater", -1.0, -2.0)));
    assert!(query_false(&kb, make_numeric_query("greater", -2.0, -1.0)));
}

// ─── ComputeNode Tests ───────────────────────────────────────

#[test]
fn test_compute_pilji_true() {
    let kb = new_kb();
    // 6 = 2 * 3
    assert!(query(&kb, make_compute_query("product", 6.0, 2.0, 3.0)));
}

#[test]
fn test_compute_pilji_false() {
    let kb = new_kb();
    // 7 != 2 * 3
    assert!(query_false(
        &kb,
        make_compute_query("product", 7.0, 2.0, 3.0)
    ));
}

#[test]
fn test_compute_sumji_true() {
    let kb = new_kb();
    // 5 = 2 + 3
    assert!(query(&kb, make_compute_query("sum", 5.0, 2.0, 3.0)));
}

#[test]
fn test_compute_sumji_false() {
    let kb = new_kb();
    // 4 != 2 + 3
    assert!(query_false(&kb, make_compute_query("sum", 4.0, 2.0, 3.0)));
}

#[test]
fn test_compute_dilcu_true() {
    let kb = new_kb();
    // 3 = 6 / 2
    assert!(query(&kb, make_compute_query("quotient", 3.0, 6.0, 2.0)));
}

#[test]
fn test_compute_dilcu_division_by_zero() {
    let kb = new_kb();
    // x / 0 is always false
    assert!(query_false(
        &kb,
        make_compute_query("quotient", 0.0, 5.0, 0.0)
    ));
}

#[test]
fn test_compute_sumji_float_tolerance() {
    let kb = new_kb();
    // 0.1 + 0.2 = 0.30000000000000004 in IEEE-754; tolerant equality answers
    // TRUE (the user means 0.3). Exact `==` would wrongly say FALSE.
    assert!(query(&kb, make_compute_query("sum", 0.3, 0.1, 0.2)));
    // A genuinely-wrong claim stays FALSE.
    assert!(query_false(&kb, make_compute_query("sum", 0.4, 0.1, 0.2)));
}

// ─── Decomposed numeric groups (surface-Lojban shape) ─────────────
//
// Surface numeric proposition event-decompose to ∃ev. head(ev) ∧ rel_x1(ev, a) ∧
// rel_x2(ev, b) ∧ ... — a LEFT-nested And where the head carries only the
// event variable and the operands live in sibling role predicates. These
// tests build that exact shape (mirroring nibli-semantics's event_decompose output)
// and pin that the numeric evaluators reach the operands.

/// Decomposed compute group: ∃_ev0. (((Compute(rel,[ev]) ∧ rel_x1(ev,x1))
/// ∧ rel_x2(ev,x2)) ∧ rel_x3(ev,x3)) — the surface shape for pilji/sumji/dilcu.
fn make_decomposed_compute_query(rel: &str, x1: f64, x2: f64, x3: f64) -> LogicBuffer {
    let mut nodes = Vec::new();
    let ev = || LogicalTerm::Variable("_ev0".to_string());
    let head = compute(&mut nodes, rel, vec![ev()]);
    let mut acc = head;
    for (i, v) in [x1, x2, x3].iter().enumerate() {
        let role = pred(
            &mut nodes,
            &format!("{rel}_x{}", i + 1),
            vec![ev(), LogicalTerm::Number(*v)],
        );
        acc = and(&mut nodes, acc, role);
    }
    let root = exists(&mut nodes, "_ev0", acc);
    LogicBuffer {
        nodes,
        roots: vec![root],
    }
}

/// Decomposed comparison group: ∃_ev0. head Pred(rel,[ev]) ∧ rel_x1(ev,a) ∧
/// rel_x2(ev,b) ∧ Zoe-padded trailing roles (zmadu/mleca arity 4, dunli 3).
fn make_decomposed_comparison_query(rel: &str, a: f64, b: f64) -> LogicBuffer {
    let mut nodes = Vec::new();
    let ev = || LogicalTerm::Variable("_ev0".to_string());
    let head = pred(&mut nodes, rel, vec![ev()]);
    let arity = if rel == "num_equal" { 3 } else { 4 };
    let mut acc = head;
    for i in 1..=arity {
        let arg = match i {
            1 => LogicalTerm::Number(a),
            2 => LogicalTerm::Number(b),
            _ => LogicalTerm::Unspecified,
        };
        let role = pred(&mut nodes, &format!("{rel}_x{i}"), vec![ev(), arg]);
        acc = and(&mut nodes, acc, role);
    }
    let root = exists(&mut nodes, "_ev0", acc);
    LogicBuffer {
        nodes,
        roots: vec![root],
    }
}

#[test]
fn test_decomposed_pilji_true() {
    let kb = new_kb();
    // 10 = 2 * 5 through the decomposed surface shape
    assert!(query(
        &kb,
        make_decomposed_compute_query("product", 10.0, 2.0, 5.0)
    ));
}

/// The UNKNOWN(non-finite) contract, pinned on BOTH numeric paths. Since the
/// `li` parse-boundary overflow guard landed, no overflowing literal can reach
/// these from the surface — but flat/raw-FOL buffers can still carry non-finite
/// Numbers, and a comparison over ±inf must NEVER be a confident TRUE/FALSE
/// (pre-guard, flat `dunli(inf, inf)` returned a confident TRUE).
#[test]
fn non_finite_comparison_is_unknown_on_both_paths() {
    let kb = new_kb();
    let inf = f64::INFINITY;

    // Flat path (try_numeric_comparison): bare Predicate node.
    let flat = |rel: &str, a: f64, b: f64| {
        let mut nodes = Vec::new();
        let root = pred(
            &mut nodes,
            rel,
            vec![LogicalTerm::Number(a), LogicalTerm::Number(b)],
        );
        LogicBuffer {
            nodes,
            roots: vec![root],
        }
    };
    for (rel, a, b) in [
        ("num_equal", inf, inf),
        ("num_equal", inf, 1.0),
        ("greater", inf, 1.0),
        ("less", 1.0, f64::NEG_INFINITY),
    ] {
        assert_eq!(
            query_result(&kb, flat(rel, a, b)),
            QueryResult::Unknown(UnknownReason::NonFinite),
            "flat {rel}({a}, {b}) must be UNKNOWN(non-finite), never definitive"
        );
    }
    // Finite controls: the guard must not widen onto meaningful comparisons.
    assert!(query(&kb, flat("num_equal", 2.0, 2.0)));
    assert!(matches!(
        query_result(&kb, flat("greater", 1.0, 2.0)),
        QueryResult::False
    ));

    // Event-decomposed path (the numeric-group guard): same contract.
    assert_eq!(
        query_result(&kb, make_decomposed_comparison_query("num_equal", inf, inf)),
        QueryResult::Unknown(UnknownReason::NonFinite),
        "decomposed dunli(inf, inf) must be UNKNOWN(non-finite)"
    );
    assert_eq!(
        query_result(&kb, make_decomposed_comparison_query("greater", inf, 1.0)),
        QueryResult::Unknown(UnknownReason::NonFinite),
        "decomposed zmadu(inf, 1) must be UNKNOWN(non-finite)"
    );
}

#[test]
fn test_decomposed_sumji_float_tolerance() {
    let kb = new_kb();
    // The surface (event-decomposed) path also uses tolerant equality:
    // 0.3 = 0.1 + 0.2 is TRUE despite IEEE-754 rounding.
    assert!(query(
        &kb,
        make_decomposed_compute_query("sum", 0.3, 0.1, 0.2)
    ));
}

#[test]
fn test_decomposed_pilji_false() {
    let kb = new_kb();
    assert!(query_false(
        &kb,
        make_decomposed_compute_query("product", 11.0, 2.0, 5.0)
    ));
}

#[test]
fn test_decomposed_sumji_true_false() {
    let kb = new_kb();
    assert!(query(
        &kb,
        make_decomposed_compute_query("sum", 5.0, 2.0, 3.0)
    ));
    assert!(query_false(
        &kb,
        make_decomposed_compute_query("sum", 6.0, 2.0, 3.0)
    ));
}

#[test]
fn test_decomposed_dilcu_true_and_division_by_zero() {
    let kb = new_kb();
    assert!(query(
        &kb,
        make_decomposed_compute_query("quotient", 3.0, 6.0, 2.0)
    ));
    // Division by zero is a definitive FALSE, not an error or fall-through.
    assert!(query_false(
        &kb,
        make_decomposed_compute_query("quotient", 3.0, 6.0, 0.0)
    ));
}

#[test]
fn test_decomposed_greater_true_false() {
    let kb = new_kb();
    assert!(query(
        &kb,
        make_decomposed_comparison_query("greater", 5.0, 3.0)
    ));
    assert!(query_false(
        &kb,
        make_decomposed_comparison_query("greater", 3.0, 5.0)
    ));
}

#[test]
fn test_decomposed_less_true_false() {
    let kb = new_kb();
    assert!(query(
        &kb,
        make_decomposed_comparison_query("less", 2.0, 3.0)
    ));
    assert!(query_false(
        &kb,
        make_decomposed_comparison_query("less", 3.0, 2.0)
    ));
}

#[test]
fn test_decomposed_num_equal_true_false() {
    let kb = new_kb();
    assert!(query(
        &kb,
        make_decomposed_comparison_query("num_equal", 3.0, 3.0)
    ));
    assert!(query_false(
        &kb,
        make_decomposed_comparison_query("num_equal", 3.0, 2.0)
    ));
}

#[test]
fn test_decomposed_negated() {
    // Not(∃ev. group) — the Not arm recurses into the Exists arm, so the
    // group verdict flips with no special handling.
    let kb = new_kb();
    let mut buf = make_decomposed_comparison_query("greater", 3.0, 5.0);
    let inner_root = buf.roots[0];
    let neg = {
        let id = buf.nodes.len() as u32;
        buf.nodes.push(LogicNode::NotNode(inner_root));
        id
    };
    buf.roots = vec![neg];
    assert!(query(&kb, buf), "NOT(3 > 5) must be TRUE");
}

#[test]
fn test_decomposed_extra_conjunct_falls_through() {
    // A group with an unrelated conjunct must NOT shortcut: the strict
    // same-relation rule bails, normal evaluation runs, and the unprovable
    // extra conjunct makes the query FALSE even though the arithmetic is true.
    let kb = new_kb();
    let mut nodes = Vec::new();
    let ev = || LogicalTerm::Variable("_ev0".to_string());
    let head = compute(&mut nodes, "product", vec![ev()]);
    let x1 = pred(
        &mut nodes,
        "pilji_x1",
        vec![ev(), LogicalTerm::Number(10.0)],
    );
    let x2 = pred(&mut nodes, "pilji_x2", vec![ev(), LogicalTerm::Number(2.0)]);
    let x3 = pred(&mut nodes, "pilji_x3", vec![ev(), LogicalTerm::Number(5.0)]);
    let extra = pred(&mut nodes, "broda", vec![ev()]);
    let a1 = and(&mut nodes, head, x1);
    let a2 = and(&mut nodes, a1, x2);
    let a3 = and(&mut nodes, a2, x3);
    let body = and(&mut nodes, a3, extra);
    let root = exists(&mut nodes, "_ev0", body);
    let buf = LogicBuffer {
        nodes,
        roots: vec![root],
    };
    assert!(
        query_false(&kb, buf),
        "an unrelated conjunct must disable the numeric-group shortcut"
    );
}

#[test]
fn test_decomposed_non_numeric_falls_through_to_store() {
    // Non-numeric operands can't compute; the group must fall through to
    // normal evaluation, where the asserted decomposed facts satisfy it.
    let kb = new_kb();
    let make = || {
        let mut nodes = Vec::new();
        let ev = || LogicalTerm::Variable("_ev0".to_string());
        let head = pred(&mut nodes, "greater", vec![ev()]);
        let x1 = pred(
            &mut nodes,
            "zmadu_x1",
            vec![ev(), LogicalTerm::Constant("alis".to_string())],
        );
        let x2 = pred(
            &mut nodes,
            "zmadu_x2",
            vec![ev(), LogicalTerm::Constant("bob".to_string())],
        );
        let a1 = and(&mut nodes, head, x1);
        let body = and(&mut nodes, a1, x2);
        let root = exists(&mut nodes, "_ev0", body);
        LogicBuffer {
            nodes,
            roots: vec![root],
        }
    };
    assert_buf(&kb, make());
    assert!(
        query(&kb, make()),
        "asserted non-numeric zmadu group must stay queryable via the store"
    );
}

#[test]
fn test_decomposed_asserted_true_group_still_true() {
    // Asserting an arithmetically-true group then querying it: the computed
    // verdict agrees with the store, so shadowing is invisible for true facts.
    let kb = new_kb();
    assert_buf(
        &kb,
        make_decomposed_compute_query("product", 10.0, 2.0, 5.0),
    );
    assert!(query(
        &kb,
        make_decomposed_compute_query("product", 10.0, 2.0, 5.0)
    ));
}

#[test]
fn test_assert_flat_numeric_comparison_rejected() {
    // The flat 2-arg form `zmadu(5, 3)` over number literals is computed ground
    // truth, not an assertable fact — reject it at assert time (the surface path
    // decomposes, so this guards the flat detection arm). A non-numeric flat
    // comparison still asserts (covered by test_greater_non_numeric_fallback).
    let kb = new_kb();
    assert!(
        kb.assert_fact_inner(make_numeric_query("greater", 5.0, 3.0), String::new())
            .is_err(),
        "asserting a flat numeric comparison must be rejected"
    );
}

#[test]
fn test_decomposed_traced_compute_check() {
    // The traced evaluator must agree with the untraced verdict and record
    // a ComputeCheck step for the group.
    let kb = new_kb();
    let (result, trace) = query_with_proof(
        &kb,
        make_decomposed_compute_query("product", 10.0, 2.0, 5.0),
    );
    assert!(result, "traced 10 = 2 × 5 must be TRUE");
    assert!(
        trace
            .steps
            .iter()
            .any(|s| matches!(&s.rule, ProofRule::ComputeCheck { .. }) && s.holds),
        "trace must contain a holding ComputeCheck step"
    );

    let (result_f, trace_f) = kb
        .query_entailment_with_proof_inner(make_decomposed_comparison_query("greater", 3.0, 5.0))
        .unwrap();
    assert!(result_f.is_false(), "traced 3 > 5 must be FALSE");
    assert!(
        trace_f
            .steps
            .iter()
            .any(|s| matches!(&s.rule, ProofRule::ComputeCheck { .. }) && !s.holds),
        "trace must contain a non-holding ComputeCheck step"
    );
}

#[test]
fn test_compute_negated() {
    let kb = new_kb();
    // NOT(7 = 2 * 3) → TRUE (because 7 != 6)
    let mut nodes = Vec::new();
    let inner = compute(
        &mut nodes,
        "product",
        vec![
            LogicalTerm::Number(7.0),
            LogicalTerm::Number(2.0),
            LogicalTerm::Number(3.0),
        ],
    );
    let root = not(&mut nodes, inner);
    assert!(query(
        &kb,
        LogicBuffer {
            nodes,
            roots: vec![root]
        }
    ));
}

#[test]
fn test_compute_node_kb_fallback() {
    // ComputeNode with non-arithmetic predicate falls back to KB lookup
    let kb = new_kb();

    // Assert: klama(alis, zarci) as a regular fact
    let mut a_nodes = Vec::new();
    let a_root = pred(
        &mut a_nodes,
        "klama",
        vec![
            LogicalTerm::Constant("alis".to_string()),
            LogicalTerm::Constant("zarci".to_string()),
        ],
    );
    assert_buf(
        &kb,
        LogicBuffer {
            nodes: a_nodes,
            roots: vec![a_root],
        },
    );

    // Query as ComputeNode — unknown to arithmetic, should fall through to KB lookup
    let mut q_nodes = Vec::new();
    let q_root = compute(
        &mut q_nodes,
        "klama",
        vec![
            LogicalTerm::Constant("alis".to_string()),
            LogicalTerm::Constant("zarci".to_string()),
        ],
    );
    assert!(query(
        &kb,
        LogicBuffer {
            nodes: q_nodes,
            roots: vec![q_root]
        }
    ));
}

// ─── Numeric quantifier-domain gap (GUARANTEES §Disclosed Sharp Edges) ────────
//
// `LogicalTerm::Number` is dropped by `collect_and_note_constants`, so a number never
// becomes a quantifier-domain member and a universal restricted to a number-bearing
// predicate is TRUE without ever testing those values. That is the DISCLOSED contract
// (pinned verdict-side by `numeric_terms_are_not_universal_domain_members`) — these tests
// pin that it is no longer SILENT. The verdict must not move; only the diagnostic is new.

/// Does the trace carry the numeric-domain-gap step, and what does it say?
fn domain_gap_detail(trace: &ProofTrace) -> Option<String> {
    trace.steps.iter().find_map(|s| match &s.rule {
        ProofRule::PredicateCheck { method, detail } if method == "numeric_domain_gap" => {
            assert!(
                s.holds,
                "the gap step must be holds:true — validate_cert \
                              requires all_hold beneath a holds:true parent"
            );
            Some(detail.clone())
        }
        _ => None,
    })
}

#[test]
fn numeric_domain_gap_is_announced_when_the_domain_is_empty() {
    // The shape the consuming project reported, and the one the existing pin uses:
    // `big(5).` alone leaves the domain EMPTY (5 is not an entity), so this goes down
    // the `members.is_empty()` / ForallVacuous path.
    let kb = new_kb();
    // Verbose ON so the `[Note]` echo branch is exercised too, not just the proof step.
    kb.set_verbose(true);
    assert_buf(&kb, compile_surface("big(5)."));
    let (result, trace) = kb
        .query_entailment_with_proof_inner(compile_surface("sum(every big, 2, 3)."))
        .unwrap();
    assert!(
        matches!(result, QueryResult::True),
        "verdict must NOT move — the disclosed contract is unchanged: {result:?}"
    );
    let detail = domain_gap_detail(&trace)
        .expect("a universal over a number-bearing predicate must announce the gap");
    assert!(
        detail.contains("(over big)"),
        "must name the restrictor relation as a hint: {detail}"
    );
    assert!(
        !detail.contains("every "),
        "must NOT spell the universal — `surface_relation` strips the place index, so \
         `every teaches.student` would read back as a universal the user never wrote: {detail}"
    );
    assert!(
        detail.contains('5'),
        "must name the excluded value: {detail}"
    );
    // The echo is latched per query: a traced query evaluates the universal twice (an
    // untraced probe, then one recording build) and the deepening loop can revisit it
    // per depth, so without the latch one `?` would print the same line repeatedly.
    assert_eq!(
        kb.inner.borrow().announced_gaps.borrow().len(),
        1,
        "the [Note] echo must be announced exactly once per query"
    );
    // …and the latch is per QUERY, not per KB epoch: the next query starts clean.
    let _ = kb
        .query_entailment_with_proof_inner(compile_surface("dog(Adam)."))
        .unwrap();
    assert!(
        kb.inner.borrow().announced_gaps.borrow().is_empty(),
        "clear_and_enable_pred_cache must reset the latch at the start of each query"
    );
}

#[test]
fn numeric_domain_gap_is_announced_when_the_domain_is_populated() {
    // The case that is genuinely silent today: another entity keeps the domain non-empty,
    // so `members.is_empty()` never fires, every member vacuously satisfies the guarded
    // `Or(Not(P), Q)`, and the trace reads as a POSITIVELY VERIFIED universal.
    let kb = new_kb();
    assert_buf(&kb, compile_surface("big(5)."));
    assert_buf(&kb, compile_surface("person(Adam)."));
    let (result, trace) = kb
        .query_entailment_with_proof_inner(compile_surface("sum(every big, 2, 2)."))
        .unwrap();
    assert!(
        matches!(result, QueryResult::True),
        "still TRUE even though the body is arithmetically FALSE of 5: {result:?}"
    );
    assert!(
        trace
            .steps
            .iter()
            .any(|s| matches!(s.rule, ProofRule::ForallVerified { .. })),
        "this must be the ForallVerified path, not ForallVacuous — otherwise the test is \
         not exercising the silent case"
    );
    let detail = domain_gap_detail(&trace)
        .expect("the populated-domain case is the SILENT one; it must announce the gap");
    assert!(
        detail.contains("(over big)"),
        "must name the restrictor relation as a hint: {detail}"
    );
    assert!(
        !detail.contains("every "),
        "must NOT spell the universal — `surface_relation` strips the place index, so \
         `every teaches.student` would read back as a universal the user never wrote: {detail}"
    );
}

#[test]
fn a_numberless_universal_gets_no_domain_gap_note() {
    // Anti-noise guard: the diagnostic must fire on the hazard, not on every universal.
    let kb = new_kb();
    assert_buf(&kb, compile_surface("dog(Adam)."));
    assert_buf(&kb, compile_surface("animal(every dog)."));
    let (_, trace) = kb
        .query_entailment_with_proof_inner(compile_surface("animal(every dog)."))
        .unwrap();
    assert_eq!(
        domain_gap_detail(&trace),
        None,
        "a restrictor holding no numbers must produce no note"
    );
}

#[test]
fn the_domain_gap_detector_fails_closed_on_an_unrecognised_shape() {
    // `numeric_domain_gap` only descends `Or(Not(g), q)` — the `every` lowering. A bare
    // (unrestricted) universal has no restrictor at all, so there is nothing to report
    // even though the KB holds a number-bearing predicate.
    let kb = new_kb();
    assert_buf(&kb, compile_surface("big(5)."));
    let (_, trace) = kb
        .query_entailment_with_proof_inner(compile_surface("all $x: big($x)."))
        .unwrap();
    assert_eq!(
        domain_gap_detail(&trace),
        None,
        "an unrestricted universal has no restrictor slot to read"
    );
}

// ─── Spurious-note regressions (adversarial review, 2026-08-01) ───────────────
//
// The first cut of the detector inferred "the restrictor holds of this number" from the
// presence of a Number key at ONE (relation, position) entry of `arg_position_index`.
// That is NECESSARY but not SUFFICIENT: the index is per-slot, flavour-blind and
// whole-KB, so it is not an evaluation of the restrictor. Each test below is a case where
// the restrictor is FALSE of the number, so nothing was skipped and a note would be a
// fabricated caveat — the one failure mode the design forbids ("a missed note is
// acceptable; a SPURIOUS note is not").

#[test]
fn a_linked_argument_restrictor_does_not_fabricate_a_gap() {
    // 5 eats BREAD; the universal restricts to eaters of an APPLE. A multi-conjunct group
    // needs every conjunct to hold of the SAME event, so at v=5 the restrictor is false.
    let kb = new_kb();
    assert_buf(&kb, compile_surface("eats(5, Bread)."));
    assert_buf(&kb, compile_surface("person(Adam)."));
    let (_, trace) = kb
        .query_entailment_with_proof_inner(compile_surface("sum(every eats(Apple), 2, 2)."))
        .unwrap();
    assert_eq!(
        domain_gap_detail(&trace),
        None,
        "5 eats bread, not an apple — the restrictor is FALSE of 5, so no value was skipped"
    );
}

#[test]
fn a_past_only_number_does_not_fabricate_a_gap_for_a_present_universal() {
    // `assert_typed_fact` indexes `fact.inner().relation`, so Past/Obligatory/… facts all
    // collapse onto the bare `("big_x1", 1)` key. The present-tense restrictor is FALSE
    // of 5 — confirmed by the direct probe below — so there is nothing to announce.
    let kb = new_kb();
    assert_buf(&kb, compile_surface("past big(5)."));
    assert_buf(&kb, compile_surface("person(Adam)."));
    assert!(
        query_false(&kb, compile_surface("big(5).")),
        "precondition: a past-only fact must not be present-tense true"
    );
    let (_, trace) = kb
        .query_entailment_with_proof_inner(compile_surface("sum(every big, 2, 2)."))
        .unwrap();
    assert_eq!(
        domain_gap_detail(&trace),
        None,
        "5 is only in the PAST extension; the present-tense universal skipped nothing"
    );
}

#[test]
fn a_where_clause_restrictor_does_not_fabricate_a_gap() {
    // `close_quantifier` nests a rel-clause OUTSIDE the description restrictor as
    // Or(Not(clause), Or(Not(desc), matrix)), so unwrapping one layer inspects only the
    // clause. 5 is big but is not a dog, so the restrictor `dog AND big` is false of it.
    let kb = new_kb();
    assert_buf(&kb, compile_surface("big(5)."));
    assert_buf(&kb, compile_surface("dog(Rex)."));
    let (_, trace) = kb
        .query_entailment_with_proof_inner(compile_surface("sum(every dog where big(it), 2, 2)."))
        .unwrap();
    assert_eq!(
        domain_gap_detail(&trace),
        None,
        "5 is big but not a dog — the conjunctive restrictor is FALSE of 5"
    );
}

#[test]
fn a_conjunctive_restrictor_reports_the_intersection_not_the_union() {
    // Mirror of the case above: the number satisfies the OTHER conjunct. Returning on the
    // first slot that holds any Number reports the UNION of the conjuncts' extensions;
    // the sound condition is the INTERSECTION.
    let kb = new_kb();
    assert_buf(&kb, compile_surface("big(Adam)."));
    assert_buf(&kb, compile_surface("dog(5)."));
    let (_, trace) = kb
        .query_entailment_with_proof_inner(compile_surface("sum(every big where dog(it), 2, 2)."))
        .unwrap();
    assert_eq!(
        domain_gap_detail(&trace),
        None,
        "5 is a dog but not big — the conjunctive restrictor is FALSE of 5"
    );
}

#[test]
fn the_diagnostic_never_mutates_the_knowledge_base() {
    // The detector evaluates the restrictor to decide whether a number is genuinely in it.
    // Evaluating a ComputeNode auto-asserts its result via `assert_typed_fact`, so a
    // compute-bearing restrictor must be REFUSED rather than evaluated: a diagnostic that
    // changes KB state is not a diagnostic.
    let kb = new_kb();
    assert_buf(&kb, compile_surface("big(5)."));
    assert_buf(&kb, compile_surface("person(Adam)."));
    let before = kb.inner.borrow().fact_store.all_facts().count();
    let _ = kb
        .query_entailment_with_proof_inner(compile_surface("sum(every big, 2, 2)."))
        .unwrap();
    let after = kb.inner.borrow().fact_store.all_facts().count();
    assert_eq!(
        before, after,
        "emitting the numeric-domain-gap diagnostic must not add facts to the store"
    );
}

/// `big(n).` in the event-decomposed shape the surface compiler produces
/// (`∃ev. big(ev) ∧ big_x1(ev,n) ∧ big_x2(ev,zoe) ∧ big_x3(ev,zoe)`), built by hand so a
/// NEGATIVE value can be asserted: `nibli_kr.pest`'s `number` rule is digits only and
/// cannot spell a sign, but `nibli-import` parses one straight out of RDF
/// (`rdf.rs` -> `lit.parse::<f64>()`), so this is a reachable store state.
fn decomposed_big_fact(n: f64) -> LogicBuffer {
    let mut nodes = Vec::new();
    let ev = || LogicalTerm::Variable("_ev0".to_string());
    let head = pred(&mut nodes, "big", vec![ev()]);
    let mut acc = head;
    for i in 1..=3 {
        let arg = if i == 1 {
            LogicalTerm::Number(n)
        } else {
            LogicalTerm::Unspecified
        };
        let role = pred(&mut nodes, &format!("big_x{i}"), vec![ev(), arg]);
        acc = and(&mut nodes, acc, role);
    }
    let root = exists(&mut nodes, "_ev0", acc);
    LogicBuffer {
        nodes,
        roots: vec![root],
    }
}

#[test]
fn reported_values_read_in_numeric_order_not_bit_order() {
    // IEEE-754 sets the sign bit on negatives, so sorting raw bit patterns puts every
    // negative AFTER every non-negative and orders negatives DESCENDING. With the 3-value
    // display cap that hides exactly the out-of-range values a reader is scanning for.
    // `f64::total_cmp` is equally total and deterministic, and numerically ascending.
    let kb = new_kb();
    assert_buf(&kb, compile_surface("person(Adam)."));
    for n in [7.0_f64, -100.0, 2.0, -3.0] {
        assert_buf(&kb, decomposed_big_fact(n));
    }
    let (_, trace) = kb
        .query_entailment_with_proof_inner(compile_surface("sum(every big, 2, 2)."))
        .unwrap();
    let detail = domain_gap_detail(&trace).expect("four number-valued bindings were skipped");
    assert!(
        detail.contains("4 numbers (-100, -3, 2, … 1 more)"),
        "values must read in ascending NUMERIC order (bit order would start at 2): {detail}"
    );
}

#[test]
fn a_compute_bearing_restrictor_is_refused_rather_than_evaluated() {
    // `subtree_has_compute`'s reason for existing. Only BUILTIN_ARITHMETIC
    // (product/sum/quotient) is turned into a `ComputeNode` by `transform_compute_nodes`,
    // and evaluating one AUTO-ASSERTS its result (`assert_typed_fact`, the trusted-oracle
    // path). Since the detector decides by EVALUATING the restrictor, such a restrictor
    // must be refused outright — a diagnostic that mutates the KB is not a diagnostic.
    // (Numeric COMPARISONS — greater/less/num_equal — stay plain `Predicate`s and are
    // decided by the pure `try_numeric_comparison`, so they are safe to evaluate.)
    let kb = new_kb();
    assert_buf(&kb, compile_surface("big(5)."));
    assert_buf(&kb, compile_surface("person(Adam)."));
    let before = kb.inner.borrow().fact_store.all_facts().count();
    let (_, trace) = kb
        .query_entailment_with_proof_inner(compile_surface("big(every sum(2, 3))."))
        .unwrap();
    assert_eq!(
        kb.inner.borrow().fact_store.all_facts().count(),
        before,
        "a compute-bearing restrictor must be refused, not evaluated — evaluating it \
         would auto-assert the computed fact"
    );
    assert_eq!(
        domain_gap_detail(&trace),
        None,
        "refused restrictors produce no note (fail closed)"
    );
}

#[test]
fn a_capped_candidate_set_is_reported_as_a_lower_bound_not_an_exact_count() {
    // The confirm loop stops at CANDIDATE_CAP, so `values.len()` is the DETECTOR'S search
    // bound, not the size of the skipped set. Printed bare, a KB with far more skipped
    // numbers rendered byte-identically to one holding exactly the cap — an under-report
    // in the one output whose entire purpose is honesty about what was skipped.
    let kb = new_kb();
    assert_buf(&kb, compile_surface("person(Adam)."));
    for n in 1..=40 {
        assert_buf(&kb, decomposed_big_fact(f64::from(n)));
    }
    let (_, trace) = kb
        .query_entailment_with_proof_inner(compile_surface("sum(every big, 2, 2)."))
        .unwrap();
    let detail = domain_gap_detail(&trace).expect("40 numbers are in the restrictor");
    assert!(
        detail.contains("at least "),
        "a capped candidate set must read as a LOWER BOUND: {detail}"
    );
    assert!(
        !detail.contains("holds of 40 numbers"),
        "must not claim an exact count it never established: {detail}"
    );
}

#[test]
fn the_gap_message_is_grammatical_in_both_number_branches() {
    // The plural branch used to read "which are not a quantifier-domain memberS" with the
    // article still attached — user-facing on all three surfaces.
    let kb = new_kb();
    assert_buf(&kb, compile_surface("person(Adam)."));
    assert_buf(&kb, compile_surface("big(5)."));
    let (_, one) = kb
        .query_entailment_with_proof_inner(compile_surface("sum(every big, 2, 2)."))
        .unwrap();
    let singular = domain_gap_detail(&one).expect("one skipped value");
    assert!(
        singular.contains("the number 5, which is not a quantifier-domain member \u{2014}"),
        "singular branch: {singular}"
    );

    assert_buf(&kb, decomposed_big_fact(9.0));
    let (_, two) = kb
        .query_entailment_with_proof_inner(compile_surface("sum(every big, 2, 2)."))
        .unwrap();
    let plural = domain_gap_detail(&two).expect("two skipped values");
    assert!(
        plural.contains("which are not quantifier-domain members \u{2014}"),
        "plural branch must drop the article: {plural}"
    );
    assert!(
        !plural.contains("a quantifier-domain members"),
        "the ungrammatical form must not reappear: {plural}"
    );
}
