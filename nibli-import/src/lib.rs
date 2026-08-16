//! Knowledge base import/export for standard formats.
//!
//! Provides RDF Turtle import, OWL class hierarchy import, and fact export.
//! Uses `nibli-engine` to inject facts via `assert_fact_direct`, so the
//! engine's assertion-ingress guards apply: the reference external compute
//! names (`exponential`, `logarithm`) are refused at ingress like any other
//! query-only shape rather than stored as unreachable facts.

pub mod export;
pub mod owl;
pub mod rdf;

use nibli_engine::NibliEngine;

/// Import RDF Turtle triples into the engine's KB.
/// Parses the Turtle text, then for each triple:
/// - `rdfs:subClassOf` → `declare_subsort`
/// - `rdf:type` → `declare_entity_sort`
/// - Other → `assert_fact_direct(predicate, [subject, object])`
///
/// Returns the number of facts/declarations imported.
pub fn import_turtle(engine: &NibliEngine, turtle_text: &str) -> Result<usize, String> {
    let triples = rdf::parse_turtle(turtle_text)?;
    owl::import_owl_classes(engine, &triples)
}

/// Import raw RDF triples (without OWL class handling) into the KB.
/// Every triple becomes a 2-argument fact: `predicate(subject, object)`.
///
/// Returns the number of facts asserted.
pub fn import_triples_raw(engine: &NibliEngine, turtle_text: &str) -> Result<usize, String> {
    let triples = rdf::parse_turtle(turtle_text)?;
    let mut count = 0;
    for triple in &triples {
        let rel = triple.predicate.local_name().to_string();
        let args = vec![
            term_to_logical(&triple.subject),
            term_to_logical(&triple.object),
        ];
        engine
            .assert_fact_direct(rel, args)
            .map_err(|e| e.to_string())?;
        count += 1;
    }
    Ok(count)
}

/// Export the engine's current truth store as fail-closed N-Triples: the
/// document plus per-fact refusals (see [`export::export_ntriples`]).
pub fn export_facts(engine: &NibliEngine) -> export::NTriplesExport {
    export::export_ntriples(engine)
}

fn term_to_logical(term: &rdf::Term) -> nibli_engine::EngineLogicalTerm {
    match term {
        rdf::Term::Iri(_) => {
            nibli_engine::EngineLogicalTerm::Constant(term.local_name().to_string())
        }
        rdf::Term::StringLiteral(s) => nibli_engine::EngineLogicalTerm::Constant(s.clone()),
        rdf::Term::NumericLiteral(n) => nibli_engine::EngineLogicalTerm::Number(*n),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_simple_triple() {
        let engine = NibliEngine::new();
        let turtle =
            r#"<http://example.org/adam> <http://example.org/likes> <http://example.org/bob> ."#;
        let count = import_triples_raw(&engine, turtle).unwrap();
        assert_eq!(count, 1);
        let facts = engine.list_facts().unwrap();
        assert!(!facts.is_empty());
    }

    #[test]
    fn test_import_owl_class_hierarchy() {
        let engine = NibliEngine::new();
        let turtle = r#"
<http://example.org/Dog> <http://www.w3.org/2000/01/rdf-schema#subClassOf> <http://example.org/Animal> .
<http://example.org/adam> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://example.org/Dog> .
"#;
        let count = import_turtle(&engine, turtle).unwrap();
        assert_eq!(count, 2); // 1 subsort + 1 entity sort
    }

    #[test]
    fn test_import_numeric_literal() {
        let engine = NibliEngine::new();
        let turtle = r#"<http://example.org/adam> <http://example.org/age> "42"^^<http://www.w3.org/2001/XMLSchema#integer> ."#;
        let count = import_triples_raw(&engine, turtle).unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_import_with_prefix() {
        let engine = NibliEngine::new();
        let turtle = r#"
@prefix ex: <http://example.org/> .
ex:adam ex:likes ex:bob .
"#;
        let count = import_triples_raw(&engine, turtle).unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn a_reference_compute_predicate_import_is_refused_not_stored_unreachable() {
        // An RDF predicate whose local name is a reserved external-compute
        // name would import as an ordinary fact no compute query ever
        // consults; ingress refuses it fail-closed instead.
        let engine = NibliEngine::new();
        let turtle = r#"<http://example.org/eight> <http://example.org/exponential> <http://example.org/two> ."#;
        let error = import_triples_raw(&engine, turtle)
            .expect_err("a reserved compute name must not import as an ordinary fact");
        assert!(error.contains("query-only"), "{error}");
        assert!(engine.list_facts().unwrap().is_empty());
    }

    #[test]
    fn export_roundtrips_through_an_independent_parser_and_reimport() {
        use rio_api::parser::TriplesParser;
        let engine = NibliEngine::new();
        let c = |s: &str| nibli_engine::EngineLogicalTerm::Constant(s.to_string());
        engine
            .assert_fact_direct("likes".to_string(), vec![c("adam"), c("bob")])
            .unwrap();
        engine
            .assert_fact_direct("knows".to_string(), vec![c("bob"), c("adam")])
            .unwrap();
        engine
            .assert_fact_direct(
                "age".to_string(),
                vec![c("adam"), nibli_engine::EngineLogicalTerm::Number(42.5)],
            )
            .unwrap();

        let out = export_facts(&engine);
        assert_eq!(
            out.exported, 3,
            "all three facts are in the exportable fragment"
        );
        assert!(
            out.refused.is_empty(),
            "nothing to refuse: {:?}",
            out.refused
        );

        // Independent parser: the document must be VALID N-Triples to a
        // foreign reader, not merely to our own rdf.rs.
        let mut parsed = 0usize;
        rio_turtle::NTriplesParser::new(out.document.as_bytes())
            .parse_all(&mut |_| -> Result<(), rio_turtle::TurtleError> {
                parsed += 1;
                Ok(())
            })
            .expect("the export must be valid N-Triples to an independent parser");
        assert_eq!(parsed, 3, "the independent parser must see every triple");

        // Re-import identity: the document re-imports (raw mode strips the
        // minted base back off) to exactly the exported facts — checked by
        // re-exporting the fresh engine and comparing documents byte-for-byte
        // (deterministic: sorted + deduplicated).
        let fresh = NibliEngine::new();
        let n = import_triples_raw(&fresh, &out.document).unwrap();
        assert_eq!(n, 3);
        let round = export_facts(&fresh);
        assert!(round.refused.is_empty(), "{:?}", round.refused);
        assert_eq!(
            round.document, out.document,
            "export -> re-import -> export must be the identity on the fragment \
             (numbers included: 42.5 rides \"42.5\"^^xsd:double)"
        );
    }

    #[test]
    fn export_refuses_every_unrepresentable_shape_with_a_reason() {
        use export::fact_to_triple;
        use nibli_reason::kb::{GroundFact, GroundTerm, StoredFact};
        let c = |s: &str| GroundTerm::Constant(s.to_string());
        let cases: Vec<(StoredFact, &str)> = vec![
            (
                StoredFact::Past(GroundFact::new("likes", vec![c("adam"), c("bob")])),
                "flavor",
            ),
            (
                StoredFact::Bare(GroundFact::new("dog", vec![c("adam")])),
                "arity 1",
            ),
            (
                StoredFact::Bare(GroundFact::new(
                    "gives",
                    vec![c("adam"), c("bob"), c("ring")],
                )),
                "arity 3",
            ),
            (
                StoredFact::Bare(GroundFact::new("type", vec![c("adam"), c("dog")])),
                "rdf:type/rdfs:subClassOf routing",
            ),
            (
                StoredFact::Bare(GroundFact::new("subClassOf", vec![c("dog"), c("animal")])),
                "rdf:type/rdfs:subClassOf routing",
            ),
            (
                StoredFact::Bare(GroundFact::new(
                    "age",
                    vec![GroundTerm::from_f64(3.0), c("adam")],
                )),
                "no IRI form",
            ),
            (
                StoredFact::Bare(GroundFact::new(
                    "likes",
                    vec![c("adam"), GroundTerm::Unspecified],
                )),
                "engine-internal",
            ),
            (
                StoredFact::Bare(GroundFact::new("likes", vec![c("adam"), c("hello world")])),
                "not IRI-local-name safe",
            ),
            (
                StoredFact::Bare(GroundFact::new(
                    "age",
                    vec![c("adam"), GroundTerm::from_f64(f64::INFINITY)],
                )),
                "non-finite",
            ),
        ];
        for (fact, needle) in cases {
            let err =
                fact_to_triple(&fact).expect_err(&format!("{fact:?} must have no triple form"));
            assert!(
                err.contains(needle),
                "refusal for {fact:?} must name its class ({needle:?}), got: {err}"
            );
        }
    }

    #[test]
    fn export_refuses_event_decomposed_kr_facts_rather_than_mangling_them() {
        // A KR-authored fact event-decomposes (`eats(Adam).` stores an event
        // Skolem plus role facts). None of that has a faithful triple form —
        // the export must refuse ALL of it loudly, never emit a mangled
        // Skolem-bearing triple.
        let engine = NibliEngine::new();
        engine.assert_text("eats(Adam).").unwrap();
        let out = export_facts(&engine);
        assert_eq!(
            out.exported, 0,
            "no decomposed fact may leak into the document: {}",
            out.document
        );
        assert!(
            !out.refused.is_empty(),
            "the refusals must say why the KB exported empty"
        );
    }
}
