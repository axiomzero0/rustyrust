# FE-a11

**Abstract.** Lowers control flow: loops, short-circuit operators, conditionals, and calls.

## Tier 1: Beginner

This function turns while loops, and-or operators, if expressions, and calls into blocks with branch terminators. Short-circuit operators guard the right operand.

## Tier 2: Silicon Reality

While loops desugar into header, body, and exit blocks with a switch on the condition discriminant. Short-circuit forms write the left value first and jump over the right side when the result is decided. Calls are terminators with a resume block, matching machine call semantics.

## Silent Failure Checklist

1. Fallbacks: none; all outcomes are explicit data.
2. Truncation: none.
3. Wrapping: none.
4. Error codes: none; diagnostics replace errors by design.
