#[allow(warnings)]
mod bindings;

use bindings::exports::lojban::nesy::engine_pipeline::Guest;
use bindings::lojban::nesy::{parser, reasoning, semantics};

struct Orchestrator;

impl Guest for Orchestrator {
    fn execute(input: String) -> bool {
        // 1. Internal WASM call to the Parser
        let parse_result = parser::parse_text(&input);

        let ast_buffer = match parse_result {
            Ok(buf) => buf,
            Err(e) => {
                eprintln!("[Orchestrator] Parse failed: {}", e);
                return false;
            }
        };

        // 2. Internal WASM call to Semantics (Zero host-side mapping!)
        let logical_forms = semantics::compile_buffer(&ast_buffer);

        // 3. Internal WASM call to Reasoning
        let mut overall_truth = true;
        for sexp in logical_forms {
            reasoning::assert_fact(&sexp);
            if !reasoning::query_entailment(&sexp) {
                overall_truth = false;
            }
        }

        overall_truth
    }
}

bindings::export!(Orchestrator with_types_in bindings);
