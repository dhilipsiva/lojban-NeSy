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
fn test_decomposed_compute_group_is_query_only() {
    // An arithmetically true formula is still executable compute, not a fact.
    // Assertion must fail closed, while querying evaluates it proof-locally.
    let kb = new_kb();
    let error = kb
        .assert_fact_inner(
            make_decomposed_compute_query("product", 10.0, 2.0, 5.0),
            "query-only product".to_string(),
        )
        .expect_err("a decomposed compute group must not enter asserted state");
    assert!(error.contains("compute formulas are query-only"));
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
fn ordinary_fact_cannot_mask_compute_backend_outage() {
    // A ComputeNode is an operational call, not an alternate spelling of a
    // store lookup. A matching asserted tuple must not turn an unavailable
    // backend into TRUE.
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

    // Query the same tuple as a ComputeNode with no backend registered.
    let mut q_nodes = Vec::new();
    let q_root = compute(
        &mut q_nodes,
        "klama",
        vec![
            LogicalTerm::Constant("alis".to_string()),
            LogicalTerm::Constant("zarci".to_string()),
        ],
    );
    assert_eq!(
        query_result(
            &kb,
            LogicBuffer {
                nodes: q_nodes,
                roots: vec![q_root]
            }
        ),
        QueryResult::Unknown(UnknownReason::BackendUnavailable)
    );
}

#[test]
fn compute_and_comparison_role_predicates_are_non_indexable() {
    // Anchor-narrowing classifier: a query-time-evaluated relation must never
    // narrow entailment candidates, and that covers its decomposed ROLE
    // predicates too — their truth is computed rather than represented by a
    // complete store extension, so an empty index entry is not "no witness".
    // Before the surface-relation check, `sum_x1` anchored `sum(some big, 2, 3).`
    // and its empty candidate set won the narrowing pick: a definitive FALSE.
    use crate::kb::is_non_indexable_relation as non_indexable;
    for rel in [
        "equals",
        "sum",
        "product",
        "quotient",
        "greater",
        "less",
        "num_equal",
        "sum_x1",
        "product_x2",
        "quotient_x3",
        "greater_x2",
        "num_equal_x1",
    ] {
        assert!(non_indexable(rel), "{rel} must be refused as an anchor");
    }
    for rel in ["dog", "dog_x1", "sum_x", "sum_x0", "summary", "foo_x12"] {
        assert!(!non_indexable(rel), "{rel} must stay indexable");
    }
}

/// `compile_surface` with `exponential` additionally REGISTERED as a compute
/// predicate — what a session that wires the external backend does
/// (nibli-host registers `exponential`/`logarithm`). The static
/// `is_non_indexable_relation` classifier cannot know registered names; only
/// the buffer-local `collect_compute_heads` sweep marks their role predicates.
fn compile_surface_with_exponential(text: &str) -> LogicBuffer {
    let ast = nibli_kr::parse_checked(text).unwrap_or_else(|e| panic!("parse '{text}': {e}"));
    let mut buf =
        nibli_semantics::compile_from_ast(ast).unwrap_or_else(|e| panic!("compile '{text}': {e}"));
    let mut preds = default_compute_predicates();
    preds.insert("exponential".to_string());
    transform_compute_nodes(&mut buf, &preds);
    buf
}

#[test]
fn registered_compute_role_predicates_do_not_anchor_narrowing() {
    // The distinguishing job of the ComputeNode-head sweep: `exponential` is
    // not a builtin, so the static classifier passes `exponential_x1` — only
    // the head's presence in this very body marks it query-time-evaluated.
    // Without the filter the empty `exponential_x1` extension anchors the
    // existential and the honest non-definitive verdict (no backend registered
    // in native tests) collapses to a definitive wrong FALSE.
    let kb = new_kb();
    assert_buf(&kb, compile_surface("big(5)."));
    let result = query_result(
        &kb,
        compile_surface_with_exponential("exponential(some big, 2, 3)."),
    );
    assert_eq!(
        result,
        QueryResult::Unknown(UnknownReason::BackendUnavailable),
        "candidates must come from big_x1 ({{5}}); 5's dispatch surfaces \
         backend-unavailable — never a definitive FALSE from an empty compute-role anchor"
    );
}

#[test]
fn stored_non_finite_witnesses_stay_reachable_through_the_index() {
    // Non-finite numbers never join the DOMAIN (`note_number` skips them), but
    // the fact store is bitwise, so a stored `big(NaN)` is still an entailment
    // WITNESS: existential narrowing draws candidates from the stored-fact
    // index, not the member list. (Also the mutation-kill for the
    // `collect_entailment_candidates -> None` mutant — the full-domain
    // fallback would lose exactly this witness.)
    let kb = new_kb();
    assert_buf(&kb, decomposed_big_fact(f64::NAN));
    assert!(query(&kb, compile_surface("big(some big).")));
}

// ─── Numbers in the quantifier domain (GUARANTEES §Disclosed Sharp Edges) ─────
//
// Since the numbers-join-the-domain change, a FINITE number asserted into a
// predicate fact IS a quantifier-domain member (`note_number` → both member
// caches): `every` checks it, `exactly N` counts it, `some` reaches it. These
// pin the corrected verdicts and the deliberate residuals (non-finite values
// skipped fail-closed here; proof-local query-time compute never growing the
// domain is pinned in compute_ingest.rs).

#[test]
fn asserted_numbers_are_universal_domain_members() {
    let kb = new_kb();
    assert_buf(&kb, compile_surface("big(5)."));
    let (result, trace) = kb
        .query_entailment_with_proof_inner(compile_surface("sum(every big, 2, 3)."))
        .unwrap();
    assert!(
        result.is_true(),
        "5 = 2 + 3 holds of the one member: {result:?}"
    );
    assert!(
        trace
            .steps
            .iter()
            .any(|s| matches!(&s.rule, ProofRule::ForallVerified { .. })),
        "the universal must be VERIFIED by checking 5, not vacuously true"
    );
    assert!(
        !trace
            .steps
            .iter()
            .any(|s| matches!(&s.rule, ProofRule::ForallVacuous)),
        "no vacuous step — the number keeps the domain non-empty"
    );
}

#[test]
fn an_arithmetically_false_body_finds_the_numeric_counterexample() {
    let kb = new_kb();
    assert_buf(&kb, compile_surface("big(5)."));
    let (result, trace) = kb
        .query_entailment_with_proof_inner(compile_surface("sum(every big, 2, 2)."))
        .unwrap();
    assert!(
        result.is_false(),
        "5 ≠ 2 + 2 — the member is checked and fails: {result:?}"
    );
    let counter = trace.steps.iter().find_map(|s| match &s.rule {
        ProofRule::ForallCounterexample { entity } => Some(entity.term.clone()),
        _ => None,
    });
    assert!(
        matches!(counter, Some(LogicalTerm::Number(n)) if n == 5.0),
        "the counterexample must be the number 5: {counter:?}"
    );
}

#[test]
fn rule_operand_numbers_join_the_domain_like_constants() {
    // Noting mirrors constants exactly: `collect_and_note_constants` walks the
    // WHOLE asserted buffer, rules included, so an asserted rule's numeric
    // operands are domain members even with no predicate fact asserting them —
    // just as a rule mentioning Adam has always noted Adam.
    let kb = new_kb();
    let mut nodes = Vec::new();
    let x = LogicalTerm::Variable("_v0".to_string());
    let restrictor = pred(&mut nodes, "big", vec![x.clone(), LogicalTerm::Unspecified]);
    let conclusion = pred(
        &mut nodes,
        "zznumeric_operand",
        vec![x, LogicalTerm::Number(2.0), LogicalTerm::Number(3.0)],
    );
    let negated_restrictor = not(&mut nodes, restrictor);
    let implication = or(&mut nodes, negated_restrictor, conclusion);
    let root = forall(&mut nodes, "_v0", implication);
    assert_buf(
        &kb,
        LogicBuffer {
            nodes,
            roots: vec![root],
        },
    );
    // 2 and 3 are members and 2 ≠ 2 + 2: the bare universal finds a
    // counterexample where an empty domain would be vacuously TRUE.
    assert!(query_false(&kb, compile_surface("all $x: sum($x, 2, 2).")));
}

#[test]
fn a_past_only_number_is_a_member_but_fails_a_present_restrictor() {
    // Domain membership is atemporal, same as constants (`past dog(Rex).` notes
    // Rex) — but the RESTRICTOR is evaluated under the query's own tense, so a
    // past-only big(5) leaves the untensed universal guard-vacuous over 5 and
    // TRUE even with an arithmetically false body.
    let kb = new_kb();
    assert_buf(&kb, compile_surface("past big(5)."));
    assert!(query(&kb, compile_surface("sum(every big, 2, 2).")));
}

/// `big(n).` in the event-decomposed shape the surface compiler produces
/// (`∃ev. big(ev) ∧ big_x1(ev,n) ∧ big_x2(ev,zoe) ∧ big_x3(ev,zoe)`), built by
/// hand so NEGATIVE and NON-FINITE values can be asserted: `nibli_kr.pest`'s
/// `number` rule is digits-only and cannot spell a sign, but `nibli-import`
/// parses signed floats straight out of RDF (`rdf.rs` → `lit.parse::<f64>()`),
/// so these are reachable store states.
fn decomposed_big_fact(n: f64) -> LogicBuffer {
    let mut nodes = Vec::new();
    let ev = || LogicalTerm::Variable("_ev0".to_string());
    let head = pred(&mut nodes, "big", vec![ev()]);
    let r1 = pred(&mut nodes, "big_x1", vec![ev(), LogicalTerm::Number(n)]);
    let mut acc = and(&mut nodes, head, r1);
    for i in 2..=3 {
        let r = pred(
            &mut nodes,
            &format!("big_x{i}"),
            vec![ev(), LogicalTerm::Unspecified],
        );
        acc = and(&mut nodes, acc, r);
    }
    let root = exists(&mut nodes, "_ev0", acc);
    LogicBuffer {
        nodes,
        roots: vec![root],
    }
}

#[test]
fn negative_numbers_join_the_domain_and_serve_as_counterexamples() {
    let kb = new_kb();
    assert_buf(&kb, decomposed_big_fact(-3.0));
    assert!(
        query_false(&kb, compile_surface("sum(every big, 2, 2).")),
        "-3 ≠ 2 + 2 — a negative member must be enumerated and fail the body"
    );
}

#[test]
fn non_finite_numbers_are_skipped_fail_closed() {
    // NaN satisfies no arithmetic and its evaluation already surfaces
    // Unknown(NonFinite); noting it would only pollute counterexample search.
    // With a NaN-only extension the domain stays number-free and the universal
    // is vacuous — the pre-change behavior, kept deliberately for non-finite
    // values.
    let kb = new_kb();
    assert_buf(&kb, decomposed_big_fact(f64::NAN));
    assert!(query(&kb, compile_surface("sum(every big, 2, 2).")));
}

#[test]
fn du_linked_numbers_count_once() {
    // Union-find passes Numbers through (`find_canonical_readonly`), so two
    // du-linked numbers are ONE entity for `exactly N`. The KR surface cannot
    // spell a ground numeric identity, but RDF import can reach this store
    // state — built flat.
    let kb = new_kb();
    let mut nodes = Vec::new();
    let b5 = pred(&mut nodes, "big", vec![LogicalTerm::Number(5.0)]);
    let b6 = pred(&mut nodes, "big", vec![LogicalTerm::Number(6.0)]);
    let eq = pred(
        &mut nodes,
        "equals",
        vec![LogicalTerm::Number(5.0), LogicalTerm::Number(6.0)],
    );
    assert_buf(
        &kb,
        LogicBuffer {
            nodes,
            roots: vec![b5, b6, eq],
        },
    );
    let count_query = |n: u32| {
        let mut q = Vec::new();
        let body = pred(&mut q, "big", vec![LogicalTerm::Variable("x".to_string())]);
        let root = q.len() as u32;
        q.push(LogicNode::CountNode(("x".to_string(), n, body)));
        LogicBuffer {
            nodes: q,
            roots: vec![root],
        }
    };
    assert!(
        query(&kb, count_query(1)),
        "5 and 6 are du-linked: one entity, count 1"
    );
    assert!(
        query_false(&kb, count_query(2)),
        "the du class must not count twice"
    );
}

// ─── Comparisons in rules are REFUSED, not silently divergent ────────────────
//
// `greater` / `less` / `num_equal` are ordinary `Predicate` nodes, but the two
// halves of the engine read them differently: a QUERY computes them
// arithmetically (`try_evaluate_numeric_group`) and the computed value wins,
// while rule compilation lowers the same atom to a plain `StoredFact` template
// looked up in a store that holds none. Every way that divergence surfaced was
// wrong, and one of them was wrong in the worst direction:
//
//   * positive guard  — INERT: the rule never fires, so the head is under-derived.
//   * negated guard   — OVERFIRES: `~greater($n, 15)` succeeds for EVERY binding
//                       because the extension is empty, so a subject whose value
//                       really is greater than 15 is still concluded. A definitive
//                       wrong TRUE.
//   * rule head       — DEAD: the derived fact is never consulted, because
//                       computation runs before the store.
//
// The refusal is on the OPERANDS, not the relation, so the relational reading
// (`greater(Alis, Bob)`, "taller than") keeps working — it is answered from the
// store on both sides and so cannot diverge.

/// Every rule position that used to accept a computable comparison now refuses it,
/// and refuses it without leaving a registry record.
#[test]
fn a_numeric_comparison_in_a_rule_is_refused_in_every_position() {
    for (position, text) in [
        (
            "positive antecedent",
            "all $x: all $n: person($x) & quantity($x, $n) & greater($n, 15) -> fit($x).",
        ),
        (
            "negated antecedent",
            "all $x: all $n: person($x) & quantity($x, $n) & ~greater($n, 15) -> rotten($x).",
        ),
        (
            "rule head",
            "all $x: all $n: quantity($x, $n) -> greater($n, 100).",
        ),
        (
            "numeric literals in an antecedent",
            "all $x: person($x) & greater(3, 1) -> fit($x).",
        ),
    ] {
        let kb = new_kb();
        let error = kb
            .assert_fact_inner(compile_surface(text), position.to_string())
            .unwrap_err();
        assert!(
            error.contains("computed comparison"),
            "{position} must be refused with the comparison contract: {error}"
        );
        assert!(
            kb.list_facts_inner().unwrap().is_empty(),
            "{position} rejection must leave no registry record"
        );
    }
}

/// The RELATIONAL reading is not collateral damage: it still asserts, still answers
/// from the store, and still works as a rule guard. This is what makes the refusal a
/// judgement about operands rather than a ban on three relation names.
#[test]
fn a_relational_comparison_still_asserts_and_still_guards_a_rule() {
    let kb = new_kb();
    assert_buf(&kb, compile_surface("greater(Alis, Bob)."));
    assert!(
        query(&kb, compile_surface("greater(Alis, Bob).")),
        "a stored relational comparison must answer from the store"
    );
    assert_buf(&kb, compile_surface("person(Ara)."));
    assert_buf(
        &kb,
        compile_surface("all $x: person($x) & greater(Alis, Bob) -> fit($x)."),
    );
    assert!(
        query(&kb, compile_surface("fit(Ara).")),
        "a relational comparison must still fire a rule — store lookup on both sides"
    );
}

/// Queries are untouched: the arithmetic path is exactly where a comparison belongs.
#[test]
fn queries_still_compute_comparisons_arithmetically() {
    let kb = new_kb();
    assert!(query(&kb, make_numeric_query("greater", 20.0, 15.0)));
    assert!(query_false(&kb, make_numeric_query("greater", 3.0, 5.0)));
}

/// The flat two-argument spelling never event-decomposes, so it reaches the guard by
/// a different arm. Direct `LogicBuffer` injection and persisted-buffer replay both
/// produce it, and neither goes through the KR surface.
#[test]
fn a_flat_raw_ir_comparison_is_refused_too() {
    let kb = new_kb();
    let error = kb
        .assert_fact_inner(make_numeric_query("greater", 3.0, 1.0), "flat".to_string())
        .unwrap_err();
    assert!(
        error.contains("computed comparison"),
        "the flat raw-IR spelling must be refused as well: {error}"
    );
}

// ─── Verdict ≡ witness enumeration ──────────────────────────────────────────
//
// `try_evaluate_numeric_group` used to have exactly ONE caller — the `ExistsNode`
// arm of `check_formula_holds_core`. `find_witnesses` has its own `ExistsNode` arm,
// which peeled the comparison group's `∃_ev` and swept domain candidates, so every
// conjunct degraded to a store lookup and found nothing (a comparison is never
// stored — the assertion guard refuses one). The boolean verdict computed TRUE while
// find/count/aggregate returned zero rows with no error: a jointly inconsistent pair.
//
// These pin the agreement. `greater($de, 15)` filtering a `quantity` join is the
// shape the quantitative direction actually needs — a rule cannot carry a comparison,
// but a QUERY may, and enumeration must honour it.

/// Every comparison relation, over and under its threshold, agreeing across the
/// boolean verdict, `query_find`, and `count_witnesses`.
#[test]
fn verdict_and_find_agree_for_every_comparison() {
    for (rel, over, under) in [
        ("greater", "quantity($x, $n) & greater($n, 15).", 20.0),
        ("less", "quantity($x, $n) & less($n, 15).", 7.0),
        ("num_equal", "quantity($x, $n) & num_equal($n, 20).", 20.0),
    ]
    .map(|(rel, text, expect)| (rel, text, expect))
    {
        let kb = new_kb();
        assert_buf(&kb, compile_surface("quantity(Varfarin, 20)."));
        assert_buf(&kb, compile_surface("quantity(Fenitoin, 7)."));

        assert!(
            query(&kb, compile_surface(over)),
            "{rel}: the boolean verdict must compute the comparison"
        );
        let rows = query_find(&kb, compile_surface(over));
        assert_eq!(
            rows.len(),
            1,
            "{rel}: enumeration must agree with the verdict, got {rows:?}"
        );
        assert!(
            rows[0]
                .iter()
                .any(|b| b.variable == "$n" && b.term == LogicalTerm::Number(under)),
            "{rel}: the surviving witness must be the one past the threshold, got {rows:?}"
        );
        assert_eq!(
            kb.count_witnesses(compile_surface(over)).unwrap(),
            1,
            "{rel}: count must agree with find"
        );
    }
}

/// Aggregation runs over the enumerated witnesses, so a threshold-filtered sum was
/// silently `None` before the hook — the shape `drug-interactions.nibli`'s dose data
/// is one line away from.
#[test]
fn aggregate_sums_only_witnesses_past_the_threshold() {
    let kb = new_kb();
    assert_buf(&kb, compile_surface("quantity(Varfarin, 20)."));
    assert_buf(&kb, compile_surface("quantity(Fenitoin, 7)."));
    let total = kb
        .aggregate(
            compile_surface("quantity($x, $n) & greater($n, 15)."),
            "$n",
            nibli_types::logic::AggregateOp::Sum,
        )
        .unwrap();
    assert_eq!(
        total,
        Some(20.0),
        "only the dose past the threshold may be summed"
    );
}

/// A non-finite operand satisfies no comparison. That is a genuine NO, not a search
/// cut, so enumeration must return zero rows rather than refusing the whole query as
/// incomplete — `witness_search_cut` deliberately excludes `Unknown(NonFinite)`, and
/// this pins that exclusion as a decision rather than a fallout.
#[test]
fn a_non_finite_operand_yields_no_witness_not_an_incomplete_refusal() {
    let kb = new_kb();
    // A non-finite value never reaches the KR surface (the `number` rule is digits
    // only), so build the stored fact the way `stored_non_finite_witnesses_...` does.
    assert_buf(&kb, decomposed_big_fact(f64::INFINITY));
    let buf = compile_surface("big($n) & greater($n, 15).");
    assert_eq!(
        kb.query_find_inner(buf)
            .expect("find must not refuse")
            .len(),
        0,
        "a non-finite operand yields no witness, not an incomplete-enumeration error"
    );
}

/// The hook must not STEAL a comparison from the store. `greater(Alis, Bob)`
/// ("taller than") has non-numeric operands, so `try_numeric_comparison` returns
/// None, the group falls through, and the stored row is still enumerated.
#[test]
fn a_relational_comparison_still_finds_its_stored_witness() {
    let kb = new_kb();
    assert_buf(&kb, compile_surface("greater(Alis, Bob)."));
    let rows = query_find(&kb, compile_surface("greater($a, $b)."));
    assert_eq!(
        rows.len(),
        1,
        "the stored relational comparison must still be found, got {rows:?}"
    );
    assert!(
        rows[0]
            .iter()
            .any(|b| b.variable == "$a" && b.term == LogicalTerm::Constant("alis".to_string())),
        "the relational row must carry its stored arguments, got {rows:?}"
    );
}

/// The comparison group binds nothing user-visible, so adding one as a filtering
/// conjunct must narrow the ROWS without changing their SHAPE — no `_ev` from the
/// comparison leaks into the reported bindings.
#[test]
fn find_row_shape_is_unchanged_by_a_comparison_conjunct() {
    let kb = new_kb();
    assert_buf(&kb, compile_surface("quantity(Varfarin, 20)."));
    let plain = query_find(&kb, compile_surface("quantity($x, $n)."));
    let filtered = query_find(&kb, compile_surface("quantity($x, $n) & greater($n, 15)."));
    assert_eq!(plain.len(), 1, "control: one stored quantity");
    assert_eq!(filtered.len(), 1, "the row survives its own filter");
    let names = |rows: &Vec<Vec<WitnessBinding>>| {
        let mut v: Vec<String> = rows[0].iter().map(|b| b.variable.clone()).collect();
        v.sort();
        v
    };
    assert_eq!(
        names(&plain),
        names(&filtered),
        "a comparison conjunct must not add a binding to the reported row"
    );
}

// ─── Compute groups under witness enumeration ───────────────────────────────
//
// The comparison hook shipped with a scope parameter that declined every non-
// comparison head, on the stated theory that a compute head "can dispatch to the
// external backend, and enumeration evaluates its body once per candidate", and that
// the excluded side "already fails CLOSED". BOTH halves were wrong.
//
// DECLINING the group is what dispatches: `find_witnesses` peels the group's `∃_ev`
// itself and recurses on the BODY, so the head reaches the FLAT `ComputeNode` arm
// carrying `[Variable(ev)]` — one argument, no operands — and is sent to the backend
// once per non-Skolem candidate. CONSUMING it is one local decision, or one call with
// the real operands. And the refusal was an accident of data, not a rule: it happened
// only because `GroundTerm`'s ordering put an event Skolem early enough to trip
// `resolve_args_for_dispatch`. An empty or Skolem-free domain got no refusal at all —
// it got zero rows and `Ok`, against a TRUE verdict.
//
// These pin the corrected contract: every group the recogniser can decide LOCALLY
// filters rows exactly as a comparison does, and every group whose routing would
// dispatch refuses explicitly, with a budget of zero calls, on every route.

/// The minimal repro, needing no backend and no facts: on a fresh KB the domain is
/// empty, so declining the group left `find` with nothing to enumerate and it returned
/// zero rows with `Ok` — while the verdict path computed TRUE. The comparison twin on
/// the same shape returns one row, so the two halves of one hook disagreed on a query
/// with no user variables at all.
#[test]
fn find_over_a_ground_arithmetic_group_agrees_with_the_verdict() {
    let kb = new_kb();
    assert!(
        query(&kb, compile_surface("product(10, 2, 5).")),
        "control: the verdict path computes 10 = 2 x 5 locally"
    );
    assert_eq!(
        query_find(&kb, compile_surface("greater(20, 15).")).len(),
        1,
        "control: a satisfied ground COMPARISON yields one empty binding set"
    );
    assert_eq!(
        query_find(&kb, compile_surface("product(10, 2, 5).")).len(),
        1,
        "a satisfied ground ARITHMETIC group must yield the same one empty binding set"
    );
    assert_eq!(
        query_find(&kb, compile_surface("product(11, 2, 5).")).len(),
        0,
        "and a false one must yield none"
    );
}

static FIND_DISPATCH_CALLS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
static FIND_DISPATCH_MAX_ARITY: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);

fn recording_find_backend(_rel: &str, args: &[LogicalTerm]) -> Result<bool, String> {
    FIND_DISPATCH_CALLS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    FIND_DISPATCH_MAX_ARITY.fetch_max(args.len(), std::sync::atomic::Ordering::SeqCst);
    Ok(false)
}

fn recording_find_batch(reqs: &[ComputeRequest]) -> Vec<Result<bool, String>> {
    FIND_DISPATCH_CALLS.fetch_add(reqs.len(), std::sync::atomic::Ordering::SeqCst);
    reqs.iter().map(|_| Ok(false)).collect()
}

/// Witness enumeration must never reach the compute backend — the stated budget is
/// ZERO calls, enforced by an explicit rule rather than by whatever happens to sort
/// first in the domain.
///
/// `GroundTerm`'s ordering puts `Constant` before `Skolem`, so on any KB with a named
/// entity the sweep dispatched for the constants FIRST and only later hit the event
/// Skolem that `resolve_args_for_dispatch` refuses. Those early calls carried a
/// one-argument `exponential(adam)` payload — the head's only argument is the event
/// variable — which no correct backend could answer. The recorded arity is the
/// evidence for that, so assert it alongside the count.
#[test]
fn find_never_dispatches_to_the_compute_backend() {
    FIND_DISPATCH_CALLS.store(0, std::sync::atomic::Ordering::SeqCst);
    FIND_DISPATCH_MAX_ARITY.store(0, std::sync::atomic::Ordering::SeqCst);
    let kb = new_kb();
    kb.set_compute_dispatch(recording_find_backend, recording_find_batch);
    assert_buf(&kb, compile_surface("dog(Adam)."));

    let _ = kb.query_find_inner(compile_surface_with_exponential(
        "dog($x) & exponential($x, 2, 3).",
    ));
    assert_eq!(
        FIND_DISPATCH_CALLS.load(std::sync::atomic::Ordering::SeqCst),
        0,
        "witness enumeration must not reach the backend at all (max arity seen: {})",
        FIND_DISPATCH_MAX_ARITY.load(std::sync::atomic::Ordering::SeqCst)
    );
}

/// A group whose routing WOULD dispatch refuses on every POSITIVE route into
/// enumeration, not just as a direct conjunct — the deontic prefixes reach dispatch
/// through the shared leaf evaluator, which the old scope parameter never covered
/// because it sat on `find_witnesses`' own `ExistsNode` arm.
///
/// Zero dispatches is asserted on every route including the negated one: the latch
/// lives at the dispatch choke point, so nothing calls out regardless of how the leaf
/// is reached. What differs is only how the refusal is REPORTED — see the negated twin
/// below.
#[test]
fn find_refuses_a_compute_group_on_every_positive_route() {
    // Ground operands throughout: a variable shared across a `~`/modal boundary is a
    // co-reference error in KR, and the route is what is under test, not the binding.
    for (route, text) in [
        ("direct conjunct", "dog($x) & exponential($x, 2, 3)."),
        ("obligation", "dog($x) & must exponential(8, 2, 3)."),
        ("permission", "dog($x) & may exponential(8, 2, 3)."),
    ] {
        FIND_DISPATCH_CALLS.store(0, std::sync::atomic::Ordering::SeqCst);
        let kb = new_kb();
        kb.set_compute_dispatch(recording_find_backend, recording_find_batch);
        assert_buf(&kb, compile_surface("dog(Adam)."));
        let err = kb
            .query_find_inner(compile_surface_with_exponential(text))
            .unwrap_err();
        assert!(
            err.contains("witness enumeration incomplete"),
            "{route}: must refuse as incomplete, got: {err}"
        );
        assert_eq!(
            FIND_DISPATCH_CALLS.load(std::sync::atomic::Ordering::SeqCst),
            0,
            "{route}: refusing must cost zero dispatches"
        );
    }
}

/// The NEGATED route refuses too, so all four routes agree.
///
/// `negate_result` collapses every non-definitive inner verdict to
/// `Unknown(NafDependent)` — deliberately, and pinned by the Lean model — and
/// `witness_search_cut` deliberately excludes `NafDependent`. Between them the fact
/// that the inner leaf was REFUSED rather than merely unprovable used to be lost, so
/// enumeration reported a definitive zero for a leaf it never decided. A monotone
/// `naf_cut_epoch` now records the laundering at the collapse point, and the leaf guard
/// consults it only when the leaf ends non-definitive.
///
/// The dispatch budget holds either way: refusing costs zero calls.
#[test]
fn a_negated_compute_leaf_under_find_refuses_as_incomplete() {
    FIND_DISPATCH_CALLS.store(0, std::sync::atomic::Ordering::SeqCst);
    let kb = new_kb();
    kb.set_compute_dispatch(recording_find_backend, recording_find_batch);
    assert_buf(&kb, compile_surface("dog(Adam)."));
    let err = kb
        .query_find_inner(compile_surface_with_exponential(
            "dog($x) & ~exponential(8, 2, 3).",
        ))
        .unwrap_err();
    assert!(
        err.contains("witness enumeration incomplete"),
        "a refused leaf under `~` must refuse the enumeration too: {err}"
    );
    assert_eq!(
        FIND_DISPATCH_CALLS.load(std::sync::atomic::Ordering::SeqCst),
        0,
        "refusing must still cost zero dispatches"
    );
}

/// Arithmetic now filters witness rows exactly as a comparison does, and agrees with
/// the verdict on both polarities across all three relations.
#[test]
fn verdict_and_find_agree_for_every_arithmetic_relation() {
    for (rel, holds, fails) in [
        ("product", "product(10, 2, 5).", "product(11, 2, 5)."),
        ("sum", "sum(5, 2, 3).", "sum(6, 2, 3)."),
        ("quotient", "quotient(3, 6, 2).", "quotient(4, 6, 2)."),
    ] {
        let kb = new_kb();
        assert!(query(&kb, compile_surface(holds)), "{rel}: verdict TRUE");
        assert_eq!(
            query_find(&kb, compile_surface(holds)).len(),
            1,
            "{rel}: find must agree with the TRUE verdict"
        );
        assert_eq!(
            kb.count_witnesses(compile_surface(holds)).unwrap(),
            1,
            "{rel}: count must agree with find"
        );

        assert!(
            query_false(&kb, compile_surface(fails)),
            "{rel}: verdict FALSE"
        );
        assert_eq!(
            query_find(&kb, compile_surface(fails)).len(),
            0,
            "{rel}: find must agree with the FALSE verdict"
        );
    }
}

/// An arithmetic conjunct narrows the rows without changing their SHAPE — the group
/// binds nothing user-visible, so no `_ev` leaks into the reported bindings. Twin of
/// `find_row_shape_is_unchanged_by_a_comparison_conjunct`.
#[test]
fn find_row_shape_is_unchanged_by_an_arithmetic_conjunct() {
    let kb = new_kb();
    assert_buf(&kb, compile_surface("quantity(Varfarin, 20)."));
    let plain = query_find(&kb, compile_surface("quantity($x, $n)."));
    let kept = query_find(
        &kb,
        compile_surface("quantity($x, $n) & product(10, 2, 5)."),
    );
    let dropped = query_find(
        &kb,
        compile_surface("quantity($x, $n) & product(11, 2, 5)."),
    );
    assert_eq!(plain.len(), 1, "control: one stored quantity");
    assert_eq!(kept.len(), 1, "a true arithmetic conjunct keeps the row");
    assert_eq!(dropped.len(), 0, "a false one drops it");
    let names = |rows: &Vec<Vec<WitnessBinding>>| {
        let mut v: Vec<String> = rows[0].iter().map(|b| b.variable.clone()).collect();
        v.sort();
        v
    };
    assert_eq!(
        names(&plain),
        names(&kept),
        "an arithmetic conjunct must not add a binding to the reported row"
    );
}

/// An arithmetic RESULT that overflows f64 is `Unknown(NonFinite)`, which
/// `witness_search_cut` deliberately excludes — so find reports zero rows without
/// refusing, and the verdict is non-definitive.
///
/// Materially different from the comparison twin
/// (`a_non_finite_operand_yields_no_witness_not_an_incomplete_refusal`): there the
/// OPERANDS are non-finite, here they are perfectly finite and only the product
/// overflows. "No witness" is a claim about f64 range, not about the data.
#[test]
fn an_overflowing_arithmetic_result_under_find_is_no_witness_not_a_refusal() {
    let kb = new_kb();
    let buf = make_decomposed_compute_query("product", f64::MAX, 1e308, 1e308);
    assert!(
        !query(&kb, buf.clone()) && !query_false(&kb, buf.clone()),
        "an overflowing result is neither TRUE nor definitively FALSE"
    );
    assert_eq!(
        kb.query_find_inner(buf)
            .expect("an overflowing result must not refuse the enumeration")
            .len(),
        0,
        "it yields no witness"
    );
}

/// The fix must not rest on `event_decompose` emitting the head leftmost. The group
/// recogniser flattens the And-tree, so a role-first buffer — reachable through raw
/// `LogicBuffer` injection and persisted-buffer replay — must decide identically.
#[test]
fn a_role_first_arithmetic_group_is_consumed_too() {
    let kb = new_kb();
    let mut nodes = Vec::new();
    let ev = || LogicalTerm::Variable("_ev0".to_string());
    // Roles FIRST, head last — the mirror of `make_decomposed_compute_query`.
    let mut acc = pred(
        &mut nodes,
        "product_x1",
        vec![ev(), LogicalTerm::Number(10.0)],
    );
    for (i, v) in [2.0_f64, 5.0].iter().enumerate() {
        let role = pred(
            &mut nodes,
            &format!("product_x{}", i + 2),
            vec![ev(), LogicalTerm::Number(*v)],
        );
        acc = and(&mut nodes, acc, role);
    }
    let head = compute(&mut nodes, "product", vec![ev()]);
    let root_body = and(&mut nodes, acc, head);
    let root = exists(&mut nodes, "_ev0", root_body);
    let buf = LogicBuffer {
        nodes,
        roots: vec![root],
    };
    assert!(query(&kb, buf.clone()), "role-first still verdicts TRUE");
    assert_eq!(
        query_find(&kb, buf).len(),
        1,
        "and role-first still yields the same one row"
    );
}

/// `NonFinite` is the ONE negated `Unknown` that still does not refuse — the sole
/// inhabitant of the "defensibly excluded" category, and therefore what gives that
/// category meaning.
///
/// It is a claim about f64 range, not about the search: a non-finite operand satisfies
/// no arithmetic, and negating that does not make a witness more likely to exist beyond
/// some budget. So it is non-cut on BOTH polarities — the positive twin is
/// `an_overflowing_arithmetic_result_under_find_is_no_witness_not_a_refusal`. Treating
/// the negated case as a cut while the positive case is not would be an asymmetry with
/// no principle behind it.
///
/// The trailing control matters: it pins that the epoch bump (or absence of one) left no
/// residue that truncates the rest of the sweep.
#[test]
fn a_negated_non_finite_group_under_find_is_zero_rows_not_a_refusal() {
    let kb = new_kb();
    assert_buf(&kb, compile_surface("dog(Adam)."));

    // ¬(∃ev. product-group(f64::MAX, 1e308, 1e308)) — finite operands, overflowing
    // result, so the inner verdict is Unknown(NonFinite) rather than a cut.
    let mut nodes = Vec::new();
    let ev = || LogicalTerm::Variable("_ev0".to_string());
    let head = compute(&mut nodes, "product", vec![ev()]);
    let mut acc = head;
    for (i, v) in [f64::MAX, 1e308, 1e308].iter().enumerate() {
        let role = pred(
            &mut nodes,
            &format!("product_x{}", i + 1),
            vec![ev(), LogicalTerm::Number(*v)],
        );
        acc = and(&mut nodes, acc, role);
    }
    let grp = exists(&mut nodes, "_ev0", acc);
    let root = not(&mut nodes, grp);
    let rows = kb
        .query_find_inner(LogicBuffer {
            nodes,
            roots: vec![root],
        })
        .expect("a negated non-finite group must not refuse the enumeration");
    assert_eq!(rows.len(), 0, "it yields no witness, got {rows:?}");

    // Control, same KB: nothing was truncated and no epoch residue leaked.
    assert_eq!(
        query_find(&kb, compile_surface("dog($x).")).len(),
        1,
        "the rest of the sweep must be unaffected"
    );
}

/// The two compute families answer a NON-NUMERIC operand differently, and the
/// difference is principled rather than an oversight — pinned here because it reads
/// like an inconsistency and will otherwise be re-litigated.
///
/// With no numbers in the domain, `$n`/`$t` still BIND (to a domain member); they just
/// bind to something that is not a number. From there:
///
/// * a COMPARISON head is an ordinary `Predicate`, so the group falls through to the
///   store — the relational reading, the one that makes `greater(Alis, Bob)` work. No
///   such fact is stored, so find reports zero rows AND the verdict is FALSE. They
///   agree, and both are right: nothing in the KB satisfies the query.
/// * an ARITHMETIC head is a `ComputeNode`, so an unresolvable operand routes to
///   dispatch. Enumeration refuses to dispatch, leaving the leaf undecided, so find
///   reports an incomplete enumeration rather than a definitive zero.
///
/// Neither is a silent wrong answer: one is a definitive no the verdict path shares,
/// the other an explicit refusal. What must never happen is a definitive zero from an
/// UNDECIDED leaf, and neither family does that.
#[test]
fn a_non_numeric_operand_answers_by_family_and_never_silently_undercounts() {
    let kb = new_kb();
    assert_buf(&kb, compile_surface("dog(Adam)."));

    // Comparison: definitive no, and the verdict path says the same thing.
    assert!(
        query_false(&kb, compile_surface("greater($n, 15).")),
        "with no numbers in the domain the comparison verdict is a definitive FALSE"
    );
    assert_eq!(
        query_find(&kb, compile_surface("greater($n, 15) & dog($x).")).len(),
        0,
        "find agrees with that FALSE — zero rows, not a refusal"
    );

    // Arithmetic: undecided, so an explicit refusal rather than a definitive zero.
    let err = kb
        .query_find_inner(compile_surface("sum($t, 2, 3) & dog($x)."))
        .unwrap_err();
    assert!(
        err.contains("witness enumeration incomplete"),
        "an undecided compute leaf must refuse, not report zero rows: {err}"
    );
}

/// The guard runs in `preflight_assertion_buffer`, before id allocation — so a
/// rejected comparison no longer consumes a fact id and leaves a hole in the
/// registry. (It used to be refused later, on the collected ground leaves.)
#[test]
fn a_refused_comparison_does_not_burn_a_fact_id() {
    let kb = new_kb();
    let first = assert_id(&kb, compile_surface("dog(Adam)."), "dog");
    assert!(
        kb.assert_fact_inner(compile_surface("greater(3, 1)."), "cmp".to_string())
            .is_err(),
        "the ground comparison must be refused"
    );
    let second = assert_id(&kb, compile_surface("cat(Bela)."), "cat");
    assert_eq!(
        second,
        first + 1,
        "a rejected comparison must not consume a fact id"
    );
}
