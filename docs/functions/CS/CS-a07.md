# CS-a07

**Abstract.** Removes comment and string contents from one source line while preserving code characters, reporting literals and comment segments it found.

## Tier 1: Beginner

This function takes one line of Rust code and produces a version where comments and quoted text are erased, plus the list of comment texts and one string literal if present. The engine uses the erased version to scan real code, and the captured texts for rules that inspect comments and strings. Errors cannot occur; an unterminated quote consumes the rest of the line.

## Tier 2: Silicon Reality

The scanner is a single-pass character state machine: line comments terminate the scan, block comments toggle an in-block flag that survives across lines, string literals are captured with escape awareness, and quote characters are classified as char literals versus lifetimes by a lookahead test that avoids erasing code between two lifetime annotations. Cost is linear in line length with no allocation beyond the output string. Brace preservation is guaranteed because quotes are erased before any brace matching runs.

## Silent Failure Checklist

1. Fallbacks: none; every character is either copied, erased, or captured.
2. Truncation: character indices are usize and bounded by the line length.
3. Wrapping: none; no arithmetic on input values.
4. Error codes: none; the function is infallible by construction.
