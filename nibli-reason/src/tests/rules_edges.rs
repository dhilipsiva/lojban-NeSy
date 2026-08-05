use super::*;

// ─── SE-conversion + universal rule + targeted witness tests ────

/// Build a 2-arg universal rule with different argument structures:
/// ∀x. restrictor(x, _) → consequent(fixed_entity, x, _)
/// This simulates "ro lo gerku cu se nelci la .bob." where SE swaps x1↔x2,
/// producing: ∀x. gerku(x) → nelci(bob, x)
fn make_universal_2arg(restrictor: &str, consequent: &str, fixed_entity: &str) -> LogicBuffer {
    let mut nodes = Vec::new();
    // restrictor(x, _)
    let restrict = pred(
        &mut nodes,
        restrictor,
        vec![
            LogicalTerm::Variable("_v0".to_string()),
            LogicalTerm::Unspecified,
        ],
    );
    // consequent(fixed_entity, x, _)
    let body = pred(
        &mut nodes,
        consequent,
        vec![
            LogicalTerm::Constant(fixed_entity.to_string()),
            LogicalTerm::Variable("_v0".to_string()),
            LogicalTerm::Unspecified,
        ],
    );
    let neg = not(&mut nodes, restrict);
    let disj = or(&mut nodes, neg, body);
    let root = forall(&mut nodes, "_v0", disj);
    LogicBuffer {
        nodes,
        roots: vec![root],
    }
}

#[test]
fn test_x2_conversion_universal_rule() {
    // Simulates the REPL demo:
    //   la .alis. gerku          → gerku(alis)
    //   ro lo gerku cu se nelci la .bob.  → ∀x. gerku(x) → nelci(bob, x)
    //   ? la .bob. nelci la .alis.        → nelci(bob, alis) = TRUE
    let kb = new_kb();
    assert_buf(&kb, make_assertion("alis", "gerku"));
    assert_buf(&kb, make_universal_2arg("gerku", "nelci", "bob"));

    // Query: nelci(bob, alis) — should be TRUE via universal rule
    let mut nodes = Vec::new();
    let root = pred(
        &mut nodes,
        "nelci",
        vec![
            LogicalTerm::Constant("bob".to_string()),
            LogicalTerm::Constant("alis".to_string()),
            LogicalTerm::Unspecified,
        ],
    );
    assert!(query(
        &kb,
        LogicBuffer {
            nodes,
            roots: vec![root]
        }
    ));
}

#[test]
fn test_x2_conversion_universal_multiple_entities() {
    // Two dogs: both should be liked by bob via universal rule
    let kb = new_kb();
    assert_buf(&kb, make_assertion("alis", "gerku"));
    assert_buf(&kb, make_assertion("rex", "gerku"));
    assert_buf(&kb, make_universal_2arg("gerku", "nelci", "bob"));

    // nelci(bob, alis) = TRUE
    let mut n1 = Vec::new();
    let r1 = pred(
        &mut n1,
        "nelci",
        vec![
            LogicalTerm::Constant("bob".to_string()),
            LogicalTerm::Constant("alis".to_string()),
            LogicalTerm::Unspecified,
        ],
    );
    assert!(query(
        &kb,
        LogicBuffer {
            nodes: n1,
            roots: vec![r1]
        }
    ));

    // nelci(bob, rex) = TRUE
    let mut n2 = Vec::new();
    let r2 = pred(
        &mut n2,
        "nelci",
        vec![
            LogicalTerm::Constant("bob".to_string()),
            LogicalTerm::Constant("rex".to_string()),
            LogicalTerm::Unspecified,
        ],
    );
    assert!(query(
        &kb,
        LogicBuffer {
            nodes: n2,
            roots: vec![r2]
        }
    ));

    // nelci(bob, carol) = FALSE (carol is not a dog)
    let mut n3 = Vec::new();
    let r3 = pred(
        &mut n3,
        "nelci",
        vec![
            LogicalTerm::Constant("bob".to_string()),
            LogicalTerm::Constant("carol".to_string()),
            LogicalTerm::Unspecified,
        ],
    );
    assert!(query_false(
        &kb,
        LogicBuffer {
            nodes: n3,
            roots: vec![r3]
        }
    ));
}

#[test]
fn test_targeted_witness_search_with_fixed_entity() {
    // Simulates the REPL demo:
    //   la .alis. gerku          → gerku(alis)
    //   ro lo gerku cu se nelci la .bob.  → ∀x. gerku(x) → nelci(bob, x)
    //   ?? ma se nelci la .bob.           → ∃x. nelci(bob, x) → includes alis
    let kb = new_kb();
    assert_buf(&kb, make_assertion("alis", "gerku"));
    assert_buf(&kb, make_universal_2arg("gerku", "nelci", "bob"));

    // Query: ∃x. nelci(bob, x) — clean-core should find only alis.
    let mut nodes = Vec::new();
    let body = pred(
        &mut nodes,
        "nelci",
        vec![
            LogicalTerm::Constant("bob".to_string()),
            LogicalTerm::Variable("x".to_string()),
            LogicalTerm::Unspecified,
        ],
    );
    let root = exists(&mut nodes, "x", body);
    let results = query_find(
        &kb,
        LogicBuffer {
            nodes,
            roots: vec![root],
        },
    );

    assert_eq!(results.len(), 1);
    let found: Vec<String> = results
        .iter()
        .filter_map(|bs| match &bs[0].term {
            LogicalTerm::Constant(c) => Some(c.clone()),
            _ => None,
        })
        .collect();
    assert!(
        found.contains(&"alis".to_string()),
        "alis should be a witness"
    );
}

#[test]
fn test_targeted_witness_search_multiple_matches() {
    // Two dogs → both should appear as witnesses for "who does bob like?"
    let kb = new_kb();
    assert_buf(&kb, make_assertion("alis", "gerku"));
    assert_buf(&kb, make_assertion("rex", "gerku"));
    assert_buf(&kb, make_universal_2arg("gerku", "nelci", "bob"));

    // Query: ∃x. nelci(bob, x) — clean-core should find alis and rex.
    let mut nodes = Vec::new();
    let body = pred(
        &mut nodes,
        "nelci",
        vec![
            LogicalTerm::Constant("bob".to_string()),
            LogicalTerm::Variable("x".to_string()),
            LogicalTerm::Unspecified,
        ],
    );
    let root = exists(&mut nodes, "x", body);
    let results = query_find(
        &kb,
        LogicBuffer {
            nodes,
            roots: vec![root],
        },
    );

    assert_eq!(results.len(), 2);
    let found: Vec<String> = results
        .iter()
        .filter_map(|bs| match &bs[0].term {
            LogicalTerm::Constant(c) => Some(c.clone()),
            _ => None,
        })
        .collect();
    assert!(
        found.contains(&"alis".to_string()),
        "alis should be a witness"
    );
    assert!(
        found.contains(&"rex".to_string()),
        "rex should be a witness"
    );
}

#[test]
fn test_conjunction_introduction_multiple_entities() {
    let kb = new_kb();
    assert_buf(&kb, make_assertion("alis", "gerku"));
    assert_buf(&kb, make_assertion("alis", "barda"));
    assert_buf(&kb, make_assertion("bob", "mlatu"));
    assert_buf(&kb, make_assertion("bob", "cmalu"));

    // alis predicates should conjoin with each other
    assert!(query_conjunction(&kb, "gerku", "alis", "barda", "alis"));
    // bob predicates should conjoin with each other
    assert!(query_conjunction(&kb, "mlatu", "bob", "cmalu", "bob"));
    // cross-entity conjunction also holds (both sides individually true)
    assert!(query_conjunction(&kb, "gerku", "alis", "mlatu", "bob"));
}

// ─── KB Reset Tests ──────────────────────────────────────────

#[test]
fn test_kb_reset_clears_facts() {
    let kb = new_kb();
    assert_buf(&kb, make_assertion("alis", "gerku"));
    assert!(query(&kb, make_query("alis", "gerku")));

    // Reset the knowledge base
    kb.inner.borrow_mut().reset();

    // After reset, previously asserted fact should no longer hold
    assert!(query_false(&kb, make_query("alis", "gerku")));
}

#[test]
fn test_kb_reset_clears_rules() {
    let kb = new_kb();
    assert_buf(&kb, make_assertion("alis", "gerku"));
    assert_buf(&kb, make_universal("gerku", "danlu"));
    assert!(query(&kb, make_query("alis", "danlu")));

    kb.inner.borrow_mut().reset();

    // After reset, re-assert the fact but not the rule
    assert_buf(&kb, make_assertion("alis", "gerku"));
    // Rule should not exist anymore
    assert!(query_false(&kb, make_query("alis", "danlu")));
}

#[test]
fn test_kb_reset_resets_skolem_counter() {
    let kb = new_kb();
    kb.set_existential_import(true).unwrap();
    // Assert a universal to trigger Skolem generation
    assert_buf(&kb, make_universal("gerku", "danlu"));
    let counter_before = kb.inner.borrow().skolem_counter;
    assert!(counter_before > 0);

    kb.inner.borrow_mut().reset();
    assert_eq!(kb.inner.borrow().skolem_counter, 0);
}

// ─── Empty buffer / edge case tests ──────────────────────────

#[test]
fn test_query_with_no_facts() {
    let kb = new_kb();
    assert!(query_false(&kb, make_query("alis", "gerku")));
}

#[test]
fn test_assert_and_query_same_fact_twice() {
    let kb = new_kb();
    assert_buf(&kb, make_assertion("alis", "gerku"));
    assert_buf(&kb, make_assertion("alis", "gerku"));
    // Should still hold and not cause issues
    assert!(query(&kb, make_query("alis", "gerku")));
}

// ─── Disjunction query tests ─────────────────────────────────

#[test]
fn test_disjunction_left_true() {
    let kb = new_kb();
    assert_buf(&kb, make_assertion("alis", "gerku"));

    let mut nodes = Vec::new();
    let left = pred(
        &mut nodes,
        "gerku",
        vec![
            LogicalTerm::Constant("alis".into()),
            LogicalTerm::Unspecified,
        ],
    );
    let right = pred(
        &mut nodes,
        "mlatu",
        vec![
            LogicalTerm::Constant("alis".into()),
            LogicalTerm::Unspecified,
        ],
    );
    let root = or(&mut nodes, left, right);
    assert!(query(
        &kb,
        LogicBuffer {
            nodes,
            roots: vec![root]
        }
    ));
}

#[test]
fn test_disjunction_right_true() {
    let kb = new_kb();
    assert_buf(&kb, make_assertion("alis", "mlatu"));

    let mut nodes = Vec::new();
    let left = pred(
        &mut nodes,
        "gerku",
        vec![
            LogicalTerm::Constant("alis".into()),
            LogicalTerm::Unspecified,
        ],
    );
    let right = pred(
        &mut nodes,
        "mlatu",
        vec![
            LogicalTerm::Constant("alis".into()),
            LogicalTerm::Unspecified,
        ],
    );
    let root = or(&mut nodes, left, right);
    assert!(query(
        &kb,
        LogicBuffer {
            nodes,
            roots: vec![root]
        }
    ));
}

#[test]
fn test_disjunction_both_false() {
    let kb = new_kb();

    let mut nodes = Vec::new();
    let left = pred(
        &mut nodes,
        "gerku",
        vec![
            LogicalTerm::Constant("alis".into()),
            LogicalTerm::Unspecified,
        ],
    );
    let right = pred(
        &mut nodes,
        "mlatu",
        vec![
            LogicalTerm::Constant("alis".into()),
            LogicalTerm::Unspecified,
        ],
    );
    let root = or(&mut nodes, left, right);
    assert!(query_false(
        &kb,
        LogicBuffer {
            nodes,
            roots: vec![root]
        }
    ));
}

// ─── Double negation tests ───────────────────────────────────

#[test]
fn test_double_negation_elimination() {
    let kb = new_kb();
    assert_buf(&kb, make_assertion("alis", "gerku"));

    // Query Not(Not(gerku(alis))) → should be TRUE
    let mut nodes = Vec::new();
    let inner = pred(
        &mut nodes,
        "gerku",
        vec![
            LogicalTerm::Constant("alis".into()),
            LogicalTerm::Unspecified,
        ],
    );
    let neg1 = not(&mut nodes, inner);
    let root = not(&mut nodes, neg1);
    assert!(query(
        &kb,
        LogicBuffer {
            nodes,
            roots: vec![root]
        }
    ));
}

// ─── Mutation-audit kill-tests (2026-07-19) ──────────────────

/// Kills rules.rs `replace + with * in register_rule` (the stratification
/// rollback pop count `typed_conditions.len() + group_edge_count`). With one
/// flat (negated) condition plus a one-condition negated-exists group the
/// correct pop count is 1 + 1 = 2; the mutant pops 1 * 1 = 1, so the phantom
/// negative dog→dog self-edge survives the rejection and wrongly rejects the
/// next, perfectly stratifiable rule. Exercised at the `register_rule` seam
/// directly — the public assert path hides the rollback behind a full
/// registry rebuild.
#[test]
fn stratification_rollback_pops_flat_plus_group_edges() {
    let kb = new_kb();
    let pv = || GroundTerm::PatternVar("x__v0".to_string());
    {
        let mut inner = kb.inner.borrow_mut();
        // dog(x) ← ¬dog(x) ∧ ¬∃ev.person(ev): the flat NAF condition is a
        // negative self-loop → rejected as unstratifiable → rollback.
        let result = crate::rules::register_rule(
            &mut inner,
            crate::kb::RuleKind::Conditional,
            "dog <- ~dog + ~exists-person-group".to_string(),
            vec!["x__v0".to_string()],
            vec![StoredFact::Bare(GroundFact::new("dog", vec![pv()]))],
            vec![StoredFact::Bare(GroundFact::new("dog", vec![pv()]))],
            vec![0],
            vec![crate::kb::NegatedExistsGroup {
                conditions: vec![StoredFact::Bare(GroundFact::new(
                    "person",
                    vec![GroundTerm::PatternVar("ev__g0".to_string())],
                ))],
                event_var: "ev__g0".to_string(),
            }],
            false,
            false,
        );
        assert!(
            result.is_err(),
            "the negative self-loop must be rejected as unstratifiable"
        );
    }
    // After a CORRECT rollback the dep graph holds no dog edges, so this
    // stratifiable rule must register (under the mutant the phantom negative
    // dog→dog edge makes it reject) and then fire.
    assert_buf(&kb, make_universal("dog", "animal"));
    assert_buf(&kb, make_assertion("rex", "dog"));
    assert!(
        query(&kb, make_query("rex", "animal")),
        "the post-rollback rule must register and derive animal(rex)"
    );
}

/// Kills the BARE-UNIVERSAL twin of the rules.rs `replace == with != in
/// compile_forall_to_rule` skolem_fn_registry dedup (the restrictor-less
/// branch, ~line 1700; the implication-branch twin at ~1631 dies to the
/// engine test `second_witness_family_survives_skolem_registry_dedup`).
/// With the registry already occupied by the first rule's base, the mutant
/// skips every later distinct base; the bare rule's dep-2 witness then falls
/// back to `unwrap_or(1)` and enumerates with the wrong dependency shape
/// (`sk(a)` instead of `sk((a, b))`), losing the entailment. Raw flat buffer
/// on purpose: the KR surface fail-closes bare universals with decomposed
/// bodies, so `KnowledgeBase::assert_fact` raw-FOL injection is the one path
/// that reaches this branch with a SURVIVING registration.
#[test]
fn bare_universal_second_witness_base_survives_registry_dedup() {
    let kb = new_kb();
    // Occupy the registry with an implication-branch dep-1 base first.
    assert_buf(&kb, make_dependent_skolem_universal("dog", "loves"));
    // ∀u ∀w ∃y. gives(u, y, w) — a dep-2 witness in the bare-universal branch.
    let mut nodes = Vec::new();
    let g = pred(
        &mut nodes,
        "gives",
        vec![
            LogicalTerm::Variable("_u0".to_string()),
            LogicalTerm::Variable("_y0".to_string()),
            LogicalTerm::Variable("_w0".to_string()),
        ],
    );
    let e = exists(&mut nodes, "_y0", g);
    let fw = forall(&mut nodes, "_w0", e);
    let root = forall(&mut nodes, "_u0", fw);
    assert_buf(
        &kb,
        LogicBuffer {
            nodes,
            roots: vec![root],
        },
    );
    assert_buf(&kb, make_assertion("adam", "dog"));
    assert_buf(&kb, make_assertion("rex", "cat"));
    // ∃y. gives(adam, y, rex) — provable only through the dep-2 witness
    // sk((adam, rex)), whose enumeration needs the registry entry.
    let mut nodes = Vec::new();
    let q = pred(
        &mut nodes,
        "gives",
        vec![
            LogicalTerm::Constant("adam".to_string()),
            LogicalTerm::Variable("y".to_string()),
            LogicalTerm::Constant("rex".to_string()),
        ],
    );
    let root = exists(&mut nodes, "y", q);
    assert!(
        query(
            &kb,
            LogicBuffer {
                nodes,
                roots: vec![root],
            },
        ),
        "the bare universal's (u, w)-dependent witness must derive gives(adam, y, rex)"
    );
}

/// Kills reasoning.rs `replace || with && in fold_negated_groups`: when the
/// positive restrictor conditions already failed DEFINITIVELY, the rule's
/// negated-exists groups must be SKIPPED. The mutant evaluates them anyway,
/// and a non-definitive group verdict (here: the cat/animal positive cycle
/// makes the ~cat check Unknown(CycleCut)) resurrects `pending`, degrading
/// the query's definitive FALSE to Unknown. Surface-compiled: the NAF group
/// shape only exists on the event-decomposed pipeline. The `dog(every wolf).`
/// rule keeps `dog` NON-index-decidable so the unbound-event combo loop
/// actually runs (an index-decidable restrictor prunes every candidate and
/// never reaches the fold at all).
#[test]
fn naf_groups_skipped_when_positive_conditions_definitively_fail() {
    let kb = new_kb();
    assert_buf(&kb, compile_surface("beautiful(every dog where ~cat(it))."));
    assert_buf(&kb, compile_surface("dog(every wolf)."));
    assert_buf(&kb, compile_surface("cat(every animal)."));
    assert_buf(&kb, compile_surface("animal(every cat)."));
    assert_buf(&kb, compile_surface("person(Bel)."));
    assert_buf(&kb, compile_surface("cat(Kim)."));
    // Bel is no dog (definitive CWA FALSE on the positive restrictor), so the
    // ~cat group must never run — its Unknown would leak into the verdict.
    assert!(
        query_false(&kb, compile_surface("beautiful(Bel).")),
        "a definitively failed restrictor must yield definitive FALSE, not Unknown"
    );
}

/// The seam-level twin of the fold_negated_groups kill (the no-unbound-event
/// call site at process_phase's tail): a FLAT rule with a negated-exists
/// group whose positive condition fails definitively for the queried entity.
/// Unmutated, the fold's short-circuit keeps the verdict a definitive FALSE;
/// under `|| with &&` the ~cat group runs, hits the cat/animal positive
/// cycle (Unknown(CycleCut)), and the query degrades to Unknown.
#[test]
fn flat_naf_group_skipped_when_positive_condition_definitively_fails() {
    let kb = new_kb();
    // Clean-core profile: no existential-import witnesses. With explicit import
    // ON, `∀x animal(x)→cat(x)` would mint a presupposition witness that IS a
    // derivable cat — the ~cat group would then find a witness and return a
    // definitive False, indistinguishable from the skip.
    kb.set_existential_import(false).unwrap();
    // Positive cycle: cat <-> animal (stratifiable; makes any cat(x) check
    // non-definitive for an underivable x).
    assert_buf(&kb, make_universal("animal", "cat"));
    assert_buf(&kb, make_universal("cat", "animal"));
    assert_buf(&kb, make_assertion("bel", "person"));
    {
        let mut inner = kb.inner.borrow_mut();
        let pv = |n: &str| GroundTerm::PatternVar(n.to_string());
        // beautiful(x) <- dog(x) ∧ ¬∃e.cat(e): flat positive condition +
        // a one-condition negated-exists group.
        crate::rules::register_rule(
            &mut inner,
            crate::kb::RuleKind::Conditional,
            "beautiful <- dog + ~exists-cat".to_string(),
            vec!["x__v0".to_string()],
            vec![StoredFact::Bare(GroundFact::new(
                "dog",
                vec![pv("x__v0"), GroundTerm::Unspecified],
            ))],
            vec![StoredFact::Bare(GroundFact::new(
                "beautiful",
                vec![pv("x__v0"), GroundTerm::Unspecified],
            ))],
            vec![],
            vec![crate::kb::NegatedExistsGroup {
                conditions: vec![StoredFact::Bare(GroundFact::new(
                    "cat",
                    vec![pv("ev__g0"), GroundTerm::Unspecified],
                ))],
                event_var: "ev__g0".to_string(),
            }],
            false,
            false,
        )
        .expect("the flat NAF-group rule must register (stratifiable)");
    }
    // dog(bel) is definitively FALSE (no dog facts or rules), so the ~cat
    // group must be skipped — evaluating it would leak the cycle's Unknown.
    assert!(
        query_false(&kb, make_query("bel", "beautiful")),
        "a definitively failed flat condition must yield definitive FALSE, not Unknown"
    );
}

fn forced_rule_digest(_: &crate::kb::RuleIdentity) -> u64 {
    0
}

#[test]
fn forced_rule_digest_collision_survives_rebuild_and_retraction() {
    let kb = new_kb();
    kb.inner
        .borrow_mut()
        .known_rules
        .force_digest_for_test(forced_rule_digest);

    let first_rule = assert_id(
        &kb,
        compile_surface("all $x: dog($x) -> animal($x)."),
        "dog -> animal",
    );
    let _second_rule = assert_id(
        &kb,
        compile_surface("all $x: cat($x) -> beautiful($x)."),
        "cat -> beautiful",
    );
    let spacer = assert_id(&kb, compile_surface("big(Spacer)."), "spacer");
    assert_buf(&kb, compile_surface("dog(Adam)."));
    assert_buf(&kb, compile_surface("cat(Bel)."));

    assert!(query(&kb, compile_surface("animal(Adam).")));
    assert!(query(&kb, compile_surface("beautiful(Bel).")));
    assert_eq!(kb.inner.borrow().known_rules.identity_count(), 2);

    kb.retract_fact_inner(spacer)
        .expect("unrelated retraction must rebuild under the forced collision");
    assert!(query(&kb, compile_surface("animal(Adam).")));
    assert!(query(&kb, compile_surface("beautiful(Bel).")));
    assert_eq!(kb.inner.borrow().known_rules.identity_count(), 2);

    kb.retract_fact_inner(first_rule)
        .expect("retracting one colliding rule must preserve the other");
    assert!(query_false(&kb, compile_surface("animal(Adam).")));
    assert!(query(&kb, compile_surface("beautiful(Bel).")));
    assert_eq!(kb.inner.borrow().known_rules.identity_count(), 1);
}

#[test]
fn alpha_equivalent_surface_rules_deduplicate_without_orphan_skolems() {
    let kb = new_kb();
    let first = assert_id(
        &kb,
        compile_surface("all $x: dog($x) -> animal($x)."),
        "first spelling",
    );
    let registry_after_first = kb.inner.borrow().skolem_fn_registry.len();
    let _renamed = assert_id(
        &kb,
        compile_surface("all $renamed: dog($renamed) -> animal($renamed)."),
        "alpha-renamed spelling",
    );

    assert_eq!(
        kb.inner.borrow().known_rules.identity_count(),
        1,
        "binder and generated-Skolem names are not semantic identity"
    );
    assert_eq!(
        kb.inner.borrow().skolem_fn_registry.len(),
        registry_after_first,
        "a skipped duplicate must not leave an unused Skolem family"
    );

    assert_buf(&kb, compile_surface("dog(Adam)."));
    assert!(query(&kb, compile_surface("animal(Adam).")));
    kb.retract_fact_inner(first)
        .expect("the surviving duplicate buffer must take over during replay");
    assert!(query(&kb, compile_surface("animal(Adam).")));
    assert_eq!(kb.inner.borrow().known_rules.identity_count(), 1);
}

#[test]
fn description_import_is_independent_of_alpha_equivalent_rule_dedup() {
    let prenex = "all $x: dog($x) -> animal($x).";
    let description = "animal(every dog).";

    // Either assertion order must retain the description's xorlo
    // presupposition. The executable rule remains one canonical identity.
    for description_first in [false, true] {
        let kb = new_kb();
        kb.set_existential_import(true).unwrap();
        if description_first {
            assert_buf(&kb, compile_surface(description));
            assert_buf(&kb, compile_surface(prenex));
        } else {
            assert_buf(&kb, compile_surface(prenex));
            assert_buf(&kb, compile_surface(description));
        }
        assert!(
            query(&kb, compile_surface("dog(some dog).")),
            "description import was lost with description_first={description_first}"
        );
        assert_eq!(
            kb.inner.borrow().known_rules.identity_count(),
            1,
            "assertion-side import must not duplicate the executable rule"
        );
    }

    // With existential import disabled, both spellings are genuinely the same
    // plain universal and no phantom witness is introduced.
    let clean_core = new_kb();
    clean_core.set_existential_import(false).unwrap();
    assert_buf(&clean_core, compile_surface(prenex));
    assert_buf(&clean_core, compile_surface(description));
    assert!(query_false(&clean_core, compile_surface("dog(some dog).")));
    assert_eq!(clean_core.inner.borrow().known_rules.identity_count(), 1);

    // The mode is mutable session configuration. Enabling it immediately
    // rebuilds the existing description without requiring a reassertion.
    clean_core.set_existential_import(true).unwrap();
    assert!(query(&clean_core, compile_surface("dog(some dog).")));
    assert_eq!(clean_core.inner.borrow().known_rules.identity_count(), 1);
}

#[test]
fn description_import_survives_a_duplicate_first_dnf_branch() {
    let kb = new_kb();
    kb.set_existential_import(true).unwrap();
    assert_buf(
        &kb,
        compile_surface("all $x: dog($x) & loves($x) -> animal($x)."),
    );
    assert_buf(
        &kb,
        compile_surface("animal(every dog where loves(it) | friend(it))."),
    );

    assert!(
        query(&kb, compile_surface("dog(some dog).")),
        "a duplicate branch-zero rule must not suppress the description import"
    );
    assert!(
        query(&kb, compile_surface("loves(some dog).")),
        "the imported witness must satisfy the first DNF restrictor branch"
    );
}

#[test]
fn retracting_a_description_does_not_taint_equal_looking_user_constant_origin() {
    let kb = new_kb();
    kb.set_existential_import(true).unwrap();
    let description_id = assert_id(&kb, compile_surface("animal(every dog)."), "description");
    let old_witness = kb
        .inner
        .borrow()
        .known_entities
        .iter()
        .find_map(|term| match term {
            GroundTerm::Skolem(symbol)
                if symbol.origin() == crate::kb::SkolemOrigin::ExistentialImport =>
            {
                Some(symbol.display_name())
            }
            _ => None,
        })
        .expect("description must mint a presupposition witness")
        .clone();

    kb.retract_fact_inner(description_id)
        .expect("description should retract by rebuilding");
    assert!(kb.inner.borrow().known_entities.is_empty());

    // Reuse the exact old internal spelling as an ordinary real entity. A
    // stale origin entry would mislabel the otherwise-correct find result.
    assert_buf(&kb, make_assertion(&old_witness, "dog"));
    let found = query_find(&kb, make_find_query("dog"));
    assert!(
        found.iter().flatten().any(
            |binding| matches!(&binding.term, LogicalTerm::Constant(name) if name == &old_witness)
        ),
        "the reused real entity must remain enumerable: {found:?}"
    );
    assert!(
        found
            .iter()
            .flatten()
            .all(|binding| { binding.origin == nibli_types::logic::WitnessOrigin::KnowledgeBase })
    );
}

#[test]
fn retraction_rebuilds_description_import_claim_from_the_surviving_assertion() {
    let prenex = "all $x: dog($x) -> animal($x).";
    let description = "animal(every dog).";

    let without_description = new_kb();
    without_description.set_existential_import(true).unwrap();
    let _prenex_id = assert_id(&without_description, compile_surface(prenex), "prenex");
    let description_id = assert_id(
        &without_description,
        compile_surface(description),
        "description",
    );
    without_description
        .retract_fact_inner(description_id)
        .expect("description should retract");
    assert!(
        query_false(&without_description, compile_surface("dog(some dog).")),
        "a surviving prenex rule must not retain the retracted description import"
    );

    let without_prenex = new_kb();
    without_prenex.set_existential_import(true).unwrap();
    let prenex_id = assert_id(&without_prenex, compile_surface(prenex), "prenex");
    let _description_id = assert_id(&without_prenex, compile_surface(description), "description");
    without_prenex
        .retract_fact_inner(prenex_id)
        .expect("prenex should retract");
    assert!(
        query(&without_prenex, compile_surface("dog(some dog).")),
        "a surviving description must reclaim its import during replay"
    );
}

#[test]
fn rule_identity_distinguishes_flat_naf_polarity_under_one_digest() {
    let kb = new_kb();
    {
        let mut inner = kb.inner.borrow_mut();
        inner.known_rules.force_digest_for_test(forced_rule_digest);
        let variable = GroundTerm::PatternVar("source-name".to_string());
        let condition = StoredFact::Bare(GroundFact::new(
            "cat",
            vec![variable.clone(), GroundTerm::Unspecified],
        ));
        let conclusion = StoredFact::Bare(GroundFact::new(
            "animal",
            vec![variable, GroundTerm::Unspecified],
        ));
        assert!(
            crate::rules::register_rule(
                &mut inner,
                crate::kb::RuleKind::Conditional,
                "positive".to_string(),
                vec!["source-name".to_string()],
                vec![condition.clone()],
                vec![conclusion.clone()],
                vec![],
                vec![],
                false,
                false,
            )
            .unwrap()
            .rule_registered
        );
        assert!(
            crate::rules::register_rule(
                &mut inner,
                crate::kb::RuleKind::Conditional,
                "negative".to_string(),
                vec!["renamed".to_string()],
                vec![StoredFact::Bare(GroundFact::new(
                    "cat",
                    vec![
                        GroundTerm::PatternVar("renamed".to_string()),
                        GroundTerm::Unspecified,
                    ],
                ))],
                vec![StoredFact::Bare(GroundFact::new(
                    "animal",
                    vec![
                        GroundTerm::PatternVar("renamed".to_string()),
                        GroundTerm::Unspecified,
                    ],
                ))],
                vec![0],
                vec![],
                false,
                false,
            )
            .unwrap()
            .rule_registered
        );
        assert_eq!(inner.known_rules.identity_count(), 2);
    }

    assert_buf(&kb, make_assertion("rex", "cat"));
    assert_buf(&kb, make_assertion("bel", "person"));
    assert!(query(&kb, make_query("rex", "animal")));
    assert!(query(&kb, make_query("bel", "animal")));
}

#[test]
fn rule_identity_distinguishes_negated_exists_groups_under_one_digest() {
    let kb = new_kb();
    kb.inner
        .borrow_mut()
        .known_rules
        .force_digest_for_test(forced_rule_digest);

    // The first rule is blocked by cat(Bel); the second must survive dedup and
    // fire because no dog(Bel) exists. Before the full identity, both NAF groups
    // were omitted from the key and the second rule was silently skipped.
    assert_buf(
        &kb,
        compile_surface("all $x: person($x) & ~cat($x) -> animal($x)."),
    );
    assert_buf(
        &kb,
        compile_surface("all $x: person($x) & ~dog($x) -> animal($x)."),
    );
    assert_buf(&kb, compile_surface("person(Bel)."));
    assert_buf(&kb, compile_surface("cat(Bel)."));

    assert_eq!(kb.inner.borrow().known_rules.identity_count(), 2);
    assert!(
        query(&kb, compile_surface("animal(Bel).")),
        "the distinct ~dog group must remain executable"
    );
}
