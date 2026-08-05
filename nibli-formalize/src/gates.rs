//! The validation gates — the "verify" firewall around the LLM's output.
//!
//! KR-only since THE DROP: every candidate runs `nibli_kr::parse_checked` →
//! nibli-semantics → the RENDER ROUND-TRIP gate (the drift-catcher: the candidate's
//! canonical re-spelling must compile to the SAME `LogicBuffer` — nibli-kr's
//! pinned fixpoint contract, `nibli-kr/src/render.rs`; AstBuffer equality is
//! deliberately NOT the contract there). The KB-authoring entry point then
//! rejects CountNodes in asserted position (opaque quoted content is inert).
//! The round-trip gate is pure Rust,
//! so it runs on native AND wasm. (The legacy Lojban chain — gerna + the
//! wasm-only camxes gate — retired with the Lojban front-end.)

use nibli_types::error::NibliError;
use nibli_types::logic::LogicBuffer;

/// Which gate rejected a candidate, carrying the compiler's own message. The
/// message is fed back verbatim into the LLM conversation as correction context.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GateError {
    /// The front-end rejected the grammar (nibli-kr's parse/resolve errors —
    /// including fail-closed dictionary-unknown aliases).
    Syntax(String),
    /// nibli-semantics rejected the semantics/arity (`NibliError::Semantic`),
    /// or the compiled formula is not assertable as KB content.
    Semantic(String),
    /// The render round-trip gate: the candidate compiles, but its canonical
    /// re-spelling (`nibli_kr::render`) fails to re-parse or compiles to a
    /// DIFFERENT `LogicBuffer` — the drift-catcher. *(native + wasm)*
    RoundTrip(String),
    /// The fresh-context semantic verifier judged that the candidate — though
    /// grammatically valid — does not MEAN what the source says (the message is
    /// the verifier's concrete mismatch list). Unlike the deterministic
    /// gates, this verdict comes from an LLM judge reading the `nibli_render`
    /// back-translation; it is best-effort advisory, but a mismatch still
    /// drives the same retry machinery as a hard gate failure.
    Verification(String),
}

impl GateError {
    /// Short human name of the gate that failed — for UI badges and logs.
    pub fn gate(&self) -> &'static str {
        match self {
            GateError::Syntax(_) => "nibli-kr",
            GateError::Semantic(_) => "semantics",
            GateError::RoundTrip(_) => "round-trip",
            GateError::Verification(_) => "semantic verifier",
        }
    }

    /// The underlying compiler message.
    pub fn message(&self) -> &str {
        match self {
            GateError::Syntax(m)
            | GateError::Semantic(m)
            | GateError::RoundTrip(m)
            | GateError::Verification(m) => m,
        }
    }
}

/// Run the local gates in fail-fast order and return the compiled FOL on
/// success. Mirrors `nibli-ui`'s `compile_text` front-end (minus
/// `nibli_reason::transform_compute_nodes`, which the translator does not need):
/// nibli-kr grammar (parse + fail-closed resolve) → nibli-semantics semantics → the render
/// round-trip gate. Which call fails determines the [`GateError`] variant: a
/// `parse_checked` failure is always a grammar error; a
/// `compile_from_ast` failure is a semantic one.
pub fn local_gates(candidate: &str) -> Result<LogicBuffer, GateError> {
    let ast = nibli_kr::parse_checked(candidate).map_err(syntax)?;
    let buf = nibli_semantics::compile_from_ast(ast.clone()).map_err(semantic)?;
    nibli_kr_round_trip(&ast, &buf)?;
    Ok(buf)
}

/// The render round-trip gate: render the accepted AST back to canonical
/// KR, re-parse and re-compile it, and demand the SAME `LogicBuffer` —
/// nibli-kr's own fixpoint contract (`parse ∘ render ∘ parse` compiles equal for
/// nibli-kr-originated buffers), enforced per candidate as a drift-catcher. Any
/// leg failing is a [`GateError::RoundTrip`] carrying the canonical
/// re-spelling, so the correction turn can steer the model toward it.
fn nibli_kr_round_trip(
    ast: &nibli_types::ast::AstBuffer,
    buf: &LogicBuffer,
) -> Result<(), GateError> {
    let rendered = nibli_kr::render::render(ast).map_err(|e| {
        GateError::RoundTrip(format!(
            "the canonical renderer could not re-spell the statement: {e}"
        ))
    })?;
    let ast2 = nibli_kr::parse_checked(&rendered).map_err(|e| {
        GateError::RoundTrip(format!(
            "the canonical re-spelling {rendered:?} failed to re-parse: {e}"
        ))
    })?;
    let buf2 = nibli_semantics::compile_from_ast(ast2).map_err(|e| {
        GateError::RoundTrip(format!(
            "the canonical re-spelling {rendered:?} failed to re-compile: {e}"
        ))
    })?;
    if buf2 != *buf {
        return Err(GateError::RoundTrip(format!(
            "the statement compiles, but its canonical re-spelling {rendered:?} compiles to \
             different logic — prefer the canonical spelling"
        )));
    }
    Ok(())
}

/// The full assertion-authoring gate the agent calls. Exact-count formulas
/// remain valid compiler/query input, but Formalize produces KB assertions, so
/// it must reject a `CountNode` in asserted position before returning text the
/// engine cannot assert. Opaque quoted content is deliberately ignored.
pub fn validate(candidate: &str) -> Result<LogicBuffer, GateError> {
    let buf = local_gates(candidate)?;
    if nibli_reason::contains_asserted_count_node(&buf) {
        return Err(GateError::Semantic(
            "exact-count formulas (exactly N and no) are query-only and cannot be \
             emitted as knowledge-base assertions. Nibli has no persistent cardinality-constraint \
             assertion. Assert only ordinary facts explicitly supported by the source; otherwise \
             the exact claim is unsupported and must not be omitted, weakened to some, or \
             fabricated around."
                .to_string(),
        ));
    }
    Ok(buf)
}

/// Validate a multi-line KB the way `nibli-ui` uses it: each non-empty,
/// non-comment line must pass the compiler, round-trip, and assertion-intent
/// gates on its own. Returns the first failing
/// line's error, tagged with its KB line number so the LLM can locate it. A
/// single-line candidate is simply validated as one statement, so this also
/// covers the single-sentence case.
pub fn validate_kb(text: &str) -> Result<(), GateError> {
    for (i, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        validate(line).map(|_| ()).map_err(|e| tag_line(e, i + 1))?;
    }
    Ok(())
}

/// Prefix a gate error with its KB line number, preserving the variant.
fn tag_line(e: GateError, line_no: usize) -> GateError {
    let msg = format!("(KB line {line_no}) {}", e.message());
    match e {
        GateError::Syntax(_) => GateError::Syntax(msg),
        GateError::Semantic(_) => GateError::Semantic(msg),
        GateError::RoundTrip(_) => GateError::RoundTrip(msg),
        GateError::Verification(_) => GateError::Verification(msg),
    }
}

fn syntax(e: NibliError) -> GateError {
    GateError::Syntax(e.to_string())
}

fn semantic(e: NibliError) -> GateError {
    GateError::Semantic(e.to_string())
}

/// The correction turn appended to the conversation when a gate rejects a
/// candidate: it names the gate, quotes the compiler's message, and asks for a
/// fix in the KB language only. Kept in one place so the phrasing is
/// consistent across gates.
pub fn feedback_for(err: &GateError) -> String {
    if let GateError::Verification(issues) = err {
        return format!(
            "That is grammatically valid but does not MEAN what the source says. An \
             independent reading of what your nibli KR actually claims reported these \
             mismatches:\n{issues}\nRevise so the meaning matches the source; output ONLY \
             the corrected nibli KR — no explanation."
        );
    }
    let (what, tool) = match err {
        GateError::Syntax(_) => ("is not valid nibli KR", "nibli-kr compiler"),
        GateError::Semantic(_) => (
            "parses but failed semantic/assertability checks (e.g. a predicate got the wrong number of arguments, or a query-only form was used as a KB assertion)",
            "semantic gate",
        ),
        GateError::RoundTrip(_) => (
            "compiles but is not canonical nibli KR (its canonical re-spelling does not compile to the same logic)",
            "round-trip gate",
        ),
        GateError::Verification(_) => unreachable!("handled above"),
    };
    format!(
        "That {what}. The {tool} reported:\n{}\nFix it and output ONLY the corrected nibli KR — no explanation.",
        err.message()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_nibli_kr_passes_local_gates() {
        // Same shape nibli-ui asserts as its default KB; runs all three
        // gates (nibli-kr, nibli-semantics, round-trip).
        local_gates("dog(Adam).").expect("valid nibli KR should pass the three local gates");
    }

    #[test]
    fn nibli_kr_garbage_fails_at_the_grammar_gate() {
        let err = local_gates("dog(Adam") // no close paren / period
            .expect_err("malformed nibli KR must be rejected");
        assert!(
            matches!(err, GateError::Syntax(_)),
            "expected Syntax, got {err:?}"
        );
        assert_eq!(err.gate(), "nibli-kr");
    }

    #[test]
    fn nibli_kr_unknown_alias_fails_closed_at_the_grammar_gate() {
        // Fail-closed name resolution (NIBLI_KR): an alias the dictionary
        // does not know is a COMPILE ERROR, not a silent new predicate.
        let err = local_gates("zzyzxq(Adam).").expect_err("unknown alias must fail closed");
        assert!(
            matches!(err, GateError::Syntax(_)),
            "expected Syntax (resolve), got {err:?}"
        );
    }

    #[test]
    fn nibli_kr_round_trip_gate_holds_on_shipped_shapes() {
        // A cross-section of the determinism corpus' construct classes: the
        // canonical fixpoint must hold for anything the front-end accepts.
        for s in [
            "dog(Adam).",
            "animal(every dog).",
            "beautiful(every person where ~cat).",
            "Kim = Adam.",
            "past dog(Dan).",
            "red(exactly 2 red).",
            "~eats(Adam).",
        ] {
            local_gates(s)
                .unwrap_or_else(|e| panic!("round-trip gate rejected shipped shape {s:?}: {e:?}"));
        }
    }

    #[test]
    fn exact_count_compiles_for_queries_but_is_rejected_as_formalized_kb_output() {
        local_gates("red(exactly 2 red).")
            .expect("the compiler/query surface must retain CountNode");

        for candidate in ["red(exactly 2 red).", "red(no red)."] {
            let err = validate(candidate)
                .expect_err("Formalize must not emit a query-only count as a KB assertion");
            assert!(matches!(err, GateError::Semantic(_)), "{err:?}");
            assert!(err.message().contains("query-only"), "{err:?}");
            assert!(err.message().contains("must not be omitted"), "{err:?}");
        }

        validate("believe(me, fact { red(exactly 2 red) }).")
            .expect("a count in opaque quoted content is not a KB constraint");
    }

    #[test]
    fn feedback_names_the_nibli_kr_gates() {
        let fb = feedback_for(&GateError::Syntax("[Syntax Error] line 1:5: nope".into()));
        assert!(fb.contains("nibli-kr compiler"));
        assert!(fb.contains("line 1:5"));
        assert!(fb.contains("corrected nibli KR"));
        let fb = feedback_for(&GateError::RoundTrip(
            "canonical re-spelling differs".into(),
        ));
        assert!(fb.contains("round-trip gate"));
        assert!(fb.contains("corrected nibli KR"));
        let fb = feedback_for(&GateError::Verification("1. off".into()));
        assert!(fb.contains("your nibli KR"));
        assert!(fb.contains("corrected nibli KR"));
    }

    #[test]
    fn validate_kb_passes_valid_multiline_and_skips_blanks_and_comments() {
        validate_kb("dog(Adam).\n# a note\n\neats(Adam).")
            .expect("every non-comment line is valid nibli KR");
    }

    #[test]
    fn validate_kb_reports_the_failing_line_number() {
        let err = validate_kb("dog(Adam).\ndog(Adam").expect_err("line 2 is malformed nibli KR");
        assert!(err.message().contains("KB line 2"), "got {err:?}");
    }
}
