// CEP:FILE: ir/mir/src/lib.rs
// CEP:WHAT: Library root for the mid-level IR.
// CEP:WHY: Lowering, optimization, and code generation share one block-based program model.
// CEP:CLASS: HPC-1
// CEP:STATUS: complete
// CEP:FAILURE: none
// CEP:ASSUMES: none
// CEP:COST: cold; module declarations only.
// CEP:EVIDENCE: cargo build -p rr-mir.
//! Mid-level IR with basic blocks, printer, hash, and verifier.
#![deny(missing_docs)]

/// The MIR program model.
pub mod model;
/// Deterministic printer and stable hash.
pub mod printer;
/// Structural verifier for MIR programs.
pub mod verifier;
