# RustyRust

A self-hosting, formally verified Rust compiler. The project compiles a Rust subset to native x86-64 Linux executables, optimizes through proof-carrying certificate checks, and maintains a machine-checkable trusted computing base inventory.

## Status

The project is in early construction. Milestone status is tracked in `docs/onboarding/ROADMAP.md`. The enforcement standard is CEP&CC 0.4 (`standards.md`); the verification goals are defined in `verification standards.md`.

## Quick start

```text
cargo build --workspace
cargo test --workspace
cargo run -p cep_lint -- .            # mechanical enforcement gate
cargo run -p cep_lint -- --self-test  # rule catalog self-verification
```

## Architecture

```text
source .rs -> lexer -> parser -> HIR -> typeck -> THIR -> MIR -> [optimizer + certificates]
     -> verifier -> LIR -> regalloc -> machine IR -> encoder -> ELF64
```

The optimization pipeline is proof-producing: every pass emits a certificate, and an independent trusted verifier accepts transformed IR only after certificate checking. The formal verification kernel lives in `verification/` and is machine-checked in CI.

## Repository layout

| Directory | Purpose |
|---|---|
| `compiler/` | Frontend, type checking, borrow checking, MIR lowering, optimization, code generation, driver |
| `ir/` | Intermediate representations: HIR, THIR, MIR, LIR, machine IR |
| `backend/` | x86-64 target, register allocation, instruction selection, encoder |
| `checker/` | Certificate format and the independent trusted verifier |
| `runtime/` | Entry stub and print routine with documented ABI contracts |
| `verification/` | Rocq kernel proofs, SMT obligations, F-star, Lean, Isabelle artifacts |
| `tools/` | `cep_lint` enforcement tool and the provenance recorder |
| `tests/` | End-to-end tests that execute generated ELF binaries |
| `benchmarks/` | Compile-time and generated-code measurement harnesses |
| `docs/` | Registry, dual-tier function documentation, architecture notes |
| `config/` | Target and feature configuration |
| `security/` | Threat model and security policy |
| `scripts/` | Verification and bootstrap scripts |
| `.cep/` | Lint configuration and waivers |
| `third_party/` | Vendored external code |
| `quarantine/` | Non-compliant code awaiting repair or deletion |

## Enforcement

`tools/cep_lint` implements the rule catalog of CEP&CC 0.4 chapter 50. Configuration data lives in `.cep/cep_lint.json`; the document and the configuration are synchronized mechanically by the `CEP-LINT-DOC-SYNC` rule. CI runs the tool over the repository, the self-test manifest, the Rocq proofs, and the SMT obligations on every push.

## License

Apache License 2.0. See `LICENSE`.
