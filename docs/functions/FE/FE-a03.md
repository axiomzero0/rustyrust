# FE-a03

**Abstract.** Parses expressions with precedence climbing from lowest to highest binding power.

## Tier 1: Beginner

This function reads arithmetic and logic expressions, grouping operators by strength the way mathematics does: multiplication binds tighter than addition, comparisons weaker still. Inputs: the parser cursor. Output: one expression tree.

## Tier 2: Silicon Reality

Precedence climbing runs one loop per level with a static operator table, so cost is linear in token count and recursion is bounded by the depth constant on every level entry. Short-circuit operators keep their own precedence levels; comparison chains are non-associative by the plus-one recursion rule.

## Silent Failure Checklist

1. Fallbacks: none.
2. Truncation: none.
3. Wrapping: none.
4. Error codes: none; diagnostics carry spans.
