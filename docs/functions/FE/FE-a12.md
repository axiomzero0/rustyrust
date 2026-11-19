# FE-a12

**Abstract.** Checks one conditional with branch type unification.

## Tier 1: Beginner

This function checks that an if condition is a boolean, checks both branches, and confirms the branches agree on their value type when both produce one. Errors carry positions.

## Tier 2: Silicon Reality

Branch checking runs against copies of the variable table so bindings stay block-scoped. Unification compares the trailing value types of both branches; statement-only conditionals with no value always unify at unit. Cost is linear in branch size with one table copy per branch.

## Silent Failure Checklist

1. Fallbacks: none; unification failures are always reported.
2. Truncation: none.
3. Wrapping: none.
4. Error codes: none; diagnostics carry spans.
