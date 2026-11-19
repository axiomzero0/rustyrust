# IR-a05

**Abstract.** Lowers one short-circuit operator into guarded blocks.

## Tier 1: Beginner

This function lowers and and or so the right side runs only when the result is undecided.

## Tier 2: Silicon Reality

The result local holds the left value; a switch jumps to the right side only for the undecided case, keyed by operator polarity. Cost is linear in operand size with three blocks per operator.

## Silent Failure Checklist

1. Fallbacks: none; all outcomes are explicit data.
2. Truncation: none.
3. Wrapping: none.
4. Error codes: none; diagnostics replace errors by design.
