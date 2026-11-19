# FE-a09

**Abstract.** Lowers one typed program to MIR.

## Tier 1: Beginner

This function converts every typed function into blocks and locals. Local zero holds the return value; parameters follow. The output passes the structural verifier.

## Tier 2: Silicon Reality

Lowering is a per-function pass with a builder that owns the block list and an insertion cursor. Cost is linear in program size. Block indices grow monotonically so printing and hashing stay stable.

## Silent Failure Checklist

1. Fallbacks: none; all outcomes are explicit data.
2. Truncation: none.
3. Wrapping: none.
4. Error codes: none; diagnostics replace errors by design.
