# FE-a10

**Abstract.** Lowers one expression to the local holding its value.

## Tier 1: Beginner

This function converts one expression into MIR statements and returns the local that stores the result. Every expression gets exactly one home.

## Tier 2: Silicon Reality

Operand expressions lower first, then the operator applies to copies of their locals, keeping MIR flat with no nested computations. Cost is linear in expression size.

## Silent Failure Checklist

1. Fallbacks: none; all outcomes are explicit data.
2. Truncation: none.
3. Wrapping: none.
4. Error codes: none; diagnostics replace errors by design.
