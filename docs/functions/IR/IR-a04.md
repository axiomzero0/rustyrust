# IR-a04

**Abstract.** Lowers one statement into MIR statements and terminators.

## Tier 1: Beginner

This function converts one typed statement: bindings record locals, assignments emit rvalues, returns patch the terminator, loops desugar into blocks.

## Tier 2: Silicon Reality

Statement lowering keeps the insertion cursor on open blocks only; terminated blocks are never written, which keeps MIR linkage valid by construction. Cost is linear in statement size.

## Silent Failure Checklist

1. Fallbacks: none; all outcomes are explicit data.
2. Truncation: none.
3. Wrapping: none.
4. Error codes: none; diagnostics replace errors by design.
