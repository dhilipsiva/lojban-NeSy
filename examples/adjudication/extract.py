#!/usr/bin/env python3
"""Convert Soufflé / CodeQL tabular output into nibli KR ground facts.

    souffle -F facts -D out taint.dl
    ./extract.py --relation carries out/TaintFlow.csv >> facts.nibli

    codeql query run --output=r.bqrs taint.ql
    codeql bqrs decode --format=csv r.bqrs | ./extract.py --relation carries --format csv -

WHY THIS ESCAPES RATHER THAN INTERPOLATES
-----------------------------------------
Analyzer columns carry identifiers taken from the code under analysis —
file paths, variable names, string literals. In a security pipeline those
are ATTACKER-INFLUENCED: whoever can name a file in the analyzed repository
can choose the bytes that land here. Interpolating them straight into KR
text would let a crafted name close the quoted term and append statements
of its own — `permits(Review, "flow:1", Waiver).` clears the attacker's own
finding, and `permits` is admitted vocabulary, so the `admits` closure does
not catch it.

So: backslash and double-quote are escaped, and anything with a control
character or an embedded newline is REFUSED rather than sanitized. A field
that cannot be represented exactly is a hard error — never a quietly
rewritten one.
"""

import argparse
import csv
import re
import sys

# Corpus arity for the relations this example admits. Fail closed on a
# column-count mismatch: a widened Soufflé relation must be a conscious edit
# here, not a silently truncated or padded fact.
ARITY = {"carries": 5, "dangerous": 3, "prevents": 2, "permits": 3}

_FORBIDDEN = re.compile(r"[\x00-\x1f\x7f]")


def kr_term(field: str, *, row: int, col: int) -> str:
    """Render one analyzer column as a quoted KR term, or die."""
    if _FORBIDDEN.search(field):
        sys.exit(
            f"extract.py: row {row} column {col}: field contains a control "
            f"character and cannot be represented as a KR term: {field!r}"
        )
    escaped = field.replace("\\", "\\\\").replace('"', '\\"')
    return f'"{escaped}"'


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--relation", required=True, help="target KR relation name")
    ap.add_argument(
        "--format",
        choices=("tsv", "csv"),
        default="tsv",
        help="tsv = Soufflé default output; csv = codeql bqrs decode --format=csv",
    )
    ap.add_argument("input", help="analyzer output file, or - for stdin")
    ap.add_argument(
        "--skip-header",
        action="store_true",
        help="drop the first row (codeql bqrs decode emits column names)",
    )
    args = ap.parse_args()

    expected = ARITY.get(args.relation)
    if expected is None:
        sys.exit(
            f"extract.py: {args.relation!r} is not an admitted relation of this "
            f"policy (known: {', '.join(sorted(ARITY))}). Add it to ARITY here AND "
            f"to the `admits` block in policy.nibli — two visible, reviewable edits."
        )

    handle = sys.stdin if args.input == "-" else open(args.input, newline="")
    with handle:
        reader = csv.reader(handle, delimiter="\t" if args.format == "tsv" else ",")
        for row_no, row in enumerate(reader, start=1):
            if args.skip_header and row_no == 1:
                continue
            if not row:
                continue
            if len(row) != expected:
                sys.exit(
                    f"extract.py: row {row_no}: {args.relation} has arity "
                    f"{expected} but the row has {len(row)} columns — refusing to "
                    f"pad or truncate. Fix the analyzer query or ARITY, not the data."
                )
            terms = ", ".join(
                kr_term(f, row=row_no, col=i) for i, f in enumerate(row, start=1)
            )
            print(f"{args.relation}({terms}).")


if __name__ == "__main__":
    main()
