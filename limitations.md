Known limitations of this implementation

Relative clause variable injection (inject_variable) replaces the first Unspecified arg. Works for the common lo X poi Y pattern. Won't handle exotic cases like rel clauses with explicit ke'a references.
Quantifiers from be/bei arguments are scoped locally within apply_selbri for WithArgs. This is correct for most cases but differs slightly from the outer quantifier wrapping scope.
jo and ju require Clone on LogicalForm (already derived). The decomposed forms are verbose but semantically precise.
