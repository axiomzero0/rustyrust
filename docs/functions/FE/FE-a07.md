# FE-a07

**Abstract.** Checks one expression and emits its typed form.

## Tier 1: Beginner

This function walks an expression, resolves variable names, applies the operator typing rules, and checks calls against signatures. The result is a typed expression tree.

## Tier 2: Silicon Reality

Operator rules split arithmetic, comparison, and logic classes with fixed result types. Calls check arity and per-argument types against the signature table. Cost is linear in expression size with bounded recursion through the depth-bounded parser input.

## Silent Failure Checklist

1. Fallbacks: none; all outcomes are explicit data.
2. Truncation: none.
3. Wrapping: none.
4. Error codes: none; diagnostics replace errors by design.
