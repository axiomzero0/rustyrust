# CS-a08

**Abstract.** Verifies rule coverage and version agreement between the standard document and the lint configuration.

## Tier 1: Beginner

This function reads the standards document, finds every rule heading, and compares the list against the configured rules in both directions, then compares the document version line against the configured standard version. Inputs: the rule parameters, the configuration, and the working directory. Output: findings that name each drift case. A missing document is a bootstrap error, not a finding.

## Tier 2: Silicon Reality

Coverage is computed with a configured heading regex whose first capture group is the rule identifier, and version agreement uses a configured version regex with one capture group. Findings anchor at the version line when present and at line one otherwise, which keeps output deterministic across runs. Cost is linear in document lines plus rules; the duplicate detection pass is quadratic in heading count, which is bounded by the catalog size.

## Silent Failure Checklist

1. Fallbacks: none; both drift directions and version disagreement are always reported.
2. Truncation: none; messages carry full rule identifiers.
3. Wrapping: none.
4. Error codes: CEP-LINT-BOOT-1 for unreadable documents and CEP-LINT-BOOT-8 for invalid patterns.
