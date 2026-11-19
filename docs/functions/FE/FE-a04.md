# FE-a04

**Abstract.** Parses primary expressions: literals, variables, calls, conditionals, and blocks.

## Tier 1: Beginner

This function reads the smallest pieces of expressions: numbers, true and false, variable names, function calls with their arguments, if-else forms, and brace blocks. Inputs: the parser cursor. Output: one expression, or diagnostics.

## Tier 2: Silicon Reality

Call arguments parse into a vector with comma separation and no arity checks at this layer; arity and types are checked by the type checker, keeping parser concerns purely syntactic. Cost is linear in expression size with bounded recursion.

## Silent Failure Checklist

1. Fallbacks: none.
2. Truncation: none.
3. Wrapping: none.
4. Error codes: none; diagnostics carry spans.
