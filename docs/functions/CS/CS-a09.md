# CS-a09

**Abstract.** Parses one JSON value from text for certificate loading.

## Tier 1: Beginner

This function reads certificate text and turns it into data the verifier can check. Malformed text yields no data rather than a crash, and the verifier reports the failure.

## Tier 2: Silicon Reality

The parser is a recursive descent over characters with objects preserving insertion order so serialization is deterministic. Cost is linear in input size; recursion depth is bounded by certificate size, which is bounded by program size. No external JSON dependency participates in the trusted checking path.

## Silent Failure Checklist

1. Fallbacks: none; malformed input yields no value.
2. Truncation: none.
3. Wrapping: none.
4. Error codes: none; absence of a value is the failure signal.
