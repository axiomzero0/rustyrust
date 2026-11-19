# Roadmap

**Abstract.** Tracks milestone status for the compiler construction plan, with deliverables and the standard that gates each milestone.

## Milestones

| Milestone | Deliverable | Status |
|---|---|---|
| M0 | Enforcement backbone: cep_lint, configuration, manifest, repository topology | complete |
| M1 | IR crates: HIR, THIR, MIR, LIR, machine IR with printer, hash, verifier | planned |
| M2 | Frontend: lexer and parser for the V0 dialect | planned |
| M3 | Semantic layer: type checker, trait solver skeleton, move analysis | planned |
| M4 | MIR lowering with control-flow desugaring | planned |
| M5 | Optimizer with certificate emission and independent verifier | planned |
| M6 | Backend: encoder, register allocation, instruction selection, ELF emission | planned |
| M7 | Driver CLI, end-to-end ELF execution tests, benchmark harness | planned |
| M8 | Formal verification: Rocq kernel proofs, SMT obligations, companion prover artifacts | planned |
| M9 | Provenance records, TCB inventory, verified bootstrap scaffolding | planned |

## Language compatibility roadmap

| Release | Language support |
|---|---|
| V0 | Safe sequential subset: i64, bool, unit, functions, let, if, while, calls, print_i64 |
| V1 | Generics, traits, associated types |
| V2 | Lifetimes, advanced borrowing, drop semantics |
| V3 | Unsafe Rust and raw pointers |
| V4 | Concurrency, atomics, unwinding |
| V5 | Procedural macros, broader standard library, FFI |
| V6 | Broad stable Rust compatibility |

## Known Limitations

The V0 dialect deviates from stable Rust: `main` returns i64 as the process exit code, arithmetic wraps with two's complement semantics in all profiles, and division by zero terminates the process. These are documented dialect decisions in `docs/architecture/RR_LANGUAGE_V0.md`, not oversights.
