//! nibli-bench-closure — release-profile timing for RECURSIVE rule evaluation.
//!
//! This bench exists because the engine has a cost cliff that is invisible in the
//! shipped corpora and was, until 2026-09-03, undisclosed. Transitive closure —
//!
//!   `all $a, $b, $c: earlier($a,$b) & earlier($b,$c) -> earlier($a,$c).`
//!
//! — is the most natural recursive rule in logic programming, it is CORRECT here,
//! and it is EXPONENTIAL in the chain length. The shipped corpora chain 2-4 hops
//! and `pins/temporal-order.nibli` closes over five, so nothing in `ci` ever pays
//! the cost. A knowledge base that grows one is not warned.
//!
//! WHY it is exponential is the part worth pinning, because it is a POLICY, not a
//! missing algorithm. Stratum-ordered materialisation (`nibli_reason::materialize`)
//! evaluates recursive SCCs bottom-up with semi-naive joins and level indexing, and
//! it handles THIS rule perfectly well — but it is requested only when exact
//! reasoning stays non-definitive (GUARANTEES §Completeness). A positive
//! reachability query resolves definitively, so it never asks, and pays the
//! backward-chaining search instead. The two directions of the SAME query on the
//! SAME knowledge base therefore differ by orders of magnitude, and that asymmetry
//! is the single most informative number this bench prints:
//!
//!   forward   `earlier(N0, Nn).`  TRUE  — backward chaining, exponential in n
//!   backward  `earlier(Nn, N0).`  FALSE — non-definitive, so saturated, ~linear
//!
//! The `adjudication` leg is the contrast: the same engine on NON-recursive rules
//! over thousands of facts (the shape `examples/adjudication/` demonstrates), which
//! is comfortable. Together the legs say where the seam between nibli and an
//! upstream Datalog engine belongs.
//!
//! Legs:
//!   forward-N    — `earlier(N0, NN).` over an N-edge chain + the closure rule.
//!                  TRUE. Backward-chained. Watch the RATIO between sizes, not the
//!                  absolute figures: it is the growth rate that is the finding.
//!   backward-N   — `earlier(NN, N0).` on the same KB. FALSE, and therefore
//!                  saturated — the same rule, evaluated the fast way.
//!   adjudication — a non-recursive policy over `flows` analyzer facts, the regime
//!                  the engine is actually suited to.
//!
//! Every verdict is asserted every iteration — a timing figure attached to a wrong
//! verdict would be meaningless. Each iteration builds a fresh engine. Prints
//! min/median/max over `NIBLI_BENCH_RUNS` iterations (default 3 — the forward legs
//! are slow by construction) after one untimed warm-up.
//!
//! Sizes are deliberately small so the bench finishes: `NIBLI_BENCH_CLOSURE_FWD`
//! (default `6,8,10`) and `NIBLI_BENCH_CLOSURE_BWD` (default `25,50`) override
//! them. Raising the forward list is how you re-measure the cliff; n=12 costs
//! roughly 18 s and n=15 does not finish in a minute.
//!
//! Run via `just bench-closure`. The recipe forces the release profile; a debug
//! build prints a loud warning and its figures must never be quoted.

use nibli_engine::NibliEngine;
use std::process::ExitCode;
use std::time::{Duration, Instant};

/// The closure rule under test. One line: `assert_text` is line-oriented.
const CLOSURE_RULE: &str = "all $a, $b, $c: earlier($a,$b) & earlier($b,$c) -> earlier($a,$c).";

/// Non-recursive policy mirroring `examples/adjudication/policy.nibli`. Generated
/// rather than `include_str!`d so this bin carries no repo-root path dependency.
const POLICY: &[&str] = &[
    "admits(\"carries\").",
    "admits(\"dangerous\").",
    "admits(\"prevents\").",
    "admits(\"permits\").",
    "derived_only(\"authorized\").",
    "derived_only(\"warns\").",
    "all $f, $d, $s, $o, $r: carries($f, $d, $s, $o, $r) & dangerous($s, $d, Exploit) & prevents(Sanitizer, $f) -> authorized($f, Release, $s).",
    "all $f, $d, $s, $o, $r: carries($f, $d, $s, $o, $r) & dangerous($s, $d, Exploit) & permits(Review, $f, Waiver) -> authorized($f, Release, $s).",
    "all $f, $d, $s, $o, $r: carries($f, $d, $s, $o, $r) & ~authorized($f, Release, $s) -> warns(Gate, $f, $s).",
];

fn build_chain(n: usize) -> Result<NibliEngine, String> {
    let engine = NibliEngine::new();
    engine
        .assert_text(CLOSURE_RULE)
        .map_err(|e| format!("closure rule: {e:?}"))?;
    for i in 0..n {
        let line = format!("earlier(N{i}, N{}).", i + 1);
        engine
            .assert_text(&line)
            .map_err(|e| format!("{line}: {e:?}"))?;
    }
    Ok(engine)
}

/// `earlier(N0, Nn).` — TRUE. Definitive, so materialisation is never requested
/// and this is the backward-chaining cost.
fn time_forward(n: usize) -> Result<Duration, String> {
    let engine = build_chain(n)?;
    let q = format!("earlier(N0, N{n}).");
    let t0 = Instant::now();
    let r = engine.query_holds(&q).map_err(|e| format!("{e:?}"))?;
    let dt = t0.elapsed();
    if !r.is_true() {
        return Err(format!("{q}: expected TRUE, got {r:?}"));
    }
    Ok(dt)
}

/// `earlier(Nn, N0).` — FALSE on the same KB. Non-definitive on the exact path, so
/// the cone IS saturated; this is the same rule evaluated bottom-up.
fn time_backward(n: usize) -> Result<Duration, String> {
    let engine = build_chain(n)?;
    let q = format!("earlier(N{n}, N0).");
    let t0 = Instant::now();
    let r = engine.query_holds(&q).map_err(|e| format!("{e:?}"))?;
    let dt = t0.elapsed();
    if !r.is_false() {
        return Err(format!("{q}: expected FALSE, got {r:?}"));
    }
    Ok(dt)
}

/// Non-recursive adjudication over `flows` analyzer facts. Returns
/// (load, clearance-query, finding-query).
///
/// The two query legs are the SAME question asked two ways, and the gap between
/// them is the non-recursive echo of the forward/backward gap above. The two
/// clearance rules are purely POSITIVE, so `authorized(F1, …)` returning FALSE
/// resolves definitively and is never materialised — an exhaustive search that
/// goes quadratic in the fact count. `warns(Gate, F1, S1)` reads clearance under
/// `~`, which requests the cone, saturates it bottom-up, and answers from a
/// complete set. Same verdict, same knowledge base, orders of magnitude apart.
fn time_adjudication(flows: usize) -> Result<(Duration, Duration, Duration), String> {
    let engine = NibliEngine::new();
    let t0 = Instant::now();
    for line in POLICY {
        engine
            .assert_text(line)
            .map_err(|e| format!("{line}: {e:?}"))?;
    }
    for i in 0..flows {
        for line in [
            format!("carries(F{i}, D{i}, S{i}, O{i}, R{i})."),
            format!("dangerous(S{i}, D{i}, Exploit)."),
        ] {
            engine
                .assert_text(&line)
                .map_err(|e| format!("{line}: {e:?}"))?;
        }
        if i % 2 == 0 {
            let line = format!("prevents(Sanitizer, F{i}).");
            engine
                .assert_text(&line)
                .map_err(|e| format!("{line}: {e:?}"))?;
        }
    }
    let t_load = t0.elapsed();

    // F1 has no sanitizer and no waiver: not cleared, therefore a finding.
    let t0 = Instant::now();
    let cleared = engine
        .query_holds("authorized(F1, Release, S1).")
        .map_err(|e| format!("{e:?}"))?;
    let t_clearance = t0.elapsed();
    if !cleared.is_false() {
        return Err(format!(
            "authorized(F1, …): expected FALSE, got {cleared:?}"
        ));
    }

    let t0 = Instant::now();
    let finding = engine
        .query_holds("warns(Gate, F1, S1).")
        .map_err(|e| format!("{e:?}"))?;
    let t_finding = t0.elapsed();
    if !finding.is_true() {
        return Err(format!(
            "warns(Gate, F1, S1): expected TRUE, got {finding:?}"
        ));
    }
    Ok((t_load, t_clearance, t_finding))
}

fn stats(mut xs: Vec<Duration>) -> (Duration, Duration, Duration) {
    xs.sort();
    let median = xs[xs.len() / 2];
    (xs[0], median, *xs.last().unwrap())
}

fn fmt(d: Duration) -> String {
    let ms = d.as_secs_f64() * 1000.0;
    if ms < 10.0 {
        format!("{ms:.1} ms")
    } else if ms < 10_000.0 {
        format!("{ms:.0} ms")
    } else {
        format!("{:.1} s", d.as_secs_f64())
    }
}

fn sizes(var: &str, default: &[usize]) -> Result<Vec<usize>, String> {
    match std::env::var(var) {
        Err(_) => Ok(default.to_vec()),
        Ok(s) => s
            .split(',')
            .map(|t| {
                t.trim()
                    .parse::<usize>()
                    .map_err(|_| format!("{var}: {t:?} is not a chain length"))
            })
            .collect(),
    }
}

fn main() -> ExitCode {
    if cfg!(debug_assertions) {
        eprintln!(
            "WARNING: debug build — these figures are NOT quotable. \
             Run `just bench-closure` (release profile)."
        );
    }
    let runs: usize = std::env::var("NIBLI_BENCH_RUNS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3);
    let fwd = match sizes("NIBLI_BENCH_CLOSURE_FWD", &[6, 8, 10]) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("nibli-bench-closure: {e}");
            return ExitCode::FAILURE;
        }
    };
    let bwd = match sizes("NIBLI_BENCH_CLOSURE_BWD", &[25, 50]) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("nibli-bench-closure: {e}");
            return ExitCode::FAILURE;
        }
    };
    const FLOWS: usize = 500;

    println!("nibli-bench-closure — recursive vs non-recursive evaluation");
    println!(
        "profile: {}   runs: {runs}",
        if cfg!(debug_assertions) {
            "debug (NOT quotable)"
        } else {
            "release"
        }
    );
    println!("rule: {CLOSURE_RULE}\n");

    // Warm-up (untimed): smallest of each family.
    let warm = |r: Result<Duration, String>| r.map(|_| ());
    if let Err(e) = fwd
        .first()
        .map(|&n| warm(time_forward(n)))
        .unwrap_or(Ok(()))
        .and_then(|_| {
            bwd.first()
                .map(|&n| warm(time_backward(n)))
                .unwrap_or(Ok(()))
        })
        .and_then(|_| time_adjudication(FLOWS).map(|_| ()))
    {
        eprintln!("nibli-bench-closure: warm-up failed: {e}");
        return ExitCode::FAILURE;
    }

    println!("FORWARD — `earlier(N0, Nn).` TRUE. Definitive, so NOT materialised:");
    println!("  the backward-chaining cost. Watch the ratio between rows.");
    let mut prev: Option<(usize, f64)> = None;
    for &n in &fwd {
        let mut xs = Vec::with_capacity(runs);
        for _ in 0..runs {
            match time_forward(n) {
                Ok(d) => xs.push(d),
                Err(e) => {
                    eprintln!("nibli-bench-closure: forward n={n}: {e}");
                    return ExitCode::FAILURE;
                }
            }
        }
        let (lo, med, hi) = stats(xs);
        let ms = med.as_secs_f64() * 1000.0;
        let growth = match prev {
            Some((pn, pms)) if pms > 0.0 && n > pn => {
                format!("   {:.1}x over n={pn}", ms / pms)
            }
            _ => String::new(),
        };
        println!(
            "  n={n:<4} min {:>9}  median {:>9}  max {:>9}{growth}",
            fmt(lo),
            fmt(med),
            fmt(hi)
        );
        prev = Some((n, ms));
    }

    println!("\nBACKWARD — `earlier(Nn, N0).` FALSE on the SAME KB. Non-definitive,");
    println!("  so the cone IS saturated: the same rule, evaluated bottom-up.");
    for &n in &bwd {
        let mut xs = Vec::with_capacity(runs);
        for _ in 0..runs {
            match time_backward(n) {
                Ok(d) => xs.push(d),
                Err(e) => {
                    eprintln!("nibli-bench-closure: backward n={n}: {e}");
                    return ExitCode::FAILURE;
                }
            }
        }
        let (lo, med, hi) = stats(xs);
        let tuples = n * (n + 1) / 2;
        println!(
            "  n={n:<4} min {:>9}  median {:>9}  max {:>9}   ({tuples} closure tuples)",
            fmt(lo),
            fmt(med),
            fmt(hi)
        );
    }

    println!("\nADJUDICATION — non-recursive policy, {FLOWS} analyzer flows");
    println!("  (the `examples/adjudication/` shape). The SAME question asked two ways:");
    println!("  clearance is purely positive so it is NOT materialised; the finding");
    println!("  reads clearance under `~`, which is.");
    let mut loads = Vec::with_capacity(runs);
    let mut clearances = Vec::with_capacity(runs);
    let mut findings = Vec::with_capacity(runs);
    for _ in 0..runs {
        match time_adjudication(FLOWS) {
            Ok((l, c, f)) => {
                loads.push(l);
                clearances.push(c);
                findings.push(f);
            }
            Err(e) => {
                eprintln!("nibli-bench-closure: adjudication: {e}");
                return ExitCode::FAILURE;
            }
        }
    }
    for (label, xs) in [
        ("load                        ", loads),
        ("clearance `authorized` FALSE", clearances),
        ("finding   `warns`      TRUE ", findings),
    ] {
        let (lo, med, hi) = stats(xs);
        println!(
            "  {label}  min {:>9}  median {:>9}  max {:>9}",
            fmt(lo),
            fmt(med),
            fmt(hi)
        );
    }

    println!(
        "\nThe finding is the RATIO, not the absolutes: forward growth is \
         multiplicative\nin n while backward and finding are not — and the \
         clearance/finding gap is the\nsame trigger policy showing up without \
         any recursion at all.\nGUARANTEES §Disclosed Sharp Edges."
    );
    ExitCode::SUCCESS
}
