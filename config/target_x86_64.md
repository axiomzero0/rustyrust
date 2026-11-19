# x86-64 Target Contract

**Abstract.** Defines the target machine assumptions of the first backend: operating system interface, ABI, memory layout, and instruction set baseline.

## Formal Definitions

- Architecture: x86-64, baseline instruction set (SSE2 guaranteed by the ABI; no extended ISA in V0).
- Operating system: Linux, static executables, no dynamic loader.
- Executable format: ELF64, one PT_LOAD segment mapped at base address 0x400000 with read and execute permissions.
- Entry: the runtime `_start` stub calls `main`, then issues the exit syscall (60) with the truncated 64-bit return value.

## Algorithm

Code generation produces a single byte image: ELF header, one program header, then the text image containing the runtime stub, the runtime `print_i64` routine, and the compiled functions. Label fixups are resolved by a two-pass assembler before image assembly.

## Invariants

1. The stack is 16-byte aligned at call sites per the System V ABI.
2. Integer arguments pass in RDI, RSI, RDX, RCX, R8, R9; the return value passes in RAX.
3. RBX, RBP, R12 through R15 are callee-saved; the compiler only reserves RBP as frame pointer.
4. `print_i64` clobbers only caller-saved registers.
5. Division by zero raises SIGFPE, which is the documented V0 trap semantics.

## Integration Points

Depends on: `rr-encoder` (byte emission), `rr-runtime` (entry stub and print routine). Depended on by: `rr-codegen` (image assembly), `tests/e2e` (execution contract).

## Known Limitations

No writable data segment; the V0 dialect has no global mutable state. No relocations and no object-file emission; linking is not supported. Page-size assumptions are limited to the ELF alignment constant.
