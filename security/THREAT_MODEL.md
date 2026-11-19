# Threat Model

**Abstract.** Defines the trust boundaries of the compiler toolchain, the assets protected, and the adversarial inputs each boundary must survive.

## Trust boundaries

1. Source input: `.rs` files passed to the compiler are untrusted. The parser and every IR stage must survive malformed input with diagnostics, bounded resource use, and no memory unsafety.
2. Generated code: ELF output is produced byte-exactly; no external linker or loader participates.
3. Lint configuration: `.cep/cep_lint.json` is repository-trusted; it gates enforcement behavior.
4. Receipts: hunt and HUNTER receipts are adversarial until signature verification is implemented; until then their absence always blocks, never silently passes.

## Adversarial inputs

- Deeply nested expressions and blocks: bounded by recursion depth limits in the parser.
- Oversized integer literals: rejected at lexing with a span.
- Invalid UTF-8: rejected at file read with a diagnostic.
- Pathological certificate content: the verifier validates structure before semantics.

## Invariants

1. No compiler panic is reachable from malformed source input; all failures are diagnostics.
2. Compiler resource use is bounded by input size with documented constants.
3. Secrets, tokens, and credentials never enter the repository; the secret scan gate enforces the lexical probe.

## Known Limitations

Fuzzing infrastructure is defined by chapter 61 quotas and is not yet running continuously; the current gate is the deterministic test suite. Signature verification of receipts is a documented refinement gap tracked under ticket RR-102.
