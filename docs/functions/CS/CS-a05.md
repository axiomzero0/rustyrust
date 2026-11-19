# CS-a05

**Abstract.** Runs the self-test manifest with exact per-rule finding count verification in isolated scenario directories.

## Tier 1: Beginner

This function reads the manifest, creates one temporary directory per scenario, writes the scenario files, runs the engine restricted to the scenario rule, and compares the finding count against the expected number. Inputs: the configuration path. Output: a pass or failure summary string. Errors come from manifest loading only; scenario deviations are reported as failures, not errors.

## Tier 2: Silicon Reality

Scenarios run with a per-scenario working directory so cross-file rules see only scenario files; cross-file rules are skipped unless the scenario declares them. The temp root is recreated per run for determinism and clean state. Counting filters by rule identifier so unrelated rule findings do not mask the target behavior. Cost is linear in scenarios times rules with one engine run per scenario.

## Silent Failure Checklist

1. Fallbacks: none; a bootstrap failure inside a scenario is a reported scenario failure.
2. Truncation: none.
3. Wrapping: none.
4. Error codes: CEP-LINT-BOOT-1 and CEP-LINT-BOOT-2 from manifest loading; scenario failures are textual.
