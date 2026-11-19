# IR-a01

**Abstract.** Prints one MIR program to canonical text used by certificates, logs, and hashes.

## Tier 1: Beginner

This function writes every function of the program as text: signature, locals, blocks, statements, and terminators. Inputs: one program. Output: the canonical text. Errors cannot occur.

## Tier 2: Silicon Reality

Printing walks functions and blocks in declaration order with fixed formatting, so two programs with equal semantics and equal shape produce identical text and identical FNV-1a hashes per section 38.19. Cost is linear in program size with one buffered string and no hash maps. The hash feeds the certificate input and output fields, so print stability is a correctness property, not a convenience.

## Silent Failure Checklist

1. Fallbacks: none; every construct has one canonical rendering.
2. Truncation: none; local and block indices render in full.
3. Wrapping: none.
4. Error codes: none; the function is infallible by construction.
