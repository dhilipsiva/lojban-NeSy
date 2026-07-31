//! nibli-bench-naf — release-profile timing for negation-as-failure evaluation.
//!
//! NAF is the engine's most expensive shape: `~p(x)` is answered by attempting to
//! prove `p(x)` and failing, so the cost of a negated literal is the cost of an
//! EXHAUSTIVE search for its positive. When the negated relation is concluded by a
//! wide multi-variable rule the search is a domain cartesian per negated occurrence.
//!
//! `utopia.nibli` is the worst shape the shipped corpora contain, and the reason
//! this bench exists:
//!
//!   * `false/1` (Article 4 multi-sig void) is concluded by ONE rule with three
//!     quantified variables and fifteen conjuncts;
//!   * Article 3's merit rules put that relation under `~`
//!     (`teaches($t,$s) & ~false($t) -> reward($t)`).
//!
//! So `reward(Esa).` — a two-hop conclusion — pays for a full sweep of the void
//! rule. `teaches(Esa, Fin).` is the control: the same KB, a stored fact, no
//! search. The gap between those two lines is the figure this bench exists to
//! report.
//!
//! Legs:
//!   load        — assert every line of the shipped `utopia.nibli` corpus
//!   naf-true    — `reward(Esa).`: fires through `~false(Esa)`, which is NOT
//!                 derivable, so the NAF search is exhaustive (the worst case —
//!                 no witness short-circuits it)
//!   naf-false   — `reward(Bela).`: Bela IS voided (Gia + Hex both Review-
//!                 credentialed, both judge+capture), so `~false(Bela)` fails on
//!                 a found witness
//!   lookup      — `teaches(Esa, Fin).`: a stored fact, for contrast
//!
//! Every verdict is asserted each iteration — a timing figure attached to a wrong
//! verdict would be meaningless. Each iteration uses a fresh engine, so no state
//! leaks between runs. Prints min/median/max over `NIBLI_BENCH_RUNS` iterations
//! (default 5 — the NAF leg is slow enough that 10 is impractical before
//! materialisation) after one untimed warm-up.
//!
//! Run via `just bench-naf`. The recipe forces the release profile; a debug build
//! prints a loud warning and its figures must never be quoted.

use nibli_engine::NibliEngine;
use std::process::ExitCode;
use std::time::{Duration, Instant};

const CORPUS: &str = include_str!("../../../utopia.nibli");

/// `~false($teacher)` is not derivable for Esa → the NAF search is exhaustive.
const NAF_TRUE_QUERY: &str = "reward(Esa).";
/// Bela is voided by the multi-sig rule → `~false(Bela)` fails on a witness.
const NAF_FALSE_QUERY: &str = "reward(Bela).";
/// A stored fact: no search at all.
const LOOKUP_QUERY: &str = "teaches(Esa, Fin).";

/// One full sequence on a fresh engine. Returns (load, naf_true, naf_false, lookup).
///
/// `materialization` is the A/B knob: this bench exists to report the gap stratum-ordered
/// materialisation opens, and quoting a before-number from a different build would be a
/// worse measurement than running both sides here. The env read lives in this BIN rather
/// than in `nibli-engine`, per the surface/core discipline the other flags follow
/// (`NIBLI_STRICT`, `NIBLI_EXISTENTIAL_IMPORT`) — the library never reads process env.
fn run_once(materialization: bool) -> Result<(Duration, Duration, Duration, Duration), String> {
    let t_start = Instant::now();
    let engine = NibliEngine::new();
    engine.set_materialization(materialization);
    let mut asserted = 0u32;
    for (line_num, line) in CORPUS.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        engine
            .assert_text(trimmed)
            .map_err(|e| format!("utopia.nibli line {}: {e:?}", line_num + 1))?;
        asserted += 1;
    }
    if asserted == 0 {
        return Err("empty corpus".into());
    }
    let t_load = t_start.elapsed();

    let t0 = Instant::now();
    let r = engine
        .query_holds(NAF_TRUE_QUERY)
        .map_err(|e| format!("{e:?}"))?;
    let t_naf_true = t0.elapsed();
    if !r.is_true() {
        return Err(format!("{NAF_TRUE_QUERY}: expected TRUE, got {r:?}"));
    }

    let t0 = Instant::now();
    let r = engine
        .query_holds(NAF_FALSE_QUERY)
        .map_err(|e| format!("{e:?}"))?;
    let t_naf_false = t0.elapsed();
    if !r.is_false() {
        return Err(format!("{NAF_FALSE_QUERY}: expected FALSE, got {r:?}"));
    }

    let t0 = Instant::now();
    let r = engine
        .query_holds(LOOKUP_QUERY)
        .map_err(|e| format!("{e:?}"))?;
    let t_lookup = t0.elapsed();
    if !r.is_true() {
        return Err(format!("{LOOKUP_QUERY}: expected TRUE, got {r:?}"));
    }

    Ok((t_load, t_naf_true, t_naf_false, t_lookup))
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
    } else {
        format!("{ms:.0} ms")
    }
}

fn main() -> ExitCode {
    let profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    if cfg!(debug_assertions) {
        eprintln!(
            "WARNING: debug build — these figures are NOT quotable. \
             Run `just bench-naf` (release profile)."
        );
    }

    let runs: usize = std::env::var("NIBLI_BENCH_RUNS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(5);

    // `NIBLI_MATERIALIZE=0` runs the same corpus through the pure backward-chaining
    // path — the OFF side of the comparison this bench reports.
    let materialization = std::env::var("NIBLI_MATERIALIZE").ok().as_deref() != Some("0");

    // Warm-up (untimed result; still verdict-checked).
    if let Err(e) = run_once(materialization) {
        eprintln!("bench-naf: sequence failed: {e}");
        return ExitCode::FAILURE;
    }

    let mut loads = Vec::with_capacity(runs);
    let mut naf_trues = Vec::with_capacity(runs);
    let mut naf_falses = Vec::with_capacity(runs);
    let mut lookups = Vec::with_capacity(runs);
    for _ in 0..runs {
        match run_once(materialization) {
            Ok((l, t, f, k)) => {
                loads.push(l);
                naf_trues.push(t);
                naf_falses.push(f);
                lookups.push(k);
            }
            Err(e) => {
                eprintln!("bench-naf: sequence failed: {e}");
                return ExitCode::FAILURE;
            }
        }
    }

    println!(
        "nibli-bench-naf — native in-process engine (nibli-engine), {profile} profile, \
         {runs} runs (fresh engine per run, 1 untimed warm-up)"
    );
    println!(
        "  materialisation: {}",
        if materialization {
            "ON (stratum-ordered; NAF answers by lookup)"
        } else {
            "OFF via NIBLI_MATERIALIZE=0 (every NAF re-proves its positive)"
        }
    );
    println!("  corpus: utopia.nibli (all verdicts asserted every run)");
    for (label, xs) in [
        ("utopia.nibli load", loads),
        ("naf-true  reward(Esa)", naf_trues),
        ("naf-false reward(Bela)", naf_falses),
        ("lookup    teaches(..)", lookups),
    ] {
        let (min, med, max) = stats(xs);
        println!(
            "  {label:<22} min {:>8}   median {:>8}   max {:>8}",
            fmt(min),
            fmt(med),
            fmt(max)
        );
    }
    println!("    (naf-true is the worst case: `~false(Esa)` has no witness, so the");
    println!("     search for `false(Esa)` is exhaustive over the void rule's domain)");
    ExitCode::SUCCESS
}
