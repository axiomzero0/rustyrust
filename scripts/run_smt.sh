#!/bin/sh
# CEP:FILE: scripts/run_smt.sh
# CEP:WHAT: Runs every SMT obligation file and compares results against recorded expectations.
# CEP:WHY: SMT results enter the verification matrix only when reproducible and bounded.
# CEP:CLASS: CEP-2
# CEP:STATUS: complete
# CEP:FAILURE: Exits non-zero when a result differs from the expectation file or a solver times out.
# CEP:ASSUMES: z3 is installed and each obligation file declares its own bounded timeout.
# CEP:COST: offline tool; runtime cost irrelevant.
# CEP:EVIDENCE: CI smt job; expected outputs under verification/smt/expected/.
set -e
Z3="${Z3BIN:-z3}"
cd "$(dirname "$0")/.."
status=0
for f in verification/smt/*.smt2; do
    name=$(basename "$f" .smt2)
    expected="verification/smt/expected/${name}.txt"
    actual=$("$Z3" "$f" 2>&1)
    if [ ! -f "$expected" ]; then
        echo "MISSING EXPECTATION: $expected"
        status=1
        continue
    fi
    if ! printf '%s\n' "$actual" | diff -q - "$expected" >/dev/null; then
        echo "MISMATCH: $name"
        printf '%s\n' "$actual" | diff - "$expected" | head -5
        status=1
    else
        echo "OK: $name"
    fi
done
exit $status
