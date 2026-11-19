# FE-a01

**Abstract.** Parses a token stream into a program of function definitions.

## Tier 1: Beginner

This function walks the token list and reads one function definition after another. Anything unexpected produces an error message with its position. Inputs: source text and its tokens. Output: a program, or diagnostics.

## Tier 2: Silicon Reality

Top-level parsing skips no tokens silently: unexpected tokens produce diagnostics one token at a time, which keeps error output deterministic. Cost is linear in token count. Recursion is bounded by the parser depth constant, enforced at every block entry, so adversarial nesting terminates with diagnostics per section 38.6.

## Silent Failure Checklist

1. Fallbacks: none; syntax errors always yield diagnostics.
2. Truncation: none.
3. Wrapping: none.
4. Error codes: none; diagnostics carry spans.
