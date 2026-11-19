# IR-a03

**Abstract.** Prints one MIR terminator in canonical form for hashing and golden comparison.

## Tier 1: Beginner

This function turns the end-of-block instruction into text: a plain jump, a branch with its targets, a function call with arguments and destination, a return, or a trap. Inputs: one terminator. Output: written lines in the canonical text. Errors cannot occur.

## Tier 2: Silicon Reality

The printer emits targets in declaration order with stable formatting, so the FNV-1a hash over the printed program is order-independent and reproducible per section 38.19. Cost is linear in the number of branch targets per terminator with one string write per element. No hash maps participate, so iteration order cannot leak into output.

## Silent Failure Checklist

1. Fallbacks: none; every terminator variant has one canonical form.
2. Truncation: none; block identifiers are usize rendered in full.
3. Wrapping: none.
4. Error codes: none; the function is infallible by construction.
