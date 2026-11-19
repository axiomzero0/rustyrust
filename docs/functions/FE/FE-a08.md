# FE-a08

**Abstract.** Runs move analysis over one typed program.

## Tier 1: Beginner

This function walks every function body and records ownership-relevant moves. In the V0 subset every type is Copy, so the analysis reports no violations.

## Tier 2: Silicon Reality

The framework is the V2 insertion point: non-Copy types will produce move-after-move violations here. Cost is linear in program size with one pass and no allocation beyond the violation vector.

## Silent Failure Checklist

1. Fallbacks: none; all outcomes are explicit data.
2. Truncation: none.
3. Wrapping: none.
4. Error codes: none; diagnostics replace errors by design.
