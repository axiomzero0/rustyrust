# LEX-a02

**Abstract.** Skips whitespace, line comments, and nested block comments from one byte offset.

## Tier 1: Beginner

This function moves a cursor forward past spaces, newlines, and comments so scanning can continue at real code. Block comments may nest; each opening marker needs one closing marker. Inputs: source text and a cursor. Output: the next cursor position.

## Tier 2: Silicon Reality

The walk is a depth-counted single pass over bytes; nesting depth is a usize counter with no allocation. Cost is linear in the skipped span. A file ending inside a comment stops the walk at end of input deterministically. Line comments end at the newline byte, matching the scanner's byte model.

## Silent Failure Checklist

1. Fallbacks: none; unterminated comments terminate at end of input without error.
2. Truncation: none.
3. Wrapping: none; depth is a usize counter.
4. Error codes: none; the function is infallible by construction.
