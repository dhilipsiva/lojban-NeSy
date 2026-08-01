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
//! ── TWO KINDS OF PIN ──────────────────────────────────────────────────────
//!
//! **Mechanism pins** guard the engine. Their fixture is INLINE in the pin file
//! and must stay that way: the fixture is not the subject, so an edit elsewhere
//! weakening it would silently weaken a guarantee about nibli. Everything under
//! `pins/` is this kind.
//!
//! **Content pins** guard a specific artifact's behaviour — a constitution, a
//! policy, a corpus. There the artifact IS the subject, so the fixture must be
//! the LIVE file, loaded with `--kb`:
//!
//! ```text
//! nibli-pin --kb path/to/constitution.nibli path/to/constitution.pins.nibli
//! ```
//!
//! Inlining a content fixture is the failure mode, not the protection: the copy
//! drifts and the pins quietly start certifying fiction. `--kb` is repeatable
//! (fixtures load in order) and each pin file gets a fresh engine, so one file
//! cannot leak state into the next. The same pin file can therefore be run
//! against several versions of an artifact.
//!
//! Composition lives at the CLI rather than in a `:load` directive on purpose:
//! it keeps the pin language closed, and it means nothing in `pins/*.nibli` can
//! reference a path outside the repo and break `just ci`.
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

/// A pre-loaded fixture KB: `(display name, source)`.
type KbFile = (String, String);

fn base_name(path: &str) -> String {
    Path::new(path)
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string())
}

fn main() -> ExitCode {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let mut kb_paths: Vec<String> = Vec::new();
    let mut paths: Vec<String> = Vec::new();
    let mut argv_error: Option<String> = None;
    let mut strata_only = false;

    let mut it = argv.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--kb" => match it.next() {
                Some(p) => kb_paths.push(p.clone()),
                None => argv_error = Some("--kb needs a path".to_string()),
            },
            "--strata" => strata_only = true,
            other if other.starts_with("--") => {
                argv_error = Some(format!("unknown flag {other:?}"));
            }
            other => paths.push(other.to_string()),
        }
    }

    // `--strata` DUMPS a KB rather than checking pins, so it needs `--kb` and takes no pin
    // file. Everything else still requires one.
    let missing_input = if strata_only {
        kb_paths.is_empty()
    } else {
        paths.is_empty()
    };
    if missing_input || argv_error.is_some() {
        if let Some(e) = argv_error {
            eprintln!("nibli-pin: {e}");
        } else if strata_only {
            eprintln!("nibli-pin: --strata needs at least one --kb <file.nibli>");
        }
        eprintln!("usage: nibli-pin [--kb <fixture.nibli>]... <pins.nibli> [more.nibli ...]");
        eprintln!("       nibli-pin --strata --kb <file.nibli>...");
        eprintln!("  --kb      load a fixture KB into a fresh engine before EACH pin file runs.");
        eprintln!("            Repeatable; loaded in the order given. Use it for CONTENT pins,");
        eprintln!("            which test a specific artifact and must read the live artifact");
        eprintln!("            rather than an inlined copy that can drift.");
        eprintln!("  --strata  load the --kb files and print the engine's STRATIFICATION to");
        eprintln!("            stdout as stable, sorted, diffable TSV, then exit. Runs no pins.");
        eprintln!("  exit 0 = pins pass, 1 = a pin regressed, 2 = harness/script error");
        return ExitCode::from(EXIT_HARNESS);
    }

    let mut total = Report {
        pins: 0,
        findings: Vec::new(),
        harness: Vec::new(),
    };

    // Read every fixture up front: an unreadable fixture means NO pin can run,
    // so there is nothing to report but a harness error.
    let mut kbs: Vec<KbFile> = Vec::new();
    for p in &kb_paths {
        match std::fs::read_to_string(p) {
            Ok(src) => kbs.push((base_name(p), src)),
            Err(e) => total
                .harness
                .push(format!("--kb {p}: unreadable ({e}) — no pins could run")),
        }
    }
    if !total.harness.is_empty() {
        eprintln!("\nHARNESS/SCRIPT ERRORS ({}):", total.harness.len());
        for h in &total.harness {
            eprintln!("  ! {h}");
        }
        eprintln!("\nnibli-pin: HARNESS ERROR (exit {EXIT_HARNESS}) — pins not trustworthy");
        return ExitCode::from(EXIT_HARNESS);
    }

    if strata_only {
        let (out, harness) = strata_dump(&kbs);
        if !harness.is_empty() {
            eprintln!("\nHARNESS/SCRIPT ERRORS ({}):", harness.len());
            for h in &harness {
                eprintln!("  ! {h}");
            }
            eprintln!("\nnibli-pin: HARNESS ERROR (exit {EXIT_HARNESS}) — dump not trustworthy");
            return ExitCode::from(EXIT_HARNESS);
        }
        print!("{out}");
        return ExitCode::from(EXIT_OK);
    }

    for path in &paths {
        match std::fs::read_to_string(path) {
            Ok(src) => {
                let name = base_name(path);
                let r = run_file_with_kb(&name, &src, &kbs);
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

/// Run a pin file against a fresh engine with no fixture — the MECHANISM-pin
/// shape, where the fixture is inline in the pin file itself.
#[cfg(test)]
fn run_file(name: &str, src: &str) -> Report {
    run_file_with_kb(name, src, &[])
}

/// Load every fixture KB into a fresh engine, then run the pin file against it.
///
/// A fixture line that fails to assert is a HARNESS error (exit 2), never a
/// finding (exit 1). The distinction is not a judgement call: if the fixture did
/// not load, no pin ran, so there is no pinned property that could have
/// regressed — reporting a finding would claim knowledge the run does not have.
/// "I could not run the test" and "the test failed" are different in kind, and
/// the exit taxonomy exists to keep them apart.
/// Load the `--kb` fixtures into a fresh engine and render the engine's stratification.
///
/// Returns `(stdout_text, harness_errors)`. A fixture that fails to load yields harness
/// errors and NO dump: numbers computed over a partly-loaded KB would be worse than no
/// numbers at all, because they look like numbers.
///
/// Format is TSV so it diffs line-by-line and parses with `split('\t')`; the header is
/// comment lines so a consumer can `startswith('#')`-skip it. Every ordering is fixed by
/// [`nibli_reason::KnowledgeBase::stratification_report`], so two runs over the same
/// input are byte-identical.
fn strata_dump(kbs: &[KbFile]) -> (String, Vec<String>) {
    use std::fmt::Write as _;

    let mut harness: Vec<String> = Vec::new();
    let engine = NibliEngine::new();
    for (kb_name, kb_src) in kbs {
        for (n, raw) in kb_src.lines().enumerate() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if line.starts_with(':') || line.starts_with('?') {
                harness.push(format!(
                    "{kb_name}:{}: a --kb fixture is plain KB text — directives and `?` \
                     queries belong in the pin file, not the artifact under test",
                    n + 1
                ));
                continue;
            }
            if let Err(e) = engine.assert_text(line) {
                harness.push(format!(
                    "{kb_name}:{}: fixture line failed to load — [{}] {e}",
                    n + 1,
                    Class::of(&e).name()
                ));
            }
        }
    }
    if !harness.is_empty() {
        return (String::new(), harness);
    }

    let rows = engine.kb().stratification_report();
    let max_stratum = rows.iter().map(|r| r.stratum).max().unwrap_or(0);
    let base = rows.iter().filter(|r| r.base).count();

    let mut out = String::new();
    out.push_str("# nibli-strata v1\n");
    out.push_str("# Produced by `nibli-pin --strata` from the engine's own dependency graph\n");
    out.push_str("# (`pred_dep_graph`), the same one `check_stratification` gates rules on.\n");
    out.push_str("# columns: predicate <TAB> stratum <TAB> base|derived <TAB> edges\n");
    out.push_str("# edges:   comma-separated, `+name` positive, `-name` negative (NAF);\n");
    out.push_str("#          empty field = no outgoing edges. An edge means \"reads\".\n");
    out.push_str("# names:   SURFACE relations — event-decomposed role predicates (`p_x1`)\n");
    out.push_str("#          are collapsed onto their anchor (`p`). `event` and `__abs_<hash>`\n");
    out.push_str("#          are compiler artifacts of `event { }` abstractions, and `equals`\n");
    out.push_str("#          is the `=` identity builtin — a disequality guard `~($a = $b)`\n");
    out.push_str("#          is a NEGATIVE edge to it and does raise the reader's stratum.\n");
    out.push_str("#          None of them are authored predicates; all are listed, not hidden,\n");
    out.push_str("#          because a dump that silently drops nodes is how re-derivations\n");
    out.push_str("#          come to disagree with the engine in the first place.\n");
    out.push_str("# order:   rows by predicate, edges by target — stable across runs.\n");
    let _ = writeln!(
        out,
        "# totals:  {} predicates, strata 0..{max_stratum}, {base} base, {} derived",
        rows.len(),
        rows.len() - base
    );
    for r in &rows {
        let edges: Vec<String> = r
            .edges
            .iter()
            .map(|e| format!("{}{}", if e.negative { '-' } else { '+' }, e.to))
            .collect();
        let _ = writeln!(
            out,
            "{}\t{}\t{}\t{}",
            r.predicate,
            r.stratum,
            if r.base { "base" } else { "derived" },
            edges.join(",")
        );
    }
    (out, harness)
}

fn run_file_with_kb(name: &str, src: &str, kbs: &[KbFile]) -> Report {
    let mut rep = Report {
        pins: 0,
        findings: Vec::new(),
        harness: Vec::new(),
    };
    let engine = NibliEngine::new();

    for (kb_name, kb_src) in kbs {
        for (n, raw) in kb_src.lines().enumerate() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if line.starts_with(':') || line.starts_with('?') {
                rep.harness.push(format!(
                    "{kb_name}:{}: a --kb fixture is plain KB text — directives and `?` queries \
                     belong in the pin file, not the artifact under test",
                    n + 1
                ));
                continue;
            }
            if let Err(e) = engine.assert_text(line) {
                rep.harness.push(format!(
                    "{kb_name}:{}: fixture line failed to load — [{}] {e}",
                    n + 1,
                    Class::of(&e).name()
                ));
            }
        }
    }
    // A fixture that did not load fully leaves the engine in an unknown state;
    // running pins against it would produce noise, not signal.
    if !rep.harness.is_empty() {
        return rep;
    }

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

    // ── --kb fixtures (CONTENT pins) ─────────────────────────────────
    //
    // A MECHANISM pin inlines its fixture, because the fixture is not the
    // subject — an external edit weakening it would silently weaken a claim
    // about the ENGINE. A CONTENT pin must load the live artifact, because the
    // artifact IS the subject, and inlining a copy means the pins slowly start
    // certifying a copy that has drifted from the thing shipped.

    const KB: &str = "person(Ara).\nchoose(Electorate, Gia).\n\
                      all $a: choose(Electorate, $a) -> permits(Review, $a).\n";

    fn kb() -> Vec<KbFile> {
        vec![("fixture.nibli".to_string(), KB.to_string())]
    }

    #[test]
    fn kb_fixture_is_visible_to_pins() {
        let r = run_file_with_kb(
            "t",
            "? permits(Review, Gia).\n# => TRUE\n\
             ? person(Ara).\n# => TRUE\n\
             :expect-pins 2\n",
            &kb(),
        );
        assert_eq!(r.findings.len(), 0, "{r:?}");
        assert_eq!(r.harness.len(), 0, "{r:?}");
        assert_eq!(r.pins, 2);
    }

    /// Without the fixture the very same pin file FAILS — proving the fixture is
    /// actually doing the work, not being quietly ignored.
    #[test]
    fn without_the_fixture_the_same_pins_fail() {
        let r = run_file("t", "? permits(Review, Gia).\n# => TRUE\n:expect-pins 1\n");
        assert_eq!(r.findings.len(), 1, "{r:?}");
    }

    /// A fixture that fails to load is a HARNESS error (exit 2), never a finding
    /// (exit 1): if the fixture did not load, no pin ran, so no pinned property
    /// could have regressed. Claiming a finding would assert knowledge the run
    /// does not have.
    #[test]
    fn a_broken_fixture_is_a_harness_error_and_runs_no_pins() {
        let broken = vec![(
            "bad.nibli".to_string(),
            "person(Ara).\nzzznotaword(Bet).\n".into(),
        )];
        let r = run_file_with_kb("t", "? person(Ara).\n# => TRUE\n:expect-pins 1\n", &broken);
        assert_eq!(
            r.findings.len(),
            0,
            "must not be reported as a finding: {r:?}"
        );
        assert!(
            r.harness
                .iter()
                .any(|h| h.contains("bad.nibli:2") && h.contains("fixture line failed to load")),
            "{r:?}"
        );
        assert_eq!(r.pins, 0, "no pin may run against a half-loaded fixture");
    }

    /// The artifact under test is plain KB text. A directive or query inside it
    /// means the two roles got mixed up.
    #[test]
    fn a_fixture_containing_pin_syntax_is_a_harness_error() {
        for bad in [":accept\nperson(Ara).\n", "? person(Ara).\n"] {
            let r = run_file_with_kb(
                "t",
                "? person(Ara).\n# => UNKNOWN\n:expect-pins 1\n",
                &[("bad.nibli".to_string(), bad.to_string())],
            );
            assert!(
                r.harness.iter().any(|h| h.contains("plain KB text")),
                "{bad:?} -> {r:?}"
            );
        }
    }

    /// Fixtures compose in order, and each pin file gets a FRESH engine — one
    /// pin file cannot leak state into the next.
    #[test]
    fn fixtures_compose_and_do_not_leak_between_files() {
        let two = vec![
            ("a.nibli".to_string(), "person(Ara).\n".to_string()),
            ("b.nibli".to_string(), "dog(Rex).\n".to_string()),
        ];
        let r = run_file_with_kb(
            "t",
            "? person(Ara).\n# => TRUE\n? dog(Rex).\n# => TRUE\n:expect-pins 2\n",
            &two,
        );
        assert_eq!(r.findings.len(), 0, "{r:?}");
        // A second run with the same fixtures starts clean: a fact asserted by
        // the FIRST pin file must not be visible to the second.
        let first = run_file_with_kb("t1", "person(Bet).\n:expect-pins 0\n", &two);
        assert_eq!(first.harness.len(), 0, "{first:?}");
        let second = run_file_with_kb("t2", "? person(Bet).\n# => FALSE\n:expect-pins 1\n", &two);
        assert_eq!(
            second.findings.len(),
            0,
            "engine leaked across files: {second:?}"
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

    // ─── --strata: the machine-readable stratification dump ──────────────────
    //
    // Exists so a consuming project reads the engine's OWN stratification instead of
    // re-deriving it from `.nibli` text with regexes. A second implementation drifts,
    // and numbers presented as "the engine computed this order" have to be the
    // engine's. These pin the properties such a consumer depends on.

    const STRATA_KB: &str = "person(Adam).\n\
                             all $x: person($x) & ~home($x) -> prisoner($x).\n\
                             all $x: prisoner($x) -> reward($x).\n";

    fn strata_of(src: &str) -> String {
        let (out, harness) = strata_dump(&[("k.nibli".to_string(), src.to_string())]);
        assert!(harness.is_empty(), "unexpected harness errors: {harness:?}");
        out
    }

    /// Parse the dump back into (predicate, stratum, kind, edges) rows.
    fn strata_rows(out: &str) -> Vec<(String, usize, String, String)> {
        out.lines()
            .filter(|l| !l.starts_with('#'))
            .map(|l| {
                let f: Vec<&str> = l.split('\t').collect();
                assert_eq!(f.len(), 4, "every row is 4 tab-separated fields: {l:?}");
                (
                    f[0].to_string(),
                    f[1].parse().expect("stratum is an integer"),
                    f[2].to_string(),
                    f[3].to_string(),
                )
            })
            .collect()
    }

    #[test]
    fn strata_dump_is_byte_identical_across_runs() {
        // The consumer diffs this across runs; `pred_dep_graph` is a HashMap, so stable
        // ordering is a property that has to be pinned, not assumed.
        let a = strata_of(STRATA_KB);
        let b = strata_of(STRATA_KB);
        assert_eq!(a, b);
        let rows = strata_rows(&a);
        let names: Vec<&str> = rows.iter().map(|r| r.0.as_str()).collect();
        let mut sorted = names.clone();
        sorted.sort_unstable();
        assert_eq!(names, sorted, "rows must be sorted by predicate");
    }

    #[test]
    fn strata_dump_marks_polarity_base_and_level() {
        let out = strata_of(STRATA_KB);
        let rows = strata_rows(&out);
        let get = |p: &str| {
            rows.iter()
                .find(|r| r.0 == p)
                .unwrap_or_else(|| panic!("missing row for {p}: {out}"))
                .clone()
        };

        let (_, home_lvl, home_kind, _) = get("home");
        let (_, pris_lvl, pris_kind, pris_edges) = get("prisoner");
        let (_, watch_lvl, _, watch_edges) = get("reward");

        assert_eq!(home_kind, "base", "nothing concludes `home`");
        assert_eq!(pris_kind, "derived", "a rule concludes `prisoner`");
        assert!(
            pris_lvl > home_lvl,
            "a NAF read must raise the reader's stratum ({pris_lvl} vs {home_lvl})"
        );
        assert_eq!(watch_lvl, pris_lvl, "a positive edge must not raise it");
        assert!(
            pris_edges.split(',').any(|e| e == "-home"),
            "the NAF edge must be marked negative: {pris_edges}"
        );
        assert!(
            watch_edges.split(',').any(|e| e == "+prisoner"),
            "the positive edge must be marked positive: {watch_edges}"
        );
    }

    #[test]
    fn strata_dump_has_a_parseable_comment_header_and_totals() {
        let out = strata_of(STRATA_KB);
        assert!(
            out.starts_with("# nibli-strata v1\n"),
            "versioned header: {out}"
        );
        let n = strata_rows(&out).len();
        assert!(
            out.lines()
                .any(|l| l.starts_with("# totals:") && l.contains(&format!("{n} predicates"))),
            "the totals line must agree with the row count: {out}"
        );
    }

    #[test]
    fn strata_dump_reports_no_rows_when_a_fixture_fails_to_load() {
        // Numbers computed over a partly-loaded KB are worse than no numbers, because
        // they still look like numbers. A bad fixture must yield harness errors and an
        // EMPTY dump, never a plausible-looking partial one.
        let (out, harness) =
            strata_dump(&[("bad.nibli".to_string(), "notaword(((.\n".to_string())]);
        assert!(out.is_empty(), "a failed load must produce no dump: {out}");
        assert!(!harness.is_empty(), "a failed load must be a harness error");
    }

    #[test]
    fn strata_dump_rejects_directives_in_a_fixture() {
        // Same rule as the pin path: a `--kb` fixture is plain KB text.
        let (_, harness) = strata_dump(&[("k.nibli".to_string(), ":strict on\n".to_string())]);
        assert!(
            harness.iter().any(|h| h.contains("plain KB text")),
            "{harness:?}"
        );
    }
}
