# CS-a02

**Abstract.** Flags numeric literals that are outside the configured allow-list and that lack a sanctioned context such as a const declaration.

## Tier 1: Beginner

This check walks the code of a file after comments and strings are removed. Every number it finds must be either in the allowed list (0 and 1) or sit inside a statement that declares a constant. Inputs: the file scan and the rule parameters. Output: one finding per unsanctioned literal. Errors cannot occur.

## Tier 2: Silicon Reality

Literals are matched with a word-bounded regex that also covers hexadecimal and underscore forms; value comparison is numeric, so 0x0 equals 0. The exemption window joins up to twelve lines around the literal, mirroring the window in section 50.3, and matches the joined text against the configured context regexes. Cost is linear in literals with a bounded window join per literal. No allocation beyond the window string.

## Silent Failure Checklist

1. Fallbacks: none; unsanctioned literals are always reported.
2. Truncation: hex values beyond i64 range clamp to i64::MAX and still match by value only for allowed lists they cannot equal.
3. Wrapping: none; the comparison path uses checked parse with clamping.
4. Error codes: none; context regexes are validated at configuration load time.
