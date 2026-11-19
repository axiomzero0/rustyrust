# CS-a03

**Abstract.** Loads the lint configuration with include merging and schema validation, producing the engine configuration or a coded bootstrap failure.

## Tier 1: Beginner

This function reads the JSON configuration file, merges any included files, collects the rule list, and validates each rule: severity range, known check name, and compilable patterns. Inputs: a configuration path. Output: a configuration, or a bootstrap error with a code from one to eight. Errors are reported with file, line, and column for JSON problems.

## Tier 2: Silicon Reality

Includes are resolved relative to the including file, tracked in a stack for cycle detection, and bounded at depth eight; merged keys never overwrite parent keys. Rule validation compiles every regex-shaped parameter eagerly so check execution cannot panic on patterns. Cost is linear in configuration size with one-time regex compilation. Severity range check implements bootstrap code seven; unknown check names implement code six.

## Silent Failure Checklist

1. Fallbacks: none; every failure is a coded bootstrap error, never a silent default.
2. Truncation: none; strings are carried verbatim.
3. Wrapping: none; numeric thresholds are validated as integral.
4. Error codes: CEP-LINT-BOOT-1 through CEP-LINT-BOOT-8 with detail strings.
