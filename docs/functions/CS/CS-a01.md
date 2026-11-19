# CS-a01

**Abstract.** Extracts function records from Rust source text: name, statement count, empty-body flag, constant-return flag, and the associated CEP comment block.

## Tier 1: Beginner

This function reads a Rust file after comments and strings have been removed, finds every `fn` definition, and records where its body starts and ends. For each function it counts the statements and looks for the comment block that sits directly above the definition, skipping attribute lines. Inputs: the raw lines, the stripped lines, and the comment blocks of one file. Output: one record per detected function. Errors cannot occur; malformed functions are skipped.

## Tier 2: Silicon Reality

The extractor runs a signature regex over each stripped line, converts byte offsets to character offsets, and performs balanced brace matching over the flattened stripped text with a semicolon guard for bodiless declarations. Statement counting is the number of semicolons inside the body span, which is a lexical approximation documented in section 50.3. Cost is linear in file size for flattening and linear per function for brace scanning; no hash maps or dynamic dispatch are involved. The associated block search walks upward over gap lines and matches a block whose end line equals the candidate line, so header blocks can be rejected by start-line identity.

## Silent Failure Checklist

1. Fallbacks: none; a brace-unbalanced function produces no record and no diagnostic.
2. Truncation: byte-to-character offsets are computed per line and are boundary-safe for regex match positions.
3. Wrapping: offsets are usize; substraction is guarded by cursor bounds.
4. Error codes: none; the function is infallible by construction.
