//! The shared session core — the ONE compile/assert/query chain every runtime
//! surface wraps with only boundary conversion.
//!
//! Before this crate, the compile chain (`nibli_kr::parse_checked` →
//! `nibli_semantics::compile_from_ast` → `nibli_reason::transform_compute_nodes`)
//! plus the compute-predicate registry and the assert/query wrappers were
//! hand-mirrored across nibli-engine (native), nibli-pipeline (WASM component),
//! nibli-wasm (wasm-bindgen), nibli-ui (Dioxus), and nibli-verify's battery —
//! the pipeline's copy literally commented "the mirror of nibli-engine's
//! compile_text, so native and WASM agree". [`CoreSession`] is that agreement
//! BY CONSTRUCTION.
//!
//! What stays surface-side, deliberately:
//! - the ERROR BOUNDARY (this crate speaks canonical [`NibliError`]; the
//!   pipeline converts to its WIT twin, nibli-wasm flattens to `String`);
//! - verdict/proof SERIALIZATION (WIT records, JSON, rendered text);
//! - the LINT policy (`nibli_kr::lint` — stdout `[Note:]` echoes vs UI note
//!   data vs none) and env reads (`NIBLI_QUIET`/`NIBLI_STRICT` are wasip2/host
//!   concerns; the browser has no process env);
//! - STORE write-through (nibli-engine's durable registry mints its own ids
//!   and reaches the KB through [`CoreSession::kb`]);
//! - compute-dispatch WIRING (the pipeline bridges to its WIT host import,
//!   the engine offers an opt-in TCP client, the browser leaves external
//!   compute unregistered) — the [`CoreSession::set_compute_dispatch`]
//!   passthrough is the seam.
//!
//! nibli-formalize's gates intentionally do NOT use this crate: they keep the
//! AST for the render round-trip gate, then mark the default compute predicates
//! locally so its KB-authoring boundary can reject query-only `ComputeNode`s
//! (the reference external names are refused by the engine's shared name guard
//! whether or not any session registered them); they also carry their own
//! `GateError` taxonomy (see nibli-formalize/src/gates.rs).

use std::collections::HashSet;

use nibli_types::error::NibliError;
use nibli_types::logic::{
    AggregateOp, FactSummary, LogicBuffer, LogicalTerm, ProofTrace, QueryResult, WitnessBinding,
};

/// Compile KR text WITHOUT compute-marking: parse + canonical claim compile only.
/// Repeated, safely exposed textual `$name` binders are factored once around
/// their ordinary connected region; ambiguous scope crossings fail closed.
/// For consumers that deliberately stop before `transform_compute_nodes`
/// (display paths; nibli-formalize's gates mirror this shape independently).
pub fn compile_unmarked(text: &str) -> Result<LogicBuffer, NibliError> {
    let ast = nibli_kr::parse_checked(text)?;
    nibli_semantics::compile_from_ast(ast)
}

/// Compatibility/intent alias for compiling KR query text without
/// compute-marking. Assertions and queries use the same canonical §6 binder
/// structure.
pub fn compile_query_unmarked(text: &str) -> Result<LogicBuffer, NibliError> {
    compile_unmarked(text)
}

/// THE compile chain: parse + semantic compile + compute-marking against the
/// given predicate set. Free-fn form for per-call-set users (nibli-ui builds
/// its set fresh each query); [`CoreSession::compile_text`] is the
/// session-owned form.
pub fn compile_text(
    text: &str,
    compute_predicates: &HashSet<String>,
) -> Result<LogicBuffer, NibliError> {
    let mut buf = compile_unmarked(text)?;
    nibli_reason::transform_compute_nodes(&mut buf, compute_predicates);
    Ok(buf)
}

/// Compatibility/intent alias for the query compile chain. The accepted claim
/// IR is byte-for-byte the same as [`compile_text`].
pub fn compile_query_text(
    text: &str,
    compute_predicates: &HashSet<String>,
) -> Result<LogicBuffer, NibliError> {
    compile_text(text, compute_predicates)
}

/// The shared session: a [`nibli_reason::KnowledgeBase`] + the compute-predicate
/// registry, with the compile/assert/query verbs every surface previously
/// hand-mirrored. No env reads, no linting, no persistence — those are
/// per-surface boundary policy (see the module doc).
pub struct CoreSession {
    kb: nibli_reason::KnowledgeBase,
    compute_predicates: HashSet<String>,
}

impl Default for CoreSession {
    fn default() -> Self {
        Self::new()
    }
}

impl CoreSession {
    /// A fresh in-memory session seeded with the built-in arithmetic compute
    /// predicates (`nibli_reason::default_compute_predicates`).
    pub fn new() -> Self {
        Self::with_kb(nibli_reason::KnowledgeBase::new())
    }

    /// Wrap an already-constructed KB (for example, one built with an empty
    /// persistent write-through mirror via `KnowledgeBase::with_store`, then
    /// populated by replaying its authoritative `LogicBuffer` registry with
    /// stable ids). Typed mirror rows alone are not a complete KB snapshot.
    pub fn with_kb(kb: nibli_reason::KnowledgeBase) -> Self {
        CoreSession {
            kb,
            compute_predicates: nibli_reason::default_compute_predicates(),
        }
    }

    /// The underlying KB, for surface-specific extras (cancel flags, predicate
    /// tracing, contradiction scans, store replay via `assert_fact_with_id`).
    pub fn kb(&self) -> &nibli_reason::KnowledgeBase {
        &self.kb
    }

    /// The current compute-predicate set (the marking input).
    pub fn compute_predicates(&self) -> &HashSet<String> {
        &self.compute_predicates
    }

    /// Register a predicate name for external compute dispatch.
    ///
    /// Fallible (decided 2026-08-09): registration flips how future compiled
    /// statements SPELL the name (`Predicate` → `ComputeNode`), so it is
    /// refused while LIVE stored statements (facts or rules, NAF bodies
    /// included) reference it — otherwise those rows become
    /// unreachable-but-listed the moment the query side starts dispatching
    /// (the assert-then-register divergence). Also refused: role spellings
    /// (`eats_x1` — register the anchor) and the engine-special relations
    /// (identity, numeric comparisons), whose built-in query semantics
    /// ComputeNode marking would silently replace. Idempotent for names
    /// already registered (the built-in arithmetic set lands here); no
    /// partial state on `Err`. The reference external names (`exponential`,
    /// `logarithm`) can never acquire live references — assertion ingress
    /// refuses them statically — so their registration is vacuously never
    /// blocked.
    pub fn register_compute_predicate(&mut self, name: String) -> Result<(), NibliError> {
        // A role spelling would strand its anchor: `transform_compute_nodes`
        // matches EXACT names, so registering `eats_x1` marks exactly the role
        // conjunct every stored/future `eats` statement carries, while the
        // anchor-collapsed reference scan below would have found no blockers.
        let collapsed = nibli_reason::role_collapsed_relation(&name);
        if collapsed != name {
            return Err(NibliError::Reasoning(format!(
                "cannot register `{name}` for external compute: role spellings collapse \
                 onto their anchor relation — register `{collapsed}` instead."
            )));
        }
        // Engine-special relations have built-in query semantics (identity
        // feeds union-find; comparisons evaluate exactly and keep a relational
        // reading) that ComputeNode marking would silently replace.
        if nibli_types::relations::is_identity(&name)
            || nibli_types::relations::is_numeric_comparison(&name)
        {
            return Err(NibliError::Reasoning(format!(
                "cannot register `{name}` for external compute: it is an engine-special \
                 relation whose query semantics are built in — marking it as compute \
                 would silently replace them."
            )));
        }
        // The scan runs even for an already-registered name, so the invariant
        // "a registered name has no live references" is enforced rather than
        // assumed: a raw sub-session ingress (`KnowledgeBase::assert_fact` on a
        // hand-built unmarked buffer) that violated it surfaces on the next
        // registration instead of returning Ok forever.
        let blocking = self.kb.stored_statement_ids_referencing(&name);
        if !blocking.is_empty() {
            const SHOWN: usize = 8;
            let shown: Vec<String> = blocking
                .iter()
                .take(SHOWN)
                .map(|id| format!("#{id}"))
                .collect();
            let more = blocking.len().saturating_sub(SHOWN);
            let tail = if more > 0 {
                format!(", … and {more} more")
            } else {
                String::new()
            };
            return Err(NibliError::Reasoning(format!(
                "cannot register `{name}` for external compute: {n} live stored \
                 statement(s) reference it ({ids}{tail}). A registered name is \
                 computed by the backend at query time, so its stored extension \
                 would become unreachable — retract the listed statements (or \
                 reset) first, then register, then re-ask them as queries.",
                n = blocking.len(),
                ids = shown.join(", "),
            )));
        }
        if self.compute_predicates.contains(&name) {
            return Ok(()); // Idempotent re-registration (the builtins land here).
        }
        self.compute_predicates.insert(name);
        // Registration is not a KB content mutation, so the content-path
        // invalidations never fire; drop the saturation so
        // `materialization_report` cannot keep presenting the name as a
        // complete stored extension.
        self.kb.invalidate_materialization();
        Ok(())
    }

    /// Register this session's external compute dispatch (per-instance; see
    /// `nibli_reason::KnowledgeBase::set_compute_dispatch` for the trust
    /// boundary). Without it, external predicates error; valid numeric built-in
    /// arithmetic still resolves in-engine.
    pub fn set_compute_dispatch(
        &self,
        eval: fn(&str, &[LogicalTerm]) -> Result<bool, String>,
        batch_eval: fn(&[nibli_reason::ComputeRequest]) -> Vec<Result<bool, String>>,
    ) {
        self.kb.set_compute_dispatch(eval, batch_eval);
    }

    /// Engine stdout diagnostics (`[Rule]`/`[Skolem]`/`[Constraint]`).
    /// Default OFF — a silent library; surfaces opt in.
    pub fn set_verbose(&self, verbose: bool) {
        self.kb.set_verbose(verbose);
    }

    /// STRICT MODE (default off — permissive warn-and-insert).
    pub fn set_strict(&self, strict: bool) {
        self.kb.set_strict(strict);
    }

    /// Legacy EXISTENTIAL-IMPORT MODE (default OFF). A profile change
    /// transactionally rebuilds the active KB so it takes effect immediately.
    pub fn set_existential_import(&self, on: bool) -> Result<(), NibliError> {
        self.kb
            .set_existential_import(on)
            .map_err(NibliError::Reasoning)
    }

    /// Whether legacy existential import is active for this whole session.
    pub fn is_existential_import(&self) -> bool {
        self.kb.is_existential_import()
    }

    /// STRATUM-ORDERED MATERIALISATION (default ON). OFF sends every
    /// negation-as-failure check back through backward chaining.
    pub fn set_materialization(&self, on: bool) {
        self.kb.set_materialization(on);
    }

    /// What the last query's saturation covered: `(complete, [(relation, why not)])`.
    /// Empty until a query has run and after any KB mutation.
    pub fn materialization_report(&self) -> (Vec<String>, Vec<(String, String)>) {
        self.kb.materialization_report()
    }

    /// THE compile chain against this session's compute-predicate set.
    pub fn compile_text(&self, text: &str) -> Result<LogicBuffer, NibliError> {
        compile_text(text, &self.compute_predicates)
    }

    /// Query-intent alias for the canonical claim compile chain.
    pub fn compile_query_text(&self, text: &str) -> Result<LogicBuffer, NibliError> {
        compile_query_text(text, &self.compute_predicates)
    }

    /// Compile KR text and assert it, splitting a multi-statement input into
    /// one INDEPENDENT fact per root (`split_roots` — connectives compile to a
    /// single root and stay one compound fact). The full input text is each
    /// root's label. Returns one `(id, compiled-sub-buffer)` pair per root so
    /// a persisting caller can store the FACT itself and replay it
    /// recompile-free; callers that only need ids map the pairs down. Reachable
    /// exact-count and executable compute formulas in asserted position
    /// (outside opaque quoted content) are query-only; the whole compiled input
    /// is preflighted so a later rejected root cannot leave earlier roots live.
    pub fn assert_text(&self, text: &str) -> Result<Vec<(u64, LogicBuffer)>, NibliError> {
        let buf = self.compile_text(text)?;
        self.kb.validate_assertion(&buf)?;
        let mut out = Vec::new();
        for sub in buf.split_roots() {
            let id = self.kb.assert_fact(sub.clone(), text.to_string())?;
            out.push((id, sub));
        }
        Ok(out)
    }

    /// Assert a fact directly by relation name and arguments, bypassing text
    /// parsing, under an optional caller-chosen id (store replay). The label
    /// is `":assert {relation}"`. Event-decomposes to the SAME shape a surface
    /// assertion produces, so the injected fact is matched by surface text
    /// queries (not just raw-FOL / same-shape direct queries). A registered
    /// compute relation is marked before assertion and therefore rejected as
    /// query-only; the reference external names (`exponential`, `logarithm`)
    /// are rejected even when unregistered. Identity stays flat; arity follows
    /// the injected-arity policy (fail-closed) — see
    /// `nibli_semantics::compile_injected_fact`.
    pub fn assert_fact_direct(
        &self,
        relation: &str,
        args: &[LogicalTerm],
        id: Option<u64>,
    ) -> Result<u64, NibliError> {
        let label = format!(":assert {}", relation);
        let buf = self.compile_injected_fact(relation, args)?;
        match id {
            Some(i) => {
                // The assert is the reasoning stage (buffer already past
                // nibli-semantics); nibli-reason's `assert_fact_with_id`
                // returns a String, so wrap as Reasoning.
                self.kb
                    .assert_fact_with_id(buf, label, i)
                    .map_err(NibliError::Reasoning)?;
                Ok(i)
            }
            None => self.kb.assert_fact(buf, label),
        }
    }

    /// Compile one direct/injected fact through the same compute-marking policy
    /// as text assertions. A relation registered for compute becomes a
    /// `ComputeNode`, allowing assertion ingress to reject it as query-only
    /// instead of silently storing an ordinary fact that compute queries ignore;
    /// the reference external names are refused by the static name guard even
    /// when unregistered, so registration order cannot re-open that hole.
    pub fn compile_injected_fact(
        &self,
        relation: &str,
        args: &[LogicalTerm],
    ) -> Result<LogicBuffer, NibliError> {
        let mut buf = nibli_semantics::compile_injected_fact(relation, args)?;
        nibli_reason::transform_compute_nodes(&mut buf, &self.compute_predicates);
        Ok(buf)
    }

    /// Assert an already-compiled buffer under a caller-chosen id — the
    /// recompile-free replay primitive. The buffer is RE-MARKED against the
    /// live compute registry first, so a row replayed out of order relative
    /// to a registration cannot store a plain fact for a name whose queries
    /// dispatch — it fails closed at preflight instead. In-order replay is
    /// unaffected: at that point in the replay the registry does not yet hold
    /// the name, and no session ever accepted a plain fact for a
    /// then-registered one.
    pub fn assert_buffer_with_id(
        &self,
        mut buffer: LogicBuffer,
        label: String,
        id: u64,
    ) -> Result<(), NibliError> {
        nibli_reason::transform_compute_nodes(&mut buffer, &self.compute_predicates);
        self.kb
            .assert_fact_with_id(buffer, label, id)
            .map_err(NibliError::Reasoning)
    }

    /// Compile a KR query and run the entailment check.
    pub fn query_text(&self, text: &str) -> Result<QueryResult, NibliError> {
        let buf = self.compile_query_text(text)?;
        self.kb.query_entailment(buf)
    }

    /// Compile a KR query, run the entailment check, and return the typed
    /// result with the canonical wire [`ProofTrace`].
    pub fn query_text_with_proof(
        &self,
        text: &str,
    ) -> Result<(QueryResult, ProofTrace), NibliError> {
        let buf = self.compile_query_text(text)?;
        self.kb.query_entailment_with_proof(buf)
    }

    /// Compile a KR query and extract all satisfying witness binding sets.
    pub fn query_find_text(&self, text: &str) -> Result<Vec<Vec<WitnessBinding>>, NibliError> {
        let buf = self.compile_query_text(text)?;
        self.kb.query_find(buf)
    }

    /// Count the distinct witness binding sets satisfying a KR query.
    pub fn count_witnesses_text(&self, text: &str) -> Result<usize, NibliError> {
        let buf = self.compile_query_text(text)?;
        self.kb.count_witnesses(buf)
    }

    /// Aggregate the numeric values bound to `variable` across all witness
    /// binding sets of a KR query. `Ok(None)` when no numeric witnesses exist.
    pub fn aggregate_text(
        &self,
        text: &str,
        variable: &str,
        op: AggregateOp,
    ) -> Result<Option<f64>, NibliError> {
        let buf = self.compile_query_text(text)?;
        self.kb.aggregate(buf, variable, op)
    }

    /// Retract a fact by id and rebuild derived state (KB only — durable
    /// tombstones are the persisting surface's concern).
    pub fn retract_fact(&self, id: u64) -> Result<(), NibliError> {
        self.kb.retract_fact(id)
    }

    /// Reset the KB, clearing all facts and rules.
    pub fn reset(&self) -> Result<(), NibliError> {
        self.kb.reset()
    }

    /// List all active (non-retracted) facts with their ids and labels.
    pub fn list_facts(&self) -> Result<Vec<FactSummary>, NibliError> {
        self.kb.list_facts()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nibli_types::logic::LogicNode;

    fn children(node: &LogicNode) -> Vec<u32> {
        match node {
            LogicNode::Predicate(_) | LogicNode::ComputeNode(_) => Vec::new(),
            LogicNode::AndNode((left, right)) | LogicNode::OrNode((left, right)) => {
                vec![*left, *right]
            }
            LogicNode::NotNode(child)
            | LogicNode::PastNode(child)
            | LogicNode::PresentNode(child)
            | LogicNode::FutureNode(child)
            | LogicNode::ObligatoryNode(child)
            | LogicNode::PermittedNode(child) => vec![*child],
            LogicNode::ExistsNode((_, body)) | LogicNode::ForAllNode((_, body)) => vec![*body],
            LogicNode::CountNode((_, _, body)) => vec![*body],
        }
    }

    fn count_exists(buffer: &LogicBuffer, node_id: u32, name: &str) -> usize {
        let node = &buffer.nodes[node_id as usize];
        usize::from(matches!(node, LogicNode::ExistsNode((var, _)) if var == name))
            + children(node)
                .into_iter()
                .map(|child| count_exists(buffer, child, name))
                .sum::<usize>()
    }

    #[test]
    fn claim_compile_closes_shared_name_once_across_conjuncts() {
        let predicates = HashSet::new();
        let text = "bite($x, Bel) & bite($x, Dana).";
        let query = compile_query_text(text, &predicates).unwrap();
        let assertion = compile_text(text, &predicates).unwrap();

        assert_eq!(assertion, query, "assertion and query IR must be canonical");
        let root = assertion.roots[0];
        let body = match &assertion.nodes[root as usize] {
            LogicNode::ExistsNode((name, body)) => {
                assert_eq!(name, "$x");
                *body
            }
            other => panic!("claim root must be the shared `$x` binder, got {other:?}"),
        };
        assert!(
            matches!(assertion.nodes[body as usize], LogicNode::AndNode(_)),
            "the one `$x` binder must dominate both connected propositions"
        );
        assert_eq!(count_exists(&assertion, root, "$x"), 1);

        let distinct = compile_unmarked("bite($p, Bel) & bite($q, Dana).").unwrap();
        assert_eq!(count_exists(&distinct, distinct.roots[0], "$p"), 1);
        assert_eq!(count_exists(&distinct, distinct.roots[0], "$q"), 1);
    }

    #[test]
    fn claim_compile_preserves_first_name_order() {
        let claim = compile_text(
            "bite($first, $second) & bite($first, $second).",
            &HashSet::new(),
        )
        .unwrap();

        let first_body = match &claim.nodes[claim.roots[0] as usize] {
            LogicNode::ExistsNode((name, body)) => {
                assert_eq!(name, "$first");
                *body
            }
            other => panic!("first surface name must be outermost, got {other:?}"),
        };
        match &claim.nodes[first_body as usize] {
            LogicNode::ExistsNode((name, _)) => assert_eq!(name, "$second"),
            other => panic!("second surface name must be the next binder, got {other:?}"),
        }
    }

    #[test]
    fn claim_compile_correlates_nonadjacent_three_conjuncts() {
        for text in [
            "bite($x, Bel) & dog(Adam) & bite($x, Dana).",
            "dog(Adam) & bite($x, Bel) & cat(Bel) & bite($x, Dana).",
        ] {
            let claim = compile_unmarked(text).unwrap();
            assert_eq!(
                count_exists(&claim, claim.roots[0], "$x"),
                1,
                "an ordinary intervening clause must not become a scope boundary: {text}"
            );
        }
    }

    #[test]
    fn query_and_assertion_compile_are_identical_for_single_clause_scopes() {
        for text in [
            "~dog($x).",
            "loves(every dog, $x).",
            "desires(me, event { dog($x) }).",
        ] {
            assert_eq!(
                compile_query_unmarked(text).unwrap(),
                compile_unmarked(text).unwrap(),
                "query and assertion aliases must compile identically: {text}"
            );
        }
    }

    #[test]
    fn claim_compile_rejects_ambiguous_scope_crossing() {
        for text in [
            "~bite($x, Bel) & bite($x, Dana).",
            "past bite($x, Bel) & bite($x, Dana).",
            "must bite($x, Bel) & bite($x, Dana).",
            "loves(some dog, $x) & bite($x, Dana).",
            "desires($x, event { dog($x) }).",
            "desires($x, event { dog($x) }) & dog($x).",
            "bite(every person, $x) & bite($x, Dana).",
            "bite($x, Dana) & bite(every person, $x).",
        ] {
            for compile in [
                compile_unmarked as fn(&str) -> Result<LogicBuffer, NibliError>,
                compile_query_unmarked,
            ] {
                let err = compile(text)
                    .expect_err("moving `$x` across a lexical boundary needs an explicit scope");
                assert!(
                    err.to_string().contains("de-re/de-dicto"),
                    "unexpected error for {text}: {err}"
                );
            }
        }
    }

    #[test]
    fn explicit_universal_scope_preserves_its_binder_and_one_dependent_witness() {
        let prenex = compile_unmarked("all $x: bite($x, Bel) & bite($x, Dana).").unwrap();
        let root = prenex.roots[0];
        assert!(
            matches!(&prenex.nodes[root as usize], LogicNode::ForAllNode((name, _)) if name == "$x"),
            "the explicit universal must remain the root binder"
        );
        assert_eq!(count_exists(&prenex, root, "$x"), 0);

        let dependent =
            compile_unmarked("all $owner: dog($owner) -> bite($w, $owner) & loves($w, $owner).")
                .unwrap();
        assert_eq!(
            count_exists(&dependent, dependent.roots[0], "$w"),
            1,
            "the conclusion's repeated witness must be one existential inside the universal"
        );
    }

    #[test]
    fn multi_root_count_rejection_happens_before_any_root_or_id_lands() {
        let session = CoreSession::new();
        let error = session
            .assert_text("person(Adam). big(exactly 1 dog).")
            .expect_err("a later query-only count must reject the whole call");
        assert!(error.to_string().contains("query-only"), "{error}");
        assert!(
            session.list_facts().unwrap().is_empty(),
            "the ordinary first root must not land before rejection"
        );
        assert_eq!(
            session.kb().next_fact_id().unwrap(),
            0,
            "the rejected call must not consume an id"
        );
    }

    // ─── Fallible compute registration (assert-then-register closure) ───────

    #[test]
    fn registering_a_referenced_name_is_refused_with_the_blocking_ids() {
        let mut session = CoreSession::new();
        let facts = session.assert_text("eats(Bela, Cheese).").unwrap();
        let fact_id = facts[0].0;
        let rule_id = session
            .assert_text("all $x: eats($x, Cheese) -> animal($x).")
            .unwrap()[0]
            .0;
        let error = session
            .register_compute_predicate("eats".to_string())
            .expect_err("live references must block registration");
        let message = error.to_string();
        assert!(message.contains("cannot register"), "{message}");
        for id in [fact_id, rule_id] {
            assert!(
                message.contains(&format!("#{id}")),
                "the refusal must name blocking id #{id}: {message}"
            );
        }
        assert!(
            !session.compute_predicates().contains("eats"),
            "a refused registration must leave no partial state"
        );
        session
            .assert_text("eats(Dana, Cheese).")
            .expect("the name must stay ordinary after a refused registration");
    }

    #[test]
    fn register_is_idempotent_and_builtins_are_ok() {
        let mut session = CoreSession::new();
        session
            .register_compute_predicate("product".to_string())
            .expect("re-registering a builtin is idempotent");
        session
            .register_compute_predicate("eats".to_string())
            .expect("a fresh unreferenced name registers");
        session
            .register_compute_predicate("eats".to_string())
            .expect("re-registering the same name is idempotent");
        let error = session
            .assert_text("eats(Bela, Cheese).")
            .expect_err("register-then-assert stays closed by the ComputeNode guard");
        assert!(error.to_string().contains("query-only"), "{error}");
    }

    #[test]
    fn retract_then_register_succeeds_and_queries_route_to_dispatch() {
        let mut session = CoreSession::new();
        let fact_id = session.assert_text("eats(Bela, Cheese).").unwrap()[0].0;
        session
            .register_compute_predicate("eats".to_string())
            .expect_err("the live fact must block registration");
        session.retract_fact(fact_id).unwrap();
        session
            .register_compute_predicate("eats".to_string())
            .expect("after retraction the registration must succeed");
        session
            .assert_text("eats(Bela, Cheese).")
            .expect_err("post-registration asserts are query-only");
        assert_eq!(
            session.query_text("eats(Bela, Cheese).").unwrap(),
            nibli_types::logic::QueryResult::Unknown(
                nibli_types::logic::UnknownReason::BackendUnavailable
            ),
            "the registered query must dispatch, never consult the retracted store"
        );
    }

    #[test]
    fn reset_then_register_succeeds() {
        let mut session = CoreSession::new();
        session.assert_text("eats(Bela, Cheese).").unwrap();
        session.reset().unwrap();
        session
            .register_compute_predicate("eats".to_string())
            .expect("reset clears the fact registry, so nothing blocks");
        assert!(
            session.compute_predicates().contains("product"),
            "the builtin seed survives reset (config, not content)"
        );
    }

    #[test]
    fn a_quoted_mention_does_not_block_registration() {
        let mut session = CoreSession::new();
        session
            .assert_text("believe(me, fact { eats(Bela, Cheese) }).")
            .unwrap();
        session
            .register_compute_predicate("eats".to_string())
            .expect("opaque quoted content is not a live reference");
    }

    #[test]
    fn role_spelled_and_engine_special_names_are_refused_at_registration() {
        let mut session = CoreSession::new();
        let err = session
            .register_compute_predicate("eats_x1".to_string())
            .expect_err("a role spelling would strand its anchor's stored facts");
        assert!(err.to_string().contains("role spellings collapse"), "{err}");
        for name in ["equals", "greater", "less", "num_equal"] {
            let err = session
                .register_compute_predicate(name.to_string())
                .expect_err("an engine-special relation must not be markable as compute");
            assert!(err.to_string().contains("engine-special"), "{name}: {err}");
        }
    }

    #[test]
    fn a_raw_injected_reference_surfaces_on_the_next_registration() {
        let mut session = CoreSession::new();
        session
            .register_compute_predicate("eats".to_string())
            .expect("a fresh unreferenced name registers");
        // Below the session seam: a hand-built UNMARKED buffer for the
        // registered name passes preflight (it is neither a ComputeNode nor a
        // reference name). The scan runs even on re-registration, so the
        // violated invariant surfaces instead of returning Ok forever.
        let raw = LogicBuffer {
            nodes: vec![LogicNode::Predicate((
                "eats".to_string(),
                vec![
                    LogicalTerm::Constant("Bela".to_string()),
                    LogicalTerm::Constant("Cheese".to_string()),
                ],
            ))],
            roots: vec![0],
        };
        session
            .kb()
            .assert_fact(raw, "raw sub-session ingress".to_string())
            .expect("raw KB ingress bypasses session marking by design");
        let err = session
            .register_compute_predicate("eats".to_string())
            .expect_err("the self-enforcing scan must surface the raw-injected reference");
        assert!(err.to_string().contains("cannot register"), "{err}");
    }

    #[test]
    fn buffer_replay_ingress_re_marks_against_the_live_registry() {
        let mut session = CoreSession::new();
        let plain = LogicBuffer {
            nodes: vec![LogicNode::Predicate((
                "eats".to_string(),
                vec![LogicalTerm::Constant("Bela".to_string())],
            ))],
            roots: vec![0],
        };
        session
            .assert_buffer_with_id(plain.clone(), "pre-registration row".to_string(), 0)
            .expect("an unregistered name replays as an ordinary fact");
        session.retract_fact(0).unwrap();
        session
            .register_compute_predicate("eats".to_string())
            .expect("no live references after retraction");
        let err = session
            .assert_buffer_with_id(plain, "out-of-order replay".to_string(), 1)
            .expect_err(
                "a replay after registration must fail closed, not store an unreachable fact",
            );
        assert!(err.to_string().contains("query-only"), "{err}");
    }

    #[test]
    fn the_id_list_in_the_refusal_is_bounded() {
        let mut session = CoreSession::new();
        let names = [
            "Ana", "Bela", "Cira", "Dana", "Elis", "Fara", "Gina", "Hana", "Ines", "Jana",
        ];
        for name in names {
            session
                .assert_text(&format!("eats({name}, Cheese)."))
                .unwrap();
        }
        let message = session
            .register_compute_predicate("eats".to_string())
            .expect_err("ten live references must block registration")
            .to_string();
        assert_eq!(
            message.matches('#').count(),
            8,
            "exactly eight ids shown: {message}"
        );
        assert!(
            message.contains("and 2 more"),
            "the remainder must be counted: {message}"
        );
    }
}
