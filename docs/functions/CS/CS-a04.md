# CS-a04

**Abstract.** Runs the engine over input paths: deterministic file discovery, per-file rule dispatch, cross-file phases, and sorted findings.

## Tier 1: Beginner

This function decides which files to scan, scans them, runs every configured rule on every file, then runs the cross-file rules that need the whole picture, and returns the sorted findings. Inputs: configuration, input paths, and a working directory. Output: findings sorted by file, line, rule, and message. Errors come from configuration, registry, or document loading only.

## Tier 2: Silicon Reality

Discovery is a sorted queue walk with exclusion patterns over full paths; glob matching runs against both absolute and repository-relative forms. Per-file dispatch is a name-keyed match over check primitives, so no rule identifier appears in engine code. Cross-file phases run documentation synchronization and rosetta agreement after the per-file pass. Cost is linear in files times rules; the sort is O(n log n) in findings.

## Silent Failure Checklist

1. Fallbacks: none; unreadable non-first-party files are skipped by design and first-party unreadability surfaces at discovery only.
2. Truncation: none; findings carry full messages.
3. Wrapping: line numbers are usize; no arithmetic on user data.
4. Error codes: CEP-LINT-BOOT-1 and CEP-LINT-BOOT-8 from discovery; propagated from cross-file checks.
