#[allow(warnings)]
mod bindings;

use crate::bindings::exports::lojban::nesy::reasoning::Guest;
use crate::bindings::lojban::nesy::ast_types::{LogicBuffer, LogicNode, LogicalTerm};
use egglog::EGraph;
use std::sync::{Mutex, OnceLock};

static EGRAPH: OnceLock<Mutex<EGraph>> = OnceLock::new();

fn get_egraph() -> &'static Mutex<EGraph> {
    EGRAPH.get_or_init(|| {
        let mut egraph = EGraph::default();

        let schema_str = r#"
            ;; ═══════════════════════════════════════════════
            ;; Lojban NeSy Engine — FOL Schema & Rules
            ;; ═══════════════════════════════════════════════

            ;; Atomic Terms
            (datatype Term
                (Var String)
                (Const String)
                (Desc String)
                (Zoe)
            )

            ;; Variadic Argument List (Linked List)
            (datatype TermList
                (Nil)
                (Cons Term TermList)
            )

            ;; Well-Formed Formulas
            (datatype Formula
                (Pred String TermList)
                (And Formula Formula)
                (Or Formula Formula)
                (Not Formula)
                (Implies Formula Formula)
                (Exists String Formula)
                (ForAll String Formula)
            )

            ;; The Knowledge Base
            (relation IsTrue (Formula))

            ;; ───────────────────────────────────────────────
            ;; STRUCTURAL REWRITES
            ;; These merge e-classes (equate terms).
            ;; Always terminating — no new facts generated.
            ;; ───────────────────────────────────────────────

            ;; Commutativity
            (rewrite (And A B) (And B A))
            (rewrite (Or A B) (Or B A))

            ;; Associativity
            (rewrite (And (And A B) C) (And A (And B C)))
            (rewrite (Or (Or A B) C) (Or A (Or B C)))

            ;; Double negation elimination
            (rewrite (Not (Not A)) A)

            ;; De Morgan's Laws
            (rewrite (Not (And A B)) (Or (Not A) (Not B)))
            (rewrite (Not (Or A B)) (And (Not A) (Not B)))

            ;; Material conditional elimination
            (rewrite (Implies A B) (Or (Not A) B))

            ;; ───────────────────────────────────────────────
            ;; INFERENCE RULES
            ;; These add new IsTrue facts to the database.
            ;; All are bounded (no recursive generation).
            ;; ───────────────────────────────────────────────

            ;; Conjunction Elimination: A ∧ B ⊢ A, B
            ;; Enables querying individual predicates from connective assertions
            ;; e.g., "la .bob. cu barda je sutra" → can now query "? la .bob. cu barda"
            (rule ((IsTrue (And A B)))
                  ((IsTrue A) (IsTrue B)))

            ;; Disjunctive Syllogism: A ∨ B, ¬A ⊢ B
            (rule ((IsTrue (Or A B)) (IsTrue (Not A)))
                  ((IsTrue B)))

            ;; Modus Ponens: A → B, A ⊢ B
            (rule ((IsTrue (Implies A B)) (IsTrue A))
                  ((IsTrue B)))

            ;; Modus Tollens: A → B, ¬B ⊢ ¬A
            (rule ((IsTrue (Implies A B)) (IsTrue (Not B)))
                  ((IsTrue (Not A))))

            ;; ───────────────────────────────────────────────
            ;; QUANTIFIER RULES
            ;; ───────────────────────────────────────────────

            ;; ∃-distribution over ∧ (forward only — sound)
            ;; ∃x.(A ∧ B) ⊢ ∃x.A ∧ ∃x.B
            ;; Combined with conjunction elimination, allows extraction of
            ;; individual predicates from existentially quantified conjunctions.
            (rule ((IsTrue (Exists v (And A B))))
                  ((IsTrue (And (Exists v A) (Exists v B)))))

            ;; ∀-distribution over ∧ (sound)
            ;; ∀x.(A ∧ B) ⊢ ∀x.A ∧ ∀x.B
            (rule ((IsTrue (ForAll v (And A B))))
                  ((IsTrue (And (ForAll v A) (ForAll v B)))))
        "#;

        egraph
            .parse_and_run_program(None, schema_str)
            .expect("Failed to load FOL schema and rules");

        Mutex::new(egraph)
    })
}

struct ReasoningComponent;

impl Guest for ReasoningComponent {
    fn assert_fact(logic: LogicBuffer) -> Result<(), String> {
        let egraph_mutex = get_egraph();
        let mut egraph = egraph_mutex.lock().unwrap();

        for &root_id in &logic.roots {
            let sexp = reconstruct_sexp(&logic, root_id);
            let command = format!("(IsTrue {})", sexp);
            if let Err(e) = egraph.parse_and_run_program(None, &command) {
                return Err(format!("Failed to assert fact: {}", e));
            }
        }
        Ok(())
    }

    fn query_entailment(logic: LogicBuffer) -> Result<bool, String> {
        let egraph_mutex = get_egraph();
        let mut egraph = egraph_mutex.lock().unwrap();

        // Saturate: run all rewrites and inference rules to fixpoint.
        // Safe because all rules are bounded (no recursive fact generation).
        if let Err(e) = egraph.parse_and_run_program(None, "(run-schedule (saturate (run)))") {
            // Fallback: if saturate is unavailable in this egglog build, use bounded run
            eprintln!(
                "[reasoning] saturate failed, falling back to bounded run: {}",
                e
            );
            if let Err(e2) = egraph.parse_and_run_program(None, "(run 100)") {
                return Err(format!("Saturation error: {}", e2));
            }
        }

        let mut all_true = true;
        for &root_id in &logic.roots {
            let sexp = reconstruct_sexp(&logic, root_id);
            let command = format!("(check (IsTrue {}))", sexp);
            match egraph.parse_and_run_program(None, &command) {
                Ok(_) => {}
                Err(e) => {
                    let msg = e.to_string();
                    if msg.contains("Check failed") {
                        all_true = false;
                    } else {
                        return Err(format!("Reasoning error: {}", msg));
                    }
                }
            }
        }
        Ok(all_true)
    }
}

/// Translates the zero-copy logic arena into egglog s-expressions.
fn reconstruct_sexp(buffer: &LogicBuffer, node_id: u32) -> String {
    match &buffer.nodes[node_id as usize] {
        LogicNode::Predicate((rel, args)) => {
            let mut args_str = String::from("(Nil)");
            for arg in args.iter().rev() {
                let term_str = match arg {
                    LogicalTerm::Variable(v) => format!("(Var \"{}\")", v),
                    LogicalTerm::Constant(c) => format!("(Const \"{}\")", c),
                    LogicalTerm::Description(d) => format!("(Desc \"{}\")", d),
                    LogicalTerm::Unspecified => "(Zoe)".to_string(),
                };
                args_str = format!("(Cons {} {})", term_str, args_str);
            }
            format!("(Pred \"{}\" {})", rel, args_str)
        }
        LogicNode::AndNode((l, r)) => {
            format!(
                "(And {} {})",
                reconstruct_sexp(buffer, *l),
                reconstruct_sexp(buffer, *r)
            )
        }
        LogicNode::OrNode((l, r)) => {
            format!(
                "(Or {} {})",
                reconstruct_sexp(buffer, *l),
                reconstruct_sexp(buffer, *r)
            )
        }
        LogicNode::NotNode(inner) => {
            format!("(Not {})", reconstruct_sexp(buffer, *inner))
        }
        LogicNode::ExistsNode((v, body)) => {
            format!("(Exists \"{}\" {})", v, reconstruct_sexp(buffer, *body))
        }
        LogicNode::ForAllNode((v, body)) => {
            format!("(ForAll \"{}\" {})", v, reconstruct_sexp(buffer, *body))
        }
    }
}

bindings::export!(ReasoningComponent with_types_in bindings);
