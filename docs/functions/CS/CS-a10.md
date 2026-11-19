# CS-a10

**Abstract.** Verifies one certificate against the output program with independent semantics re-derivation.

## Tier 1: Beginner

This function checks the hashes recorded in a certificate against the actual program, then re-derives every recorded optimization from first principles. If anything disagrees, the certificate is rejected and the optimized program is not used.

## Tier 2: Silicon Reality

The checker duplicates the wrap-i64 semantics table rather than calling optimizer code, which is the trust boundary required by G16. Hash checks bind evidence to the exact program via canonical printed IR. Cost is linear in obligations plus one program scan per dead-store obligation; the quadratic bound is acceptable at V0 program sizes and documented as a limitation.

## Silent Failure Checklist

1. Fallbacks: none; every disagreement is a rejection.
2. Truncation: none.
3. Wrapping: the re-derivation table mirrors dialect wrapping semantics.
4. Error codes: none; rejection reasons are returned as data.
