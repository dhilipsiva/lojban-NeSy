use crate::bindings::lojban::nesy::ast_types::{
    Bridi, Connective, Conversion, Gadri, PlaceTag, Selbri, Sumti,
};
use crate::dictionary::JbovlasteSchema;
use crate::ir::{LogicalForm, LogicalTerm};
use lasso::Rodeo;

/// Tracks a quantifier introduced by a `lo` description,
/// with an optional relative clause restrictor.
struct QuantifierEntry {
    var: lasso::Spur,
    desc_id: u32,
    restrictor: Option<LogicalForm>,
}

pub struct SemanticCompiler {
    pub interner: Rodeo,
    pub var_counter: usize,
}

impl SemanticCompiler {
    pub fn new() -> Self {
        Self {
            interner: Rodeo::new(),
            var_counter: 0,
        }
    }

    fn fresh_var(&mut self) -> lasso::Spur {
        let v = format!("_v{}", self.var_counter);
        self.var_counter += 1;
        self.interner.get_or_intern(&v)
    }

    // ─── Selbri Introspection ────────────────────────────────────

    /// Recursively extracts the arity of the structural head of the relation.
    fn get_selbri_arity(&self, selbri_id: u32, selbris: &[Selbri]) -> usize {
        match &selbris[selbri_id as usize] {
            Selbri::Root(g) => JbovlasteSchema::get_arity_or_default(g.as_str()),
            Selbri::Tanru((_, head_id)) => self.get_selbri_arity(*head_id, selbris),
            Selbri::Converted((_, inner_id)) => self.get_selbri_arity(*inner_id, selbris),
            Selbri::Negated(inner_id) => self.get_selbri_arity(*inner_id, selbris),
            Selbri::Grouped(inner_id) => self.get_selbri_arity(*inner_id, selbris),
            Selbri::WithArgs((core_id, _)) => self.get_selbri_arity(*core_id, selbris),
            Selbri::Connected((left_id, _, _)) => self.get_selbri_arity(*left_id, selbris),
            Selbri::Compound(parts) => parts
                .last()
                .map(|s| JbovlasteSchema::get_arity_or_default(s.as_str()))
                .unwrap_or(2),
        }
    }

    /// Extracts the string name of the structural head (for non-quantified descriptions).
    fn get_selbri_head_name<'a>(&self, selbri_id: u32, selbris: &'a [Selbri]) -> &'a str {
        match &selbris[selbri_id as usize] {
            Selbri::Root(r) => r.as_str(),
            Selbri::Tanru((_, head_id)) => self.get_selbri_head_name(*head_id, selbris),
            Selbri::Converted((_, inner_id)) => self.get_selbri_head_name(*inner_id, selbris),
            Selbri::Negated(inner_id) => self.get_selbri_head_name(*inner_id, selbris),
            Selbri::Grouped(inner_id) => self.get_selbri_head_name(*inner_id, selbris),
            Selbri::WithArgs((core_id, _)) => self.get_selbri_head_name(*core_id, selbris),
            Selbri::Connected((left_id, _, _)) => self.get_selbri_head_name(*left_id, selbris),
            Selbri::Compound(parts) => parts.last().map(|s| s.as_str()).unwrap_or("entity"),
        }
    }

    // ─── Sumti Resolution ────────────────────────────────────────

    /// Resolve a single sumti into a LogicalTerm, collecting any quantifier entries generated.
    fn resolve_sumti(
        &mut self,
        sumti: &Sumti,
        sumtis: &[Sumti],
        selbris: &[Selbri],
        sentences: &[Bridi],
    ) -> (LogicalTerm, Vec<QuantifierEntry>) {
        match sumti {
            Sumti::ProSumti(p) => {
                let term = if matches!(p.as_str(), "da" | "de" | "di") {
                    LogicalTerm::Variable(self.interner.get_or_intern(p.as_str()))
                } else {
                    LogicalTerm::Constant(self.interner.get_or_intern(p.as_str()))
                };
                (term, vec![])
            }

            Sumti::Name(n) => (
                LogicalTerm::Constant(self.interner.get_or_intern(n.as_str())),
                vec![],
            ),

            Sumti::Description((gadri, desc_id)) => {
                if matches!(gadri, Gadri::Lo) {
                    let var = self.fresh_var();
                    (
                        LogicalTerm::Variable(var),
                        vec![QuantifierEntry {
                            var,
                            desc_id: *desc_id,
                            restrictor: None,
                        }],
                    )
                } else {
                    // le/la descriptions: non-quantified specific referent
                    let desc_str = self.get_selbri_head_name(*desc_id, selbris);
                    (
                        LogicalTerm::Description(self.interner.get_or_intern(desc_str)),
                        vec![],
                    )
                }
            }

            // FIX 1.3: Tagged sumti — resolve the inner sumti.
            // Positional assignment is handled by compile_bridi, not here.
            Sumti::Tagged((_tag, inner_id)) => {
                let inner = &sumtis[*inner_id as usize];
                self.resolve_sumti(inner, sumtis, selbris, sentences)
            }

            // FIX 1.4: Restricted sumti — resolve inner + compile relative clause.
            Sumti::Restricted((inner_id, rel_clause)) => {
                let inner = &sumtis[*inner_id as usize];
                let (term, mut quants) = self.resolve_sumti(inner, sumtis, selbris, sentences);

                // Compile the relative clause body as a full bridi
                let rel_body = self.compile_bridi(
                    &sentences[rel_clause.body_sentence as usize],
                    selbris,
                    sumtis,
                    sentences,
                );

                // Inject the bound variable as x1 of the restrictor and attach
                // to the most recent quantifier (the one created by the inner sumti).
                if let Some(last) = quants.last_mut() {
                    last.restrictor = Some(Self::inject_variable(rel_body, last.var));
                }
                // If no quantifier exists (e.g., "mi poi barda"), the restrictor
                // is dropped. This is rare and semantically unusual.

                (term, quants)
            }

            Sumti::QuotedLiteral(q) => (
                LogicalTerm::Constant(self.interner.get_or_intern(q.as_str())),
                vec![],
            ),

            Sumti::Unspecified => (LogicalTerm::Unspecified, vec![]),
        }
    }

    /// Inject a variable as x1 of a logical form, replacing Unspecified in
    /// the first argument position. Used to bind relative clause restrictors
    /// to their head noun's variable.
    fn inject_variable(form: LogicalForm, var: lasso::Spur) -> LogicalForm {
        match form {
            LogicalForm::Predicate { relation, mut args } => {
                if !args.is_empty() && matches!(args[0], LogicalTerm::Unspecified) {
                    args[0] = LogicalTerm::Variable(var);
                } else if args.is_empty() {
                    args.push(LogicalTerm::Variable(var));
                }
                LogicalForm::Predicate { relation, args }
            }
            // Tanru produce And — inject into both branches
            LogicalForm::And(l, r) => LogicalForm::And(
                Box::new(Self::inject_variable(*l, var)),
                Box::new(Self::inject_variable(*r, var)),
            ),
            LogicalForm::Not(inner) => {
                LogicalForm::Not(Box::new(Self::inject_variable(*inner, var)))
            }
            // Quantifiers, Or — pass through unchanged
            other => other,
        }
    }

    // ─── Selbri Application ──────────────────────────────────────

    /// Recursively instantiates a Selbri against a set of arguments, correctly
    /// mapping intersective Tanru, argument-swapping conversions, negation,
    /// grouping, be/bei binding, and connectives across the AST tree.
    fn apply_selbri(
        &mut self,
        selbri_id: u32,
        args: &[LogicalTerm],
        selbris: &[Selbri],
        sumtis: &[Sumti],
        sentences: &[Bridi],
    ) -> LogicalForm {
        match &selbris[selbri_id as usize] {
            Selbri::Root(g) => LogicalForm::Predicate {
                relation: self.interner.get_or_intern(g.as_str()),
                args: args.to_vec(),
            },

            // Tanru → Intersective Conjunction: sutra gerku(x) = sutra(x) ∧ gerku(x)
            Selbri::Tanru((mod_id, head_id)) => {
                let left = self.apply_selbri(*mod_id, args, selbris, sumtis, sentences);
                let right = self.apply_selbri(*head_id, args, selbris, sumtis, sentences);
                LogicalForm::And(Box::new(left), Box::new(right))
            }

            // SE-conversion: permute argument positions
            Selbri::Converted((conv, inner_id)) => {
                let mut permuted = args.to_vec();
                match conv {
                    Conversion::Se if permuted.len() >= 2 => permuted.swap(0, 1),
                    Conversion::Te if permuted.len() >= 3 => permuted.swap(0, 2),
                    Conversion::Ve if permuted.len() >= 4 => permuted.swap(0, 3),
                    Conversion::Xe if permuted.len() >= 5 => permuted.swap(0, 4),
                    _ => {}
                }
                self.apply_selbri(*inner_id, &permuted, selbris, sumtis, sentences)
            }

            // FIX 1.2: Selbri negation → ¬P(args)
            Selbri::Negated(inner_id) => {
                let inner = self.apply_selbri(*inner_id, args, selbris, sumtis, sentences);
                LogicalForm::Not(Box::new(inner))
            }

            // FIX 1.7: ke/ke'e grouping — transparent wrapper, just recurse
            Selbri::Grouped(inner_id) => {
                self.apply_selbri(*inner_id, args, selbris, sumtis, sentences)
            }

            // FIX 1.5: be/bei clause — bound arguments fill x2, x3, ... of the core
            Selbri::WithArgs((core_id, bound_ids)) => {
                let core_arity = self.get_selbri_arity(*core_id, selbris);
                let mut merged = Vec::with_capacity(core_arity);
                let mut inner_quantifiers: Vec<QuantifierEntry> = Vec::new();

                // x1 from outer context
                merged.push(if !args.is_empty() {
                    args[0].clone()
                } else {
                    LogicalTerm::Unspecified
                });

                // x2, x3, ... from be/bei bound arguments
                for bound_id in bound_ids.iter() {
                    let bound_sumti = &sumtis[*bound_id as usize];
                    let (term, quants) =
                        self.resolve_sumti(bound_sumti, sumtis, selbris, sentences);
                    inner_quantifiers.extend(quants);
                    merged.push(term);
                }

                // Pad remaining positions with outer args (if any) or Unspecified
                let bound_count = 1 + bound_ids.len(); // x1 + be/bei args
                for i in merged.len()..core_arity {
                    if i < args.len() && i >= bound_count {
                        merged.push(args[i].clone());
                    } else {
                        merged.push(LogicalTerm::Unspecified);
                    }
                }

                let mut form = self.apply_selbri(*core_id, &merged, selbris, sumtis, sentences);

                // Wrap with quantifiers from be/bei arguments (e.g., "nelci be lo gerku")
                for entry in inner_quantifiers.into_iter().rev() {
                    let desc_arity = self.get_selbri_arity(entry.desc_id, selbris);
                    let mut restrictor_args = vec![LogicalTerm::Variable(entry.var)];
                    while restrictor_args.len() < desc_arity {
                        restrictor_args.push(LogicalTerm::Unspecified);
                    }
                    let restrictor = self.apply_selbri(
                        entry.desc_id,
                        &restrictor_args,
                        selbris,
                        sumtis,
                        sentences,
                    );
                    let mut body = LogicalForm::And(Box::new(restrictor), Box::new(form));
                    if let Some(rel_restrictor) = entry.restrictor {
                        body = LogicalForm::And(Box::new(rel_restrictor), Box::new(body));
                    }
                    form = LogicalForm::Exists(entry.var, Box::new(body));
                }

                form
            }

            // FIX 1.6: Selbri connectives — je/ja/jo/ju
            Selbri::Connected((left_id, conn, right_id)) => {
                let left = self.apply_selbri(*left_id, args, selbris, sumtis, sentences);
                let right = self.apply_selbri(*right_id, args, selbris, sumtis, sentences);

                match conn {
                    // je (AND): P(x) ∧ Q(x)
                    Connective::Je => LogicalForm::And(Box::new(left), Box::new(right)),

                    // ja (OR): P(x) ∨ Q(x)
                    Connective::Ja => LogicalForm::Or(Box::new(left), Box::new(right)),

                    // jo (IFF): (¬A ∨ B) ∧ (¬B ∨ A)
                    Connective::Jo => {
                        let not_l = LogicalForm::Not(Box::new(left.clone()));
                        let not_r = LogicalForm::Not(Box::new(right.clone()));
                        LogicalForm::And(
                            Box::new(LogicalForm::Or(Box::new(not_l), Box::new(right))),
                            Box::new(LogicalForm::Or(Box::new(not_r), Box::new(left))),
                        )
                    }

                    // ju (XOR): (A ∨ B) ∧ ¬(A ∧ B)
                    Connective::Ju => LogicalForm::And(
                        Box::new(LogicalForm::Or(
                            Box::new(left.clone()),
                            Box::new(right.clone()),
                        )),
                        Box::new(LogicalForm::Not(Box::new(LogicalForm::And(
                            Box::new(left),
                            Box::new(right),
                        )))),
                    ),
                }
            }

            Selbri::Compound(parts) => {
                let head = parts.last().map(|s| s.as_str()).unwrap_or("unknown");
                LogicalForm::Predicate {
                    relation: self.interner.get_or_intern(head),
                    args: args.to_vec(),
                }
            }
        }
    }

    // ─── Top-Level Bridi Compilation ─────────────────────────────

    pub fn compile_bridi(
        &mut self,
        bridi: &Bridi,
        selbris: &[Selbri],
        sumtis: &[Sumti],
        sentences: &[Bridi],
    ) -> LogicalForm {
        let target_arity = self.get_selbri_arity(bridi.relation, selbris);

        // FIX 1.3: Positional argument vector for place-tagged sumti.
        // Tagged sumti go to their explicit position; untagged fill remaining slots.
        let mut positioned: Vec<Option<LogicalTerm>> = vec![None; target_arity];
        let mut untagged: Vec<LogicalTerm> = Vec::new();
        let mut quantifiers: Vec<QuantifierEntry> = Vec::new();

        for &term_id in bridi.head_terms.iter().chain(bridi.tail_terms.iter()) {
            let sumti = &sumtis[term_id as usize];

            match sumti {
                Sumti::Tagged((tag, inner_id)) => {
                    let inner = &sumtis[*inner_id as usize];
                    let (term, quants) = self.resolve_sumti(inner, sumtis, selbris, sentences);
                    quantifiers.extend(quants);
                    let idx = match tag {
                        PlaceTag::Fa => 0,
                        PlaceTag::Fe => 1,
                        PlaceTag::Fi => 2,
                        PlaceTag::Fo => 3,
                        PlaceTag::Fu => 4,
                    };
                    if idx < target_arity {
                        positioned[idx] = Some(term);
                    }
                }
                other => {
                    let (term, quants) = self.resolve_sumti(other, sumtis, selbris, sentences);
                    quantifiers.extend(quants);
                    untagged.push(term);
                }
            }
        }

        // Merge: tagged args at their positions, untagged fill remaining None slots
        let mut untagged_iter = untagged.into_iter();
        let args: Vec<LogicalTerm> = positioned
            .into_iter()
            .map(|slot| {
                slot.or_else(|| untagged_iter.next())
                    .unwrap_or(LogicalTerm::Unspecified)
            })
            .collect();

        // Construct the main relation via tree traversal
        let mut final_form = self.apply_selbri(bridi.relation, &args, selbris, sumtis, sentences);

        // Wrap with existential quantifiers and restrictors (inner-to-outer)
        for entry in quantifiers.into_iter().rev() {
            let desc_arity = self.get_selbri_arity(entry.desc_id, selbris);

            let mut restrictor_args = Vec::with_capacity(desc_arity);
            restrictor_args.push(LogicalTerm::Variable(entry.var));
            while restrictor_args.len() < desc_arity {
                restrictor_args.push(LogicalTerm::Unspecified);
            }

            // Description selbris map structurally just like the main relation
            let desc_restrictor =
                self.apply_selbri(entry.desc_id, &restrictor_args, selbris, sumtis, sentences);
            let mut body = LogicalForm::And(Box::new(desc_restrictor), Box::new(final_form));

            // Conjoin relative clause restrictor if present
            if let Some(rel_restrictor) = entry.restrictor {
                body = LogicalForm::And(Box::new(rel_restrictor), Box::new(body));
            }

            final_form = LogicalForm::Exists(entry.var, Box::new(body));
        }

        // FIX 1.1: Sentence-level negation
        if bridi.negated {
            final_form = LogicalForm::Not(Box::new(final_form));
        }

        final_form
    }
}
