# IR-a02

**Abstract.** Verifies structural validity of MIR programs: use-def validity, terminator validity, and block linkage.

## Tier 1: Beginner

This function checks that every statement reads and writes locals that exist, and that every branch targets a block that exists. Inputs: one program. Output: one diagnostic per violation, or none. Errors cannot occur; bad programs produce diagnostics, not crashes.

## Tier 2: Silicon Reality

The verifier runs after lowering and after each optimization pass per section 38.18. Cost is linear in program size: one local-table bounds check per read, write, and branch target. Diagnostics carry function, block, and local identifiers so failures point at the exact mutation site. The cheap mode runs per pass in-process; the full mode runs in CI over golden programs.

## Silent Failure Checklist

1. Fallbacks: none; violations are always reported.
2. Truncation: none.
3. Wrapping: none.
4. Error codes: none; diagnostics replace errors by design.
