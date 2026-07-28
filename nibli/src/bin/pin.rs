//! nibli-pin — a native pin runner for KB-level behavioural gates.
//!
//! The problem it solves: some properties a knowledge base relies on are
//! EMERGENT from the engine rather than stated by it. The rights-floor
//! stratification firewall is the motivating case — a KB asserts
//! `entitled(every person, event { eats() }).` and then relies on the engine
//! REFUSING any rule that would punish the absence of that right. That refusal
//! comes out of an asymmetry between `rules::flatten_consequent` (descends an
//! abstraction body, so its predicate becomes a dependency edge) and
//! `rules::collect_ground_facts` (honours `__abs_` opacity, so no actuality
//! leaks). Nothing in the KB's own repo can pin that; a refactor upstream would
//! break the guarantee silently. So the pin suite lives HERE, next to the code
//! whose behaviour it constrains.
//!
//! Native by design: no wasm, no fuel, no component build — it links
//! nibli-engine directly so `just` can run it in seconds.
//!
//! ── FILE FORMAT ───────────────────────────────────────────────────────────
//!
//! A pin file is nibli KR with annotations, following the `determinism-corpus.nibli`
//! house style (`? <query>` + `# => <verdict>`):
//!
//! ```text
//! # comments and blank lines are skipped
//! person(Adam).                       # a plain assertion: must SUCCEED
//!
//! :accept                             # explicit, COUNTED must-succeed pin
//! all $x: person($x) & ~home($x) -> prisoner($x).
//!
//! :refuse reasoning /'prisoner' -> 'eats'/
//! all $x: person($x) & ~eats($x) -> prisoner($x).
//!
//! ? entitled(Adam, event { eats() }).
//! # => TRUE
//!
//! :expect-pins 4                      # anti-hollowing floor (exact count)
//! ```
//!
//! `:refuse <class> /<regex>/` and `:accept` are DUALS and are scoped to the
//! NEXT statement only — never sticky. `<class>` is the `NibliError` variant
//! (`syntax` | `semantic` | `reasoning` | `backend`), matched on the TYPE, not on
//! the message, so a misspelled predicate (a `Syntax` error) can never satisfy a
//! `:refuse reasoning` pin. The regex additionally constrains the message, so a
//! stratification refusal cannot be satisfied by some unrelated reasoning error.
//!
//! Every `?` query MUST carry a `# =>` annotation; an unpinned query is a
//! harness error, not a silent pass. `RESOURCE_EXCEEDED` is deliberately NOT a
//! pinnable verdict: fuel is a host-level budget the native engine does not
//! enforce, so pinning it would let a real trap go green under wasm. If a query
//! returns it, that is reported as infrastructure failure.
//!
//! ── EXIT CODES ────────────────────────────────────────────────────────────
//!
//! * `0` — every pin passed.
//! * `1` — a REAL FINDING: a pin mismatched (wrong verdict, wrong refusal class
//!   or message, unexpected acceptance, unexpected refusal). The property under
//!   test regressed.
//! * `2` — HARNESS OR SCRIPT ERROR: a malformed directive, an unpinned query, an
//!   unpinnable pinned verdict, a resource-exceeded result, an `:expect-pins`
//!   mismatch, or an unreadable file. Nothing was learned about the property.
//!
//! Keeping 1 and 2 distinct is the point: CI must not read "your pin file has a
//! typo" as "the soundness firewall broke", nor the reverse.

use nibli_engine::{EngineError, NibliEngine};
use std::path::Path;
use std::process::ExitCode;

const EXIT_OK: u8 = 0;
const EXIT_FINDING: u8 = 1;
const EXIT_HARNESS: u8 = 2;

/// What the next statement is expected to do. Set by `:accept` / `:refuse`,
/// consumed by the next statement — deliberately not sticky.
enum Expect {
    /// No directive seen: a statement must still succeed, but it is not a
    /// counted pin (the file is mostly ordinary KB text).
    Default,
    /// `:accept` — must succeed, and COUNTS as a pin.
    Accept,
    /// `:refuse <class> /<regex>/` — must fail with this error class, and the
    /// Display message must contain the (literal, un-anchored) needle.
    Refuse { class: Class, needle: String },
}

#[derive(PartialEq, Clone, Copy, Debug)]
enum Class {
    Syntax,
    Semantic,
    Reasoning,
    Backend,
}

impl Class {
    fn parse(s: &str) -> Option<Self> {
        match s {
            "syntax" => Some(Self::Syntax),
            "semantic" => Some(Self::Semantic),
            "reasoning" => Some(Self::Reasoning),
            "backend" => Some(Self::Backend),
            _ => None,
        }
    }

    /// The CLASS of an actual error — matched on the variant, never the string.
    fn of(e: &EngineError) -> Self {
        match e {
            EngineError::Syntax(_) => Self::Syntax,
            EngineError::Semantic(_) => Self::Semantic,
            EngineError::Reasoning(_) => Self::Reasoning,
            EngineError::Backend(_) => Self::Backend,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Syntax => "syntax",
            Self::Semantic => "semantic",
            Self::Reasoning => "reasoning",
            Self::Backend => "backend",
        }
    }
}

/// Verdicts a query may be pinned to. `RESOURCE_EXCEEDED` is intentionally
/// absent — see the module docs.
fn is_pinnable_verdict(v: &str) -> bool {
    // `display_query_result` renders "UNKNOWN (<reason>)" for the Unknown arm, so
    // accept an UNKNOWN prefix but never a resource-exhaustion label.
    v == "TRUE" || v == "FALSE" || v == "UNKNOWN" || v.starts_with("UNKNOWN (")
}

#[derive(Debug)]
struct Report {
    pins: usize,
    findings: Vec<String>,
    harness: Vec<String>,
}

fn main() -> ExitCode {
    let paths: Vec<String> = std::env::args().skip(1).collect();
    if paths.is_empty() {
        eprintln!("usage: nibli-pin <file.nibli> [more.nibli ...]");
        eprintln!("  exit 0 = pins pass, 1 = a pin regressed, 2 = harness/script error");
        return ExitCode::from(EXIT_HARNESS);
    }

    let mut total = Report {
        pins: 0,
        findings: Vec::new(),
        harness: Vec::new(),
    };

    for path in &paths {
        match std::fs::read_to_string(path) {
            Ok(src) => {
                let name = Path::new(path)
                    .file_name()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_else(|| path.clone());
                let r = run_file(&name, &src);
                println!(
                    "  {name}: {} pins, {} findings, {} harness errors",
                    r.pins,
                    r.findings.len(),
                    r.harness.len()
                );
                total.pins += r.pins;
                total.findings.extend(r.findings);
                total.harness.extend(r.harness);
            }
            Err(e) => total.harness.push(format!("{path}: unreadable ({e})")),
        }
    }

    if !total.harness.is_empty() {
        eprintln!("\nHARNESS/SCRIPT ERRORS ({}):", total.harness.len());
        for h in &total.harness {
            eprintln!("  ! {h}");
        }
    }
    if !total.findings.is_empty() {
        eprintln!(
            "\nFINDINGS ({}) — a pinned property regressed:",
            total.findings.len()
        );
        for f in &total.findings {
            eprintln!("  ✗ {f}");
        }
    }

    // Harness errors dominate: if the script is malformed we did not actually
    // test the property, so reporting a "finding" would be a lie either way.
    if !total.harness.is_empty() {
        eprintln!("\nnibli-pin: HARNESS ERROR (exit {EXIT_HARNESS}) — pins not trustworthy");
        return ExitCode::from(EXIT_HARNESS);
    }
    if !total.findings.is_empty() {
        eprintln!(
            "\nnibli-pin: {} FINDING(S) (exit {EXIT_FINDING})",
            total.findings.len()
        );
        return ExitCode::from(EXIT_FINDING);
    }
    println!("nibli-pin: PASS — {} pins", total.pins);
    ExitCode::from(EXIT_OK)
}

/// Parse `:refuse <class> /<needle>/` into an `Expect`, or an error string.
fn parse_refuse(rest: &str) -> Result<Expect, String> {
    let rest = rest.trim();
    let (class_tok, pattern) = rest
        .split_once(char::is_whitespace)
        .ok_or_else(|| format!(":refuse needs a class and /pattern/ (got {rest:?})"))?;
    let class = Class::parse(class_tok.trim()).ok_or_else(|| {
        format!(
            ":refuse unknown class {:?} (want syntax|semantic|reasoning|backend)",
            class_tok.trim()
        )
    })?;
    let pattern = pattern.trim();
    let needle = pattern
        .strip_prefix('/')
        .and_then(|p| p.strip_suffix('/'))
        .ok_or_else(|| format!(":refuse pattern must be /slash-delimited/ (got {pattern:?})"))?;
    if needle.is_empty() {
        return Err(":refuse pattern must not be empty".to_string());
    }
    Ok(Expect::Refuse {
        class,
        needle: needle.to_string(),
    })
}

fn run_file(name: &str, src: &str) -> Report {
    let mut rep = Report {
        pins: 0,
        findings: Vec::new(),
        harness: Vec::new(),
    };
    let engine = NibliEngine::new();
    let mut expect = Expect::Default;
    let mut expect_pins: Option<usize> = None;

    // Queries are `? <text>` and must be followed by a `# => <verdict>` line, so
    // walk with an index rather than a plain for-loop.
    let lines: Vec<&str> = src.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let raw = lines[i];
        let line = raw.trim();
        i += 1;

        if line.is_empty() {
            continue;
        }
        // A `# =>` reaching here is an annotation with no query above it.
        if let Some(v) = line.strip_prefix("# =>") {
            rep.harness.push(format!(
                "{name}:{}: stray `# =>{}` annotation — no `?` query precedes it",
                i,
                v.trim_end()
            ));
            continue;
        }
        if line.starts_with('#') {
            continue;
        }

        // ── directives ──
        if let Some(rest) = line.strip_prefix(":refuse") {
            if !matches!(expect, Expect::Default) {
                rep.harness.push(format!(
                    "{name}:{i}: directive follows an unconsumed directive — each applies to the NEXT statement only"
                ));
            }
            match parse_refuse(rest) {
                Ok(e) => expect = e,
                Err(msg) => rep.harness.push(format!("{name}:{i}: {msg}")),
            }
            continue;
        }
        if line == ":accept" {
            if !matches!(expect, Expect::Default) {
                rep.harness.push(format!(
                    "{name}:{i}: directive follows an unconsumed directive — each applies to the NEXT statement only"
                ));
            }
            expect = Expect::Accept;
            continue;
        }
        if let Some(rest) = line.strip_prefix(":expect-pins") {
            match rest.trim().parse::<usize>() {
                Ok(n) => expect_pins = Some(n),
                Err(_) => rep.harness.push(format!(
                    "{name}:{i}: :expect-pins needs a number (got {rest:?})"
                )),
            }
            continue;
        }
        if line.starts_with(':') {
            rep.harness
                .push(format!("{name}:{i}: unknown directive {line:?}"));
            continue;
        }

        // ── query: `? <text>` + mandatory `# => <verdict>` ──
        if let Some(qtext) = line.strip_prefix('?') {
            if !matches!(expect, Expect::Default) {
                rep.harness.push(format!(
                    "{name}:{i}: :accept/:refuse applies to an ASSERTION, not a `?` query"
                ));
                expect = Expect::Default;
            }
            let qtext = qtext.trim();
            // The annotation must be the very next non-blank line.
            let mut j = i;
            while j < lines.len() && lines[j].trim().is_empty() {
                j += 1;
            }
            let Some(pinned) = lines
                .get(j)
                .map(|l| l.trim())
                .and_then(|l| l.strip_prefix("# =>"))
                .map(|v| v.trim().to_string())
            else {
                rep.harness.push(format!(
                    "{name}:{i}: query {qtext:?} has no `# => <verdict>` annotation (pins are mandatory)"
                ));
                continue;
            };
            i = j + 1;

            if !is_pinnable_verdict(&pinned) {
                rep.harness.push(format!(
                    "{name}:{i}: {pinned:?} is not a pinnable verdict \
                     (TRUE|FALSE|UNKNOWN only — RESOURCE_EXCEEDED is a resource outcome, \
                     not a logical one, and is runtime-dependent by design)"
                ));
                continue;
            }

            rep.pins += 1;
            match engine.query_holds(qtext) {
                Err(e) => {
                    // A query that will not COMPILE has no verdict at all — it must
                    // never be compared against a verdict string.
                    rep.harness.push(format!(
                        "{name}:{i}: query {qtext:?} failed to compile, so it has no verdict to \
                         compare with the pinned {pinned:?} — [{}] {e}",
                        Class::of(&e).name()
                    ));
                }
                Ok(result) => {
                    let actual = nibli_engine::display_query_result(&result);
                    if actual.starts_with("RESOURCE_EXCEEDED") {
                        rep.harness.push(format!(
                            "{name}:{i}: query {qtext:?} exhausted a resource ({actual}) — \
                             infrastructure outcome, not a verdict; pin excluded"
                        ));
                    } else if !verdict_matches(&pinned, &actual) {
                        rep.findings.push(format!(
                            "{name}:{i}: query {qtext:?} pinned {pinned:?} but got {actual:?}"
                        ));
                    }
                }
            }
            continue;
        }

        // ── statement (assertion / rule) ──
        let outcome = engine.assert_text(line);
        let taken = std::mem::replace(&mut expect, Expect::Default);
        match (taken, outcome) {
            (Expect::Default, Ok(_)) => {}
            (Expect::Default, Err(e)) => {
                rep.findings.push(format!(
                    "{name}:{i}: {line:?} failed to load — [{}] {e}",
                    Class::of(&e).name()
                ));
            }
            (Expect::Accept, Ok(_)) => rep.pins += 1,
            (Expect::Accept, Err(e)) => {
                rep.pins += 1;
                rep.findings.push(format!(
                    "{name}:{i}: :accept but {line:?} was REFUSED — [{}] {e}",
                    Class::of(&e).name()
                ));
            }
            (Expect::Refuse { class, needle }, Ok(_)) => {
                rep.pins += 1;
                rep.findings.push(format!(
                    "{name}:{i}: :refuse {} /{needle}/ but {line:?} was ACCEPTED \
                     — the guarantee this pin protects is GONE",
                    class.name()
                ));
            }
            (Expect::Refuse { class, needle }, Err(e)) => {
                rep.pins += 1;
                let got = Class::of(&e);
                let msg = e.to_string();
                if got != class {
                    rep.findings.push(format!(
                        "{name}:{i}: :refuse {} but {line:?} failed as [{}] instead — {msg} \
                         (a different error class is NOT the property under test)",
                        class.name(),
                        got.name()
                    ));
                } else if !msg.contains(&needle) {
                    rep.findings.push(format!(
                        "{name}:{i}: :refuse {} /{needle}/ matched the class but not the message — got {msg}",
                        class.name()
                    ));
                }
            }
        }
    }

    if !matches!(expect, Expect::Default) {
        rep.harness.push(format!(
            "{name}: file ends with an unconsumed :accept/:refuse directive"
        ));
    }
    match expect_pins {
        None => rep.harness.push(format!(
            "{name}: no `:expect-pins <n>` floor — without it a hollowed-out file passes vacuously"
        )),
        Some(n) if n != rep.pins => rep.harness.push(format!(
            "{name}: :expect-pins {n} but {} pins ran — pins were added or lost; \
             adjust the floor consciously in the same diff",
            rep.pins
        )),
        Some(_) => {}
    }
    rep
}

/// A bare `UNKNOWN` pin matches any `UNKNOWN (reason)`; everything else is exact.
fn verdict_matches(pinned: &str, actual: &str) -> bool {
    if pinned == "UNKNOWN" {
        return actual == "UNKNOWN" || actual.starts_with("UNKNOWN (");
    }
    pinned == actual
}

// ═══════════════════════════════════════════════════════════════════════
// Harness self-tests
//
// A pin runner that cannot FAIL is worse than no runner, because it reads as a
// guarantee. Each test below pins one defect found in the throwaway predecessor
// (the "51 pins in seconds" runner): every one of these was a way for a broken
// KB — or a broken engine — to go green.
// ═══════════════════════════════════════════════════════════════════════
#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal preamble: a floor plus the closure link, so firewall refusals are
    /// reachable. Ends WITHOUT a directive.
    const FLOOR: &str = "entitled(every person, event { eats() }).\n\
                         all $anyone: prisoner($anyone) -> person($anyone).\n";

    fn run(src: &str) -> Report {
        run_file("t", src)
    }

    /// Baseline: a well-formed file with one refusal pin and one accept pin is
    /// clean, and both count.
    #[test]
    fn well_formed_file_passes_and_counts_pins() {
        let r = run(&format!(
            "{FLOOR}\
             :refuse reasoning /'prisoner' -> 'eats'/\n\
             all $x: person($x) & ~eats($x) -> prisoner($x).\n\
             :accept\n\
             all $x: person($x) & ~home($x) -> prisoner($x).\n\
             :expect-pins 2\n"
        ));
        assert_eq!(r.findings.len(), 0, "{:?}", r.findings);
        assert_eq!(r.harness.len(), 0, "{:?}", r.harness);
        assert_eq!(r.pins, 2);
    }

    /// DEFECT 1: the predecessor could not tell a stratification refusal from a
    /// syntax typo, so a misspelled predicate PASSED as a firewall test. The
    /// class is matched on the `NibliError` variant, so a typo is a `syntax`
    /// error and cannot satisfy `:refuse reasoning`.
    #[test]
    fn a_typo_cannot_masquerade_as_a_firewall_refusal() {
        let r = run(&format!(
            "{FLOOR}\
             :refuse reasoning /Unstratifiable negation/\n\
             all $x: persson($x) & ~eats($x) -> prisoner($x).\n\
             :expect-pins 1\n"
        ));
        assert_eq!(r.findings.len(), 1, "a typo must be a FINDING: {r:?}");
        assert!(
            r.findings[0].contains("failed as [syntax]"),
            "the finding must name the wrong class: {}",
            r.findings[0]
        );
    }

    /// Same class, wrong reason: a reasoning error that is not the pinned one
    /// must not satisfy the pin either.
    #[test]
    fn right_class_wrong_message_is_a_finding() {
        let r = run(&format!(
            "{FLOOR}\
             :refuse reasoning /this text appears in no diagnostic/\n\
             all $x: person($x) & ~eats($x) -> prisoner($x).\n\
             :expect-pins 1\n"
        ));
        assert_eq!(r.findings.len(), 1, "{r:?}");
        assert!(
            r.findings[0].contains("matched the class but not the message"),
            "{}",
            r.findings[0]
        );
    }

    /// THE CORE REGRESSION: if the firewall breaks, the punishing rule starts
    /// LOADING and the refusal pin must fail loudly. Simulated by omitting the
    /// floor, which is exactly the state a broken opacity asymmetry would leave.
    #[test]
    fn firewall_gone_is_a_finding_not_a_pass() {
        let r = run("all $anyone: prisoner($anyone) -> person($anyone).\n\
             :refuse reasoning /'prisoner' -> 'eats'/\n\
             all $x: person($x) & ~eats($x) -> prisoner($x).\n\
             :expect-pins 1\n");
        assert_eq!(r.findings.len(), 1, "{r:?}");
        assert!(
            r.findings[0].contains("was ACCEPTED") && r.findings[0].contains("GONE"),
            "{}",
            r.findings[0]
        );
    }

    /// DEFECT 2: no positive-load expectation existed, so a control could only be
    /// run by inverting the exit code. `:accept` is a first-class counted pin, and
    /// an over-broad protected set trips it.
    #[test]
    fn accept_catches_an_over_broad_protected_set() {
        // `~eats` IS inside the cone, so pinning it as a control must FAIL —
        // this is the shape that catches a protected set that silently widened.
        let r = run(&format!(
            "{FLOOR}\
             :accept\n\
             all $x: person($x) & ~eats($x) -> prisoner($x).\n\
             :expect-pins 1\n"
        ));
        assert_eq!(r.findings.len(), 1, "{r:?}");
        assert!(r.findings[0].contains("was REFUSED"), "{}", r.findings[0]);
    }

    /// DEFECT 3: the expectation flag was global and STICKY. A directive applies
    /// to the next statement only; the statement after it is judged as default.
    #[test]
    fn directives_are_scoped_to_the_next_statement_only() {
        // The refusal is consumed by the first rule. The SAME rule repeated is
        // refused again (the first attempt rolled back), but with the directive
        // spent it is now a default must-succeed — so it must surface as a FINDING
        // rather than silently reusing the pin. Repeating the identical rule is
        // deliberate: a rule that merely *happens* not to be in the protected set
        // would load for reasons unrelated to directive scoping and prove nothing.
        let r = run(&format!(
            "{FLOOR}\
             :refuse reasoning /'prisoner' -> 'eats'/\n\
             all $x: person($x) & ~eats($x) -> prisoner($x).\n\
             all $x: person($x) & ~eats($x) -> prisoner($x).\n\
             :expect-pins 1\n"
        ));
        assert_eq!(r.pins, 1, "only the directed statement is a pin: {r:?}");
        assert_eq!(r.findings.len(), 1, "{r:?}");
        assert!(
            r.findings[0].contains("failed to load"),
            "the undirected statement must be judged as must-succeed: {}",
            r.findings[0]
        );
    }

    /// Two directives in a row means one was never consumed — a script bug, and
    /// the kind that silently disarms a pin.
    #[test]
    fn stacked_directives_are_a_harness_error() {
        let r = run(&format!(
            "{FLOOR}:accept\n:accept\nperson(Bet).\n:expect-pins 1\n"
        ));
        assert!(
            r.harness.iter().any(|h| h.contains("unconsumed directive")),
            "{r:?}"
        );
    }

    #[test]
    fn trailing_directive_at_eof_is_a_harness_error() {
        let r = run(&format!("{FLOOR}:expect-pins 0\n:accept\n"));
        assert!(
            r.harness
                .iter()
                .any(|h| h.contains("unconsumed :accept/:refuse")),
            "{r:?}"
        );
    }

    /// DEFECT 4: unpinned queries passed silently.
    #[test]
    fn an_unannotated_query_is_a_harness_error() {
        let r = run(&format!("{FLOOR}? eats(Adam).\n:expect-pins 0\n"));
        assert!(
            r.harness.iter().any(|h| h.contains("no `# => <verdict>`")),
            "{r:?}"
        );
        assert_eq!(r.pins, 0, "an unpinned query must not count as a pin");
    }

    /// DEFECT 5: a query that failed to COMPILE was rendered as a comparable
    /// verdict string, so a broken query could match a pin.
    #[test]
    fn an_uncompilable_query_is_a_harness_error_not_a_verdict() {
        let r = run(&format!(
            "{FLOOR}? zzznotaword(Adam).\n# => FALSE\n:expect-pins 1\n"
        ));
        assert_eq!(
            r.findings.len(),
            0,
            "must NOT be reported as a verdict mismatch: {r:?}"
        );
        assert!(
            r.harness
                .iter()
                .any(|h| h.contains("failed to compile") && h.contains("no verdict")),
            "{r:?}"
        );
    }

    /// DEFECT 6: `RESOURCE_EXCEEDED` was pinnable as an ordinary verdict, which
    /// under wasm would let a fuel trap go green. It is structurally unpinnable.
    #[test]
    fn resource_exceeded_is_not_a_pinnable_verdict() {
        let r = run(&format!(
            "{FLOOR}? eats(Adam).\n# => RESOURCE_EXCEEDED\n:expect-pins 1\n"
        ));
        assert!(
            r.harness
                .iter()
                .any(|h| h.contains("not a pinnable verdict")),
            "{r:?}"
        );
        assert!(!is_pinnable_verdict("RESOURCE_EXCEEDED"));
        assert!(is_pinnable_verdict("TRUE"));
        assert!(is_pinnable_verdict("UNKNOWN (cycle-cut)"));
    }

    /// A wrong verdict is a FINDING (the property changed), not a harness error.
    #[test]
    fn a_wrong_verdict_is_a_finding() {
        let r = run(&format!(
            "{FLOOR}? eats(Adam).\n# => TRUE\n:expect-pins 1\n"
        ));
        assert_eq!(r.harness.len(), 0, "{r:?}");
        assert_eq!(r.findings.len(), 1, "{r:?}");
        assert!(
            r.findings[0].contains("pinned \"TRUE\""),
            "{}",
            r.findings[0]
        );
    }

    /// DEFECT 7: no anti-hollowing floor. A file whose pins were deleted must not
    /// pass vacuously, and a missing floor is itself an error.
    #[test]
    fn expect_pins_floor_catches_hollowing_and_is_mandatory() {
        let drifted = run(&format!("{FLOOR}:accept\nperson(Bet).\n:expect-pins 7\n"));
        assert!(
            drifted
                .harness
                .iter()
                .any(|h| h.contains(":expect-pins 7 but 1 pins ran")),
            "{drifted:?}"
        );
        let missing = run(&format!("{FLOOR}:accept\nperson(Bet).\n"));
        assert!(
            missing
                .harness
                .iter()
                .any(|h| h.contains("no `:expect-pins")),
            "{missing:?}"
        );
    }

    /// Malformed directives fail closed rather than being ignored.
    #[test]
    fn malformed_directives_are_harness_errors() {
        for (src, needle) in [
            (":refuse\nperson(Bet).\n", "needs a class"),
            (":refuse nonsense /x/\nperson(Bet).\n", "unknown class"),
            (":refuse reasoning bare\nperson(Bet).\n", "slash-delimited"),
            (":refuse reasoning //\nperson(Bet).\n", "must not be empty"),
            (":expect-pins lots\n", "needs a number"),
            (":wat\n", "unknown directive"),
            ("# => TRUE\n", "stray"),
        ] {
            let r = run(&format!("{FLOOR}{src}"));
            assert!(
                r.harness.iter().any(|h| h.contains(needle)),
                "expected {needle:?} for {src:?}, got {:?}",
                r.harness
            );
        }
    }

    /// An `:accept`/`:refuse` aimed at a `?` query is a category error.
    #[test]
    fn a_directive_on_a_query_is_a_harness_error() {
        let r = run(&format!(
            "{FLOOR}:accept\n? eats(Adam).\n# => FALSE\n:expect-pins 1\n"
        ));
        assert!(
            r.harness
                .iter()
                .any(|h| h.contains("applies to an ASSERTION")),
            "{r:?}"
        );
    }

    /// The shipped pin file must actually pass — otherwise `just verify-pins` is
    /// red on a fresh clone and nobody trusts it.
    #[test]
    fn shipped_rights_floor_pins_pass() {
        let src = include_str!("../../../pins/rights-floor.nibli");
        let r = run_file("rights-floor.nibli", src);
        assert_eq!(r.findings.len(), 0, "{:?}", r.findings);
        assert_eq!(r.harness.len(), 0, "{:?}", r.harness);
        assert!(r.pins >= 12, "pin coverage collapsed: {} pins", r.pins);
    }
}
