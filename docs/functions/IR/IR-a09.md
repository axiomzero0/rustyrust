# IR-a09

**Abstract.** Removes dead stores in one function and emits obligations.

## Tier 1: Beginner

This component is part of the optimization pipeline. It transforms the program and records evidence describing exactly what changed, so an independent checker can confirm the transformation preserved meaning.

## Tier 2: Silicon Reality

The implementation follows the HPC pass contract: explicit legality conditions, one pass per file, verification after the pass, and certificate emission with input, intermediate, and output hashes over canonical printed IR. Cost is linear in program size for the scan and constant per obligation record.

## Silent Failure Checklist

1. Fallbacks: none; legality failures leave the statement unchanged.
2. Truncation: none.
3. Wrapping: arithmetic follows the documented wrap-i64 dialect semantics.
4. Error codes: none; obligations and diagnostics are data.
