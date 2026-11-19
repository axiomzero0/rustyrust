# FE-a06

**Abstract.** Checks one statement and emits typed statements.

## Tier 1: Beginner

This function checks one statement: bindings, assignments, effect expressions, returns, and loops. Type errors are reported with positions and the statement is dropped, so later stages only see typed input.

## Tier 2: Silicon Reality

Statement checking mutates one scoped variable table per block; scoping is restored by the caller through table copies. Cost is linear in statement size. Each failure path produces exactly one diagnostic to keep output deterministic.

## Silent Failure Checklist

1. Fallbacks: none; all outcomes are explicit data.
2. Truncation: none.
3. Wrapping: none.
4. Error codes: none; diagnostics replace errors by design.
