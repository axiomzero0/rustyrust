// CEP:FILE: ir/machine/src/lib.rs
// CEP:WHAT: Library root for the machine IR after register assignment.
// CEP:WHY: Code generation consumes one frame-and-instruction model per function.
// CEP:CLASS: HPC-1
// CEP:STATUS: complete
// CEP:FAILURE: none
// CEP:ASSUMES: Frame slots are eight bytes and rbp-relative; documented in the target contract.
// CEP:COST: cold; type definitions only.
// CEP:EVIDENCE: cargo build -p rr-machine.
//! Machine IR after register assignment.
#![deny(missing_docs)]
use rr_lir::Inst;

/// A machine-level function with a concrete frame layout.
#[derive(Debug, Clone)]
pub struct MachFn {
    /// Function name; the entry point is named main.
    pub name: String,
    /// Instructions with virtual registers resolved.
    pub insts: Vec<Inst>,
    /// Total frame bytes below rbp, sixteen-byte aligned.
    pub frame_size: i32,
    /// Highest slot index in use.
    pub slots: u32,
}
