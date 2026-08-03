//! Doc-fence gate: every statement inside a ```nibli-kr fence under
//! `mdbook/src/` must compile through the SHIPPED front-end — the same
//! `NibliEngine::assert_text` path as the REPL's `:load` and `nibli-validate`.
//!
//! Why here and not in the `docs` CI job: that job is Nix + `mdbook build`
//! with no Rust toolchain at all (~2 min), and DOCS_TODO's deferral ("keep
//! docs job free of full cargo build") protects exactly that lightness. `just
//! ci` already compiles the workspace, so this rides along for free.
//!
//! Why `nibli-verify`: it is `publish = false` (so reaching outside the
//! package dir has no packaging consequence), it already depends on
//! nibli-engine, and `ci` already builds it for `verify-nibli-kr-seam`.

use std::fs;
use std::path::{Path, PathBuf};

use nibli_engine::NibliEngine;

/// Resolved from the manifest dir so the test is CWD-independent.
const MDBOOK_SRC: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../mdbook/src");

const FENCE_LANG: &str = "nibli-kr";

/// Coverage floors. A path typo or a broken scanner must fail LOUDLY rather
/// than pass by checking nothing (the discipline `nibli-ui`'s
/// `shipped_examples_compile` floor established). Set just under today's
/// 22 statements / 6 files so ordinary doc edits never touch this file.
const MIN_STATEMENTS: usize = 20;
const MIN_FILES: usize = 5;

struct Stmt {
    file: String,
    line: usize,
    /// Fence ordinal within the file — statements in one fence share a KB.
    fence: usize,
    text: String,
}

/// `(fence_char, run_length, remainder)` when `t` opens with 3+ ` or ~.
fn fence_run(t: &str) -> Option<(char, usize, &str)> {
    let ch = t.chars().next()?;
    if ch != '`' && ch != '~' {
        return None;
    }
    let n = t.chars().take_while(|&c| c == ch).count();
    // ` and ~ are one byte each, so the char count is also the byte offset.
    if n < 3 { None } else { Some((ch, n, &t[n..])) }
}

/// CommonMark-shaped fence scan.
///
/// State carries the opening fence's CHARACTER and RUN LENGTH, so (a) a fence
/// carrying an info string never closes a block, and (b) a longer outer fence
/// may legally contain a shorter inner one. A plain `^```(\w*)$` regex once
/// inverted fence state on the hyphenated `nibli-kr` tag in the book's
/// verify_book.py; this construction cannot make that mistake.
fn extract(rel: &str, src: &str) -> Vec<Stmt> {
    let mut out = Vec::new();
    let mut open: Option<(char, usize, bool)> = None;
    let mut fence_no = 0usize;

    for (i, raw) in src.lines().enumerate() {
        let t = raw.trim_start();
        let indent = raw.len() - t.len();

        match open {
            Some((ch, len, is_kr)) => {
                if let Some((fch, flen, rest)) = fence_run(t)
                    && fch == ch
                    && flen >= len
                    && rest.trim().is_empty()
                {
                    open = None;
                    continue;
                }
                if !is_kr {
                    continue;
                }
                let s = raw.trim();
                // Blank lines and whole-line `#` comments are corpus
                // convention, not statements (the same skip the .nibli
                // corpora and nibli-ui's guard use). A TRAILING `#` comment
                // stays attached — it is legal KR and must compile.
                if s.is_empty() || s.starts_with('#') {
                    continue;
                }
                out.push(Stmt {
                    file: rel.to_string(),
                    line: i + 1,
                    fence: fence_no,
                    text: s.to_string(),
                });
            }
            None => {
                // 4+ spaces of indent is an indented code block, not a fence.
                if indent > 3 {
                    continue;
                }
                if let Some((fch, flen, rest)) = fence_run(t) {
                    let lang = rest
                        .trim()
                        .split_whitespace()
                        .next()
                        .unwrap_or("")
                        .to_ascii_lowercase();
                    if lang == FENCE_LANG {
                        fence_no += 1;
                    }
                    open = Some((fch, flen, lang == FENCE_LANG));
                }
            }
        }
    }

    assert!(
        open.is_none(),
        "{rel}: unterminated code fence — everything after it is swallowed, \
         so any ```{FENCE_LANG} fence below it is silently unchecked"
    );
    out
}

fn markdown_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let rd = fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("cannot read {}: {e} — did mdbook/src move?", dir.display()));
    let mut paths: Vec<PathBuf> = rd.map(|e| e.unwrap().path()).collect();
    paths.sort(); // deterministic report order
    for p in paths {
        if p.is_dir() {
            markdown_files(&p, out);
        } else if p.extension().is_some_and(|e| e == "md") {
            out.push(p);
        }
    }
}

#[test]
fn mdbook_kr_fences_compile() {
    let root = Path::new(MDBOOK_SRC);
    let mut files = Vec::new();
    markdown_files(root, &mut files);

    let mut statements = Vec::new();
    let mut files_with_fences = 0usize;
    for path in &files {
        let rel = format!("mdbook/src/{}", path.strip_prefix(root).unwrap().display());
        let stmts = extract(&rel, &fs::read_to_string(path).unwrap());
        if !stmts.is_empty() {
            files_with_fences += 1;
        }
        statements.extend(stmts);
    }

    // One KB per FENCE: a fence is what a reader pastes into `:load`, so its
    // lines are asserted CUMULATIVELY (this also catches a pair of lines that
    // are individually fine but jointly unstratifiable). reset() between
    // fences keeps every failure attributable to one fence.
    let engine = NibliEngine::new();
    let mut current: Option<(String, usize)> = None;
    let mut failures: Vec<String> = Vec::new();

    for st in &statements {
        let key = (st.file.clone(), st.fence);
        if current.as_ref() != Some(&key) {
            engine.reset();
            current = Some(key);
        }
        if let Err(e) = engine.assert_text(&st.text) {
            failures.push(format!(
                "  {}:{}\n      {}\n      -> {e}",
                st.file, st.line, st.text
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "{} statement(s) in ```{FENCE_LANG} fences do not compile:\n\n{}\n\n\
         Every line in a ```{FENCE_LANG} fence must be ONE complete statement \
         that compiles through the shipped front-end. `:load` and \
         nibli-validate are both line-oriented, so a statement wrapped across \
         lines is a doc bug even when the joined text would parse. Prose \
         examples, REPL transcripts, output, and pseudo-syntax belong in a \
         ```text fence.",
        failures.len(),
        failures.join("\n"),
    );

    assert!(
        statements.len() >= MIN_STATEMENTS && files_with_fences >= MIN_FILES,
        "doc-fence coverage collapsed: {} statement(s) across {} file(s) under \
         {MDBOOK_SRC} (floor {MIN_STATEMENTS}/{MIN_FILES}) — the scanner or the \
         path is broken, not the docs",
        statements.len(),
        files_with_fences,
    );
}
