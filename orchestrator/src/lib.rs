#[allow(warnings)]
mod bindings;

use bindings::Guest;
use bindings::lojban::nesy::{parser, reasoning, semantics};

struct EnginePipeline;

impl Guest for EnginePipeline {
    fn execute(input: String) -> bool {
        // REPL Command Routing
        let is_query = input.starts_with("?");
        let text = if is_query { input[1..].trim() } else { &input };

        // --- Phase 1: Zero-Copy Parse ---
        let ast = match parser::parse_text(text) {
            Ok(ast) => ast,
            Err(e) => {
                println!("[WASM] Parser Error: {}", e);
                return false;
            }
        };

        // --- Phase 2: Zero-Copy Semantics ---
        let sexps = match semantics::compile_buffer(&ast) {
            Ok(s) => s,
            Err(e) => {
                println!("[WASM] Semantics Error: {}", e);
                return false;
            }
        };

        let mut final_result = true;

        // --- Phase 3: Reasoning ---
        for sexp in sexps {
            if is_query {
                match reasoning::query_entailment(&sexp) {
                    Ok(result) => {
                        println!(
                            "[WASM] Query Entailment: {}",
                            if result { "TRUE" } else { "FALSE" }
                        );
                        final_result = result;
                    }
                    Err(e) => println!("[WASM] Query Error: {}", e),
                }
            } else {
                if let Err(e) = reasoning::assert_fact(&sexp) {
                    println!("[WASM] Assert Error: {}", e);
                    continue;
                }
                println!("[WASM] Fact Asserted: {}", sexp);
            }
        }

        final_result
    }
}

bindings::export!(EnginePipeline with_types_in bindings);
