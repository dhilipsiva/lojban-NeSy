//! Shipped-artifact parity: `grammars/nibli.tmLanguage.json`'s keyword
//! alternation is a MIRROR of `nibli_lexicon::RESERVED_WORDS`, and
//! `grammars/README.md` says so in prose — "Keywords must stay equal to
//! `nibli_lexicon::RESERVED_WORDS` (and the pest `kw_*` rules)". The pest
//! mirror has been pinned all along by nibli-kr's parser conformance test;
//! the TextMate mirror was pinned by nothing, and nothing in the repo
//! referenced `grammars/` at all — no recipe, no CI job, no test.
//!
//! Dependency-free on purpose (nibli-verify carries no serde_json). Every
//! structural assumption below is its own assert with its own message, so a
//! reformatted grammar fails as "the extractor no longer finds X" and never as
//! a silently empty comparison.

/// nibli-verify is `publish = false`, so `include_str!` may reach the repo
/// root (cf. `include_str!("../../determinism-corpus.nibli")` in
/// nibli_kr_seam_gate.rs). `include_str!` rather than a runtime read so a
/// moved or renamed grammar is a COMPILE error, not a silently-skipped gate.
const TMLANGUAGE: &str = include_str!("../../grammars/nibli.tmLanguage.json");

#[test]
fn tmlanguage_keywords_match_reserved_words() {
    let raw = keyword_match_pattern(TMLANGUAGE);

    // The \b anchors are load-bearing, not cosmetic: unanchored, TextMate
    // would paint the `we` inside `wealth` and the `the` inside `theory`.
    // NOTE the doubling — JSON escapes the regex backslash, so the FILE holds
    // `\\b`.
    let inner = raw
        .strip_prefix(r#"\\b("#)
        .and_then(|s| s.strip_suffix(r#")\\b"#))
        .unwrap_or_else(|| {
            panic!(
                "the tmLanguage `keyword` match must stay a word-boundary-anchored \
                 alternation of the form \\\\b(a|b|…)\\\\b — found {raw:?}. Without \
                 the \\b anchors the editor paints the `we` inside `wealth`."
            )
        });

    let file_order: Vec<&str> = inner.split('|').collect();

    // Splitting on '|' is only sound if every alternate is a bare identifier
    // (no groups, no escapes, no character classes). Assert it, and get the
    // RESERVED_WORDS shape invariant re-checked on the mirror for free.
    for w in &file_order {
        assert!(
            !w.is_empty()
                && w.starts_with(|c: char| c.is_ascii_lowercase())
                && w.chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'),
            "alternate {w:?} is not a bare `[a-z][a-z0-9_]*` identifier — the \
             `keyword` match must stay a plain alternation or splitting on `|` \
             is the wrong extraction"
        );
    }

    let mut spellings = file_order.clone();
    spellings.sort_unstable();

    let reserved = nibli_lexicon::reserved::RESERVED_WORDS;
    assert_eq!(
        spellings, reserved,
        "grammars/nibli.tmLanguage.json `keyword` match and RESERVED_WORDS diverge \
         (grammars/README.md: \"Keywords must stay equal to \
         nibli_lexicon::RESERVED_WORDS (and the pest kw_* rules)\"). Edit the \
         alternation in the SAME commit as nibli-lexicon/src/reserved.rs. The pest \
         twin is pinned separately by nibli-kr's parser conformance test — expect \
         that one to fire too."
    );
    assert_eq!(
        file_order, reserved,
        "the tmLanguage alternation holds the right SET in the wrong ORDER — keep \
         it in RESERVED_WORDS order so the two files diff side by side. (\\b \
         anchoring makes order semantically irrelevant to TextMate; this is a \
         reviewability rule.)"
    );
}

/// The raw, still-JSON-escaped `"match"` string of the `keyword` repository
/// entry. Whitespace-exact by design: a reformat breaks it LOUDLY here rather
/// than degrading the comparison.
fn keyword_match_pattern(src: &str) -> &str {
    let entry = src
        .find(r#""keyword": {"#)
        .expect("tmLanguage `repository` must still contain a `keyword` entry");
    let rest = &src[entry..];
    let m = rest
        .find(r#""match":"#)
        .expect("the `keyword` entry must still carry a `match` regex");
    let after = &rest[m + r#""match":"#.len()..];
    let start = after.find('"').expect("`match` must be a JSON string") + 1;
    let body = &after[start..];

    let bytes = body.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => i += 2,
            b'"' => return &body[..i],
            _ => i += 1,
        }
    }
    panic!("unterminated `match` string in the tmLanguage `keyword` entry");
}
