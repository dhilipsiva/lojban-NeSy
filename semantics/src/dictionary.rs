// Include the perfect hash map generated at compile time
include!(concat!(env!("OUT_DIR"), "/generated_dictionary.rs"));

pub struct JbovlasteSchema;

impl JbovlasteSchema {
    /// Retrieves the arity of a predicate in O(1) time with zero allocation.
    /// Returns None for unknown words (not in jbovlaste).
    pub fn get_arity(word: &str) -> Option<usize> {
        JBOVLASTE_ARITIES.get(word).copied()
    }

    /// Retrieves the arity, defaulting to 2 for unknown words.
    /// Use this only when a fallback is acceptable (e.g., lujvo not in dictionary).
    pub fn get_arity_or_default(word: &str) -> usize {
        JBOVLASTE_ARITIES.get(word).copied().unwrap_or(2)
    }
}
