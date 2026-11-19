# LEX-a01

**Abstract.** Scans source text into tokens with spans or positioned diagnostics.

## Tier 1: Beginner

This function reads the source file character by character, skips whitespace and comments, and emits tokens for words, numbers, and symbols. Numbers too large to store are reported as errors with their location. Inputs: one source file. Output: a token list, or a list of diagnostics. Malformed input never crashes the tool.

## Tier 2: Silicon Reality

The scanner is a single byte-indexed pass with nested block-comment tracking and greedy two-character operator matching ordered longest first. Integer literals parse through i64 with a documented range check so overflow is a diagnostic, not a wrap. Cost is linear in source size; memory is one token per source token with no backtracking. Spans are byte offsets usable by every later diagnostic.

## Silent Failure Checklist

1. Fallbacks: none; invalid characters always produce diagnostics.
2. Truncation: integer literals are range-checked, never clamped.
3. Wrapping: none; parsing rejects overflow.
4. Error codes: none; diagnostics carry spans.
