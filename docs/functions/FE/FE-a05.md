# FE-a05

**Abstract.** Type checks one program into a typed program.

## Tier 1: Beginner

This function reads every function of the parsed program, collects the signatures, checks each body against them, and validates the main function. Errors carry positions.

## Tier 2: Silicon Reality

Signature collection is a single pass with duplicate detection; body checking walks statements recursively with a scoped variable table. Cost is linear in program size. The trait solver reports zero goals for V0, which the checker asserts.

## Silent Failure Checklist

1. Fallbacks: none; all outcomes are explicit data.
2. Truncation: none.
3. Wrapping: none.
4. Error codes: none; diagnostics replace errors by design.
