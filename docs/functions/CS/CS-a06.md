# CS-a06

**Abstract.** Binary entry point mapping options, self-test, and lint runs onto the documented exit codes.

## Tier 1: Beginner

This function parses the command line, prints help or version when asked, runs the self-test when asked, otherwise loads the configuration, runs the engine over the given paths, prints the report, and exits with the configured code. Inputs: process arguments. Output: process exit status. Usage errors exit with three; bootstrap failures exit with two.

## Tier 2: Silicon Reality

The version banner prints the tool version from the package metadata and the standard version from configuration data, binding the binary to the enforced revision at run time rather than compile time. Findings map to the configured fail-on severity list; the report is written to standard output in the sorted format. Cost is one configuration load and one engine pass.

## Silent Failure Checklist

1. Fallbacks: none; every failure path has a coded message and exit status.
2. Truncation: none.
3. Wrapping: exit codes are u8 per process semantics.
4. Error codes: exit 0 clean or non-blocking findings, exit 1 blocking findings, exit 2 bootstrap, exit 3 usage.
