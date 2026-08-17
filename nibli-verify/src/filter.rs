//! Conservative fragment filter: is a `(KB, query)` case inside the cleanly-mappable
//! Horn / NAF-free classical fragment?
//!
//! Over-skipping is safe (it only lowers coverage); under-skipping would judge a
//! non-classical case against a classical oracle and raise a false alarm, so every
//! check here errs toward SKIP. Two layers:
//!   1. a SOURCE token scan for genuine negation — a universal rule's implication
//!      arrow also compiles to `Not` (`Or(Not(A),B)`), indistinguishable from a real
//!      `na` once flattened, so genuine negation must be caught before translation;
//!   2. a buffer scan for the non-classical node kinds (compute / deontic /
//!      exact-count / abstraction), plus the `du` shape gate below. Tense nodes are
//!      NOT rejected here: they are handled downstream by `tense::flavorize`, which
//!      rewrites the verified tense shapes to flavor-suffixed predicates and skips
//!      unsupported IR shapes (tense×abstraction, nested wrappers). Source-level
//!      tense×deontic stacks fail compilation before this filter.

use nibli_types::logic::{LogicBuffer, LogicNode, LogicalTerm};

/// True if a KR source line contains genuine negation. KR spells it with the
/// `~` operator (claim negation and `where ~pred` bodies alike), and `~`
/// appears nowhere else in the grammar, so a substring check is exact. The
/// `no` determiner (exact-count-0) compiles to a CountNode and is caught by
/// the buffer scan instead.
pub fn source_has_negation(line: &str) -> bool {
    line.contains('~')
}

/// Whether a relation name belongs to the numeric-comparison family — either the bare
/// name (`greater`) or one of its event-decomposition role predicates (`greater_x2`).
///
/// NEITHER oracle has a theory of arithmetic, and nothing in the translators supplies
/// one. `tptp.rs` emits FOF only, with numbers as `num_<n>` Herbrand constants, so
/// Vampire reads `greater(num_20, num_15)` as an uninterpreted relation with zero
/// axioms — `CounterSatisfiable`, against an engine that computes TRUE. `asp.rs` folds
/// the same group through `regroup_event` to `goal :- greater(num_5, num_3).`, likewise
/// unentailed. Making clingo compare them as integers is not a `BodyLit` arm: it needs
/// the GLOBAL number rendering changed, and mixing `num_<n>` constants with integers
/// under a relational operator is vacuously true by ASP-Core-2 term order — a wrong
/// oracle verdict, not a skip.
///
/// So these are skipped in every filter. Nothing generates one today, which is why the
/// hole was silent; this closes it before a generator or curated case walks into it.
/// `equals` is the contrasting case and the reason this needs its own predicate: it is
/// the one relation nibli never event-decomposes, so `equals_args` can match it flatly
/// and both oracles have a native theory for identity.
///
/// The `_xN` suffix strip is deliberately re-implemented here rather than reaching for
/// `nibli_reason::materialize::surface_relation`: nibli-verify stays independent of the
/// thing it checks (the same reason `asp.rs` re-derives the event regrouping).
fn is_numeric_comparison_relation(rel: &str) -> bool {
    let base = match rel.rsplit_once("_x") {
        Some((head, tail)) if !tail.is_empty() && tail.bytes().all(|b| b.is_ascii_digit()) => head,
        _ => rel,
    };
    nibli_types::relations::is_numeric_comparison(base)
}

/// Ground `du` in the ONE verified shape both oracles can judge: the buffer's sole root,
/// with exactly two `Constant` args — i.e. a bare `la .X. cu du la .Y.` fact or query
/// (`du` is never event-decomposed, so this is precisely how nibli-semantics compiles it). The
/// Vampire path maps it to TPTP native `=` (congruence closure over a definite theory
/// derives exactly the union-find's reflexive/symmetric/transitive/substitutive
/// consequences, in both directions); the ASP path canonicalizes the equivalence classes
/// away before regrouping (`asp::DuClasses`). Everything else — `du` under a rule or
/// negation, `du` with variable/number/description args — is skipped conservatively:
/// nibli's semantics there (contradiction records for `na du`, tensed inertness, exact
/// numeric `dunli` vs `du`) is not what either oracle would judge.
fn du_mappable(buf: &LogicBuffer, idx: usize, args: &[LogicalTerm]) -> bool {
    buf.roots.as_slice() == [idx as u32]
        && args.len() == 2
        && args.iter().all(|a| matches!(a, LogicalTerm::Constant(_)))
}

/// `Some(reason)` if the buffer contains a node outside the classical FOL fragment.
/// (Tense nodes pass — `tense::flavorize` is the tense gate.)
pub fn buffer_non_classical(buf: &LogicBuffer) -> Option<&'static str> {
    for (idx, node) in buf.nodes.iter().enumerate() {
        let reason = match node {
            LogicNode::ComputeNode(_) => "compute predicate",
            LogicNode::ObligatoryNode(_) | LogicNode::PermittedNode(_) => "deontic",
            LogicNode::CountNode(_) => "exact-count quantifier",
            LogicNode::Predicate((rel, _)) if rel.starts_with("__abs_") => "abstraction",
            LogicNode::Predicate((rel, _)) if is_numeric_comparison_relation(rel) => {
                "numeric comparison (no theory of arithmetic)"
            }
            LogicNode::Predicate((rel, args))
                if rel == "equals" && !du_mappable(buf, idx, args) =>
            {
                "equality (nested or non-ground)"
            }
            _ => continue,
        };
        return Some(reason);
    }
    None
}

/// `Some(reason)` if the buffer is outside the **ASP-mappable** (stratified-NAF +
/// closed-world) fragment. Two differences from `buffer_non_classical`: negation-as-failure
/// (`NotNode`) is ACCEPTED (the whole point of the clingo oracle), and `__abs_` ABSTRACTIONS
/// (`lo nu`/`lo du'u`/…) are ACCEPTED — the translator models an abstraction as an opaque constant
/// keyed by its lossless marker identity (`asp::abs_const_of`), so a deontic-NAF rule like GDPR's
/// `ro lo prenu poi na zanru cu se bilga lo nu se vimcu` maps. The other non-classical node kinds
/// (compute / deontic modal / exact-count) are still rejected; tense nodes pass through to
/// `tense::flavorize`, which rewrites the verified shapes (tense × restrictor-NAF now
/// flavorizes to a flavor-suffixed `not`) rather than mis-judging them. Ground sole-root `du`
/// equality is ACCEPTED (see
/// [`du_mappable`]; the translator canonicalizes the classes away); any other `du` shape is
/// skipped.
///
/// (`se bilga` / `se curmi` compile to the PLAIN gismu `bilga`/`curmi`, not a deontic modal node,
/// so the deontic reading rides for free once the abstraction in the head is mapped.)
pub fn buffer_asp_mappable(buf: &LogicBuffer) -> Option<&'static str> {
    buffer_asp_mappable_with(buf, false)
}

/// Whether a NON-GROUND `equals` — `~($a = $b)`, a disequality guard between two rule
/// variables — may ride into the ASP program as clingo's `!=` builtin.
///
/// This is the documented seam for the one `equals` shape beyond ground `du` that the ASP
/// oracle can judge exactly (the sibling of [`count_case_guard`]). Two conditions, and the
/// caller owns the second because it is a property of the WHOLE KB, not of one buffer:
///
/// 1. **The translator must emit `!=`.** `asp::pos_or_naf` / `peel_antecedent_literal`
///    intercept `equals` before `regroup_event`. Without that, `~($a = $b)` renders as
///    `not equals(A, B)`, which — since nothing ever puts `equals/2` in a head — is
///    vacuously true for every pair including `A == A`. Relaxing this filter without that
///    interception does not produce a skip, it produces a WRONG oracle verdict.
/// 2. **No ground `du` facts in the KB** (`allow_nonground_equals` is set only then). With
///    an empty union-find the engine decides a non-ground `equals` by plain structural
///    comparison — `materialize::builtin_holds` is written on exactly that assumption, and
///    `materialize::saturate` refuses the whole KB the moment `equivalence_parent` is
///    non-empty. Pairing ASP's class-canonical constants against the engine's union-find
///    *plus* its `DU_VARIANT_BOUND`-truncated variant search is unverified, so that
///    combination stays skipped.
pub fn buffer_asp_mappable_with(
    buf: &LogicBuffer,
    allow_nonground_equals: bool,
) -> Option<&'static str> {
    for (idx, node) in buf.nodes.iter().enumerate() {
        let reason = match node {
            LogicNode::ComputeNode(_) => "compute predicate",
            LogicNode::ObligatoryNode(_) | LogicNode::PermittedNode(_) => "deontic",
            LogicNode::CountNode(_) => "exact-count quantifier",
            LogicNode::Predicate((rel, _)) if is_numeric_comparison_relation(rel) => {
                "numeric comparison (no theory of arithmetic)"
            }
            LogicNode::Predicate((rel, args))
                if rel == "equals"
                    && !du_mappable(buf, idx, args)
                    && !(allow_nonground_equals && comparable_equals(args)) =>
            {
                "equality (nested or non-ground)"
            }
            _ => continue,
        };
        return Some(reason);
    }
    None
}

/// An `equals` whose two arguments are both terms clingo can compare with `==`/`!=` after
/// `DuClasses` canonicalization: variables and constants only. A `Number` belongs to the
/// compute fragment and a `Description`/`Unspecified` is an opaque referent whose identity
/// nibli decides structurally in ways the oracle does not model — both stay skipped.
fn comparable_equals(args: &[LogicalTerm]) -> bool {
    args.len() == 2
        && args
            .iter()
            .all(|a| matches!(a, LogicalTerm::Variable(_) | LogicalTerm::Constant(_)))
}

/// The ASP filter for the QUERY buffer: like [`buffer_asp_mappable`], but a sole-root
/// exact-count query (`Count(v, n, body)` — `re lo gerku cu danlu`) is ACCEPTED: the
/// translator maps it to a clingo `#count` aggregate. Count nodes anywhere else (a
/// count ASSERTION, or a count nested under other structure) stay skipped.
pub fn buffer_asp_mappable_query(buf: &LogicBuffer) -> Option<&'static str> {
    for (idx, node) in buf.nodes.iter().enumerate() {
        let reason = match node {
            LogicNode::ComputeNode(_) => "compute predicate",
            LogicNode::ObligatoryNode(_) | LogicNode::PermittedNode(_) => "deontic",
            LogicNode::CountNode(_) if buf.roots.as_slice() != [idx as u32] => {
                "exact-count quantifier (nested / non-root)"
            }
            LogicNode::Predicate((rel, _)) if is_numeric_comparison_relation(rel) => {
                "numeric comparison (no theory of arithmetic)"
            }
            LogicNode::Predicate((rel, args))
                if rel == "equals" && !du_mappable(buf, idx, args) =>
            {
                "equality (nested or non-ground)"
            }
            _ => continue,
        };
        return Some(reason);
    }
    None
}

/// Case-level guard for an exact-count QUERY. Both former skip conditions were
/// CANONIZED by the 2026-07-02 count-semantics decision (GUARANTEES
/// §Aggregation), so nothing is skipped today:
/// - **KBs with rules**: every verifier engine is explicitly clean-core, so a
///   universal creates no existential-import witness; the ASP program likewise
///   has no implicit domain member. Both sides count the derivable entities.
/// - **KBs with `du`**: the engine now counts one representative per
///   du-equivalence class, matching the translator's canonicalization.
///
/// The guard is retained as the documented seam for any FUTURE count shape
/// that needs a conservative skip.
pub fn count_case_guard(_kb: &[LogicBuffer], _query: &LogicBuffer) -> Option<&'static str> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_abstraction_marker() -> String {
        // Event kind + Predicate("", []). Minimal but structurally complete v1 key.
        let mut key = vec![0xa0, 0x10];
        key.extend_from_slice(&0_u64.to_be_bytes());
        key.extend_from_slice(&0_u64.to_be_bytes());
        nibli_types::abstraction::encode_v1(&key)
    }

    #[test]
    fn detects_negation_tokens() {
        assert!(source_has_negation(
            "obliged(every person where ~approves, event { removes() })."
        ));
        assert!(source_has_negation("~dog(Adam)."));
        assert!(!source_has_negation("animal(every dog)."));
        // The `no` determiner is count territory (buffer scan), not `~`.
        assert!(!source_has_negation("goes(no dog)."));
    }

    #[test]
    fn flags_non_classical_nodes() {
        let compute = LogicBuffer {
            nodes: vec![LogicNode::ComputeNode(("product".into(), vec![]))],
            roots: vec![0],
        };
        assert_eq!(buffer_non_classical(&compute), Some("compute predicate"));

        let abs = LogicBuffer {
            nodes: vec![LogicNode::Predicate((
                valid_abstraction_marker(),
                vec![LogicalTerm::Constant("x".into())],
            ))],
            roots: vec![0],
        };
        assert_eq!(buffer_non_classical(&abs), Some("abstraction"));

        let plain = LogicBuffer {
            nodes: vec![LogicNode::Predicate((
                "gerku".into(),
                vec![LogicalTerm::Constant("adam".into())],
            ))],
            roots: vec![0],
        };
        assert_eq!(buffer_non_classical(&plain), None);
    }

    /// The one accepted `du` shape: sole root, two constants — a bare fact/query.
    fn ground_equals() -> LogicBuffer {
        LogicBuffer {
            nodes: vec![LogicNode::Predicate((
                "equals".into(),
                vec![
                    LogicalTerm::Constant("adam".into()),
                    LogicalTerm::Constant("bel".into()),
                ],
            ))],
            roots: vec![0],
        }
    }

    #[test]
    fn ground_sole_root_equals_is_mappable_in_both_fragments() {
        // `la .X. cu du la .Y.` — Vampire judges it as native `=`; the ASP translator
        // canonicalizes the equivalence class away. Accepted by BOTH filters.
        assert_eq!(buffer_non_classical(&ground_equals()), None);
        assert_eq!(buffer_asp_mappable(&ground_equals()), None);
    }

    #[test]
    fn nested_or_non_ground_equals_is_skipped_in_both_fragments() {
        // `du` with a variable arg (e.g. inside a rule) — not the verified shape.
        let non_ground = LogicBuffer {
            nodes: vec![LogicNode::Predicate((
                "equals".into(),
                vec![
                    LogicalTerm::Variable("x".into()),
                    LogicalTerm::Constant("bel".into()),
                ],
            ))],
            roots: vec![0],
        };
        assert_eq!(
            buffer_non_classical(&non_ground),
            Some("equality (nested or non-ground)")
        );
        assert_eq!(
            buffer_asp_mappable(&non_ground),
            Some("equality (nested or non-ground)")
        );

        // Ground `du` that is NOT the sole root (e.g. wrapped in `na du` — a negative-fact
        // /contradiction record in nibli, NOT NAF) — skipped, never mis-judged.
        let negated = LogicBuffer {
            nodes: vec![
                LogicNode::Predicate((
                    "equals".into(),
                    vec![
                        LogicalTerm::Constant("adam".into()),
                        LogicalTerm::Constant("bel".into()),
                    ],
                )),
                LogicNode::NotNode(0),
            ],
            roots: vec![1],
        };
        assert_eq!(
            buffer_non_classical(&negated),
            Some("equality (nested or non-ground)")
        );
        assert_eq!(
            buffer_asp_mappable(&negated),
            Some("equality (nested or non-ground)")
        );

        // Numeric `du` (`li pa du li re`) — exact numeric identity is `dunli`/compute
        // territory; skipped.
        let numeric = LogicBuffer {
            nodes: vec![LogicNode::Predicate((
                "equals".into(),
                vec![LogicalTerm::Number(1.0), LogicalTerm::Number(2.0)],
            ))],
            roots: vec![0],
        };
        assert_eq!(
            buffer_non_classical(&numeric),
            Some("equality (nested or non-ground)")
        );
        assert_eq!(
            buffer_asp_mappable(&numeric),
            Some("equality (nested or non-ground)")
        );
    }

    #[test]
    fn asp_mappable_accepts_naf_rejects_non_classical() {
        // NAF (NotNode) is accepted by the ASP filter (unlike the classical one).
        let naf = LogicBuffer {
            nodes: vec![
                LogicNode::Predicate(("gerku".into(), vec![LogicalTerm::Variable("x".into())])),
                LogicNode::NotNode(0),
            ],
            roots: vec![1],
        };
        assert_eq!(buffer_asp_mappable(&naf), None);

        // The non-classical reject list still applies (compute / deontic / …).
        let compute = LogicBuffer {
            nodes: vec![LogicNode::ComputeNode(("product".into(), vec![]))],
            roots: vec![0],
        };
        assert_eq!(buffer_asp_mappable(&compute), Some("compute predicate"));

        // Abstractions (`lo nu` → `__abs_`) are ACCEPTED by the ASP filter (modeled as opaque
        // constants) though still rejected by the classical one.
        let abs = LogicBuffer {
            nodes: vec![LogicNode::Predicate((
                valid_abstraction_marker(),
                vec![LogicalTerm::Variable("v".into())],
            ))],
            roots: vec![0],
        };
        assert_eq!(buffer_non_classical(&abs), Some("abstraction"));
        assert_eq!(buffer_asp_mappable(&abs), None);
    }

    /// A numeric comparison must be skipped by EVERY filter, in both the flat and the
    /// event-decomposed spelling. Neither oracle has a theory of arithmetic, so an
    /// admitted comparison is a false alarm (`Diverge`) rather than a missed case.
    /// Nothing generates one today — which is exactly why this needs pinning.
    #[test]
    fn a_numeric_comparison_is_not_mappable_by_any_filter() {
        const REASON: &str = "numeric comparison (no theory of arithmetic)";

        // Flat two-argument spelling (raw-IR injection, persisted-buffer replay).
        for rel in ["greater", "less", "num_equal"] {
            let flat = LogicBuffer {
                nodes: vec![LogicNode::Predicate((
                    rel.into(),
                    vec![LogicalTerm::Number(5.0), LogicalTerm::Number(3.0)],
                ))],
                roots: vec![0],
            };
            assert_eq!(buffer_non_classical(&flat), Some(REASON), "{rel} classical");
            assert_eq!(buffer_asp_mappable(&flat), Some(REASON), "{rel} asp");
            assert_eq!(
                buffer_asp_mappable_with(&flat, true),
                Some(REASON),
                "{rel} asp, non-ground equals allowed"
            );
            assert_eq!(
                buffer_asp_mappable_query(&flat),
                Some(REASON),
                "{rel} asp query"
            );
        }

        // Event-decomposed spelling — what the KR surface actually produces. The role
        // predicate carries the operand, so the ROLE name has to be caught too.
        let decomposed = LogicBuffer {
            nodes: vec![
                LogicNode::Predicate(("greater".into(), vec![LogicalTerm::Variable("ev".into())])),
                LogicNode::Predicate((
                    "greater_x1".into(),
                    vec![
                        LogicalTerm::Variable("ev".into()),
                        LogicalTerm::Number(20.0),
                    ],
                )),
                LogicNode::AndNode((0, 1)),
                LogicNode::ExistsNode(("ev".into(), 2)),
            ],
            roots: vec![3],
        };
        assert_eq!(buffer_non_classical(&decomposed), Some(REASON));
        assert_eq!(buffer_asp_mappable(&decomposed), Some(REASON));
        assert_eq!(buffer_asp_mappable_query(&decomposed), Some(REASON));

        // The suffix strip must not over-reach: an ordinary relation whose name merely
        // ends in a role-shaped suffix, or contains a comparison name as a substring,
        // stays mappable.
        for rel in ["dog_x1", "greatest", "lesson", "ungreater"] {
            let ordinary = LogicBuffer {
                nodes: vec![LogicNode::Predicate((
                    rel.into(),
                    vec![LogicalTerm::Constant("adam".into())],
                ))],
                roots: vec![0],
            };
            assert_eq!(
                buffer_non_classical(&ordinary),
                None,
                "{rel} must stay mappable"
            );
        }
    }
}
