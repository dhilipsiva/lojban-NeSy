use super::*;

fn raw_fact(relation: &str, constant: &str) -> LogicBuffer {
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

fn raw_implication(antecedent: &str, consequent: &str) -> LogicBuffer {
    let mut nodes = Vec::new();
    let variable = LogicalTerm::Variable("x".to_string());
    let antecedent = pred(&mut nodes, antecedent, vec![variable.clone()]);
    let consequent = pred(&mut nodes, consequent, vec![variable]);
    let not_antecedent = not(&mut nodes, antecedent);
    let implication = or(&mut nodes, not_antecedent, consequent);
    let root = forall(&mut nodes, "x", implication);
    LogicBuffer {
        nodes,
        roots: vec![root],
    }
}

fn raw_cycle_body(nodes: &mut Vec<LogicNode>) -> u32 {
    let variable = LogicalTerm::Variable("x".to_string());
    let person = pred(nodes, "person", vec![variable.clone()]);
    let dog = pred(nodes, "dog", vec![variable]);
    let not_dog = not(nodes, dog);
    and(nodes, person, not_dog)
}

fn raw_cycle_query() -> LogicBuffer {
    let mut nodes = Vec::new();
    let body = raw_cycle_body(&mut nodes);
    let root = exists(&mut nodes, "x", body);
    LogicBuffer {
        nodes,
        roots: vec![root],
    }
}

fn raw_cycle_exact_zero_query() -> LogicBuffer {
    let mut nodes = Vec::new();
    let body = raw_cycle_body(&mut nodes);
    let root = nodes.len() as u32;
    nodes.push(LogicNode::CountNode(("x".to_string(), 0, body)));
    LogicBuffer {
        nodes,
        roots: vec![root],
    }
}

fn raw_cycle_kb() -> KnowledgeBase {
    let kb = KnowledgeBase::new();
    kb.assert_fact(raw_fact("person", "adam"), "person-adam".to_string())
        .expect("person assertion");
    kb.assert_fact(raw_implication("dog", "cat"), "dog-to-cat".to_string())
        .expect("dog-to-cat rule");
    kb.assert_fact(raw_implication("cat", "dog"), "cat-to-dog".to_string())
        .expect("cat-to-dog rule");
    kb
}

#[test]
fn public_raw_collections_refuse_a_non_definitive_cycle_naf_leaf() {
    let kb = raw_cycle_kb();

    let query = raw_cycle_query();
    assert_eq!(
        kb.query_entailment(query.clone()).unwrap(),
        QueryResult::Unknown(UnknownReason::NafDependent),
        "premise: the neutral cycle under negation must remain NAF-dependent"
    );
    assert_incomplete_enumeration(kb.query_find(query.clone()));
    assert_incomplete_enumeration(kb.count_witnesses(query.clone()));
    assert_incomplete_enumeration(kb.aggregate(query, "x", nibli_types::logic::AggregateOp::Sum));

    assert_eq!(
        kb.query_entailment(raw_cycle_exact_zero_query()).unwrap(),
        QueryResult::Unknown(UnknownReason::NafDependent),
        "CountNode has a separate evaluator and remains non-definitive"
    );
}

fn raw_cycle_with_absorbing_false_query(unknown_first: bool) -> LogicBuffer {
    let mut nodes = Vec::new();
    let variable = LogicalTerm::Variable("x".to_string());
    let person = pred(&mut nodes, "person", vec![variable.clone()]);
    let dog = pred(&mut nodes, "dog", vec![variable.clone()]);
    let not_dog = not(&mut nodes, dog);
    let person_again = pred(&mut nodes, "person", vec![variable]);
    let not_person = not(&mut nodes, person_again);
    let pair = if unknown_first {
        and(&mut nodes, not_dog, not_person)
    } else {
        and(&mut nodes, not_person, not_dog)
    };
    let body = and(&mut nodes, person, pair);
    let root = exists(&mut nodes, "x", body);
    LogicBuffer {
        nodes,
        roots: vec![root],
    }
}

fn raw_cycle_or_true_with_absorbing_false_query(
    non_definitive_first: bool,
    choice_first: bool,
) -> LogicBuffer {
    let mut nodes = Vec::new();
    let variable = LogicalTerm::Variable("x".to_string());
    let dog = pred(&mut nodes, "dog", vec![variable.clone()]);
    let non_definitive = not(&mut nodes, dog);
    let person = pred(&mut nodes, "person", vec![variable.clone()]);
    let choice = if non_definitive_first {
        or(&mut nodes, non_definitive, person)
    } else {
        or(&mut nodes, person, non_definitive)
    };
    let person_again = pred(&mut nodes, "person", vec![variable]);
    let definitive_false = not(&mut nodes, person_again);
    let body = if choice_first {
        and(&mut nodes, choice, definitive_false)
    } else {
        and(&mut nodes, definitive_false, choice)
    };
    let root = exists(&mut nodes, "x", body);
    LogicBuffer {
        nodes,
        roots: vec![root],
    }
}

fn raw_multi_root_cycle_with_absorbing_false_query(unknown_first: bool) -> LogicBuffer {
    let mut nodes = Vec::new();
    let unknown_body = raw_cycle_body(&mut nodes);
    let unknown_root = exists(&mut nodes, "x", unknown_body);

    let variable = LogicalTerm::Variable("y".to_string());
    let person = pred(&mut nodes, "person", vec![variable.clone()]);
    let person_again = pred(&mut nodes, "person", vec![variable]);
    let not_person = not(&mut nodes, person_again);
    let false_body = and(&mut nodes, person, not_person);
    let false_root = exists(&mut nodes, "y", false_body);

    let roots = if unknown_first {
        vec![unknown_root, false_root]
    } else {
        vec![false_root, unknown_root]
    };
    LogicBuffer { nodes, roots }
}

fn assert_definitive_empty_collections(kb: &KnowledgeBase, query: LogicBuffer) {
    assert_eq!(
        kb.query_find(query.clone()).unwrap(),
        Vec::<Vec<WitnessBinding>>::new()
    );
    assert_eq!(kb.count_witnesses(query.clone()).unwrap(), 0);
    assert_eq!(
        kb.aggregate(query, "x", nibli_types::logic::AggregateOp::Sum)
            .unwrap(),
        None
    );
}

#[test]
fn definitive_false_absorbs_a_non_definitive_conjunct_in_either_order() {
    for unknown_first in [true, false] {
        let kb = raw_cycle_kb();
        let query = raw_cycle_with_absorbing_false_query(unknown_first);
        assert_eq!(
            kb.query_entailment(query.clone()).unwrap(),
            QueryResult::False,
            "premise: False absorbs Unknown for conjunction order; unknown_first={unknown_first}"
        );
        assert_definitive_empty_collections(&kb, query);
    }

    for non_definitive_first in [true, false] {
        for choice_first in [true, false] {
            let kb = raw_cycle_kb();
            let query =
                raw_cycle_or_true_with_absorbing_false_query(non_definitive_first, choice_first);
            assert_eq!(
                kb.query_entailment(query.clone()).unwrap(),
                QueryResult::False,
                "premise: outer False must absorb the inner Unknown OR True"
            );
            assert_definitive_empty_collections(&kb, query);
        }
    }

    for unknown_first in [true, false] {
        let kb = raw_cycle_kb();
        let query = raw_multi_root_cycle_with_absorbing_false_query(unknown_first);
        assert_eq!(
            kb.query_entailment(query.clone()).unwrap(),
            QueryResult::False,
            "premise: raw roots are conjunctive; False absorbs Unknown; unknown_first={unknown_first}"
        );
        assert_definitive_empty_collections(&kb, query);
    }
}
