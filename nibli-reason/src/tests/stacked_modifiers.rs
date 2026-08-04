use super::*;

const STACK_ERR: &str = "nested tense/deontic LogicNode wrappers are unsupported";

#[derive(Clone, Copy, Debug)]
enum StackOrder {
    DeonticOuter,
    TemporalOuter,
}

#[derive(Clone, Copy, Debug)]
enum RulePlacement {
    Antecedent,
    Conclusion,
    Naf,
}

fn stacked(nodes: &mut Vec<LogicNode>, inner: u32, order: StackOrder) -> u32 {
    match order {
        // Obligatory(Past(P)) — the former surface spelling `must past P`.
        StackOrder::DeonticOuter => {
            let temporal = past(nodes, inner);
            obligatory(nodes, temporal)
        }
        // Past(Obligatory(P)) — the inverse raw-IR order.
        StackOrder::TemporalOuter => {
            let deontic = obligatory(nodes, inner);
            past(nodes, deontic)
        }
    }
}

fn atom(nodes: &mut Vec<LogicNode>, relation: &str, term: LogicalTerm) -> u32 {
    pred(nodes, relation, vec![term])
}

fn stacked_fact(order: StackOrder) -> LogicBuffer {
    let mut nodes = Vec::new();
    let leaf = atom(&mut nodes, "dog", LogicalTerm::Constant("rex".to_string()));
    let root = stacked(&mut nodes, leaf, order);
    LogicBuffer {
        nodes,
        roots: vec![root],
    }
}

fn stacked_rule(order: StackOrder, placement: RulePlacement) -> LogicBuffer {
    let mut nodes = Vec::new();
    let var = || LogicalTerm::Variable("_v0".to_string());
    let base_antecedent = atom(&mut nodes, "dog", var());
    let base_conclusion = atom(&mut nodes, "animal", var());

    let (antecedent, conclusion) = match placement {
        RulePlacement::Antecedent => (stacked(&mut nodes, base_antecedent, order), base_conclusion),
        RulePlacement::Conclusion => (base_antecedent, stacked(&mut nodes, base_conclusion, order)),
        RulePlacement::Naf => {
            let absent = atom(&mut nodes, "cat", var());
            let not_absent = not(&mut nodes, absent);
            let stacked_naf = stacked(&mut nodes, not_absent, order);
            (
                and(&mut nodes, base_antecedent, stacked_naf),
                base_conclusion,
            )
        }
    };
    let negated_antecedent = not(&mut nodes, antecedent);
    let implication = or(&mut nodes, negated_antecedent, conclusion);
    let root = forall(&mut nodes, "_v0", implication);
    LogicBuffer {
        nodes,
        roots: vec![root],
    }
}

fn expect_reasoning_stack_error<T: std::fmt::Debug>(result: Result<T, NibliError>) {
    match result {
        Err(NibliError::Reasoning(message)) => {
            assert!(message.contains(STACK_ERR), "{message}");
            assert!(message.contains("one flavor"), "{message}");
        }
        other => panic!("expected fail-closed stacked-wrapper error, got {other:?}"),
    }
}

#[test]
fn stacked_wrappers_reject_every_query_and_assertion_ingress_in_both_orders() {
    for order in [StackOrder::DeonticOuter, StackOrder::TemporalOuter] {
        let kb = new_kb();
        expect_reasoning_stack_error(kb.assert_fact(stacked_fact(order), "stacked".into()));
        expect_reasoning_stack_error(kb.query_entailment(stacked_fact(order)));
        expect_reasoning_stack_error(kb.query_find(stacked_fact(order)));
        // An error is returned before a proof trace can be constructed.
        expect_reasoning_stack_error(kb.query_entailment_with_proof(stacked_fact(order)));

        // Validation precedes materialisation, so both runtime settings expose
        // exactly the same rejection instead of building state from a bad query.
        for enabled in [false, true] {
            kb.set_materialization(enabled);
            expect_reasoning_stack_error(kb.query_entailment(stacked_fact(order)));
        }
    }
}

#[test]
fn stacked_wrappers_reject_rule_antecedent_conclusion_and_naf_in_both_orders() {
    for order in [StackOrder::DeonticOuter, StackOrder::TemporalOuter] {
        for placement in [
            RulePlacement::Antecedent,
            RulePlacement::Conclusion,
            RulePlacement::Naf,
        ] {
            let kb = new_kb();
            expect_reasoning_stack_error(
                kb.assert_fact(stacked_rule(order, placement), format!("{placement:?}")),
            );
            assert!(kb.list_facts().unwrap().is_empty());
        }
    }
}

#[test]
fn rejection_is_atomic_and_persisted_rows_cannot_bypass_it() {
    let kb = new_kb();
    let first = kb
        .assert_fact(make_assertion("rex", "dog"), "valid-first".into())
        .unwrap();

    // A valid first root followed by an invalid stacked root is rejected before
    // either root mutates the store or an assertion id is consumed.
    let mut mixed = stacked_fact(StackOrder::DeonticOuter);
    let valid = mixed.nodes.len() as u32;
    mixed.nodes.push(LogicNode::Predicate((
        "cat".to_string(),
        vec![LogicalTerm::Constant("milo".to_string())],
    )));
    mixed.roots.insert(0, valid);
    expect_reasoning_stack_error(kb.assert_fact(mixed, "mixed".into()));
    assert_eq!(
        kb.list_facts()
            .unwrap()
            .iter()
            .map(|fact| fact.id)
            .collect::<Vec<_>>(),
        vec![first]
    );

    let second = kb
        .assert_fact(make_assertion("milo", "cat"), "valid-second".into())
        .unwrap();
    assert_eq!(second, first + 1, "rejection must not consume a fact id");
    kb.rebuild().unwrap();

    // Persistence replay uses a pre-assigned id. Validate before advancing the
    // counter or inserting a durable record, then prove ordinary retraction still works.
    let err = kb
        .assert_fact_with_id(
            stacked_fact(StackOrder::TemporalOuter),
            "legacy-stacked-row".into(),
            99,
        )
        .unwrap_err();
    assert!(err.contains(STACK_ERR), "{err}");
    kb.retract_fact(first).unwrap();
    kb.retract_fact(second).unwrap();
    assert!(kb.list_facts().unwrap().is_empty());

    let third = kb
        .assert_fact(make_assertion("nora", "person"), "valid-third".into())
        .unwrap();
    assert_eq!(
        third,
        second + 1,
        "bad persisted id must not advance counter"
    );
}

#[test]
fn separate_formula_paths_keep_independent_single_flavors() {
    // The validator is path-sensitive, not buffer-global: sibling facts and
    // separate rule literals may choose different single flavors.
    let kb = new_kb();
    let mut nodes = Vec::new();
    let p = atom(&mut nodes, "dog", LogicalTerm::Constant("rex".to_string()));
    let p = past(&mut nodes, p);
    let q = atom(
        &mut nodes,
        "animal",
        LogicalTerm::Constant("milo".to_string()),
    );
    let q = obligatory(&mut nodes, q);
    let root = and(&mut nodes, p, q);
    kb.assert_fact(
        LogicBuffer {
            nodes,
            roots: vec![root],
        },
        "separate-siblings".into(),
    )
    .unwrap();

    let mut rule_nodes = Vec::new();
    let ante = atom(
        &mut rule_nodes,
        "dog",
        LogicalTerm::Variable("_v0".to_string()),
    );
    let ante = past(&mut rule_nodes, ante);
    let negated = not(&mut rule_nodes, ante);
    let cons = atom(
        &mut rule_nodes,
        "animal",
        LogicalTerm::Variable("_v0".to_string()),
    );
    let cons = obligatory(&mut rule_nodes, cons);
    let body = or(&mut rule_nodes, negated, cons);
    let root = forall(&mut rule_nodes, "_v0", body);
    kb.assert_fact(
        LogicBuffer {
            nodes: rule_nodes,
            roots: vec![root],
        },
        "past-to-obligatory".into(),
    )
    .unwrap();
}
