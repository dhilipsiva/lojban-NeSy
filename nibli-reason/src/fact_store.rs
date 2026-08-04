//! Fact store abstraction for the reasoning engine.
//!
//! The `FactStore` trait decouples the KB from its storage backend.
//! `InMemoryFactStore` is the default (fast, bounded by available memory).
//! Persistent backends (e.g., redb) implement the same trait for lazy
//! on-demand loading from disk.

use std::collections::{HashMap, HashSet};

use crate::kb::StoredFact;

/// Abstract fact storage backend for the reasoning engine.
///
/// Implementations must maintain a predicate index (relation name → facts)
/// for O(1) amortized lookup on the hot path.
///
/// `Send` supertrait: a `Box<dyn FactStore>` lives inside `KnowledgeBaseInner`,
/// so without this bound the whole `KnowledgeBase` (and everything embedding it —
/// `NibliEngine`, tavla's `GossipNode`) is `!Send`, which makes it unusable as
/// async-graphql shared state (`Arc<Mutex<GossipNode>>` requires the inner to be
/// `Send`). All impls (in-memory, redb, WASI) hold only `Send` fields. We do not
/// require `Sync`: `KnowledgeBase` wraps its inner in `RefCell` (single-threaded
/// by design), and the server provides cross-thread safety via the outer `Mutex`.
pub trait FactStore: Send {
    /// Look up all facts for a given predicate relation.
    fn lookup_predicate(&self, relation: &str) -> Option<&HashSet<StoredFact>>;

    /// Check if a specific fact is asserted.
    fn contains(&self, fact: &StoredFact) -> bool;

    /// Insert a fact into the store.
    fn insert(&mut self, fact: StoredFact);

    /// Remove all facts (for reset/rebuild).
    fn clear(&mut self);

    /// Iterate over all stored facts, in no particular order. Boxed because
    /// `FactStore` is used as a trait object (`Box<dyn FactStore>` in the KB) —
    /// an RPITIT return would break dyn-compatibility.
    fn all_facts(&self) -> Box<dyn Iterator<Item = &StoredFact> + '_>;

    /// Number of facts stored.
    fn len(&self) -> usize;

    /// Whether the store is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Remove a specific fact from the store. Returns true if it was present.
    fn remove(&mut self, fact: &StoredFact) -> bool;

    /// Clone the store into a new boxed instance (for hypothetical reasoning).
    fn clone_box(&self) -> Box<dyn FactStore>;
}

/// In-memory fact store: the predicate index (relation name → fact set) IS the
/// single storage — `all_facts()` iterates the buckets. (The old shape kept a
/// second flat `HashSet` purely so `all_facts()` could return `&HashSet`,
/// deep-cloning every fact into both — 2× the fact-corpus heap.)
#[derive(Clone)]
pub struct InMemoryFactStore {
    predicate_index: HashMap<String, HashSet<StoredFact>>,
}

impl InMemoryFactStore {
    pub fn new() -> Self {
        Self {
            predicate_index: HashMap::new(),
        }
    }
}

impl Default for InMemoryFactStore {
    fn default() -> Self {
        Self::new()
    }
}

impl FactStore for InMemoryFactStore {
    fn lookup_predicate(&self, relation: &str) -> Option<&HashSet<StoredFact>> {
        self.predicate_index.get(relation)
    }

    fn contains(&self, fact: &StoredFact) -> bool {
        if let Some(set) = self.predicate_index.get(fact.relation()) {
            set.contains(fact)
        } else {
            false
        }
    }

    fn insert(&mut self, fact: StoredFact) {
        let rel = fact.relation().to_string();
        self.predicate_index.entry(rel).or_default().insert(fact);
    }

    fn clear(&mut self) {
        self.predicate_index.clear();
    }

    fn all_facts(&self) -> Box<dyn Iterator<Item = &StoredFact> + '_> {
        Box::new(self.predicate_index.values().flatten())
    }

    fn len(&self) -> usize {
        self.predicate_index.values().map(HashSet::len).sum()
    }

    fn remove(&mut self, fact: &StoredFact) -> bool {
        let Some(set) = self.predicate_index.get_mut(fact.relation()) else {
            return false;
        };
        let removed = set.remove(fact);
        if removed && set.is_empty() {
            self.predicate_index.remove(fact.relation());
        }
        removed
    }

    fn clone_box(&self) -> Box<dyn FactStore> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod abstraction_store_boundary_tests {
    use super::*;
    use crate::KnowledgeBase;
    use crate::kb::{GroundFact, GroundTerm, canonicalize_stored_fact_abstraction_marker};

    fn valid_marker_with_digest(digest: &str) -> String {
        let mut key = vec![0xa0, 0x10];
        key.extend_from_slice(&1_u64.to_be_bytes());
        key.push(b'p');
        key.extend_from_slice(&0_u64.to_be_bytes());
        let key_hex: String = key.iter().map(|byte| format!("{byte:02x}")).collect();
        format!("__abs_v1_{digest}_{key_hex}")
    }

    fn marker_fact(relation: &str, arity: usize) -> StoredFact {
        StoredFact::Bare(GroundFact::new(
            relation,
            (0..arity)
                .map(|index| GroundTerm::Constant(format!("a{index}")))
                .collect(),
        ))
    }

    fn preloaded(fact: StoredFact) -> Box<dyn FactStore> {
        let mut store = InMemoryFactStore::new();
        store.insert(fact);
        Box::new(store)
    }

    #[test]
    fn with_store_validates_preloaded_abstraction_rows() {
        let legacy = marker_fact("__abs_0123456789abcdef", 1);
        assert!(KnowledgeBase::with_store(preloaded(legacy)).is_err());

        let forged_relation = valid_marker_with_digest("0000000000000000");
        let mut canonical = marker_fact(&forged_relation, 1);
        canonicalize_stored_fact_abstraction_marker(&mut canonical).unwrap();
        assert_ne!(forged_relation, canonical.relation());
        assert!(
            KnowledgeBase::with_store(preloaded(marker_fact(&forged_relation, 1))).is_err(),
            "a custom store cannot be installed with a stale predicate index"
        );

        assert!(
            KnowledgeBase::with_store(preloaded(marker_fact(canonical.relation(), 2))).is_err(),
            "non-unary internal markers must fail closed"
        );
        assert!(
            KnowledgeBase::with_store(preloaded(canonical)).is_ok(),
            "canonical unary v1 rows remain installable"
        );
    }
}
