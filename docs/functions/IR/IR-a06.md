# IR-a06

**Abstract.** Lowers one conditional into branch blocks and a join block.

## Tier 1: Beginner

This function lowers if expressions: the condition switches to branch blocks, both branches store into one result local, and flow joins at one block.

## Tier 2: Silicon Reality

Branch blocks patch their own goto to the join when open; branches ending in return skip the patch, which keeps the graph valid. Cost is linear in branch size with one switch and one result local per conditional.

## Silent Failure Checklist

1. Fallbacks: none; all outcomes are explicit data.
2. Truncation: none.
3. Wrapping: none.
4. Error codes: none; diagnostics replace errors by design.
