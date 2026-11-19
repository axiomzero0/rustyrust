# Architecture

**Abstract.** Describes the staged compiler pipeline of RustyRust, the verification boundaries between stages, and the data that flows across each boundary.

## Pipeline

```text
frontend          semantic           verified core        optimization            backend            artifact
.rs -> tokens -> HIR -> typeck -> THIR -> MIR -> [passes + certificates] -> LIR -> machine -> ELF64
                                                             |
                                                       trusted verifier
```

## Formal Definitions

Each stage is a workspace crate with a single responsibility. The IR crates define the data: `rr-hir` (source syntax with spans), `rr-thir` (typed high-level expressions), `rr-mir` (block-based transfer functions with explicit locals), `rr-lir` (instruction selection output with virtual registers), `rr-machine` (physical-register instructions with frame layout). The compiler crates transform data between IRs; the backend crates lower to bytes; the checker crates verify transformations independently of the optimizer.

## Algorithm

1. The frontend lexes and parses the V0 dialect into HIR with byte spans.
2. The type checker resolves names, validates types, and emits THIR.
3. The MIR lowering stage desugars control flow into basic blocks with terminators.
4. The optimizer runs ordered passes; each pass emits a certificate of obligations.
5. The verifier re-derives every obligation from the certificate data and the MIR semantics, independent of optimizer code.
6. The backend selects instructions, allocates registers, encodes bytes, and emits a static ELF64 executable with the runtime stub linked in.

## Invariants

1. Every IR mutation is followed by IR verification in debug and CI runs.
2. Every pass certificate is checked before optimized IR enters code generation.
3. Compilation is deterministic: identical inputs, configuration, and toolchain produce identical ELF bytes.
4. Integer arithmetic in the V0 dialect wraps; division by zero is a defined trap.
5. No component trusts an SMT answer without an independently checkable artifact.

## Integration Points

Depends on: `verification/semantics` (Rocq kernel constrains the MIR arithmetic semantics), `config/target_x86_64.md` (target contract). Depended on by: `tests/e2e` (executes generated ELF), `benchmarks` (measures both compile time and generated-code time).

## Known Limitations

Register allocation uses a linear scan over a fixed physical pool with rbp-relative local slots; the full virtual-register allocator is a later milestone. The backend emits static ELF64 executables for Linux x86-64 only; object files and a linker are future work. Division proofs are excluded from the first Rocq kernel theorem and covered by SMT obligations instead.
