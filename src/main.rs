use bumpalo::Bump;
use reedline::{DefaultPrompt, Reedline, Signal};

mod ast;
mod dictionary;
mod ir;
mod lexer;
mod preprocessor;
mod reasoning;
mod semantic;

use ast::parse_to_ast;
use dictionary::JbovlasteSchema;
use lexer::tokenize;
use preprocessor::preprocess;
use reasoning::ReasoningCore;
use semantic::SemanticCompiler;

fn main() {
    println!("==================================================");
    println!(" Lojban Neuro-Symbolic Engine - Bare-Metal V1 MVP ");
    println!("==================================================");

    // 1. Load the dictionary schema
    println!("Loading jbovlaste dictionary schema...");
    let schema = JbovlasteSchema::load_from_file("jbovlaste-en.xml");
    println!("Loaded {} predicate arities.", schema.arities.len());

    // 2. Initialize persistent state
    let mut reasoner = ReasoningCore::new(&schema);
    let mut compiler = SemanticCompiler::new(schema);

    let mut line_editor = Reedline::create();
    let prompt = DefaultPrompt::default();

    println!("Type ':quit' to exit.");

    loop {
        let sig = line_editor.read_line(&prompt);
        match sig {
            Ok(Signal::Success(buffer)) => {
                let input = buffer.trim();

                if input.is_empty() {
                    continue;
                }
                if input == ":quit" || input == ":q" {
                    break;
                }

                println!("\n[1] Raw Input: {}", input);

                // Phase 1: Lexing
                let raw_tokens = tokenize(input);
                println!("[2] Lexed Tokens: {} items", raw_tokens.len());

                // Phase 2: Preprocessing
                let normalized_tokens = preprocess(raw_tokens.into_iter(), input);

                // Phase 3: Reconstruct sanitized string for structural parsing
                let sanitized_input = normalized_tokens
                    .iter()
                    .filter_map(|t| match t {
                        preprocessor::NormalizedToken::Standard(_, s) => Some(*s),
                        preprocessor::NormalizedToken::Quoted(s) => Some(*s),
                        preprocessor::NormalizedToken::Glued(parts) => Some(parts[0]),
                    })
                    .collect::<Vec<&str>>()
                    .join(" ");

                // Phase 4: Structural Parsing (AST via Bump Arena)
                let arena = Bump::new();
                match parse_to_ast(&sanitized_input, &arena) {
                    Ok(ast) => {
                        println!("[3] AST Generated: {} Bridi nodes", ast.len());

                        for bridi in ast.iter() {
                            let lir = compiler.compile_bridi(bridi);
                            println!("[4] Logical Form (LIR): {:?}", lir);

                            // Dynamically generate the S-Expression
                            let sexp = compiler.to_sexp(&lir);

                            reasoner.assert_fact(&sexp);
                            println!("[5] Fact asserted into egglog Truth Relation.");

                            println!("[6] Verifying Graph Entailment...");
                            reasoner.query(&sexp);
                        }
                    }
                    Err(e) => {
                        eprintln!("[!] Parser Error: {}", e);
                    }
                }
                // The `arena` goes out of scope and frees all AST memory instantly.
            }
            Ok(Signal::CtrlD) | Ok(Signal::CtrlC) => {
                println!("Aborting.");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
}
