//! Native nibli engine library: calls nibli-kr/nibli-semantics/nibli-reason directly as Rust crates.
//! No WASM, no Wasmtime — full stack traces for debugging.

use std::cell::RefCell;
use std::path::Path;

use nibli_store::NibliStore;

pub use nibli_reason::ComputeRequest as EngineComputeRequest;
pub use nibli_types::logic::{
    AggregateOp as EngineAggregateOp, FactSummary as EngineFactSummary,
    LogicBuffer as EngineLogicBuffer, LogicNode as EngineLogicNode,
    LogicalTerm as EngineLogicalTerm, QueryResult as EngineQueryResult,
    ResourceKind as EngineResourceKind, UnknownReason as EngineUnknownReason,
    WitnessBinding as EngineWitnessBinding, WitnessOrigin as EngineWitnessOrigin,
};

/// The pipeline's typed error (`Syntax`/`Semantic`/`Reasoning`/`Backend`),
/// re-exported so embedders, tests, and the server can pattern-match the error
/// CLASS instead of string-parsing the `[Xxx Error]` Display prefix.
pub use nibli_types::error::NibliError as EngineError;
use nibli_types::logic;

mod compute_client;

// ═══════════════════════════════════════════════════════════════════════
// PROOF TRACE CONVERSION
// ═══════════════════════════════════════════════════════════════════════
//
// The canonical proof types ARE the wire types now (serde-derived in nibli-types),
// so there is no canonical->wire conversion; `nibli-protocol` only supplies JSON
// helpers. Readable rendering lives in `nibli-render`. Term display is an inherent
// method on the canonical `LogicalTerm` enum.

pub fn display_term(term: &EngineLogicalTerm) -> String {
    term.trace_display()
}

pub fn display_query_result(result: &EngineQueryResult) -> String {
    match result.detail_label() {
        Some(detail) => format!("{} ({})", result.status_label(), detail),
        None => result.status_label().to_string(),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// ENGINE WRAPPER
// ═══════════════════════════════════════════════════════════════════════

pub struct NibliEngine {
    /// The shared compile/assert/query core (nibli-session) — the same
    /// CoreSession the pipeline/wasm/ui surfaces wrap, so native and WASM
    /// agree BY CONSTRUCTION.
    core: nibli_session::CoreSession,
    store: RefCell<Option<NibliStore>>,
}

impl Default for NibliEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl NibliEngine {
    /// Access the underlying KnowledgeBase for sort/constraint declarations.
    pub fn kb(&self) -> &nibli_reason::KnowledgeBase {
        self.core.kb()
    }

    /// Install a cooperative cancellation flag on the underlying reasoning core.
    /// When the flag is raised, an in-flight query aborts via the error channel
    /// (returned as a `String` error from the query methods). The native
    /// nibli-server watchdog uses this to free a blocking thread when a request's
    /// wall-clock budget elapses, instead of letting a pathological query run to
    /// completion. No clock is read inside the engine.
    pub fn set_cancel_flag(&self, flag: std::sync::Arc<std::sync::atomic::AtomicBool>) {
        self.core.kb().set_cancel_flag(flag);
    }

    /// Remove any installed cancellation flag.
    pub fn clear_cancel_flag(&self) {
        self.core.kb().clear_cancel_flag();
    }

    /// Enable/disable the engine's informational stdout diagnostics
    /// (`[Rule]`/`[Skolem]`/`[Constraint] Registered`). Default OFF — nibli-engine
    /// is a silent library, so the server/validate/tavla do not spam stdout on a
    /// per-query corpus re-assertion. Interactive callers (the native `nibli`
    /// REPL) opt in. Configuration — survives `reset()`.
    pub fn set_verbose(&self, verbose: bool) {
        self.core.kb().set_verbose(verbose);
    }

    /// Enable/disable STRICT MODE (default off — permissive warn-and-insert):
    /// when on, an arity mismatch or integrity-constraint violation REJECTS the
    /// offending fact and fails the assertion. Like `set_verbose`, the library
    /// stays permissive by default; embedders opt in programmatically (the
    /// runtime surfaces read `NIBLI_STRICT=1` — nibli-host forwards it into the
    /// guest, where `nibli-pipeline::Session::new` applies it).
    pub fn set_strict(&self, strict: bool) {
        self.core.kb().set_strict(strict);
    }

    /// Enable/disable legacy EXISTENTIAL-IMPORT MODE (default OFF). ON makes a
    /// description universal mint logical witnesses that participate in every
    /// quantifier/find/count surface. The change transactionally rebuilds the KB.
    pub fn set_existential_import(&self, on: bool) -> Result<(), EngineError> {
        self.core.set_existential_import(on)
    }

    /// Whether legacy existential import is active for this whole engine.
    pub fn is_existential_import(&self) -> bool {
        self.core.is_existential_import()
    }

    /// Enable/disable STRATUM-ORDERED MATERIALISATION (default ON). When on, the
    /// relations a query reads under `~` are saturated bottom-up in stratum order and
    /// each NAF check becomes a set-membership test. OFF restores the pure
    /// backward-chaining path. The runtime surfaces read `NIBLI_MATERIALIZE=0`
    /// (nibli-host forwards it into the guest, where `nibli-pipeline::Session::new`
    /// applies it).
    pub fn set_materialization(&self, on: bool) {
        self.core.kb().set_materialization(on);
    }

    /// What the last query's saturation covered: `(completed, [(relation, why not)])`.
    /// The only way to see whether a slow `~p(x)` actually got the lookup.
    pub fn materialization_report(&self) -> (Vec<String>, Vec<(String, String)>) {
        self.core.kb().materialization_report()
    }

    /// Register this engine's external compute dispatch (per-instance). Without
    /// it, external predicates (e.g. `tenfa`/`dugri`) return an error; built-in
    /// arithmetic (pilji/sumji/dilcu) works regardless. Replaces the old
    /// thread-local registration that the multithreaded server could not use.
    /// See `nibli_reason::KnowledgeBase::set_compute_dispatch` for the trust boundary.
    pub fn set_compute_dispatch(
        &self,
        eval: fn(&str, &[EngineLogicalTerm]) -> Result<bool, String>,
        batch_eval: fn(&[nibli_reason::ComputeRequest]) -> Vec<Result<bool, String>>,
    ) {
        self.core.kb().set_compute_dispatch(eval, batch_eval);
    }

    /// Enable external compute dispatch to a Python-style JSON-Lines backend at
    /// `addr` (e.g. `"127.0.0.1:5555"`). Wires the native TCP client as this
    /// engine's compute dispatch, so registered external predicates (e.g.
    /// `tenfa`/`dugri`) are evaluated by the backend; built-in arithmetic
    /// (pilji/sumji/dilcu) is still resolved in-engine. Opt-in — engines that do
    /// not call this leave external compute unregistered (`set_compute_dispatch`
    /// isolation preserved). The address is stored per-thread; in the
    /// multithreaded server each `spawn_blocking` worker connects lazily and
    /// reuses its connection. Register the external predicate names separately
    /// via `register_compute_predicate`. Trust boundary: the backend is a
    /// plaintext, unauthenticated peer in the trusted computing base.
    pub fn enable_compute_backend(&self, addr: &str) {
        compute_client::set_addr(addr);
        self.core.kb().set_compute_dispatch(
            compute_client::native_eval_fn,
            compute_client::native_batch_eval_fn,
        );
    }

    /// Create an engine without persistence (existing behavior).
    pub fn new() -> Self {
        NibliEngine {
            core: nibli_session::CoreSession::new(),
            store: RefCell::new(None),
        }
    }

    /// Create an engine with disk persistence at the given path.
    /// Opens a `RedbFactStore` for typed fact persistence and replays
    /// the legacy `NibliStore` (LogicBuffer-level) for backward compatibility.
    pub fn open(db_path: &Path) -> Result<Self, String> {
        let mut store = NibliStore::open(db_path, "local".to_string())
            .map_err(|e| format!("Store error: {e}"))?;

        // Upgrade a legacy v2 registry to v3. Engine-written DBs hold bare
        // `LogicBuffer` payloads (never `StoredAssertion::Text`), so this is a
        // version restamp only — NOT `migrate_v2_text_rows` (decoding a bare
        // buffer as a `StoredAssertion` is a category error). Any genuinely
        // undecodable row still fails closed in `replay_from_store` below.
        if store.needs_migration() {
            store
                .finalize_v3()
                .map_err(|e| format!("Store error: {e}"))?;
        }

        // Open persistent typed fact store alongside the legacy store.
        let typed_db_path = db_path.with_extension("typed.redb");
        // The fact REGISTRY (the store opened above) is the durable source of
        // truth: retraction tombstones live there, and remote merges land
        // there. The typed store is only the KB's write-through mirror — a
        // store-level retraction never touches its rows. Discard it WITHOUT
        // decoding first: old hash-only/corrupt mirror rows are recoverable
        // because the canonical LogicBuffer registry below rebuilds the mirror.
        // Direct RedbFactStore::open callers retain the strict validation path.
        let typed_store = nibli_store::typed_store::RedbFactStore::open_for_rebuild(&typed_db_path)
            .map_err(|e| format!("TypedStore error: {e}"))?;

        let kb = nibli_reason::KnowledgeBase::with_store(Box::new(typed_store))
            .map_err(|e| format!("TypedStore compatibility error: {e}"))?;
        let engine = NibliEngine {
            core: nibli_session::CoreSession::with_kb(kb),
            store: RefCell::new(Some(store)),
        };
        engine.replay_from_store()?;
        Ok(engine)
    }

    /// Replay all persisted facts into the in-memory KB.
    fn replay_from_store(&self) -> Result<(), String> {
        let store = self.store.borrow();
        let Some(store) = store.as_ref() else {
            return Ok(()); // No store configured — nothing to replay.
        };
        let facts = store
            .all_active_facts()
            .map_err(|e| format!("Store error: {e}"))?;
        for fact in &facts {
            let buf: logic::LogicBuffer = postcard::from_bytes(&fact.payload)
                .map_err(|e| format!("Deserialize error: {e}"))?;
            self.core
                .kb()
                .assert_fact_with_id(buf, fact.label.clone(), fact.id)
                .map_err(|e| format!("Replay error (fact {}): {e}", fact.id))?;
        }
        Ok(())
    }

    /// Validate KR text without asserting — returns Ok if it parses and compiles.
    /// This is intentionally compile-only: a CountNode is valid query IR even
    /// though `assert_text` will reject it as query-only.
    pub fn validate(&self, text: &str) -> Result<(), String> {
        self.compile_text(text)
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    /// Register a predicate name for external compute dispatch.
    pub fn register_compute_predicate(&mut self, name: String) {
        self.core.register_compute_predicate(name);
    }

    fn compile_text(&self, input: &str) -> Result<logic::LogicBuffer, EngineError> {
        // The SOLE text→AST seam — every public text method funnels through
        // here, delegating to the SHARED chain (nibli-session), the same core
        // the WASM surfaces wrap; `EngineError` is the re-exported `NibliError`.
        self.core.compile_text(input)
    }

    fn compile_query_text(&self, input: &str) -> Result<logic::LogicBuffer, EngineError> {
        self.core.compile_query_text(input)
    }

    /// Persist one compiled root and install it in the live KB under the same
    /// source id. The durable registry and KB counters are both consulted so
    /// even a caller that used the exposed lower-level KB cannot cause reuse.
    /// Pure assertion preflight happens before serialization, id lookup, or a
    /// durable write. After that, persistence happens before reasoning; a later
    /// reasoning rejection deletes that row before the error is returned, so
    /// reopen cannot resurrect a failed assertion.
    fn persist_and_assert(
        &self,
        store: &mut NibliStore,
        buffer: logic::LogicBuffer,
        label: String,
    ) -> Result<u64, EngineError> {
        self.core.kb().validate_assertion(&buffer)?;
        let payload = postcard::to_allocvec(&buffer)
            .map_err(|e| EngineError::Reasoning(format!("Serialize error: {e}")))?;
        let durable_id = store
            .next_fact_id()
            .map_err(|e| EngineError::Reasoning(format!("Store error: {e}")))?;
        let live_id = self.core.kb().next_fact_id()?;
        let fact_id = durable_id.max(live_id);

        store
            .insert_fact(fact_id, label.clone(), payload)
            .map_err(|e| EngineError::Reasoning(format!("Store error: {e}")))?;

        if let Err(reason) = self.core.kb().assert_fact_with_id(buffer, label, fact_id) {
            return match store.delete_fact(fact_id) {
                Ok(()) => Err(EngineError::Reasoning(reason)),
                Err(rollback) => Err(EngineError::Reasoning(format!(
                    "{reason} (additionally, durable rollback of fact {fact_id} failed: {rollback})"
                ))),
            };
        }

        Ok(fact_id)
    }

    /// Reset the knowledge base, clearing all facts and rules.
    pub fn reset(&self) {
        self.core.kb().reset().ok();
        if let Ok(mut store) = self.store.try_borrow_mut()
            && let Some(s) = store.as_mut()
        {
            let _ = s.clear();
        }
    }

    /// Parse KR text, compile to FOL, and assert into the knowledge base.
    ///
    /// A bare-`.i` multi-sentence text becomes N INDEPENDENT facts — one per root —
    /// each with its own id, store record, and retraction (connectives compile to a
    /// single root and stay one fact). Returns the minted ids in root order. A
    /// single-sentence text yields exactly one id. Exact-count formulas in
    /// asserted position (outside opaque quoted content) are query-only. The
    /// whole compiled input is preflighted before any independently retractable
    /// root receives an id or durable row.
    pub fn assert_text(&self, text: &str) -> Result<Vec<u64>, EngineError> {
        let mut store = self.store.try_borrow_mut().map_err(|_| {
            EngineError::Reasoning("Store error: persistence state is already borrowed".to_string())
        })?;

        // No-store path: the shared core's assert loop IS this behavior.
        let Some(s) = store.as_mut() else {
            return Ok(self
                .core
                .assert_text(text)?
                .into_iter()
                .map(|(id, _)| id)
                .collect());
        };

        // Store write-through path (engine-specific): the durable registry and
        // live KB share one collision-free id space, so the per-root loop runs
        // here with persistence in the middle.
        let buf = self.compile_text(text)?;
        self.core.kb().validate_assertion(&buf)?;
        let label = text.to_string();
        let parts = buf.split_roots();
        let mut ids = Vec::with_capacity(parts.len());
        for sub in parts {
            ids.push(self.persist_and_assert(s, sub, label.clone())?);
        }
        Ok(ids)
    }

    /// Assert a fact directly by relation name and arguments, bypassing text
    /// parsing. Uses the same durable id/rollback path as [`Self::assert_text`]
    /// when persistence is configured (label `":assert {relation}"`) and is
    /// event-decomposed to the surface shape.
    pub fn assert_fact_direct(
        &self,
        relation: String,
        args: Vec<EngineLogicalTerm>,
    ) -> Result<u64, EngineError> {
        let mut store = self.store.try_borrow_mut().map_err(|_| {
            EngineError::Reasoning("Store error: persistence state is already borrowed".to_string())
        })?;
        let Some(store) = store.as_mut() else {
            return self.core.assert_fact_direct(&relation, &args, None);
        };

        let buffer = nibli_semantics::compile_injected_fact(&relation, &args)?;
        self.persist_and_assert(store, buffer, format!(":assert {relation}"))
    }

    /// Parse KR query, run entailment check, return result + formatted proof + JSON proof.
    pub fn query_text_with_proof(
        &self,
        text: &str,
    ) -> Result<(EngineQueryResult, String, String), EngineError> {
        let (result, trace) = self.core.query_text_with_proof(text)?;
        // `trace` IS the wire `ProofTrace` (canonical == wire now) — no conversion.
        let formatted = nibli_render::render_proof_text(&trace, nibli_render::Register::Spec);
        let json = nibli_protocol::proof_trace_to_json(&trace);
        Ok((result, formatted, json))
    }

    /// Parse a KR query, run the entailment check, and return the typed
    /// result together with the raw wire [`nibli_protocol::ProofTrace`] — for
    /// callers/tests that need structured proof access (the plain-English "why"
    /// summary, the collapsed macro-DAG view) rather than the pre-formatted text.
    pub fn query_text_raw_proof(
        &self,
        text: &str,
    ) -> Result<(EngineQueryResult, nibli_protocol::ProofTrace), EngineError> {
        self.core.query_text_with_proof(text)
    }

    /// Evaluate a KR query against the KB and return the typed query result.
    pub fn query_holds(&self, text: &str) -> Result<EngineQueryResult, EngineError> {
        self.core.query_text(text)
    }

    /// Parse a KR query and extract all satisfying witness bindings.
    pub fn query_find_text(
        &self,
        text: &str,
    ) -> Result<Vec<Vec<EngineWitnessBinding>>, EngineError> {
        self.core.query_find_text(text)
    }

    /// Count the number of distinct witness binding sets satisfying a KR query.
    /// Exposes `nibli_reason::KnowledgeBase::count_witnesses` at the embedding level.
    pub fn count_witnesses_text(&self, text: &str) -> Result<usize, EngineError> {
        self.core.count_witnesses_text(text)
    }

    /// Aggregate the numeric values bound to `variable` across all witness binding
    /// sets of a KR query, applying `op` (Sum/Min/Max/Avg). Returns `Ok(None)`
    /// when no numeric witnesses are found. Exposes `nibli_reason::KnowledgeBase::aggregate`.
    pub fn aggregate_text(
        &self,
        text: &str,
        variable: &str,
        op: nibli_types::logic::AggregateOp,
    ) -> Result<Option<f64>, EngineError> {
        self.core.aggregate_text(text, variable, op)
    }

    /// Compile KR text to the typed FOL `LogicBuffer` without asserting.
    ///
    /// Returns the IR directly — the caller renders it (e.g. via
    /// `nibli_render::render_logic_tree` / `render_logic_buffer`). No
    /// S-expression string is produced.
    pub fn compile_debug(&self, text: &str) -> Result<EngineLogicBuffer, EngineError> {
        self.compile_text(text)
    }

    /// Query-intent alias for [`Self::compile_debug`]. Assertions and queries
    /// consume the same canonical typed FOL `LogicBuffer`, including
    /// connected-clause `$name` co-reference.
    pub fn compile_query_debug(&self, text: &str) -> Result<EngineLogicBuffer, EngineError> {
        self.compile_query_text(text)
    }

    /// List all active (non-retracted) facts with their IDs and labels.
    pub fn list_facts(&self) -> Result<Vec<EngineFactSummary>, EngineError> {
        self.core.kb().list_facts()
    }

    /// Retract a fact by ID and rebuild derived state.
    ///
    /// When persistence is configured, the retraction is also written through to
    /// the on-disk store as a tombstone, so a subsequent `open()` does NOT replay
    /// (resurrect) the retracted fact. The in-memory KB is retracted first (this
    /// validates the ID and rebuilds derived state); the durable tombstone is only
    /// written if that succeeds, keeping both layers consistent.
    pub fn retract_fact(&self, id: u64) -> Result<(), EngineError> {
        self.core.kb().retract_fact(id)?;

        let mut store = self.store.try_borrow_mut().map_err(|_| {
            EngineError::Reasoning("Store error: persistence state is already borrowed".to_string())
        })?;
        if let Some(s) = store.as_mut() {
            // Idempotent at the store layer: retracting an already-tombstoned or
            // never-persisted-but-known fact is fine. A NotFound here means a caller
            // mutated the exposed lower-level KB directly — that is not a durability
            // failure, so swallow it.
            match s.retract_fact(id) {
                Ok(()) => {}
                Err(nibli_store::StoreError::NotFound(_)) => {}
                Err(e) => return Err(EngineError::Reasoning(format!("Store error: {e}"))),
            }
        }
        Ok(())
    }

    /// Scan for contradictions (asserted store + derived positives for `~P`;
    /// not a full closure proof — see
    /// [`nibli_reason::KnowledgeBase::check_contradictions`]).
    pub fn check_contradictions(&self) -> Vec<String> {
        self.core.kb().check_contradictions()
    }

    /// Enable tracing for a predicate (interactive debugging).
    pub fn trace_predicate(&self, predicate: &str) {
        self.core.kb().trace_predicate(predicate);
    }

    /// Disable tracing for a predicate.
    pub fn untrace_predicate(&self, predicate: &str) {
        self.core.kb().untrace_predicate(predicate);
    }

    /// List all currently traced predicates.
    pub fn traced_predicates(&self) -> Vec<String> {
        self.core.kb().traced_predicates()
    }
}

#[cfg(test)]
mod tests {
    use super::NibliEngine;
    use std::fs;
    use std::path::{Path, PathBuf};

    fn temp_db_path(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join("nibli_engine_tests");
        fs::create_dir_all(&dir).unwrap();
        dir.join(format!("{name}.redb"))
    }

    fn cleanup(path: &Path) {
        let _ = fs::remove_file(path);
    }

    /// The persisted payload is now `nibli_types::logic::LogicBuffer` serialized
    /// directly via serde/postcard (the `StoredLogicBuffer` mirror was deleted).
    /// This pins that round-trip over every node + term variant — the property the
    /// replay path (`replay_from_store`) and the write path (`assert_text`) depend on.
    #[test]
    fn logic_buffer_serde_postcard_roundtrip_covers_all_variants() {
        use nibli_types::logic::{LogicBuffer, LogicNode, LogicalTerm};

        let buf = LogicBuffer {
            nodes: vec![
                LogicNode::Predicate((
                    "gerku".into(),
                    vec![
                        LogicalTerm::Constant("adam".into()),
                        LogicalTerm::Variable("x".into()),
                        LogicalTerm::Description("le-dog".into()),
                        LogicalTerm::Unspecified,
                    ],
                )),
                LogicNode::Predicate(("danlu".into(), vec![LogicalTerm::Constant("adam".into())])),
                LogicNode::AndNode((0, 1)),
                LogicNode::ExistsNode(("x".into(), 2)),
                LogicNode::PastNode(0),
                LogicNode::NotNode(1),
                LogicNode::ForAllNode(("y".into(), 5)),
                LogicNode::ComputeNode((
                    "product".into(),
                    vec![LogicalTerm::Number(3.0), LogicalTerm::Number(4.0)],
                )),
                LogicNode::CountNode(("z".into(), 2, 0)),
                LogicNode::OrNode((0, 1)),
                LogicNode::PresentNode(0),
                LogicNode::FutureNode(0),
                LogicNode::ObligatoryNode(0),
                LogicNode::PermittedNode(0),
            ],
            roots: vec![2, 3],
        };

        let bytes = postcard::to_allocvec(&buf).unwrap();
        let decoded: LogicBuffer = postcard::from_bytes(&bytes).unwrap();
        assert_eq!(buf, decoded);
    }

    #[test]
    fn persistent_assert_does_not_mutate_kb_when_store_is_unavailable() {
        let path = temp_db_path("atomic_assert_store_busy");
        cleanup(&path);

        let engine = NibliEngine::open(&path).expect("Persistent engine should open");
        let _borrow = engine.store.borrow();

        let err = engine
            .assert_text("big(some dog).")
            .expect_err("Store borrow conflict should abort assertion");
        assert!(
            err.to_string().contains("Store error"),
            "Expected store error, got: {err}"
        );
        assert!(
            engine
                .query_holds("big(some dog).")
                .expect("Query should still run")
                .is_false(),
            "Failed persistent assertions must not leak into the live KB"
        );

        drop(_borrow);
        let store = engine.store.borrow();
        let facts = store
            .as_ref()
            .unwrap()
            .all_active_facts()
            .expect("Store should remain empty");
        assert!(
            facts.is_empty(),
            "Failed persistent assertions must not leak into the store"
        );

        drop(store);
        cleanup(&path);
    }
}
