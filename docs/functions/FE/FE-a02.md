# FE-a02

**Abstract.** Parses one statement of the V0 dialect.

## Tier 1: Beginner

This function reads one statement: a variable binding, an assignment, a return, a loop, or an expression run for effect. Each form ends with a semicolon. Inputs: the parser cursor. Output: one statement, or diagnostics for malformed forms.

## Tier 2: Silicon Reality

Statement dispatch is keyword-driven with fallthrough to expression statements, so the grammar stays flat and termination is linear. Assignment targets are validated syntactically; semantic checks happen later. Cost is linear in statement size.

## Silent Failure Checklist

1. Fallbacks: none.
2. Truncation: none.
3. Wrapping: none.
4. Error codes: none; diagnostics carry spans.
